//! The legacy ABI has four lists rather than a single media-browser export.
//! Preserve that choice explicitly; do not invent remote category IDs.
use super::*;
use crate::extension_accounts::AccountRuntime;
use music_player_extensions::abi::TrackInfo;
use music_player_provider::MediaEntry;
use music_player_storage::extension_source_accounts::AccountProtocol;
use music_player_types::source::LocalSourceRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegacySourceList {
    Tracks,
    Albums,
    Artists,
    Playlists,
}

impl SourceResolver {
    pub async fn extension_legacy_media(
        &self,
        account_id: &str,
        list: LegacySourceList,
        parent: Option<&LocalSourceRef>,
        query: Option<String>,
        offset: u64,
        limit: u32,
    ) -> Result<SourceMediaPage, ProviderError> {
        tokio::time::timeout(RESOLVE_TIMEOUT, async {
            if limit == 0 || offset > u64::from(u32::MAX) {
                return Err(failure("invalid source v1 browse range"));
            }
            if parent.is_some_and(|p| p.account_id != account_id || p.kind == ResourceKind::Item) {
                return Err(failure("invalid extension browse parent"));
            }
            let accounts = &self.extension_dispatch()?.accounts;
            let runtime = match parent {
                Some(parent) => accounts.for_local_source(parent).await,
                None => accounts.for_account(account_id).await,
            }
            .map_err(extension_failure)?;
            if runtime.protocol() != AccountProtocol::V1 {
                return Err(failure("source account does not use the v1 list protocol"));
            }
            let items = match list {
                LegacySourceList::Tracks => runtime
                    .legacy_tracks(parent, query, offset, limit)
                    .await
                    .map_err(extension_failure)?
                    .into_iter()
                    .map(|item| legacy_track(&runtime, item))
                    .collect::<Result<Vec<_>, _>>()?,
                LegacySourceList::Albums => runtime
                    .legacy_albums(parent, query, offset, limit)
                    .await
                    .map_err(extension_failure)?
                    .into_iter()
                    .map(|item| container(&runtime, ResourceKind::Album, item.id, item.title))
                    .collect::<Result<Vec<_>, _>>()?,
                LegacySourceList::Artists => runtime
                    .legacy_artists(parent, query, offset, limit)
                    .await
                    .map_err(extension_failure)?
                    .into_iter()
                    .map(|item| container(&runtime, ResourceKind::Artist, item.id, item.name))
                    .collect::<Result<Vec<_>, _>>()?,
                LegacySourceList::Playlists => runtime
                    .legacy_playlists(parent, query, offset, limit)
                    .await
                    .map_err(extension_failure)?
                    .into_iter()
                    .map(|item| container(&runtime, ResourceKind::Playlist, item.id, item.name))
                    .collect::<Result<Vec<_>, _>>()?,
            };
            // V1 returns neither a total nor an explicit continuation. A full
            // page may require one final empty request, never a fixed-size cap.
            let next_offset = if items.len() == limit as usize {
                let next = offset + u64::from(limit);
                if next > u64::from(u32::MAX) {
                    return Err(failure(
                        "source v1 browse offset exceeds its protocol range",
                    ));
                }
                Some(next)
            } else {
                None
            };
            Ok(SourceMediaPage { items, next_offset })
        })
        .await
        .map_err(|_| failure("saved source browse timed out"))?
    }
}

fn container(
    runtime: &AccountRuntime,
    kind: ResourceKind,
    item_id: String,
    title: String,
) -> Result<MediaEntry, ProviderError> {
    Ok(MediaEntry {
        id: runtime
            .source_handle(kind, item_id)
            .map_err(extension_failure)?
            .to_handle(),
        title,
        item_type: Some(
            match kind {
                ResourceKind::Album => "Album",
                ResourceKind::Artist => "Artist",
                ResourceKind::Playlist => "Playlist",
                _ => return Err(failure("invalid source v1 container kind")),
            }
            .into(),
        ),
        media_type: None,
        is_container: true,
        season_number: None,
        episode_number: None,
        track: None,
    })
}

fn legacy_track(runtime: &AccountRuntime, item: TrackInfo) -> Result<MediaEntry, ProviderError> {
    let source = runtime
        .source_handle(ResourceKind::Item, item.id.clone())
        .map_err(extension_failure)?;
    let SourceHandle::Local(source) = source else {
        return Err(failure("source account does not use the v1 list protocol"));
    };
    let track = source_extension_v1::browse_track(&source, item)?;
    Ok(MediaEntry {
        id: track.id.clone(),
        title: track.title.clone(),
        // V1 provides tags, not Emby item/media types or episode numbering.
        item_type: None,
        media_type: None,
        is_container: false,
        season_number: None,
        episode_number: None,
        track: Some(track),
    })
}
