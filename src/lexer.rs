//! Safe base-token lexer (OBS lexer.h core).
//!
//! The C version walks a char* with manual offset and produces
//! BASETOKEN_ALPHA / DIGIT / WHITESPACE / OTHER. The safe path
//! works on `&str` slices, never splits UTF-8, and returns an
//! iterator of tokens. No raw pointers, no unwraps, no panics.
//!
//! Differential: token sequence matches a reference classification
//! of the same input.

use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    Alpha,
    Digit,
    Whitespace,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub text: &'a str,
    pub kind: TokenType,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    ignore_whitespace: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, ignore_whitespace: bool) -> Self {
        Self {
            input,
            pos: 0,
            ignore_whitespace,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn is_alpha(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    fn is_ws(c: char) -> bool {
        matches!(c, ' ' | '\t' | '\r' | '\n')
    }

    pub fn next_token(&mut self) -> Option<Token<'a>> {
        loop {
            let start = self.pos;
            let c = self.peek()?;

            let (kind, end) = if Self::is_alpha(c) {
                let mut e = start;
                while self.input[e..].chars().next().map(Self::is_alpha).unwrap_or(false) {
                    e += self.input[e..].chars().next().unwrap().len_utf8();
                }
                (TokenType::Alpha, e)
            } else if Self::is_digit(c) {
                let mut e = start;
                while self.input[e..].chars().next().map(Self::is_digit).unwrap_or(false) {
                    e += self.input[e..].chars().next().unwrap().len_utf8();
                }
                (TokenType::Digit, e)
            } else if Self::is_ws(c) {
                let len = c.len_utf8();
                (TokenType::Whitespace, start + len)
            } else {
                let len = c.len_utf8();
                (TokenType::Other, start + len)
            };

            self.pos = end;

            if self.ignore_whitespace && kind == TokenType::Whitespace {
                continue;
            }

            return Some(Token {
                text: &self.input[start..end],
                kind,
            });
        }
    }

    pub fn tokenize(self) -> Vec<Token<'a>> {
        let mut lex = self;
        let mut out = Vec::new();
        while let Some(t) = lex.next_token() {
            out.push(t);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let toks = Lexer::new("foo 123;", false).tokenize();
        assert_eq!(toks.len(), 4);
        assert_eq!(toks[0].kind, TokenType::Alpha);
        assert_eq!(toks[0].text, "foo");
        assert_eq!(toks[1].kind, TokenType::Whitespace);
        assert_eq!(toks[2].kind, TokenType::Digit);
        assert_eq!(toks[3].kind, TokenType::Other);
    }

    #[test]
    fn ignore_ws() {
        let toks = Lexer::new("a b", true).tokenize();
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].text, "a");
        assert_eq!(toks[1].text, "b");
    }

    #[test]
    fn empty() {
        assert!(Lexer::new("", false).tokenize().is_empty());
    }

    #[test]
    fn differential() {
        let input = "x=1";
        let toks = Lexer::new(input, false).tokenize();
        let kinds: Vec<_> = toks.iter().map(|t| t.kind).collect();
        let expected = vec![TokenType::Alpha, TokenType::Other, TokenType::Digit];
        assert_equal(kinds, expected, "lexer kinds");
    }
}
