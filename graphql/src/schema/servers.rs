//! Saved servers, and which one the library is reading from.
//!
//! Connecting is deliberately dull: it makes a server the current provider and
//! nothing else. It does not navigate anywhere, and — this is the point — it
//! cannot interrupt playback. A provider is where the *screens* read from;
//! where the audio comes out is the receiver, a different trait behind
//! different state, and nothing here can reach it.

use async_graphql::*;
use music_player_provider::ProviderConfig;
use music_player_storage::{
    saved_servers::{self, NewServer},
    Database,
};
use super::objects::server::{Server, ServerInput, SourceKind};
use super::provider;

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
        let db = ctx.data::<Database>().unwrap();
        // A discovered peer has no saved row, so fall back to the live config.
        if let Ok(Some(row)) = saved_servers::get(db.get_connection(), &config.id).await {
            return Ok(Some(Server::from_row(row, Some(config.id.as_str()))));
        }
        Ok(Some(Server {
            id: ID(config.id.clone()),
            kind: config.kind.clone(),
            name: config.name.clone(),
            url: config.url.clone(),
            username: config.username.clone(),
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
    /// Save a server, or update the one already stored at that url.
    async fn add_server(&self, ctx: &Context<'_>, input: ServerInput) -> Result<Server, Error> {
        let db = ctx.data::<Database>().unwrap();
        if input.url.trim().is_empty() {
            return Err(Error::new("a server needs a url"));
        }
        if provider::state(ctx).registry().get(&input.kind).is_none() {
            return Err(Error::new(format!("unknown kind of server: {}", input.kind)));
        }

        let server = NewServer::new(input.kind, input.name, input.url)
            .with_credentials(input.username, input.password);
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
        provider::state(ctx)
            .connect(config)
            .await
            .map_err(provider::err)?;

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
