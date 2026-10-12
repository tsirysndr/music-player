//! Remote clients mutate the daemon manager, never a second local registry.
use music_player_playback::{
    extension_accounts::{AccountError, ExtensionAccounts},
    source_resolver::SourceResolver,
};
use tonic::{Request, Response, Status};

use crate::api::music::v1alpha1::{
    extensions_service_server::ExtensionsService, InstalledExtension, ListExtensionsRequest,
    ListExtensionsResponse, RescanExtensionsRequest, RescanExtensionsResponse,
    SetExtensionEnabledRequest, SetExtensionEnabledResponse,
};

pub struct Extensions {
    sources: SourceResolver,
}
impl Extensions {
    pub fn new(sources: SourceResolver) -> Self {
        Self { sources }
    }
    fn manager(&self) -> Result<&ExtensionAccounts, Status> {
        self.sources
            .extension_accounts()
            .map_err(|error| Status::failed_precondition(error.to_string()))
    }
    async fn catalog(&self) -> Result<Vec<InstalledExtension>, Status> {
        Ok(self
            .manager()?
            .installed()
            .await
            .map_err(account_error)?
            .into_iter()
            .map(|item| {
                let enabled = item.enabled();
                let manifest = item.manifest;
                InstalledExtension {
                    id: manifest.id,
                    name: manifest.name,
                    version: manifest.version,
                    author: manifest.author,
                    description: manifest.description,
                    capabilities: manifest
                        .capabilities
                        .iter()
                        .map(|c| c.as_str().into())
                        .collect(),
                    allowed_hosts: manifest.permissions.allowed_hosts,
                    library_read: manifest.permissions.library_read,
                    topics: manifest.topics,
                    directory: item.dir.to_string_lossy().into_owned(),
                    enabled,
                }
            })
            .collect())
    }
}
fn account_error(error: AccountError) -> Status {
    match error {
        AccountError::Busy => Status::resource_exhausted(error.to_string()),
        AccountError::Storage => Status::internal(error.to_string()),
        _ => Status::failed_precondition(error.to_string()),
    }
}

#[tonic::async_trait]
impl ExtensionsService for Extensions {
    async fn list_extensions(
        &self,
        _: Request<ListExtensionsRequest>,
    ) -> Result<Response<ListExtensionsResponse>, Status> {
        Ok(Response::new(ListExtensionsResponse {
            extensions: self.catalog().await?,
        }))
    }
    async fn set_extension_enabled(
        &self,
        request: Request<SetExtensionEnabledRequest>,
    ) -> Result<Response<SetExtensionEnabledResponse>, Status> {
        let request = request.into_inner();
        let mut extension = self
            .catalog()
            .await?
            .into_iter()
            .find(|item| item.id == request.id)
            .ok_or_else(|| Status::not_found("Extension is not installed"))?;
        self.manager()?
            .set_enabled(
                &request.id,
                request.enabled,
                &chrono::Utc::now().to_rfc3339(),
            )
            .await
            .map_err(account_error)?;
        extension.enabled = request.enabled;
        Ok(Response::new(SetExtensionEnabledResponse {
            extension: Some(extension),
        }))
    }
    async fn rescan_extensions(
        &self,
        _: Request<RescanExtensionsRequest>,
    ) -> Result<Response<RescanExtensionsResponse>, Status> {
        self.manager()?.rescan().await.map_err(account_error)?;
        Ok(Response::new(RescanExtensionsResponse {
            extensions: self.catalog().await?,
        }))
    }
}

#[cfg(test)]
mod tests;
