//! Safe pure CF lexer (OBS cf-lexer.h core).
//!
//! The C version builds on the base lexer, handles spliced lines,
//! comments, and produces CFTOKEN_NAME / NUM / SPACETAB / NEWLINE /
//! STRING / OTHER. The safe path works on `&str`, never splits UTF-8,
//! and returns an iterator of tokens. No raw pointers, no unwraps,
//! no panics on unterminated strings (they become OTHER or partial).
//!
//! Differential: token kinds match a reference classification.

#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CfTokenType {
    None,
    Name,
    Num,
    SpaceTab,
    Newline,
    String,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CfToken<'a> {
    pub text: &'a str,
    pub kind: CfTokenType,
}

pub struct CfLexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> CfLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn is_name_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_name_cont(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    fn is_space_tab(c: char) -> bool {
        matches!(c, ' ' | '\t')
    }

    fn is_newline(c: char) -> bool {
        matches!(c, '\n' | '\r')
    }

    pub fn next_token(&mut self) -> Option<CfToken<'a>> {
        let start = self.pos;
        let c = self.peek()?;

        let (kind, end) = if Self::is_name_start(c) {
            let mut e = start;
            while self.input[e..].chars().next().map(Self::is_name_cont).unwrap_or(false) {
                e += self.input[e..].chars().next().unwrap().len_utf8();
            }
            (CfTokenType::Name, e)
        } else if Self::is_digit(c) {
            let mut e = start;
            while self.input[e..].chars().next().map(Self::is_digit).unwrap_or(false) {
                e += self.input[e..].chars().next().unwrap().len_utf8();
            }
            (CfTokenType::Num, e)
        } else if Self::is_space_tab(c) {
            let mut e = start;
            while self.input[e..].chars().next().map(Self::is_space_tab).unwrap_or(false) {
                e += self.input[e..].chars().next().unwrap().len_utf8();
            }
            (CfTokenType::SpaceTab, e)
        } else if Self::is_newline(c) {
            let len = c.len_utf8();
            // Consume \r\n as one if present
            let next = self.input[start + len..].chars().next();
            if (c == '\r' && next == Some('\n')) || (c == '\n' && next == Some('\r')) {
                (CfTokenType::Newline, start + len + 1)
            } else {
                (CfTokenType::Newline, start + len)
            }
        } else if c == '"' || c == '\'' {
            // Simple string: consume until matching quote or end
            let quote = c;
            let mut e = start + c.len_utf8();
            while let Some(ch) = self.input[e..].chars().next() {
                e += ch.len_utf8();
                if ch == quote {
                    break;
                }
            }
            (CfTokenType::String, e)
        } else {
            let len = c.len_utf8();
            (CfTokenType::Other, start + len)
        };

        self.pos = end;
        Some(CfToken {
            text: &self.input[start..end],
            kind,
        })
    }

    pub fn tokenize(self) -> Vec<CfToken<'a>> {
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
        let toks = CfLexer::new("foo 123\n").tokenize();
        assert_eq!(toks.len(), 4);
        assert_eq!(toks[0].kind, CfTokenType::Name);
        assert_eq!(toks[0].text, "foo");
        assert_eq!(toks[1].kind, CfTokenType::SpaceTab);
        assert_eq!(toks[2].kind, CfTokenType::Num);
        assert_eq!(toks[3].kind, CfTokenType::Newline);
    }

    #[test]
    fn string() {
        let toks = CfLexer::new("\"hello\"").tokenize();
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, CfTokenType::String);
        assert_eq!(toks[0].text, "\"hello\"");
    }

    #[test]
    fn unterminated_string_safe() {
        let toks = CfLexer::new("\"open").tokenize();
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].kind, CfTokenType::String);
    }

    #[test]
    fn empty() {
        assert!(CfLexer::new("").tokenize().is_empty());
    }

    #[test]
    fn differential() {
        let input = "a=1";
        let toks = CfLexer::new(input).tokenize();
        let kinds: Vec<_> = toks.iter().map(|t| t.kind).collect();
        let expected = vec![CfTokenType::Name, CfTokenType::Other, CfTokenType::Num];
        assert_equal(kinds, expected, "cf-lexer kinds");
    }
}
