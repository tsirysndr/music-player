//! The signed-in atproto account.
//!
//! Deliberately built on the *same* session the atradio write-through uses,
//! rather than a session of its own. Signing in here writes that shared
//! session file, so scrobbling and station sync start working without a second
//! login — and signing out stops them, which is the honest consequence of the
//! same fact.
//!
//! Password login, not OAuth: the daemon has no browser to hand off to and no
//! callback to receive one on. An app password is what atproto offers for
//! exactly this case, and it is what the `atradio` CLI already uses.

use anyhow::Error;
use serde::{Deserialize, Serialize};

use crate::atradio;

/// Who is signed in, as a client needs to render it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub did: String,
    /// Without the leading `@` — clients add it, because whether it is shown
    /// is a presentation choice.
    pub handle: String,
    /// What the user calls themselves. Absent for accounts that set none, in
    /// which case the handle is the name.
    pub display_name: Option<String>,
    /// The avatar's url, from the account's Bluesky profile record. Absent is
    /// normal: plenty of accounts have none, and clients draw initials.
    pub avatar: Option<String>,
}

impl Account {
    /// What to show as the name: the display name when there is one, else the
    /// handle, which is always present.
    pub fn label(&self) -> &str {
        self.display_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(&self.handle)
    }
}

/// The account signed in right now, if any.
///
/// Reads the shared session, so an `atradio login` at the terminal shows up
/// here too — one account, however it was established.
pub async fn current() -> Option<Account> {
    // A daemon configured by environment variables has credentials but no
    // session until something uses them. Establishing one here is what makes
    // the UI show a CLI-configured account as signed in, which it is.
    if atradio::profile().is_none() {
        atradio::ensure_session().await;
    }
    let profile = atradio::profile()?;
    let avatar = fetch_avatar(&profile.did).await;
    Some(Account {
        did: profile.did,
        handle: profile.handle,
        display_name: profile.display_name,
        avatar,
    })
}

/// Sign in with a handle and an app password.
///
/// The handle is accepted with or without a leading `@`: it is displayed with
/// one everywhere, so it is the natural thing to type, and rejecting it would
/// be pedantry.
pub async fn sign_in(handle: &str, password: &str) -> Result<Account, Error> {
    let identifier = handle.trim().trim_start_matches('@');
    if identifier.is_empty() {
        return Err(Error::msg("enter your handle"));
    }
    if password.is_empty() {
        return Err(Error::msg("enter your app password"));
    }

    let profile = atradio::sign_in(identifier, password).await?;
    let avatar = fetch_avatar(&profile.did).await;

    // The account's repo is now readable, so pull it in — likes, and whatever
    // else follows the same identity. Detached: signing in should return as
    // soon as the credentials are accepted, not wait on a repo that may be
    // large, and a sync failure is not a failed sign-in.
    tokio::spawn(async {
        let db = crate::shared().await;
        crate::rocksky_likes::sync(db.get_connection().clone()).await;
    });

    Ok(Account {
        did: profile.did,
        handle: profile.handle,
        display_name: profile.display_name,
        avatar,
    })
}

/// Forget the session. Also ends atradio write-through, which is the same
/// session — said plainly here because it is not obvious from the button.
pub fn sign_out() {
    atradio::sign_out();
}

/// The account's avatar, from its public Bluesky profile.
///
/// Public, so it needs no session and cannot fail the sign-in: an account
/// without an avatar, or an appview that is down, gives `None` and the client
/// draws initials instead.
async fn fetch_avatar(did: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct ProfileView {
        avatar: Option<String>,
    }

    let response = super::atproto::http()
        .ok()?
        .get("https://public.api.bsky.app/xrpc/app.bsky.actor.getProfile")
        .query(&[("actor", did)])
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    response.json::<ProfileView>().await.ok()?.avatar
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name falls back to the handle, which every account has.
    #[test]
    fn the_label_prefers_a_display_name() {
        let mut account = Account {
            handle: "tsiry.rocksky.app".into(),
            ..Default::default()
        };
        assert_eq!(account.label(), "tsiry.rocksky.app");

        account.display_name = Some("Tsiry".into());
        assert_eq!(account.label(), "Tsiry");

        // A display name of spaces is not a name.
        account.display_name = Some("   ".into());
        assert_eq!(account.label(), "tsiry.rocksky.app");
    }
}
