//! State for the TUI's server switcher.
//!
//! An fzf-style overlay over the saved servers, opened with `C`. Picking one
//! repoints every library screen at it; picking "This machine" goes back to
//! the daemon's own files. Neither interrupts playback — a server is where the
//! library is *read from*, which is a different question from where the audio
//! comes out.
//!
//! It follows the fuzzy finder's pattern (a flag on `App` that swallows keys
//! while it is up) rather than introducing a second modal mechanism.

use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher, Utf32Str,
};

/// One saved server, as the daemon reports it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServerEntry {
    /// Empty for the local library, which is always the first row.
    pub id: String,
    pub kind: String,
    pub name: String,
    pub url: String,
    pub connected: bool,
}

impl ServerEntry {
    /// The row this machine is always offered as.
    pub fn local(connected: bool) -> Self {
        Self {
            id: String::new(),
            kind: "local".into(),
            name: "This machine".into(),
            url: "the daemon's own library".into(),
            connected,
        }
    }

    /// What the fuzzy matcher sees, and what the row shows.
    pub fn display(&self) -> String {
        format!("{}  —  {}", self.name, self.url)
    }

    pub fn is_local(&self) -> bool {
        self.id.is_empty()
    }
}

/// A matched row, with the character positions to highlight.
#[derive(Clone, Debug, Default)]
pub struct SwitcherResult {
    pub entry: ServerEntry,
    pub display: String,
    pub indices: Vec<u32>,
}

/// Which field the add-server form is on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AddField {
    #[default]
    Kind,
    Name,
    Url,
    Username,
    Password,
}

impl AddField {
    pub const ALL: [AddField; 5] = [
        AddField::Kind,
        AddField::Name,
        AddField::Url,
        AddField::Username,
        AddField::Password,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AddField::Kind => "Type",
            AddField::Name => "Name",
            AddField::Url => "URL",
            AddField::Username => "Username",
            AddField::Password => "Password",
        }
    }

    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|f| *f == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> Self {
        let index = Self::ALL.iter().position(|f| *f == self).unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// The "add a server" form, reachable with Ctrl-n from the switcher.
#[derive(Clone, Debug, Default)]
pub struct AddServerForm {
    pub active: bool,
    pub focus: AddField,
    /// Index into [`ServerSwitcher::kinds`].
    pub kind: usize,
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub submitting: bool,
    /// Why the daemon refused. Empty when it did not.
    pub error: String,
}

impl AddServerForm {
    fn field_mut(&mut self) -> Option<&mut String> {
        match self.focus {
            AddField::Kind => None,
            AddField::Name => Some(&mut self.name),
            AddField::Url => Some(&mut self.url),
            AddField::Username => Some(&mut self.username),
            AddField::Password => Some(&mut self.password),
        }
    }

    pub fn value(&self, field: AddField) -> &str {
        match field {
            AddField::Kind => "",
            AddField::Name => &self.name,
            AddField::Url => &self.url,
            AddField::Username => &self.username,
            AddField::Password => &self.password,
        }
    }

    pub fn push(&mut self, c: char) {
        self.error.clear();
        if let Some(field) = self.field_mut() {
            field.push(c);
        }
    }

    pub fn backspace(&mut self) {
        self.error.clear();
        if let Some(field) = self.field_mut() {
            field.pop();
        }
    }

    pub fn clear_field(&mut self) {
        self.error.clear();
        if let Some(field) = self.field_mut() {
            field.clear();
        }
    }

    /// A url is the one thing a server cannot be saved without.
    pub fn can_submit(&self) -> bool {
        !self.url.trim().is_empty() && !self.submitting
    }
}

/// The switcher overlay. `active` is what makes the key handler take over.
pub struct ServerSwitcher {
    pub active: bool,
    pub query: String,
    pub selected_index: usize,
    /// Every saved server, plus the local row at the front.
    pub entries: Vec<ServerEntry>,
    pub results: Vec<SwitcherResult>,
    /// The kinds the daemon supports, as (key, label).
    pub kinds: Vec<(String, String)>,
    pub loading: bool,
    pub error: String,
    pub form: AddServerForm,
    matcher: Matcher,
}

impl Default for ServerSwitcher {
    fn default() -> Self {
        Self {
            active: false,
            query: String::new(),
            selected_index: 0,
            entries: vec![],
            results: vec![],
            kinds: vec![],
            loading: false,
            error: String::new(),
            form: AddServerForm::default(),
            matcher: Matcher::new(Config::DEFAULT),
        }
    }
}

impl ServerSwitcher {
    pub fn open(&mut self) {
        self.active = true;
        self.query.clear();
        self.selected_index = 0;
        self.error.clear();
        self.form = AddServerForm::default();
        self.refresh();
    }

    pub fn close(&mut self) {
        self.active = false;
        self.form.active = false;
    }

    /// Replace the list with what the daemon has, keeping the local row first.
    pub fn set_servers(&mut self, servers: Vec<ServerEntry>) {
        let anything_connected = servers.iter().any(|server| server.connected);
        let mut entries = vec![ServerEntry::local(!anything_connected)];
        entries.extend(servers);
        self.entries = entries;
        self.loading = false;
        self.refresh();
    }

    pub fn selected(&self) -> Option<&ServerEntry> {
        self.results
            .get(
                self.selected_index
                    .min(self.results.len().saturating_sub(1)),
            )
            .map(|result| &result.entry)
    }

    pub fn move_down(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.results.len() - 1);
        }
    }

    pub fn move_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Re-rank the rows for the current query, best first.
    pub fn refresh(&mut self) {
        if self.query.is_empty() {
            self.results = self
                .entries
                .iter()
                .map(|entry| SwitcherResult {
                    display: entry.display(),
                    entry: entry.clone(),
                    indices: vec![],
                })
                .collect();
        } else {
            let pattern = Pattern::parse(&self.query, CaseMatching::Ignore, Normalization::Smart);
            let mut buf = Vec::new();
            let mut matched: Vec<(u32, SwitcherResult)> = Vec::new();
            for entry in &self.entries {
                let display = entry.display();
                let haystack = Utf32Str::new(&display, &mut buf);
                let mut indices = Vec::new();
                if let Some(score) = pattern.indices(haystack, &mut self.matcher, &mut indices) {
                    indices.sort_unstable();
                    indices.dedup();
                    matched.push((
                        score,
                        SwitcherResult {
                            entry: entry.clone(),
                            display,
                            indices,
                        },
                    ));
                }
            }
            matched.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
            self.results = matched.into_iter().map(|(_, result)| result).collect();
        }
        self.selected_index = self
            .selected_index
            .min(self.results.len().saturating_sub(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(name: &str, url: &str, connected: bool) -> ServerEntry {
        ServerEntry {
            id: format!("id-{name}"),
            kind: "subsonic".into(),
            name: name.into(),
            url: url.into(),
            connected,
        }
    }

    fn switcher() -> ServerSwitcher {
        let mut switcher = ServerSwitcher::default();
        switcher.open();
        switcher.set_servers(vec![
            server("Living room NAS", "http://nas.lan:4533", false),
            server("Jellyfin", "http://media.lan:8096", false),
        ]);
        switcher
    }

    /// There is always somewhere to read from, so the local row leads.
    #[test]
    fn the_local_library_is_always_offered_first() {
        let switcher = switcher();
        assert!(switcher.entries[0].is_local());
        assert_eq!(switcher.results.len(), 3);
    }

    /// Nothing connected *is* the local library being connected.
    #[test]
    fn the_local_row_is_connected_when_nothing_else_is() {
        let mut switcher = switcher();
        assert!(switcher.entries[0].connected);

        switcher.set_servers(vec![
            server("Living room NAS", "http://nas.lan:4533", true),
            server("Jellyfin", "http://media.lan:8096", false),
        ]);
        assert!(!switcher.entries[0].connected);
    }

    #[test]
    fn filters_fuzzily_over_name_and_url() {
        let mut switcher = switcher();
        switcher.query = "jelly".into();
        switcher.refresh();
        assert_eq!(switcher.results.len(), 1);
        assert_eq!(switcher.results[0].entry.name, "Jellyfin");

        // The url is searchable too — a server is often known by its address.
        switcher.query = "nas.lan".into();
        switcher.refresh();
        assert_eq!(switcher.results[0].entry.name, "Living room NAS");
    }

    #[test]
    fn a_query_matching_nothing_leaves_no_selection_to_activate() {
        let mut switcher = switcher();
        switcher.query = "zzzzz".into();
        switcher.refresh();
        assert!(switcher.results.is_empty());
        assert!(switcher.selected().is_none());
    }

    #[test]
    fn selection_stays_inside_the_list() {
        let mut switcher = switcher();
        for _ in 0..10 {
            switcher.move_down();
        }
        assert_eq!(switcher.selected_index, 2);
        for _ in 0..10 {
            switcher.move_up();
        }
        assert_eq!(switcher.selected_index, 0);
    }

    /// Filtering must not leave the cursor pointing past the end.
    #[test]
    fn narrowing_the_query_pulls_the_selection_back() {
        let mut switcher = switcher();
        switcher.move_down();
        switcher.move_down();
        assert_eq!(switcher.selected_index, 2);

        switcher.query = "jelly".into();
        switcher.refresh();
        assert_eq!(switcher.selected_index, 0);
        assert_eq!(switcher.selected().unwrap().name, "Jellyfin");
    }

    #[test]
    fn the_form_types_into_the_focused_field() {
        let mut form = AddServerForm {
            focus: AddField::Name,
            ..Default::default()
        };
        for c in "NAS".chars() {
            form.push(c);
        }
        assert_eq!(form.name, "NAS");

        form.focus = AddField::Url;
        for c in "http://nas".chars() {
            form.push(c);
        }
        assert_eq!(form.url, "http://nas");
        assert_eq!(form.name, "NAS", "the other fields are untouched");

        form.backspace();
        assert_eq!(form.url, "http://na");
        form.clear_field();
        assert!(form.url.is_empty());
    }

    /// The kind is a choice, not a text field; typing must not corrupt it.
    #[test]
    fn typing_on_the_kind_field_does_nothing() {
        let mut form = AddServerForm::default();
        form.push('x');
        assert_eq!(form.kind, 0);
        assert!(form.name.is_empty());
    }

    #[test]
    fn a_url_is_required_to_submit() {
        let mut form = AddServerForm::default();
        assert!(!form.can_submit());
        form.focus = AddField::Url;
        form.push('h');
        assert!(form.can_submit());
        // Not while one is already in flight.
        form.submitting = true;
        assert!(!form.can_submit());
    }

    #[test]
    fn tab_cycles_through_every_form_field() {
        let mut field = AddField::Kind;
        for _ in 0..AddField::ALL.len() {
            field = field.next();
        }
        assert_eq!(field, AddField::Kind);
        assert_eq!(AddField::Kind.previous(), AddField::Password);
    }

    /// Reopening must not show the last attempt.
    #[test]
    fn opening_resets_the_query_and_the_form() {
        let mut switcher = switcher();
        switcher.query = "jelly".into();
        switcher.form.active = true;
        switcher.form.url = "http://old".into();
        switcher.error = "stale".into();

        switcher.open();
        assert!(switcher.query.is_empty());
        assert!(!switcher.form.active);
        assert!(switcher.form.url.is_empty());
        assert!(switcher.error.is_empty());
    }
}
