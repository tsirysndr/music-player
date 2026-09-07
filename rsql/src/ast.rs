//! The parsed filter, before it knows anything about SQL.
//!
//! Keeping the AST free of schema knowledge is what lets one filter be checked
//! against the track schema in one place and the album schema in another, and
//! lets an extension inspect a filter without a database in hand.

use crate::error::{Error, Result};
use crate::lexer::{tokenize, Comparison, Spanned, Token};

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    And(Vec<Node>),
    Or(Vec<Node>),
    Compare(Constraint),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Constraint {
    pub field: String,
    pub op: Comparison,
    /// One value for most operators, several for `=in=` / `=out=`, none for
    /// `=null=` / `=notnull=`.
    pub values: Vec<String>,
    /// Byte offset of the field name, for error reporting.
    pub at: usize,
}

impl Node {
    /// Every field mentioned anywhere in the filter, in order of appearance.
    pub fn fields(&self) -> Vec<&str> {
        let mut out = Vec::new();
        self.walk(&mut |c| out.push(c.field.as_str()));
        out
    }

    /// Visit every comparison in the tree.
    pub fn walk<'a>(&'a self, visit: &mut impl FnMut(&'a Constraint)) {
        match self {
            Node::And(children) | Node::Or(children) => {
                for child in children {
                    child.walk(visit);
                }
            }
            Node::Compare(constraint) => visit(constraint),
        }
    }
}

/// Parse an RSQL filter.
///
/// Precedence follows RSQL: `;` (AND) binds tighter than `,` (OR), so
/// `a==1,b==2;c==3` is `a==1 OR (b==2 AND c==3)`.
pub fn parse(input: &str) -> Result<Node> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err(Error::new("the filter is empty", 0));
    }
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
        end: input.len(),
    };
    let node = parser.or_expression()?;
    if let Some(spanned) = parser.peek() {
        return Err(Error::new("unexpected trailing input", spanned.at));
    }
    Ok(node)
}

struct Parser<'a> {
    tokens: &'a [Spanned],
    pos: usize,
    /// Offset just past the input, for errors that are about what is *missing*.
    end: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'a Spanned> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&'a Spanned> {
        let spanned = self.tokens.get(self.pos);
        if spanned.is_some() {
            self.pos += 1;
        }
        spanned
    }

    /// Where the *next* token starts, or the end of input.
    fn here(&self) -> usize {
        self.peek().map(|t| t.at).unwrap_or(self.end)
    }

    fn eat(&mut self, token: &Token) -> bool {
        if self.peek().map(|t| &t.token) == Some(token) {
            self.pos += 1;
            return true;
        }
        false
    }

    fn or_expression(&mut self) -> Result<Node> {
        let mut branches = vec![self.and_expression()?];
        while self.eat(&Token::Or) {
            branches.push(self.and_expression()?);
        }
        Ok(if branches.len() == 1 {
            branches.pop().unwrap()
        } else {
            Node::Or(branches)
        })
    }

    fn and_expression(&mut self) -> Result<Node> {
        let mut branches = vec![self.primary()?];
        while self.eat(&Token::And) {
            branches.push(self.primary()?);
        }
        Ok(if branches.len() == 1 {
            branches.pop().unwrap()
        } else {
            Node::And(branches)
        })
    }

    fn primary(&mut self) -> Result<Node> {
        if self.eat(&Token::OpenParen) {
            let node = self.or_expression()?;
            if !self.eat(&Token::CloseParen) {
                return Err(Error::new("expected ')'", self.here()));
            }
            return Ok(node);
        }
        self.constraint().map(Node::Compare)
    }

    fn constraint(&mut self) -> Result<Constraint> {
        let at = self.here();
        let field = match self.next().map(|s| &s.token) {
            Some(Token::Ident(name)) => name.clone(),
            // A value here means the user wrote something like `==x`, or a
            // bare word with no operator at all.
            Some(_) => return Err(Error::new("expected a field name", at)),
            None => return Err(Error::new("expected a field name", self.end)),
        };

        let op_at = self.here();
        let op = match self.next().map(|s| &s.token) {
            Some(Token::Op(op)) => *op,
            _ => {
                return Err(Error::new(
                    format!("expected an operator after '{field}'"),
                    op_at,
                ))
            }
        };

        if op.is_unary() {
            return Ok(Constraint {
                field,
                op,
                values: Vec::new(),
                at,
            });
        }

        let values = if self.eat(&Token::OpenParen) {
            let mut values = Vec::new();
            loop {
                let value_at = self.here();
                match self.next().map(|s| &s.token) {
                    Some(Token::Value(value)) => values.push(value.clone()),
                    Some(Token::Ident(word)) => values.push(word.clone()),
                    _ => return Err(Error::new("expected a value", value_at)),
                }
                // Inside a list a comma separates items rather than being OR.
                if !self.eat(&Token::Or) {
                    break;
                }
            }
            if !self.eat(&Token::CloseParen) {
                return Err(Error::new("expected ')'", self.here()));
            }
            values
        } else {
            let value_at = self.here();
            match self.next().map(|s| &s.token) {
                Some(Token::Value(value)) => vec![value.clone()],
                Some(Token::Ident(word)) => vec![word.clone()],
                _ => {
                    return Err(Error::new(
                        format!("expected a value after '{field}'"),
                        value_at,
                    ))
                }
            }
        };

        if op.is_list() && values.is_empty() {
            return Err(Error::new("expected at least one value", at));
        }
        if !op.is_list() && values.len() > 1 {
            return Err(Error::new(
                format!("'{field}' takes one value; use =in= for a list"),
                at,
            ));
        }
        Ok(Constraint {
            field,
            op,
            values,
            at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare(field: &str, op: Comparison, values: &[&str]) -> Node {
        Node::Compare(Constraint {
            field: field.into(),
            op,
            values: values.iter().map(|v| v.to_string()).collect(),
            at: 0,
        })
    }

    /// Compare structure while ignoring the recorded offsets.
    fn normalize(node: &Node) -> Node {
        match node {
            Node::And(children) => Node::And(children.iter().map(normalize).collect()),
            Node::Or(children) => Node::Or(children.iter().map(normalize).collect()),
            Node::Compare(c) => Node::Compare(Constraint { at: 0, ..c.clone() }),
        }
    }

    #[test]
    fn parses_a_single_comparison() {
        assert_eq!(
            normalize(&parse("artist==Radiohead").unwrap()),
            compare("artist", Comparison::Eq, &["Radiohead"])
        );
    }

    /// RSQL binds AND tighter than OR, so this must not come out as a flat list.
    #[test]
    fn and_binds_tighter_than_or() {
        assert_eq!(
            normalize(&parse("a==1,b==2;c==3").unwrap()),
            Node::Or(vec![
                compare("a", Comparison::Eq, &["1"]),
                Node::And(vec![
                    compare("b", Comparison::Eq, &["2"]),
                    compare("c", Comparison::Eq, &["3"]),
                ]),
            ])
        );
    }

    #[test]
    fn parentheses_override_precedence() {
        assert_eq!(
            normalize(&parse("(a==1,b==2);c==3").unwrap()),
            Node::And(vec![
                Node::Or(vec![
                    compare("a", Comparison::Eq, &["1"]),
                    compare("b", Comparison::Eq, &["2"]),
                ]),
                compare("c", Comparison::Eq, &["3"]),
            ])
        );
    }

    /// A `( … )` after an operator is a value list, not a nested group — the
    /// same comma means two different things depending on where it appears.
    #[test]
    fn parses_value_lists() {
        assert_eq!(
            normalize(&parse("genre=in=(rock,jazz,'trip hop')").unwrap()),
            compare("genre", Comparison::In, &["rock", "jazz", "trip hop"])
        );
    }

    #[test]
    fn parses_unary_operators() {
        assert_eq!(
            normalize(&parse("genre=null=").unwrap()),
            compare("genre", Comparison::Null, &[])
        );
    }

    #[test]
    fn collects_the_fields_used() {
        let node = parse("artist==X;(year>2000,genre==rock)").unwrap();
        assert_eq!(node.fields(), vec!["artist", "year", "genre"]);
    }

    #[test]
    fn rejects_malformed_filters() {
        for (input, at) in [
            ("artist", 6),   // no operator
            ("artist==", 8), // no value
            ("(artist==X", 10),
            ("==X", 0),
        ] {
            let e = parse(input).unwrap_err();
            assert_eq!(e.at, at, "{input}: {e}");
        }
        assert!(parse("   ").is_err());
    }
}
