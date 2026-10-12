//! Saved servers, and which one the library is reading from.
//!
//! Connecting is deliberately dull: it makes a server the current provider and
//! nothing else. It does not navigate anywhere, and — this is the point — it
//! cannot interrupt playback. A provider is where the *screens* read from;
//! where the audio comes out is the receiver, a different trait behind
//! different state, and nothing here can reach it.

use super::objects::server::{Server, ServerInput, SourceKind};
use super::provider;
use async_graphql::*;
use music_player_provider::{ProviderConfig, ProviderError};
use music_player_storage::{
    saved_servers::{self, NewServer, PasswordUpdate},
    Database,
};

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Default)]
pub struct ServersQuery;

#[Object]
impl ServersQuery {
    /// Every saved server, with the connected one flagged.
    async fn saved_servers(&self, ctx: &Context<'_>) -> Result<Vec<Server>, Error> {
        let db = ctx.data::<Database>().unwrap();
        let connected = provider::state(ctx).config().await;
        let rows = saved_servers::list(db.get_connection())
            .await
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|row| Server::from_row(row, connected.as_ref().map(|c| c.id.as_str())))
            .collect())
    }

    async fn saved_server(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Server>, Error> {
        let db = ctx.data::<Database>().unwrap();
        let connected = provider::state(ctx).config().await;
        Ok(saved_servers::get(db.get_connection(), &id)
            .await
            .map_err(|e| Error::new(e.to_string()))?
            .map(|row| Server::from_row(row, connected.as_ref().map(|c| c.id.as_str()))))
    }

    /// The server the library screens are currently reading from, if any.
    /// `null` means the local library.
    async fn connected_server(&self, ctx: &Context<'_>) -> Result<Option<Server>, Error> {
        let Some(config) = provider::state(ctx).config().await else {
            return Ok(None);
        };
        // Saving edits does not rebuild the connection. Its actual config may
        // differ from the saved row until the caller reconnects.
        Ok(Some(Server {
            id: ID(config.id),
            kind: config.kind,
            name: config.name,
            url: config.url,
            username: config.username,
            has_password: config.password.is_some(),
            connected: true,
        }))
    }

    /// The kinds of server this build can talk to, straight from the registry.
    async fn source_kinds(&self, ctx: &Context<'_>) -> Result<Vec<SourceKind>, Error> {
        Ok(provider::state(ctx)
            .registry()
            .describe()
            .into_iter()
            .map(Into::into)
            .collect())
    }
}

#[derive(Default)]
pub struct ServersMutation;

#[Object]
impl ServersMutation {
    /// Save by kind/url/username, or edit the exact account named by input.id.
    async fn add_server(&self, ctx: &Context<'_>, input: ServerInput) -> Result<Server, Error> {
        let db = ctx.data::<Database>().unwrap();
        let factory = provider::state(ctx)
            .registry()
            .get(&input.kind)
            .ok_or_else(|| Error::new(format!("unknown kind of server: {}", input.kind)))?;

        // A hosted backend has one address, and it is the factory's — not
        // whatever a client happened to send. Its form has no url field, so
        // demanding one here is what made Rocksky unusable.
        let url = match factory.fixed_url() {
            Some(fixed) => fixed.to_string(),
            None if input.url.trim().is_empty() => return Err(Error::new("a server needs a url")),
            None => input.url.clone(),
        };

        let password_update =
            PasswordUpdate::from_fields(input.password, input.password_value, input.clear_password)
                .map_err(|e| Error::new(e.to_string()))?;
        let server = NewServer::new(input.kind, input.name, url)
            .with_credentials(input.username, None)
            .with_id(input.id.map(|id| id.to_string()))
            .with_password_update(password_update);
        let row = saved_servers::upsert(db.get_connection(), &server, &now())
            .await
            .map_err(|e| Error::new(e.to_string()))?;
        let connected = provider::state(ctx).config().await;
        Ok(Server::from_row(
            row,
            connected.as_ref().map(|c| c.id.as_str()),
        ))
    }

    /// Forget a server. Disconnects first if it is the one in use, so the
    /// screens fall back to the local library rather than reading from
    /// something that is no longer listed.
    async fn delete_server(&self, ctx: &Context<'_>, id: ID) -> Result<bool, Error> {
        let db = ctx.data::<Database>().unwrap();
        let state = provider::state(ctx);
        if state.config().await.map(|c| c.id) == Some(id.to_string()) {
            state.disconnect().await;
        }
        saved_servers::delete(db.get_connection(), &id)
            .await
            .map_err(|e| Error::new(e.to_string()))
    }

    /// Point the library screens at a saved server.
    ///
    /// Connects first and swaps second, so a server that is unreachable leaves
    /// the previous one in place. Playback is untouched either way.
    async fn connect_to_server(&self, ctx: &Context<'_>, id: ID) -> Result<Server, Error> {
        let db = ctx.data::<Database>().unwrap();
        let row = saved_servers::get(db.get_connection(), &id)
            .await
            .map_err(|e| Error::new(e.to_string()))?
            .ok_or_else(|| Error::new("no such server"))?;

        let config = ProviderConfig {
            id: row.id.clone(),
            kind: row.kind.clone(),
            name: row.name.clone(),
            url: row.url.clone(),
            username: row.username.clone(),
            password: row.password.clone(),
        };
        let snapshot = &row;
        let connected = provider::state(ctx)
            .connect_checked(config, |identity| async move {
                match identity {
                    Some(remote) => {
                        saved_servers::bind_remote_identity(db.get_connection(), snapshot, &remote)
                            .await
                            .map_err(ProviderError::other)?;
                        Ok(())
                    }
                    None if snapshot.kind == "emby" => Err(ProviderError::Other(
                        "Emby did not confirm the remote account identity".into(),
                    )),
                    None => Ok(()),
                }
            })
            .await
            .map_err(provider::err)?;
        if connected.provider.capabilities().liked {
            provider::restamp_queue_likes(ctx, connected);
        }

        Ok(Server::from_row(row, Some(id.as_str())))
    }

    /// Back to the local library. Returns what was disconnected.
    async fn disconnect_from_server(&self, ctx: &Context<'_>) -> Result<Option<Server>, Error> {
        let Some(config) = provider::state(ctx).disconnect().await else {
            return Ok(None);
        };
        Ok(Some(Server {
            id: ID(config.id),
            kind: config.kind,
            name: config.name,
            url: config.url,
            username: config.username,
            has_password: config.password.is_some(),
            connected: false,
        }))
    }
}
