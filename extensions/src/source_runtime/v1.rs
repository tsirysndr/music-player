//! Legacy calls share the v2 scheduler budgets, disable state and cancellation
//! boundary. V1 has no remote release export: its lease retains only the local
//! instance/reservation, and must never be described as server session cleanup.
use super::*;
use crate::abi as legacy;

pub struct LegacySourceLease {
    _permit: Permit,
}
pub struct ResolvedLegacySource {
    pub response: legacy::StreamResponse,
    pub lease: LegacySourceLease,
}

impl SourceRuntime {
    pub async fn source_info_v1(&self) -> Result<legacy::SourceInfo, RuntimeError> {
        self.call(|ext| {
            let info: legacy::SourceInfo = ext
                .call_source_v1("source_info", &())?
                .ok_or_else(|| anyhow::anyhow!("missing source v1 description"))?;
            if info.name.trim().is_empty() {
                return Err(anyhow::anyhow!("empty source v1 name"));
            }
            Ok(info)
        })
        .await
    }

    async fn browse_v1<T: serde::de::DeserializeOwned + Send + 'static>(
        &self,
        function: &'static str,
        request: legacy::BrowseRequest,
    ) -> Result<Vec<T>, RuntimeError> {
        if request.limit == 0 {
            return Err(RuntimeError::InvalidRequest);
        }
        self.call(move |ext| {
            let items: Vec<T> = ext.call_source_v1(function, &request)?.unwrap_or_default();
            if items.len() > request.limit as usize {
                return Err(anyhow::anyhow!("source v1 page exceeds requested limit"));
            }
            Ok(items)
        })
        .await
    }
    pub async fn list_albums_v1(
        &self,
        request: legacy::BrowseRequest,
    ) -> Result<Vec<legacy::SourceAlbum>, RuntimeError> {
        self.browse_v1("list_albums", request).await
    }
    pub async fn list_artists_v1(
        &self,
        request: legacy::BrowseRequest,
    ) -> Result<Vec<legacy::SourceArtist>, RuntimeError> {
        self.browse_v1("list_artists", request).await
    }
    pub async fn list_tracks_v1(
        &self,
        request: legacy::BrowseRequest,
    ) -> Result<Vec<legacy::TrackInfo>, RuntimeError> {
        self.browse_v1("list_tracks", request).await
    }
    pub async fn list_playlists_v1(
        &self,
        request: legacy::BrowseRequest,
    ) -> Result<Vec<legacy::SourcePlaylist>, RuntimeError> {
        self.browse_v1("list_playlists", request).await
    }

    pub async fn resolve_audio_v1(
        &self,
        request: legacy::StreamRequest,
    ) -> Result<ResolvedLegacySource, RuntimeError> {
        if request.track_id.trim().is_empty() {
            return Err(RuntimeError::InvalidRequest);
        }
        let permit = self.reserve(None)?;
        let core = self.core.clone();
        let (tx, rx) = oneshot::channel();
        self.enqueue(Box::new(move || {
            let result = invoke(&core, &tx, |ext| {
                ext.call_source_v1("get_stream_url", &request)?
                    .ok_or_else(|| anyhow::anyhow!("missing source v1 stream response"))
            });
            match result {
                Ok(response) => {
                    #[cfg(test)]
                    if let Some(hook) = core.before_resolve_delivery.lock().unwrap().take() {
                        hook();
                    }
                    let _ = tx.send(Ok(ResolvedLegacySource {
                        response,
                        lease: LegacySourceLease { _permit: permit },
                    }));
                }
                Err(failure) => {
                    // No v1 remote release protocol exists. Do not call v2 or
                    // pretend a timed-out legacy resolve has remote cleanup.
                    drop(permit);
                    let _ = tx.send(Err(failure.error));
                }
            }
        }));
        rx.await.unwrap_or(Err(RuntimeError::Unavailable))
    }
}

#[cfg(test)]
mod tests;
