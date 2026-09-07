//! State for the TUI's smart-playlist form.
//!
//! A small modal with four fields. It follows the fuzzy-finder overlay's
//! pattern — a flag on `App` that swallows key events while it is up — rather
//! than introducing a second modal mechanism.
//!
//! The live match count is deliberately not refreshed on every keystroke: it is
//! a round trip to the daemon, and a filter is invalid for most of the time it
//! is being typed. It refreshes when the user moves off the filter field or
//! presses Enter on it.

use music_player_rsql::TRACKS;

/// Which field has focus. Tab moves forward, Shift+Tab back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Field {
    #[default]
    Name,
    Filter,
    SortBy,
    Limit,
}

impl Field {
    pub const ALL: [Field; 4] = [Field::Name, Field::Filter, Field::SortBy, Field::Limit];

    pub fn label(self) -> &'static str {
        match self {
            Field::Name => "Name",
            Field::Filter => "Filter",
            Field::SortBy => "Sort by",
            Field::Limit => "Limit",
        }
    }

    /// What the field expects, shown when it is empty.
    pub fn placeholder(self) -> &'static str {
        match self {
            Field::Name => "My mix",
            Field::Filter => "genre==rock;year>2000",
            Field::SortBy => "playcount, random, … (empty = library order)",
            Field::Limit => "0 = all",
        }
    }

    pub fn next(self) -> Self {
        let index = Field::ALL.iter().position(|f| *f == self).unwrap_or(0);
        Field::ALL[(index + 1) % Field::ALL.len()]
    }

    pub fn previous(self) -> Self {
        let index = Field::ALL.iter().position(|f| *f == self).unwrap_or(0);
        Field::ALL[(index + Field::ALL.len() - 1) % Field::ALL.len()]
    }
}

/// Descending vs ascending, as the form shows it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Order {
    #[default]
    Asc,
    Desc,
}

impl Order {
    pub fn toggled(self) -> Self {
        match self {
            Order::Asc => Order::Desc,
            Order::Desc => Order::Asc,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Order::Asc => "asc",
            Order::Desc => "desc",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Order::Asc => "ascending",
            Order::Desc => "descending",
        }
    }
}

/// The form's state. `active` is what makes the key handler take over.
#[derive(Clone, Debug, Default)]
pub struct SmartPlaylistForm {
    pub active: bool,
    pub focus: Field,
    pub name: String,
    pub filter: String,
    pub sort_by: String,
    pub order: Order,
    pub limit: String,
    /// Matches for the current filter. `None` until a preview has come back.
    pub preview_count: Option<u32>,
    /// Why the filter did not compile. Empty when it did.
    pub error: String,
    pub previewing: bool,
    pub submitting: bool,
}

impl SmartPlaylistForm {
    /// Open an empty form.
    pub fn open(&mut self) {
        *self = Self {
            active: true,
            ..Default::default()
        };
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    /// The field that currently has focus, as a mutable string.
    fn focused_mut(&mut self) -> &mut String {
        match self.focus {
            Field::Name => &mut self.name,
            Field::Filter => &mut self.filter,
            Field::SortBy => &mut self.sort_by,
            Field::Limit => &mut self.limit,
        }
    }

    pub fn value(&self, field: Field) -> &str {
        match field {
            Field::Name => &self.name,
            Field::Filter => &self.filter,
            Field::SortBy => &self.sort_by,
            Field::Limit => &self.limit,
        }
    }

    pub fn push(&mut self, c: char) {
        self.focused_mut().push(c);
        self.invalidate_preview();
    }

    pub fn backspace(&mut self) {
        self.focused_mut().pop();
        self.invalidate_preview();
    }

    pub fn clear_field(&mut self) {
        self.focused_mut().clear();
        self.invalidate_preview();
    }

    /// A typed-over filter makes the last count stale; it is cleared rather
    /// than left to look current.
    fn invalidate_preview(&mut self) {
        if matches!(self.focus, Field::Filter | Field::SortBy | Field::Limit) {
            self.preview_count = None;
            self.error.clear();
        }
    }

    /// The limit as a number. Anything unparseable — including a half-typed
    /// value — reads as "no limit" rather than an arbitrary cap.
    pub fn limit_value(&self) -> u32 {
        self.limit.trim().parse().unwrap_or(0)
    }

    /// Whether the form has enough to submit.
    pub fn can_submit(&self) -> bool {
        !self.name.trim().is_empty() && !self.submitting
    }

    /// The line shown under the filter box.
    pub fn status_line(&self) -> String {
        if self.previewing {
            return "Checking…".to_string();
        }
        if !self.error.is_empty() {
            return self.error.clone();
        }
        match self.preview_count {
            Some(1) => "1 track matches".to_string(),
            Some(count) => format!("{count} tracks match"),
            None => format!("Fields: {}", TRACKS.field_names().join(", ")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_cycles_through_every_field() {
        let mut field = Field::Name;
        for _ in 0..Field::ALL.len() {
            field = field.next();
        }
        assert_eq!(field, Field::Name, "a full cycle returns to the start");

        assert_eq!(Field::Name.previous(), Field::Limit, "wraps backwards");
        assert_eq!(Field::Filter.previous(), Field::Name);
    }

    #[test]
    fn typing_lands_in_the_focused_field() {
        let mut form = SmartPlaylistForm::default();
        form.open();
        for c in "Mix".chars() {
            form.push(c);
        }
        assert_eq!(form.name, "Mix");

        form.focus = Field::Filter;
        for c in "genre==rock".chars() {
            form.push(c);
        }
        assert_eq!(form.filter, "genre==rock");
        // The name is untouched by editing another field.
        assert_eq!(form.name, "Mix");

        form.backspace();
        assert_eq!(form.filter, "genre==roc");
        form.clear_field();
        assert!(form.filter.is_empty());
    }

    /// A count left over from the previous filter would read as current.
    #[test]
    fn editing_the_filter_clears_a_stale_count() {
        let mut form = SmartPlaylistForm::default();
        form.open();
        form.focus = Field::Filter;
        form.preview_count = Some(42);
        form.push('x');
        assert!(form.preview_count.is_none());

        // Editing the name does not, though — it does not affect the match.
        form.focus = Field::Name;
        form.preview_count = Some(42);
        form.push('y');
        assert_eq!(form.preview_count, Some(42));
    }

    /// A half-typed limit must not silently cap the playlist.
    #[test]
    fn an_unparseable_limit_means_no_limit() {
        let mut form = SmartPlaylistForm::default();
        for (raw, expected) in [("", 0), ("25", 25), ("  10 ", 10), ("abc", 0), ("-5", 0)] {
            form.limit = raw.to_string();
            assert_eq!(form.limit_value(), expected, "{raw:?}");
        }
    }

    #[test]
    fn a_name_is_required_to_submit() {
        let mut form = SmartPlaylistForm::default();
        form.open();
        assert!(!form.can_submit());
        form.name = "Mix".into();
        assert!(form.can_submit());
        // Not while a submit is already in flight.
        form.submitting = true;
        assert!(!form.can_submit());
    }

    #[test]
    fn the_status_line_reports_what_it_knows() {
        let mut form = SmartPlaylistForm::default();
        form.open();
        // Nothing checked yet: offer the vocabulary instead.
        assert!(form.status_line().starts_with("Fields:"));
        assert!(form.status_line().contains("playcount"));

        form.preview_count = Some(1);
        assert_eq!(form.status_line(), "1 track matches");
        form.preview_count = Some(12);
        assert_eq!(form.status_line(), "12 tracks match");

        // An error outranks a count.
        form.error = "unknown field 'nope'".into();
        assert_eq!(form.status_line(), "unknown field 'nope'");
        // And "checking" outranks both.
        form.previewing = true;
        assert_eq!(form.status_line(), "Checking…");
    }

    #[test]
    fn the_order_toggles_and_names_itself() {
        assert_eq!(Order::Asc.toggled(), Order::Desc);
        assert_eq!(Order::Desc.toggled(), Order::Asc);
        assert_eq!(Order::Desc.as_str(), "desc");
        assert_eq!(Order::Asc.label(), "ascending");
    }

    /// Reopening must not carry the previous rule over.
    #[test]
    fn opening_resets_the_form() {
        let mut form = SmartPlaylistForm {
            active: false,
            name: "old".into(),
            filter: "genre==jazz".into(),
            preview_count: Some(3),
            error: "stale".into(),
            ..Default::default()
        };
        form.open();
        assert!(form.active);
        assert!(form.name.is_empty());
        assert!(form.filter.is_empty());
        assert!(form.preview_count.is_none());
        assert!(form.error.is_empty());
        assert_eq!(form.focus, Field::Name);
    }
}
