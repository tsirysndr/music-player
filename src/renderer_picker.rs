//! State for the TUI's "play to" picker.
//!
//! An fzf-style overlay over the renderers the daemon can see — Chromecasts,
//! UPnP/DLNA renderers and other music-player instances — opened with `P`.
//!
//! A *renderer* is a sink: where the audio comes out. That is the opposite
//! question from the [server switcher](crate::server_switcher), which is where
//! the library is read *from*. Keeping the two overlays separate is the point;
//! conflating them is what made the web client's status row open the wrong one.

use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher, Utf32Str,
};

/// One place the audio could come out.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RendererEntry {
    /// Empty for this machine, which is always the first row.
    pub id: String,
    pub name: String,
    /// The daemon's `app` field: `chromecast`, `dlna`, `music-player`, …
    pub kind: String,
    /// Reached over the cast API rather than the peer one.
    pub cast: bool,
    pub playing: bool,
}

impl RendererEntry {
    pub fn local(playing: bool) -> Self {
        Self {
            id: String::new(),
            name: "This computer".into(),
            kind: "local".into(),
            cast: false,
            playing,
        }
    }

    /// How each kind describes itself in the list.
    pub fn label(&self) -> &str {
        match self.kind.as_str() {
            "local" => "local playback",
            "chromecast" => "Chromecast",
            "dlna" => "UPnP / DLNA",
            "music-player" => "music-player",
            "xbmc" => "Kodi",
            other => other,
        }
    }

    /// What the fuzzy matcher sees, and what the row shows.
    pub fn display(&self) -> String {
        format!("{}  —  {}", self.name, self.label())
    }

    pub fn is_local(&self) -> bool {
        self.id.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct RendererResult {
    pub entry: RendererEntry,
    pub display: String,
    pub indices: Vec<u32>,
}

/// The picker overlay. `active` is what makes the key handler take over.
pub struct RendererPicker {
    pub active: bool,
    pub query: String,
    pub selected_index: usize,
    /// Every renderer, plus the local row at the front.
    pub entries: Vec<RendererEntry>,
    pub results: Vec<RendererResult>,
    pub loading: bool,
    pub error: String,
    matcher: Matcher,
}

impl Default for RendererPicker {
    fn default() -> Self {
        Self {
            active: false,
            query: String::new(),
            selected_index: 0,
            entries: vec![],
            results: vec![],
            loading: false,
            error: String::new(),
            matcher: Matcher::new(Config::DEFAULT),
        }
    }
}

impl RendererPicker {
    pub fn open(&mut self) {
        self.active = true;
        self.query.clear();
        self.selected_index = 0;
        self.error.clear();
        self.refresh();
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    /// Replace the list with what the daemon found, local row first.
    pub fn set_renderers(&mut self, renderers: Vec<RendererEntry>) {
        let elsewhere = renderers.iter().any(|renderer| renderer.playing);
        let mut entries = vec![RendererEntry::local(!elsewhere)];
        entries.extend(renderers);
        self.entries = entries;
        self.loading = false;
        self.refresh();
    }

    pub fn selected(&self) -> Option<&RendererEntry> {
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
                .map(|entry| RendererResult {
                    display: entry.display(),
                    entry: entry.clone(),
                    indices: vec![],
                })
                .collect();
        } else {
            let pattern = Pattern::parse(&self.query, CaseMatching::Ignore, Normalization::Smart);
            let mut buf = Vec::new();
            let mut matched: Vec<(u32, RendererResult)> = Vec::new();
            for entry in &self.entries {
                let display = entry.display();
                let haystack = Utf32Str::new(&display, &mut buf);
                let mut indices = Vec::new();
                if let Some(score) = pattern.indices(haystack, &mut self.matcher, &mut indices) {
                    indices.sort_unstable();
                    indices.dedup();
                    matched.push((
                        score,
                        RendererResult {
                            entry: entry.clone(),
                            display,
                            indices,
                        },
                    ));
                }
            }
            matched.sort_by(|a, b| b.0.cmp(&a.0));
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

    fn renderer(name: &str, kind: &str, playing: bool) -> RendererEntry {
        RendererEntry {
            id: format!("id-{name}"),
            name: name.into(),
            kind: kind.into(),
            cast: kind != "music-player",
            playing,
        }
    }

    fn picker() -> RendererPicker {
        let mut picker = RendererPicker::default();
        picker.open();
        picker.set_renderers(vec![
            renderer("Kitchen speaker", "chromecast", false),
            renderer("Living room TV", "dlna", false),
            renderer("studio", "music-player", false),
        ]);
        picker
    }

    /// There is always somewhere to play, so this machine leads.
    #[test]
    fn this_computer_is_always_offered_first() {
        let picker = picker();
        assert!(picker.entries[0].is_local());
        assert_eq!(picker.results.len(), 4);
    }

    /// Nothing playing elsewhere *is* this machine playing.
    #[test]
    fn the_local_row_plays_when_nothing_else_does() {
        let mut picker = picker();
        assert!(picker.entries[0].playing);

        picker.set_renderers(vec![renderer("Kitchen speaker", "chromecast", true)]);
        assert!(!picker.entries[0].playing);
        assert!(picker.entries[1].playing);
    }

    /// The kind is searchable, not just the name — "dlna" should find the TV.
    #[test]
    fn filters_fuzzily_over_name_and_kind() {
        let mut picker = picker();
        picker.query = "kitchen".into();
        picker.refresh();
        assert_eq!(picker.results.len(), 1);
        assert_eq!(picker.results[0].entry.name, "Kitchen speaker");

        picker.query = "dlna".into();
        picker.refresh();
        assert_eq!(picker.results[0].entry.name, "Living room TV");
    }

    #[test]
    fn a_query_matching_nothing_leaves_no_selection() {
        let mut picker = picker();
        picker.query = "zzzzz".into();
        picker.refresh();
        assert!(picker.results.is_empty());
        assert!(picker.selected().is_none());
    }

    #[test]
    fn selection_stays_inside_the_list() {
        let mut picker = picker();
        for _ in 0..10 {
            picker.move_down();
        }
        assert_eq!(picker.selected_index, 3);
        for _ in 0..10 {
            picker.move_up();
        }
        assert_eq!(picker.selected_index, 0);
    }

    /// Filtering must not leave the cursor pointing past the end.
    #[test]
    fn narrowing_the_query_pulls_the_selection_back() {
        let mut picker = picker();
        picker.move_down();
        picker.move_down();
        picker.move_down();
        assert_eq!(picker.selected_index, 3);

        picker.query = "kitchen".into();
        picker.refresh();
        assert_eq!(picker.selected_index, 0);
        assert_eq!(picker.selected().unwrap().name, "Kitchen speaker");
    }

    /// A peer daemon is reached over the device API, everything else over the
    /// cast one — the row has to carry which.
    #[test]
    fn a_peer_is_not_a_cast_target() {
        let picker = picker();
        let peer = picker
            .entries
            .iter()
            .find(|entry| entry.kind == "music-player")
            .unwrap();
        assert!(!peer.cast);
        assert!(picker.entries[1].cast);
    }

    #[test]
    fn every_kind_names_itself() {
        assert_eq!(RendererEntry::local(true).label(), "local playback");
        assert_eq!(renderer("x", "chromecast", false).label(), "Chromecast");
        assert_eq!(renderer("x", "dlna", false).label(), "UPnP / DLNA");
        assert_eq!(renderer("x", "xbmc", false).label(), "Kodi");
        // Anything unrecognised says what the daemon called it.
        assert_eq!(renderer("x", "sonos", false).label(), "sonos");
    }

    /// Reopening must not show the last query.
    #[test]
    fn opening_resets_the_query() {
        let mut picker = picker();
        picker.query = "kitchen".into();
        picker.error = "stale".into();
        picker.open();
        assert!(picker.query.is_empty());
        assert!(picker.error.is_empty());
        assert_eq!(picker.results.len(), 4);
    }
}
