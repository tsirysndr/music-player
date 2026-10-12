//! Stable, credential-free native and extension references. A handle is not a playback URL.

use std::{error::Error, fmt};
use url::form_urlencoded;

const NAMESPACE: &str = "mp-source:";
const MAX_HANDLE_BYTES: usize = 16 * 1024;
const EXTENSION_RESOLVER_PREFIX: &str = "extension:";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RemoteIdentity {
    pub server_id: String,
    pub user_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    Item,
    Album,
    Artist,
    Playlist,
    Container,
}

impl ResourceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Item => "item",
            Self::Album => "album",
            Self::Artist => "artist",
            Self::Playlist => "playlist",
            Self::Container => "container",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceRef {
    pub resolver: String,
    pub account_id: String,
    pub remote: RemoteIdentity,
    pub kind: ResourceKind,
    pub item_id: String,
}

/// A persisted extension account/configuration scope, without a claim about
/// remote server or user identity. The host must verify the saved revision and
/// effective configuration before use; parsing alone does not authenticate it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LocalSourceRef {
    pub resolver: String,
    pub account_id: String,
    pub config_revision: u64,
    pub kind: ResourceKind,
    pub item_id: String,
}

/// Shared handle boundary. The wire version selects the identity model, not
/// the installed plugin protocol; neither variant may fall back to the other.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SourceHandle {
    Remote(SourceRef),
    Local(LocalSourceRef),
}

/// Errors deliberately exclude the supplied handle or URL from diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceError {
    NotHandle,
    TooLong,
    UnsupportedVersion,
    InvalidFields,
    UnsupportedResolver,
    InvalidKind,
    InvalidRevision,
    EmptyIdentity,
    NonCanonical,
    ConflictingTrack,
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotHandle => "not a source handle",
            Self::TooLong => "source handle exceeds 16 KiB",
            Self::UnsupportedVersion => "unsupported source handle version",
            Self::InvalidFields => "missing, duplicate or unknown source handle field",
            Self::UnsupportedResolver => "unsupported source resolver",
            Self::InvalidKind => "invalid source resource kind",
            Self::InvalidRevision => "source configuration revision must be between 1 and i64::MAX",
            Self::EmptyIdentity => "empty source identity",
            Self::NonCanonical => "noncanonical source handle",
            Self::ConflictingTrack => "track id and uri have conflicting source identities",
        })
    }
}

impl Error for SourceError {}

impl SourceRef {
    /// Bind a source to an installed plugin identity, independently of its
    /// package version or currently loaded instance. Matches manifest ID syntax.
    pub fn extension_resolver(extension_id: &str) -> Result<String, SourceError> {
        if !valid_extension_id(extension_id) {
            return Err(SourceError::UnsupportedResolver);
        }
        Ok(format!("{EXTENSION_RESOLVER_PREFIX}{extension_id}"))
    }

    /// The plugin owning this reference. This only recognizes the namespace;
    /// availability and saved account binding must be checked by the host.
    pub fn extension_id(&self) -> Option<&str> {
        self.resolver
            .strip_prefix(EXTENSION_RESOLVER_PREFIX)
            .filter(|id| valid_extension_id(id))
    }

    /// Encode public identity fields in their canonical order. Callers must
    /// validate externally supplied fields with `parse` before accepting them.
    pub fn to_handle(&self) -> String {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("resolver", &self.resolver)
            .append_pair("account", &self.account_id)
            .append_pair("server", &self.remote.server_id)
            .append_pair("user", &self.remote.user_id)
            .append_pair("kind", self.kind.as_str())
            .append_pair("id", &self.item_id)
            .finish();
        format!("{NAMESPACE}v1?{query}")
    }

    /// Recognize the reserved namespace even when its version or payload is
    /// invalid. URI scheme casing must not enable a legacy-path fallback.
    pub fn is_handle(value: &str) -> bool {
        value
            .get(..NAMESPACE.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(NAMESPACE))
    }

    pub fn parse(value: &str) -> Result<Self, SourceError> {
        if value.len() > MAX_HANDLE_BYTES {
            return Err(SourceError::TooLong);
        }
        if !Self::is_handle(value) {
            return Err(SourceError::NotHandle);
        }
        let versioned = value
            .strip_prefix(NAMESPACE)
            .ok_or(SourceError::NonCanonical)?;
        let query = versioned
            .strip_prefix("v1?")
            .ok_or(SourceError::UnsupportedVersion)?;
        let mut fields = [None, None, None, None, None, None];
        for (key, value) in form_urlencoded::parse(query.as_bytes()) {
            let index = match key.as_ref() {
                "resolver" => 0,
                "account" => 1,
                "server" => 2,
                "user" => 3,
                "kind" => 4,
                "id" => 5,
                _ => return Err(SourceError::InvalidFields),
            };
            if fields[index].replace(value.into_owned()).is_some() {
                return Err(SourceError::InvalidFields);
            }
        }
        let [Some(resolver), Some(account_id), Some(server_id), Some(user_id), Some(kind), Some(item_id)] =
            fields
        else {
            return Err(SourceError::InvalidFields);
        };
        if [
            &resolver,
            &account_id,
            &server_id,
            &user_id,
            &kind,
            &item_id,
        ]
        .iter()
        .any(|field| field.trim().is_empty())
        {
            return Err(SourceError::EmptyIdentity);
        }
        if resolver != "emby"
            && !resolver
                .strip_prefix(EXTENSION_RESOLVER_PREFIX)
                .is_some_and(valid_extension_id)
        {
            return Err(SourceError::UnsupportedResolver);
        }
        let kind = match kind.as_str() {
            "item" => ResourceKind::Item,
            "album" => ResourceKind::Album,
            "artist" => ResourceKind::Artist,
            "playlist" => ResourceKind::Playlist,
            "container" => ResourceKind::Container,
            _ => return Err(SourceError::InvalidKind),
        };
        let source = Self {
            resolver,
            account_id,
            remote: RemoteIdentity { server_id, user_id },
            kind,
            item_id,
        };
        // Also rejects lossy UTF-8 decoding, malformed percent escapes,
        // alternate escaping, reordered keys, fragments and empty separators.
        if source.to_handle() != value {
            return Err(SourceError::NonCanonical);
        }
        Ok(source)
    }
}

impl LocalSourceRef {
    /// Construct a checked local reference from a plugin ID (without the
    /// `extension:` prefix). No remote identity is synthesized.
    pub fn for_extension(
        extension_id: &str,
        account_id: String,
        config_revision: u64,
        kind: ResourceKind,
        item_id: String,
    ) -> Result<Self, SourceError> {
        let source = Self {
            resolver: SourceRef::extension_resolver(extension_id)?,
            account_id,
            config_revision,
            kind,
            item_id,
        };
        Self::parse(&source.to_handle())
    }

    pub fn extension_id(&self) -> Option<&str> {
        self.resolver
            .strip_prefix(EXTENSION_RESOLVER_PREFIX)
            .filter(|id| valid_extension_id(id))
    }

    /// Encode public fields. As with `SourceRef`, struct literals can contain
    /// invalid fields: validate external values with `parse` before use.
    pub fn to_handle(&self) -> String {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("resolver", &self.resolver)
            .append_pair("account", &self.account_id)
            .append_pair("scope", "local")
            .append_pair("revision", &self.config_revision.to_string())
            .append_pair("kind", self.kind.as_str())
            .append_pair("id", &self.item_id)
            .finish();
        format!("{NAMESPACE}v2?{query}")
    }

    pub fn parse(value: &str) -> Result<Self, SourceError> {
        if value.len() > MAX_HANDLE_BYTES {
            return Err(SourceError::TooLong);
        }
        if !SourceRef::is_handle(value) {
            return Err(SourceError::NotHandle);
        }
        let versioned = value
            .strip_prefix(NAMESPACE)
            .ok_or(SourceError::NonCanonical)?;
        let query = versioned
            .strip_prefix("v2?")
            .ok_or(SourceError::UnsupportedVersion)?;
        let mut fields = [None, None, None, None, None, None];
        for (key, value) in form_urlencoded::parse(query.as_bytes()) {
            let index = match key.as_ref() {
                "resolver" => 0,
                "account" => 1,
                "scope" => 2,
                "revision" => 3,
                "kind" => 4,
                "id" => 5,
                _ => return Err(SourceError::InvalidFields),
            };
            if fields[index].replace(value.into_owned()).is_some() {
                return Err(SourceError::InvalidFields);
            }
        }
        let [Some(resolver), Some(account_id), Some(scope), Some(revision), Some(kind), Some(item_id)] =
            fields
        else {
            return Err(SourceError::InvalidFields);
        };
        if [&resolver, &account_id, &item_id]
            .iter()
            .any(|field| field.trim().is_empty())
        {
            return Err(SourceError::EmptyIdentity);
        }
        if scope != "local" {
            return Err(SourceError::InvalidFields);
        }
        if !resolver
            .strip_prefix(EXTENSION_RESOLVER_PREFIX)
            .is_some_and(valid_extension_id)
        {
            return Err(SourceError::UnsupportedResolver);
        }
        let config_revision = revision
            .parse::<u64>()
            .map_err(|_| SourceError::InvalidRevision)?;
        if config_revision == 0 || config_revision > i64::MAX as u64 {
            return Err(SourceError::InvalidRevision);
        }
        let kind = match kind.as_str() {
            "item" => ResourceKind::Item,
            "album" => ResourceKind::Album,
            "artist" => ResourceKind::Artist,
            "playlist" => ResourceKind::Playlist,
            "container" => ResourceKind::Container,
            _ => return Err(SourceError::InvalidKind),
        };
        let source = Self {
            resolver,
            account_id,
            config_revision,
            kind,
            item_id,
        };
        // Exact re-encoding also rejects alternate revision spellings, lossy
        // UTF-8, malformed/alternate escapes, reordered keys and fragments.
        if source.to_handle() != value {
            return Err(SourceError::NonCanonical);
        }
        Ok(source)
    }
}

impl SourceHandle {
    pub fn parse(value: &str) -> Result<Self, SourceError> {
        if value.starts_with("mp-source:v2?") {
            LocalSourceRef::parse(value).map(Self::Local)
        } else {
            // Preserve the complete remote parser, including its strict
            // errors for unknown versions and noncanonical namespace casing.
            SourceRef::parse(value).map(Self::Remote)
        }
    }

    pub fn to_handle(&self) -> String {
        match self {
            Self::Remote(source) => source.to_handle(),
            Self::Local(source) => source.to_handle(),
        }
    }

    /// Recognizes even malformed and unknown-version reserved handles so
    /// callers can reject them rather than resolve via the browsing provider.
    pub fn is_handle(value: &str) -> bool {
        SourceRef::is_handle(value)
    }

    pub fn extension_id(&self) -> Option<&str> {
        match self {
            Self::Remote(source) => source.extension_id(),
            Self::Local(source) => source.extension_id(),
        }
    }

    pub fn account_id(&self) -> &str {
        match self {
            Self::Remote(source) => &source.account_id,
            Self::Local(source) => &source.account_id,
        }
    }

    pub fn resolver(&self) -> &str {
        match self {
            Self::Remote(source) => &source.resolver,
            Self::Local(source) => &source.resolver,
        }
    }

    pub fn kind(&self) -> ResourceKind {
        match self {
            Self::Remote(source) => source.kind,
            Self::Local(source) => source.kind,
        }
    }

    pub fn item_id(&self) -> &str {
        match self {
            Self::Remote(source) => &source.item_id,
            Self::Local(source) => &source.item_id,
        }
    }
}

impl From<SourceRef> for SourceHandle {
    fn from(source: SourceRef) -> Self {
        Self::Remote(source)
    }
}

impl From<LocalSourceRef> for SourceHandle {
    fn from(source: LocalSourceRef) -> Self {
        Self::Local(source)
    }
}

fn valid_extension_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// Fill only an absent mirror of a valid source handle. Legacy pairs remain
/// untouched; a bare remote id is never promoted into a different identity.
/// Every failure leaves both input strings unchanged.
pub fn normalize_track(
    id: &mut String,
    uri: &mut String,
) -> Result<Option<SourceRef>, SourceError> {
    let source_id = SourceRef::is_handle(id)
        .then(|| SourceRef::parse(id))
        .transpose()?;
    let source_uri = SourceRef::is_handle(uri)
        .then(|| SourceRef::parse(uri))
        .transpose()?;
    match (source_id, source_uri) {
        (None, None) => Ok(None),
        (Some(source), None) if uri.is_empty() => {
            uri.clone_from(id);
            Ok(Some(source))
        }
        (None, Some(source)) if id.is_empty() => {
            id.clone_from(uri);
            Ok(Some(source))
        }
        (Some(source), Some(other)) if source == other => Ok(Some(source)),
        _ => Err(SourceError::ConflictingTrack),
    }
}

/// Normalize either identity model without translating one into the other.
/// Full identity must match, including local configuration revision. Only an
/// empty mirror can be filled; malformed handles and conflicts mutate neither
/// input. Native-only callers can keep using `normalize_track`.
pub fn normalize_source_track(
    id: &mut String,
    uri: &mut String,
) -> Result<Option<SourceHandle>, SourceError> {
    let source_id = SourceHandle::is_handle(id)
        .then(|| SourceHandle::parse(id))
        .transpose()?;
    let source_uri = SourceHandle::is_handle(uri)
        .then(|| SourceHandle::parse(uri))
        .transpose()?;
    match (source_id, source_uri) {
        (None, None) => Ok(None),
        (Some(source), None) if uri.is_empty() => {
            uri.clone_from(id);
            Ok(Some(source))
        }
        (None, Some(source)) if id.is_empty() => {
            id.clone_from(uri);
            Ok(Some(source))
        }
        (Some(source), Some(other)) if source == other => Ok(Some(source)),
        _ => Err(SourceError::ConflictingTrack),
    }
}

#[cfg(test)]
mod handle_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn source(account: &str) -> SourceRef {
        SourceRef {
            resolver: "emby".into(),
            account_id: account.into(),
            remote: RemoteIdentity {
                server_id: "server-1".into(),
                user_id: "user-1".into(),
            },
            kind: ResourceKind::Item,
            item_id: "55508".into(),
        }
    }

    #[test]
    fn extension_identity_roundtrips_without_colliding_with_native_or_other_plugins() {
        let native = source("saved-a");
        let mut first = native.clone();
        first.resolver = SourceRef::extension_resolver("com.example.Emby_2").unwrap();
        let mut second = first.clone();
        second.resolver = SourceRef::extension_resolver("com.example.other").unwrap();
        assert_eq!(first.extension_id(), Some("com.example.Emby_2"));
        assert_eq!(native.extension_id(), None);
        for value in [&native, &first, &second] {
            assert_eq!(SourceRef::parse(&value.to_handle()), Ok(value.clone()));
        }
        let handles: HashSet<_> = [&native, &first, &second]
            .map(SourceRef::to_handle)
            .into_iter()
            .collect();
        assert_eq!(handles.len(), 3);
        let mut id = first.to_handle();
        let mut uri = String::new();
        assert_eq!(normalize_track(&mut id, &mut uri), Ok(Some(first.clone())));
        assert_eq!(id, uri);
        let mut other_uri = second.to_handle();
        let before = (id.clone(), other_uri.clone());
        assert_eq!(
            normalize_track(&mut id, &mut other_uri),
            Err(SourceError::ConflictingTrack)
        );
        assert_eq!((id, other_uri), before);
    }

    #[test]
    fn invalid_extension_namespace_is_not_accepted_as_an_opaque_resolver() {
        for id in ["", "with space", "a/b", "a:b", "雪"] {
            assert_eq!(
                SourceRef::extension_resolver(id),
                Err(SourceError::UnsupportedResolver)
            );
            let mut value = source("saved-a");
            value.resolver = format!("extension:{id}");
            assert_eq!(value.extension_id(), None);
            assert_eq!(
                SourceRef::parse(&value.to_handle()),
                Err(SourceError::UnsupportedResolver)
            );
        }
    }

    #[test]
    fn canonical_handle_round_trips_opaque_identity_and_resource_kinds() {
        let mut value = source("old id&+/%雪");
        value.item_id = "item?x=1#two".into();
        assert_eq!(value.to_handle(), "mp-source:v1?resolver=emby&account=old+id%26%2B%2F%25%E9%9B%AA&server=server-1&user=user-1&kind=item&id=item%3Fx%3D1%23two");
        for kind in [
            ResourceKind::Item,
            ResourceKind::Album,
            ResourceKind::Artist,
            ResourceKind::Playlist,
            ResourceKind::Container,
        ] {
            value.kind = kind;
            assert_eq!(SourceRef::parse(&value.to_handle()), Ok(value.clone()));
        }
    }

    #[test]
    fn same_item_on_two_accounts_and_different_remote_users_stays_distinct() {
        let a = source("saved-a");
        let b = source("saved-b");
        let mut other_user = a.clone();
        other_user.remote.user_id = "user-2".into();
        let mut other_server = a.clone();
        other_server.remote.server_id = "server-2".into();
        let values = [a, b, other_user, other_server];
        assert_eq!(values.iter().cloned().collect::<HashSet<_>>().len(), 4);
        assert_eq!(
            values
                .iter()
                .map(SourceRef::to_handle)
                .collect::<HashSet<_>>()
                .len(),
            4
        );
    }

    #[test]
    fn malformed_reserved_handles_never_become_legacy_tracks() {
        for invalid in [
            "mp-source:",
            "mp-source:v2?resolver=emby",
            "MP-SOURCE:v1?",
            "mp-source:v1?",
        ] {
            assert!(SourceRef::is_handle(invalid));
            let mut id = invalid.to_owned();
            let mut uri = String::new();
            assert!(normalize_track(&mut id, &mut uri).is_err());
            assert_eq!(id, invalid);
            assert!(uri.is_empty());
        }
        assert!(!SourceRef::is_handle(
            "https://example.invalid/mp-source:v1"
        ));
    }

    #[test]
    fn rejects_duplicate_unknown_missing_and_empty_identity_fields() {
        let valid = source("saved-a").to_handle();
        for invalid in [
            format!("{valid}&account=saved-b"),
            format!("{valid}&%61ccount=saved-b"),
            format!("{valid}&api_key=synthetic"),
            valid.replace("&server=server-1", ""),
            valid.replace("account=saved-a", "account="),
            valid.replace("server=server-1", "server="),
            valid.replace("user=user-1", "user=+"),
            valid.replace("id=55508", "id="),
            valid.replace("resolver=emby", "resolver=jellyfin"),
            valid.replace("kind=item", "kind=movie"),
        ] {
            assert!(SourceRef::parse(&invalid).is_err());
        }
    }

    #[test]
    fn rejects_noncanonical_or_lossy_query_decoding() {
        let valid = source("saved-a").to_handle();
        for invalid in [
            valid.replace(
                "resolver=emby&account=saved-a",
                "account=saved-a&resolver=emby",
            ),
            valid.replace("id=55508", "id=%35%35%35%30%38"),
            valid.replace("id=55508", "id=%FF"),
            valid.replace("id=55508", "id=%zz"),
            format!("{valid}#fragment"),
            format!("{valid}&"),
        ] {
            assert_eq!(SourceRef::parse(&invalid), Err(SourceError::NonCanonical));
        }
    }

    #[test]
    fn encoded_handle_size_is_bounded_in_bytes() {
        let mut value = source("saved-a");
        value.item_id.clear();
        let overhead = value.to_handle().len();
        value.item_id = "x".repeat(MAX_HANDLE_BYTES - overhead);
        assert_eq!(SourceRef::parse(&value.to_handle()), Ok(value.clone()));
        value.item_id.push('x');
        assert_eq!(
            SourceRef::parse(&value.to_handle()),
            Err(SourceError::TooLong)
        );
    }

    #[test]
    fn fills_only_missing_handle_mirrors_and_preserves_legacy_tracks() {
        let source = source("saved-a");
        let handle = source.to_handle();
        for (mut id, mut uri) in [
            (handle.clone(), String::new()),
            (String::new(), handle.clone()),
            (handle.clone(), handle.clone()),
        ] {
            assert_eq!(normalize_track(&mut id, &mut uri), Ok(Some(source.clone())));
            assert_eq!(id, handle);
            assert_eq!(uri, handle);
        }
        let mut id = "local-track".to_owned();
        let mut uri = "/music/track.mp3".to_owned();
        assert_eq!(normalize_track(&mut id, &mut uri), Ok(None));
        assert_eq!(id, "local-track");
        assert_eq!(uri, "/music/track.mp3");
    }

    #[test]
    fn conflicting_inputs_are_rejected_without_partial_mutation() {
        let a = source("saved-a").to_handle();
        let b = source("saved-b").to_handle();
        for (mut id, mut uri) in [
            (a.clone(), b),
            ("55508".into(), a.clone()),
            (a.clone(), "55508".into()),
            (a.clone(), "https://example.invalid/stream".into()),
            (a, "mp-source:v2?".into()),
        ] {
            let before = (id.clone(), uri.clone());
            assert!(normalize_track(&mut id, &mut uri).is_err());
            assert_eq!((id, uri), before);
        }
    }
}
