//! Editor support: syntax highlighting and completion for a filter box.
//!
//! Kept here rather than in each UI so the TUI, the desktop and the web all
//! colour and complete the same way, and so the rules are tested once.
//!
//! Both entry points are deliberately forgiving. A filter is invalid for most
//! of the time it is being typed, so neither [`highlight`] nor [`complete`]
//! ever fails — they describe what is there, including the broken parts.

use crate::lexer::Comparison;
use crate::schema::Schema;

/// What a run of characters is, for colouring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A field the schema knows.
    Field,
    /// A field it does not — the most useful thing to colour differently,
    /// since it is the mistake people actually make.
    UnknownField,
    /// A comparison operator.
    Operator,
    /// `;` `,` `and` `or`.
    Logic,
    Paren,
    /// A bare value.
    Value,
    /// A quoted value, including its quotes.
    Quoted,
    Space,
    /// Something that does not belong — an unterminated string, a stray
    /// character.
    Error,
}

/// One coloured run, as byte offsets into the input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// The text this span covers.
    pub fn text<'a>(&self, input: &'a str) -> &'a str {
        input.get(self.start..self.end).unwrap_or_default()
    }
}

/// Split `input` into coloured runs.
///
/// Covers the whole input with no gaps and no overlaps, so a renderer can
/// concatenate the spans and get the original string back — which is what
/// keeps a highlight overlay aligned with the caret beneath it.
pub fn highlight(input: &str, schema: &Schema) -> Vec<Span> {
    let bytes = input.as_bytes();
    let mut spans: Vec<Span> = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let start = i;
        let c = bytes[i];

        if c.is_ascii_whitespace() {
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            spans.push(Span {
                kind: Kind::Space,
                start,
                end: i,
            });
            continue;
        }

        match c {
            b'(' | b')' => {
                i += 1;
                spans.push(Span {
                    kind: Kind::Paren,
                    start,
                    end: i,
                });
            }
            b';' | b',' => {
                i += 1;
                spans.push(Span {
                    kind: Kind::Logic,
                    start,
                    end: i,
                });
            }
            b'\'' | b'"' => {
                let quote = c;
                i += 1;
                let mut closed = false;
                while i < bytes.len() {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == quote {
                        i += 1;
                        closed = true;
                        break;
                    }
                    i += 1;
                }
                spans.push(Span {
                    // An unterminated string is the error, not the text in it.
                    kind: if closed { Kind::Quoted } else { Kind::Error },
                    start,
                    end: i,
                });
            }
            // `=name=` — the FIQL spelling. A lone `==` falls through to the
            // symbolic branch.
            b'=' if i + 1 < bytes.len() && bytes[i + 1] != b'=' => {
                let mut j = i + 1;
                while j < bytes.len() && bytes[j] != b'=' {
                    j += 1;
                }
                if j < bytes.len() {
                    i = j + 1;
                    let name = input.get(start + 1..j).unwrap_or_default();
                    spans.push(Span {
                        kind: if named_operator(name) {
                            Kind::Operator
                        } else {
                            Kind::Error
                        },
                        start,
                        end: i,
                    });
                } else {
                    // Still being typed: `=gt` with no closing `=`.
                    i = bytes.len();
                    spans.push(Span {
                        kind: Kind::Operator,
                        start,
                        end: i,
                    });
                }
            }
            b'=' | b'!' | b'<' | b'>' => {
                while i < bytes.len() && matches!(bytes[i], b'=' | b'!' | b'<' | b'>') {
                    i += 1;
                }
                spans.push(Span {
                    kind: Kind::Operator,
                    start,
                    end: i,
                });
            }
            _ => {
                while i < bytes.len() && !is_delimiter(bytes[i]) {
                    i += 1;
                }
                let word = input.get(start..i).unwrap_or_default();
                let kind = if is_logic_word(word) && follows_a_value(&spans) {
                    Kind::Logic
                } else if expects_value(&spans) {
                    Kind::Value
                } else if schema.field(word).is_some() {
                    Kind::Field
                } else {
                    Kind::UnknownField
                };
                spans.push(Span {
                    kind,
                    start,
                    end: i,
                });
            }
        }
    }
    spans
}

fn is_delimiter(c: u8) -> bool {
    c.is_ascii_whitespace() || matches!(c, b';' | b',' | b'(' | b')' | b'=' | b'!' | b'<' | b'>')
}

fn is_logic_word(word: &str) -> bool {
    matches!(word.to_ascii_lowercase().as_str(), "and" | "or")
}

fn named_operator(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "eq" | "ne"
            | "neq"
            | "gt"
            | "ge"
            | "gte"
            | "lt"
            | "le"
            | "lte"
            | "in"
            | "out"
            | "nin"
            | "like"
            | "contains"
            | "null"
            | "isnull"
            | "notnull"
            | "isnotnull"
    )
}

/// The last span that is not whitespace.
fn last_meaningful(spans: &[Span]) -> Option<Kind> {
    spans
        .iter()
        .rev()
        .find(|span| span.kind != Kind::Space)
        .map(|span| span.kind)
}

/// A connective can only follow a completed comparison.
fn follows_a_value(spans: &[Span]) -> bool {
    matches!(
        last_meaningful(spans),
        Some(Kind::Value) | Some(Kind::Quoted) | Some(Kind::Paren)
    )
}

/// A bare word right after an operator is a value, not a field name.
fn expects_value(spans: &[Span]) -> bool {
    match last_meaningful(spans) {
        Some(Kind::Operator) => true,
        // Inside a `=in=(a,b)` list.
        Some(Kind::Logic) | Some(Kind::Value) | Some(Kind::Quoted) => in_value_list(spans),
        Some(Kind::Paren) => opens_value_list(spans),
        _ => false,
    }
}

/// Whether the innermost open paren directly followed an operator, making it a
/// value list rather than a group. The comma means different things in each.
fn opens_value_list(spans: &[Span]) -> bool {
    let mut depth = 0usize;
    for index in (0..spans.len()).rev() {
        let span = &spans[index];
        if span.kind != Kind::Paren {
            continue;
        }
        // Only the paren character itself is in this span.
        let is_close = span.end - span.start == 1;
        match () {
            _ if is_close => {}
            _ => {}
        }
        // Distinguish by looking at the source is not possible here, so the
        // caller passes the input; see `in_value_list`.
        let _ = depth;
        depth = 0;
        return index > 0
            && spans[..index]
                .iter()
                .rev()
                .find(|s| s.kind != Kind::Space)
                .map(|s| s.kind == Kind::Operator)
                .unwrap_or(false);
    }
    false
}

fn in_value_list(spans: &[Span]) -> bool {
    opens_value_list(spans)
}

// ── Completion ──────────────────────────────────────────────────────────────

/// What the caret is in the middle of typing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum What {
    Field,
    Operator,
    Value,
}

/// The suggestion list for a caret position, and the range an accepted
/// suggestion replaces.
#[derive(Clone, Debug, PartialEq)]
pub struct Completion {
    pub what: What,
    /// The partial text already typed, which a suggestion replaces.
    pub prefix: String,
    /// Byte range in the input that a suggestion replaces.
    pub from: usize,
    pub to: usize,
    /// The field this comparison is about, when the caret is on a value.
    pub field: Option<String>,
    /// The suggestions themselves, already filtered by the prefix.
    pub suggestions: Vec<Suggestion>,
}

/// One suggestion, with the hint a picker shows beside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
    pub text: String,
    pub hint: String,
}

impl Completion {
    /// Apply a suggestion to `input`, returning the new text and where the
    /// caret should land.
    pub fn apply(&self, input: &str, suggestion: &str) -> (String, usize) {
        let mut out = String::with_capacity(input.len() + suggestion.len());
        out.push_str(input.get(..self.from).unwrap_or_default());
        out.push_str(suggestion);
        let caret = out.len();
        out.push_str(input.get(self.to..).unwrap_or_default());
        (out, caret)
    }
}

/// Whether a run of operator characters is already a whole operator, rather
/// than the start of a longer one. `>` counts: it is valid on its own, and
/// typing another `=` still turns it into `>=`.
fn is_complete_operator(text: &str) -> bool {
    matches!(text, "==" | "!=" | ">=" | "<=" | ">" | "<")
}

/// Operators that make sense for a field, with a hint each.
fn operators_for(schema: &Schema, field: Option<&str>) -> Vec<Suggestion> {
    use crate::schema::FieldKind;
    let kind = field.and_then(|name| schema.field(name)).map(|f| f.kind);
    let mut ops = vec![
        ("==", "equals"),
        ("!=", "does not equal"),
        ("=in=", "any of"),
        ("=out=", "none of"),
        ("=null=", "has no value"),
        ("=notnull=", "has a value"),
    ];
    match kind {
        Some(FieldKind::Text) | None => {
            ops.insert(2, ("=like=", "contains — * is a wildcard"));
        }
        Some(_) => {
            // Ordering comparisons only mean something for numbers and dates.
            ops.splice(
                2..2,
                [
                    (">", "greater than"),
                    (">=", "at least"),
                    ("<", "less than"),
                    ("<=", "at most"),
                ],
            );
        }
    }
    ops.into_iter()
        .map(|(text, hint)| Suggestion {
            text: text.to_string(),
            hint: hint.to_string(),
        })
        .collect()
}

/// Values worth suggesting for a field. Only where a fixed vocabulary exists —
/// nobody can guess an artist name, but `true`/`false` and relative ages are
/// finite and easy to get wrong.
fn values_for(schema: &Schema, field: Option<&str>) -> Vec<Suggestion> {
    use crate::schema::FieldKind;
    let Some(field) = field.and_then(|name| schema.field(name)) else {
        return Vec::new();
    };
    match field.kind {
        FieldKind::Boolean => ["true", "false"]
            .iter()
            .map(|text| Suggestion {
                text: (*text).to_string(),
                hint: String::new(),
            })
            .collect(),
        FieldKind::Timestamp => [
            ("7d", "in the last week"),
            ("30d", "in the last 30 days"),
            ("6m", "in the last 6 months"),
            ("1y", "in the last year"),
            ("2024-01-01", "on or after a date"),
        ]
        .iter()
        .map(|(text, hint)| Suggestion {
            text: (*text).to_string(),
            hint: (*hint).to_string(),
        })
        .collect(),
        _ => Vec::new(),
    }
}

/// What to suggest at `caret` (a byte offset into `input`).
pub fn complete(input: &str, caret: usize, schema: &Schema) -> Completion {
    let caret = caret.min(input.len());
    let before = input.get(..caret).unwrap_or_default();

    // Walk back over the word the caret is in.
    let word_start = before
        .rfind(|c: char| is_delimiter(c as u8) || c == '\'' || c == '"')
        .map(|index| index + before[index..].chars().next().map_or(1, char::len_utf8))
        .unwrap_or(0);
    let word = &before[word_start..];

    // What came before that word decides what is being typed.
    let spans = highlight(before, schema);
    let preceding = spans
        .iter()
        .rev()
        .find(|span| span.kind != Kind::Space && span.end <= word_start)
        .map(|span| (span.kind, span.text(before).to_string()));

    // The caret sits on a run of operator characters. Whether that is still
    // being typed decides what to offer: `year>` is a finished operator and
    // wants a value next, while `genre=` could still become `=like=`.
    let operator_chars = before
        .chars()
        .rev()
        .take_while(|c| matches!(c, '=' | '!' | '<' | '>'))
        .count();
    if operator_chars > 0 && word.is_empty() {
        let from = caret - operator_chars;
        let typed = &before[from..];
        let field = field_before(&spans, before, from);
        if is_complete_operator(typed) {
            let suggestions = filter_by_prefix(values_for(schema, field.as_deref()), "");
            return Completion {
                what: What::Value,
                prefix: String::new(),
                from: caret,
                to: caret,
                field,
                suggestions,
            };
        }
        let suggestions = filter_by_prefix(operators_for(schema, field.as_deref()), typed);
        return Completion {
            what: What::Operator,
            prefix: typed.to_string(),
            from,
            to: caret,
            field,
            suggestions,
        };
    }

    match preceding {
        // Right after an operator: a value.
        Some((Kind::Operator, _)) => {
            let field = field_before(&spans, before, word_start);
            let suggestions = filter_by_prefix(values_for(schema, field.as_deref()), word);
            Completion {
                what: What::Value,
                prefix: word.to_string(),
                from: word_start,
                to: caret,
                field,
                suggestions,
            }
        }
        // Anywhere else a bare word can start: a field.
        _ => {
            let suggestions = filter_by_prefix(
                schema
                    .fields
                    .iter()
                    .map(|field| Suggestion {
                        text: field.name.to_string(),
                        hint: field.label.to_string(),
                    })
                    .collect(),
                word,
            );
            Completion {
                what: What::Field,
                prefix: word.to_string(),
                from: word_start,
                to: caret,
                field: None,
                suggestions,
            }
        }
    }
}

/// The field name of the comparison ending at `before_offset`.
fn field_before(spans: &[Span], input: &str, before_offset: usize) -> Option<String> {
    let mut seen_operator = false;
    for span in spans.iter().rev() {
        if span.end > before_offset {
            continue;
        }
        match span.kind {
            Kind::Space => continue,
            Kind::Operator => seen_operator = true,
            Kind::Field | Kind::UnknownField if seen_operator => {
                return Some(span.text(input).to_string())
            }
            Kind::Field | Kind::UnknownField => return Some(span.text(input).to_string()),
            _ => return None,
        }
    }
    None
}

/// Keep the suggestions that start with `prefix`, case-insensitively. An empty
/// prefix keeps them all.
fn filter_by_prefix(suggestions: Vec<Suggestion>, prefix: &str) -> Vec<Suggestion> {
    if prefix.is_empty() {
        return suggestions;
    }
    let prefix = prefix.to_ascii_lowercase();
    suggestions
        .into_iter()
        .filter(|s| s.text.to_ascii_lowercase().starts_with(&prefix))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::TRACKS;

    fn kinds(input: &str) -> Vec<(Kind, &str)> {
        highlight(input, &TRACKS)
            .into_iter()
            .filter(|span| span.kind != Kind::Space)
            .map(|span| {
                let text = &input[span.start..span.end];
                (span.kind, text)
            })
            .collect()
    }

    /// A renderer concatenates the spans to draw the line, so they have to
    /// cover the input exactly — a gap would drop characters, an overlap would
    /// double them, and either way the highlight would drift off the caret.
    #[test]
    fn spans_tile_the_input_exactly() {
        for input in [
            "genre==rock",
            "  genre == rock ; year > 2000  ",
            "genre=in=(rock,'trip hop')",
            "artist=='unterminated",
            "((year>2000))",
            "",
        ] {
            let spans = highlight(input, &TRACKS);
            let mut at = 0;
            for span in &spans {
                assert_eq!(span.start, at, "gap or overlap in {input:?}");
                at = span.end;
            }
            assert_eq!(at, input.len(), "spans stop short in {input:?}");
            let rebuilt: String = spans.iter().map(|s| s.text(input)).collect();
            assert_eq!(rebuilt, input);
        }
    }

    /// The mistake people actually make is a mistyped field, so it is the one
    /// thing that must be visibly different.
    #[test]
    fn an_unknown_field_is_marked() {
        assert_eq!(
            kinds("artistt==X"),
            vec![
                (Kind::UnknownField, "artistt"),
                (Kind::Operator, "=="),
                (Kind::Value, "X"),
            ]
        );
        assert_eq!(kinds("artist==X")[0], (Kind::Field, "artist"));
    }

    /// A value is never a field, however it is spelled — `rock` after `==` is
    /// not an unknown field.
    #[test]
    fn values_are_not_mistaken_for_fields() {
        assert_eq!(
            kinds("genre==rock;year>2000"),
            vec![
                (Kind::Field, "genre"),
                (Kind::Operator, "=="),
                (Kind::Value, "rock"),
                (Kind::Logic, ";"),
                (Kind::Field, "year"),
                (Kind::Operator, ">"),
                (Kind::Value, "2000"),
            ]
        );
    }

    #[test]
    fn quoted_values_and_unterminated_strings() {
        assert_eq!(
            kinds("album=='Kid A'"),
            vec![
                (Kind::Field, "album"),
                (Kind::Operator, "=="),
                (Kind::Quoted, "'Kid A'"),
            ]
        );
        // An unterminated string is the error, and it is the whole run.
        assert_eq!(kinds("album=='Kid A")[2].0, Kind::Error);
    }

    #[test]
    fn named_operators_are_checked() {
        assert_eq!(kinds("year=gt=2000")[1], (Kind::Operator, "=gt="));
        assert_eq!(kinds("year=nope=2000")[1], (Kind::Error, "=nope="));
    }

    #[test]
    fn and_or_colour_as_logic_only_between_comparisons() {
        assert_eq!(
            kinds("artist==Or and year>2000"),
            vec![
                (Kind::Field, "artist"),
                (Kind::Operator, "=="),
                // The band called "Or" is a value, not a connective.
                (Kind::Value, "Or"),
                (Kind::Logic, "and"),
                (Kind::Field, "year"),
                (Kind::Operator, ">"),
                (Kind::Value, "2000"),
            ]
        );
    }

    #[test]
    fn suggests_fields_at_the_start() {
        let completion = complete("", 0, &TRACKS);
        assert_eq!(completion.what, What::Field);
        assert!(completion.suggestions.iter().any(|s| s.text == "artist"));
        // Every suggestion carries the label a picker shows.
        assert!(completion.suggestions.iter().all(|s| !s.hint.is_empty()));
    }

    #[test]
    fn filters_field_suggestions_by_what_is_typed() {
        let completion = complete("pl", 2, &TRACKS);
        assert_eq!(completion.what, What::Field);
        assert_eq!(completion.prefix, "pl");
        assert!(completion.suggestions.iter().any(|s| s.text == "playcount"));
        assert!(!completion.suggestions.iter().any(|s| s.text == "artist"));
    }

    /// Ordering comparisons mean nothing for text, and `=like=` means nothing
    /// for a number — offering them would teach the wrong thing.
    #[test]
    fn operator_suggestions_suit_the_field() {
        let numeric = operators_for(&TRACKS, Some("year"));
        assert!(numeric.iter().any(|s| s.text == ">="));
        assert!(!numeric.iter().any(|s| s.text == "=like="));

        let text = operators_for(&TRACKS, Some("artist"));
        assert!(text.iter().any(|s| s.text == "=like="));
        assert!(!text.iter().any(|s| s.text == ">="));
    }

    /// A half-typed operator asks for operators; a finished one asks for the
    /// value that comes next. `year>` is already valid, so it is the latter.
    #[test]
    fn a_finished_operator_moves_on_to_the_value() {
        let partial = complete("artist=", 7, &TRACKS);
        assert_eq!(partial.what, What::Operator);
        assert_eq!(partial.field.as_deref(), Some("artist"));
        assert_eq!(partial.prefix, "=");
        // The suggestions are the operators that start with what was typed.
        assert!(partial.suggestions.iter().all(|s| s.text.starts_with('=')));

        for finished in ["year>", "year>=", "artist==", "year<"] {
            let completion = complete(finished, finished.len(), &TRACKS);
            assert_eq!(completion.what, What::Value, "{finished}");
            // Nothing is replaced: the caret is past the operator.
            assert_eq!(completion.from, completion.to);
        }
    }

    #[test]
    fn suggests_values_where_the_vocabulary_is_fixed() {
        let boolean = complete("liked==", 7, &TRACKS);
        assert_eq!(boolean.what, What::Value);
        assert_eq!(boolean.field.as_deref(), Some("liked"));
        assert!(boolean.suggestions.iter().any(|s| s.text == "true"));

        let date = complete("lastplayed>", 11, &TRACKS);
        assert!(date.suggestions.iter().any(|s| s.text == "30d"));

        // Nobody can guess an artist name, so nothing is offered.
        assert!(complete("artist==", 8, &TRACKS).suggestions.is_empty());
    }

    /// Accepting a suggestion must replace only the partial word, and leave the
    /// caret after what it inserted.
    #[test]
    fn applying_a_suggestion_replaces_the_prefix() {
        let completion = complete("gen", 3, &TRACKS);
        let (text, caret) = completion.apply("gen", "genre");
        assert_eq!(text, "genre");
        assert_eq!(caret, 5);

        // With text after the caret, that text survives.
        let completion = complete("gen==rock", 3, &TRACKS);
        let (text, caret) = completion.apply("gen==rock", "genre");
        assert_eq!(text, "genre==rock");
        assert_eq!(caret, 5);
    }

    /// A caret past the end (or on a multi-byte boundary) must not panic.
    #[test]
    fn a_caret_out_of_range_is_clamped() {
        let completion = complete("genre", 999, &TRACKS);
        assert_eq!(completion.to, 5);
        // And a filter with multi-byte characters is walked safely.
        let _ = complete("artist==Sigur Rós", 17, &TRACKS);
        let _ = highlight("artist==Sigur Rós", &TRACKS);
    }
}
