//! The signed-in atproto account.
//!
//! One account for the whole daemon: the same session the atradio
//! write-through and the Rocksky repo reads use, so signing in here turns
//! those on and signing out turns them off. That is a real consequence rather
//! than a side effect, and the sign-out message says so.

use async_graphql::*;
use music_player_storage::account;

/// Who is signed in, as a client needs to render it.
#[derive(SimpleObject, Clone, Debug)]
pub struct Account {
    pub did: String,
    /// Without the leading `@`; whether to show one is the client's choice.
    pub handle: String,
    /// Absent for accounts that set none, where the handle is the name.
    pub display_name: Option<String>,
    /// Absent is normal — plenty of accounts have no avatar, and a client
    /// draws initials instead.
    pub avatar: Option<String>,
}

impl From<account::Account> for Account {
    fn from(account: account::Account) -> Self {
        Self {
            did: account.did,
            handle: account.handle,
            display_name: account.display_name,
            avatar: account.avatar,
        }
    }
}

#[derive(Default)]
pub struct AccountQuery;

#[Object]
impl AccountQuery {
    /// The account signed in right now. `null` means nobody is.
    ///
    /// An `atradio login` at the terminal shows up here too: there is one
    /// session, however it was established.
    async fn account(&self, _ctx: &Context<'_>) -> Option<Account> {
        account::current().await.map(Into::into)
    }
}

#[derive(Default)]
pub struct AccountMutation;

#[Object]
impl AccountMutation {
    /// Sign in with a handle and an app password.
    ///
    /// An app password, not the account password: atproto issues them for
    /// exactly this, they can be revoked one at a time, and the daemon has no
    /// browser to run an OAuth flow in.
    async fn sign_in(
        &self,
        _ctx: &Context<'_>,
        handle: String,
        password: String,
    ) -> Result<Account, Error> {
        account::sign_in(&handle, &password)
            .await
            .map(Into::into)
            .map_err(|e| Error::new(e.to_string()))
    }

    /// Forget the session. Scrobbling and station sync stop with it — they are
    /// the same session, not separate logins.
    async fn sign_out(&self, _ctx: &Context<'_>) -> Result<bool, Error> {
        account::sign_out();
        Ok(true)
    }
}
