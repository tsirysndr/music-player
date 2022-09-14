use std::sync::Arc;

use music_player_storage::Database;

use crate::api::v1alpha1::{
    addons_service_server::AddonsService, GetAddonDetailsRequest, GetAddonDetailsResponse,
    GetAddonsRequest, GetAddonsResponse, SetAddonEnabledRequest, SetAddonEnabledResponse,
};
use crate::objects::v1alpha1::Addon;

pub struct Addons {
    db: Arc<Database>,
}

impl Addons {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl AddonsService for Addons {
    async fn get_addons(
        &self,
        _request: tonic::Request<GetAddonsRequest>,
    ) -> Result<tonic::Response<GetAddonsResponse>, tonic::Status> {
        let response = GetAddonsResponse { addons: vec![] };
        Ok(tonic::Response::new(response))
    }
    async fn get_addon_details(
        &self,
        _request: tonic::Request<GetAddonDetailsRequest>,
    ) -> Result<tonic::Response<GetAddonDetailsResponse>, tonic::Status> {
        let response = GetAddonDetailsResponse {
            addon: Some(Addon::default()),
        };
        Ok(tonic::Response::new(response))
    }
    async fn set_addon_enabled(
        &self,
        _request: tonic::Request<SetAddonEnabledRequest>,
    ) -> Result<tonic::Response<SetAddonEnabledResponse>, tonic::Status> {
        let response = SetAddonEnabledResponse {};
        Ok(tonic::Response::new(response))
    }
}
