//! Tokenizer for RSQL.
//!
//! The grammar is small enough that hand-writing this beats a parser
//! generator, and it lets every token keep its byte offset — a filter typed
//! into the smart-playlist form is worth pointing at the exact character that
//! is wrong.

use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// A field name, e.g. `artist`.
    Ident(String),
    /// A quoted or bare literal.
    Value(String),
    /// A comparison operator, normalised (`=gt=` and `>` are both `Gt`).
    Op(Comparison),
    /// `;` or `and`
    And,
    /// `,` or `or` — at the top level. Inside `( … )` a comma separates list
    /// items instead, which the parser resolves from context.
    Or,
    OpenParen,
    CloseParen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    In,
    Out,
    /// Substring match, case-insensitive. `*` in the value is a wildcard.
    Like,
    /// The field has no value (NULL or empty string).
    Null,
    NotNull,
}

impl Comparison {
    fn from_symbolic(text: &str) -> Option<Self> {
        Some(match text {
            "==" => Comparison::Eq,
            "!=" => Comparison::Ne,
            ">=" => Comparison::Ge,
            "<=" => Comparison::Le,
            ">" => Comparison::Gt,
            "<" => Comparison::Lt,
            _ => return None,
        })
    }

    fn from_named(name: &str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "eq" => Comparison::Eq,
            "ne" | "neq" => Comparison::Ne,
            "gt" => Comparison::Gt,
            "ge" | "gte" => Comparison::Ge,
            "lt" => Comparison::Lt,
            "le" | "lte" => Comparison::Le,
            "in" => Comparison::In,
            "out" | "nin" => Comparison::Out,
            "like" | "contains" => Comparison::Like,
            "null" | "isnull" => Comparison::Null,
            "notnull" | "isnotnull" => Comparison::NotNull,
            _ => return None,
        })
    }

    /// Whether the operator takes a list of values rather than one.
    pub fn is_list(self) -> bool {
        matches!(self, Comparison::In | Comparison::Out)
    }

    /// Whether the operator takes no value at all.
    pub fn is_unary(self) -> bool {
        matches!(self, Comparison::Null | Comparison::NotNull)
    }
}

/// A token together with where it started, in bytes from the start of input.
#[derive(Clone, Debug, PartialEq)]
pub struct Spanned {
    pub token: Token,
    pub at: usize,
}

/// Characters that end a bare (unquoted) value or identifier.
fn is_delimiter(c: char) -> bool {
    c.is_whitespace() || matches!(c, ';' | ',' | '(' | ')' | '=' | '!' | '<' | '>')
}

pub fn tokenize(input: &str) -> Result<Vec<Spanned>> {
    let chars: Vec<char> = input.chars().collect();
    // Byte offset of each char, so spans point into the original string.
    let offsets: Vec<usize> = input.char_indices().map(|(i, _)| i).collect();
    let at = |i: usize| offsets.get(i).copied().unwrap_or(input.len());

    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        let start = at(i);
        match c {
            ';' => {
                tokens.push(Spanned {
                    token: Token::And,
                    at: start,
                });
                i += 1;
            }
            ',' => {
                tokens.push(Spanned {
                    token: Token::Or,
                    at: start,
                });
                i += 1;
            }
            '(' => {
                tokens.push(Spanned {
                    token: Token::OpenParen,
                    at: start,
                });
                i += 1;
            }
            ')' => {
                tokens.push(Spanned {
                    token: Token::CloseParen,
                    at: start,
                });
                i += 1;
            }
            '\'' | '"' => {
                let (value, next) = quoted(&chars, i, c, start)?;
                tokens.push(Spanned {
                    token: Token::Value(value),
                    at: start,
                });
                i = next;
            }
            // `=name=` — the FIQL form of a named operator. A lone `==` is
            // handled by the symbolic branch below.
            '=' if i + 1 < chars.len() && chars[i + 1] != '=' => {
                let mut j = i + 1;
                while j < chars.len() && chars[j] != '=' {
                    j += 1;
                }
                if j >= chars.len() {
                    return Err(Error::new("unterminated operator", start));
                }
                let name: String = chars[i + 1..j].iter().collect();
                let op = Comparison::from_named(&name)
                    .ok_or_else(|| Error::new(format!("unknown operator '={name}='"), start))?;
                tokens.push(Spanned {
                    token: Token::Op(op),
                    at: start,
                });
                i = j + 1;
            }
            '=' | '!' | '<' | '>' => {
                // Longest match first: `>=` before `>`.
                let two: String = chars[i..(i + 2).min(chars.len())].iter().collect();
                let (op, len) = match Comparison::from_symbolic(&two) {
                    Some(op) => (op, 2),
                    None => {
                        let one = c.to_string();
                        match Comparison::from_symbolic(&one) {
                            Some(op) => (op, 1),
                            None => {
                                return Err(Error::new(format!("unexpected '{c}'"), start));
                            }
                        }
                    }
                };
                tokens.push(Spanned {
                    token: Token::Op(op),
                    at: start,
                });
                i += len;
            }
            _ => {
                let mut j = i;
                while j < chars.len() && !is_delimiter(chars[j]) {
                    j += 1;
                }
                let word: String = chars[i..j].iter().collect();
                // `and` / `or` read as keywords only where a connective can
                // appear; anywhere else they are an ordinary word.
                let token = match word.to_ascii_lowercase().as_str() {
                    "and" if expects_connective(&tokens) => Token::And,
                    "or" if expects_connective(&tokens) => Token::Or,
                    _ if expects_value(&tokens) => Token::Value(word),
                    _ => Token::Ident(word),
                };
                tokens.push(Spanned { token, at: start });
                i = j;
            }
        }
    }
    Ok(tokens)
}

/// A connective can follow a completed comparison, i.e. a value or a `)`.
fn expects_connective(tokens: &[Spanned]) -> bool {
    matches!(
        tokens.last().map(|t| &t.token),
        Some(Token::Value(_)) | Some(Token::CloseParen)
    )
}

/// A bare word right after an operator, or inside that operator's value list,
/// is a value rather than a field name.
///
/// The comma is the reason this needs care: in `a==1,b==2` it separates two
/// comparisons and `b` is a field, while in `=in=(a,b)` it separates values and
/// `b` is one. Only an open paren that directly followed an operator starts a
/// value list.
fn expects_value(tokens: &[Spanned]) -> bool {
    match tokens.last().map(|t| &t.token) {
        Some(Token::Op(_)) => true,
        Some(Token::OpenParen) => opens_value_list(tokens, tokens.len() - 1),
        Some(Token::Or) => in_value_list(tokens),
        _ => false,
    }
}

/// Whether the `(` at `index` is the start of a value list rather than a group.
fn opens_value_list(tokens: &[Spanned], index: usize) -> bool {
    index > 0 && matches!(tokens[index - 1].token, Token::Op(_))
}

/// Whether the position after `tokens` sits inside an unclosed value list.
fn in_value_list(tokens: &[Spanned]) -> bool {
    let mut depth = 0usize;
    for index in (0..tokens.len()).rev() {
        match &tokens[index].token {
            Token::CloseParen => depth += 1,
            Token::OpenParen if depth == 0 => return opens_value_list(tokens, index),
            Token::OpenParen => depth -= 1,
            _ => {}
        }
    }
    false
}

/// Read a quoted string, honouring `\` escapes. Returns the value and the
/// index just past the closing quote.
fn quoted(chars: &[char], start: usize, quote: char, at: usize) -> Result<(String, usize)> {
    let mut value = String::new();
    let mut i = start + 1;
    while i < chars.len() {
        match chars[i] {
            '\\' if i + 1 < chars.len() => {
                value.push(chars[i + 1]);
                i += 2;
            }
            c if c == quote => return Ok((value, i + 1)),
            c => {
                value.push(c);
                i += 1;
            }
        }
    }
    Err(Error::new("unterminated string", at))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(input: &str) -> Vec<Token> {
        tokenize(input)
            .unwrap()
            .into_iter()
            .map(|t| t.token)
            .collect()
    }

    #[test]
    fn reads_a_simple_comparison() {
        assert_eq!(
            kinds("artist==Radiohead"),
            vec![
                Token::Ident("artist".into()),
                Token::Op(Comparison::Eq),
                Token::Value("Radiohead".into()),
            ]
        );
    }

    #[test]
    fn accepts_both_operator_spellings() {
        assert_eq!(kinds("year=gt=2000"), kinds("year>2000"));
        assert_eq!(kinds("year=ge=2000"), kinds("year>=2000"));
    }

    #[test]
    fn reads_quoted_values_with_delimiters_inside() {
        assert_eq!(
            kinds(r#"album=="Kid A; and B""#),
            vec![
                Token::Ident("album".into()),
                Token::Op(Comparison::Eq),
                Token::Value("Kid A; and B".into()),
            ]
        );
    }

    /// `and`/`or` are only connectives between comparisons — a track by a band
    /// actually called "Or" must still be searchable.
    #[test]
    fn and_or_are_keywords_only_between_comparisons() {
        assert_eq!(
            kinds("artist==Or and year>2000"),
            vec![
                Token::Ident("artist".into()),
                Token::Op(Comparison::Eq),
                Token::Value("Or".into()),
                Token::And,
                Token::Ident("year".into()),
                Token::Op(Comparison::Gt),
                Token::Value("2000".into()),
            ]
        );
    }

    #[test]
    fn reads_value_lists() {
        assert_eq!(
            kinds("genre=in=(rock,jazz)"),
            vec![
                Token::Ident("genre".into()),
                Token::Op(Comparison::In),
                Token::OpenParen,
                Token::Value("rock".into()),
                Token::Or,
                Token::Value("jazz".into()),
                Token::CloseParen,
            ]
        );
    }

    #[test]
    fn reports_where_the_error_is() {
        let e = tokenize("artist=nope=x").unwrap_err();
        assert_eq!(e.at, 6);
        assert!(e.message.contains("unknown operator"));
        assert_eq!(tokenize("artist=='unclosed").unwrap_err().at, 8);
    }
}
