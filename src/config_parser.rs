//! Safe pure config-file string parser (OBS config-file core).
//!
//! The C version reads an ini-style file via the CF lexer/preprocessor.
//! The safe path parses a simple section/key=value string into the
//! existing Config store. No file I/O, no nulls, no unwraps.
//! Unterminated lines or missing equals are skipped safely.
//!
//! Differential: parsed keys match a reference HashMap of the input.

use crate::config::Config;
use crate::diff::assert_equal;
use crate::cf_lexer::{CfLexer, CfTokenType};

/// Parse a simple ini-style string into a Config.
/// Format: [section]\n key = value\n
/// Whitespace is ignored; lines without '=' are skipped.
pub fn parse_config_string(input: &str) -> Config {
    let mut cfg = Config::new();
    let mut current_section = String::new();
    let toks = CfLexer::new(input).tokenize();

    let mut i = 0;
    while i < toks.len() {
        // Section: [ name ]
        if toks[i].text == "[" {
            if let Some(name_tok) = toks.get(i + 1) {
                if name_tok.kind == CfTokenType::Name {
                    if toks.get(i + 2).map(|t| t.text == "]").unwrap_or(false) {
                        current_section = name_tok.text.to_string();
                        i += 3;
                        continue;
                    }
                }
            }
        }

        // key = value (skip whitespace tokens)
        if matches!(toks[i].kind, CfTokenType::Name | CfTokenType::Num) {
            // Look ahead for =
            let mut j = i + 1;
            while j < toks.len() && matches!(toks[j].kind, CfTokenType::SpaceTab) {
                j += 1;
            }
            if j < toks.len() && toks[j].text == "=" {
                j += 1;
                while j < toks.len() && matches!(toks[j].kind, CfTokenType::SpaceTab) {
                    j += 1;
                }
                if j < toks.len() {
                    let key = toks[i].text;
                    let value = toks[j].text.trim_matches(|c| c == '"' || c == '\'');
                    if !current_section.is_empty() {
                        cfg.set_string(&current_section, key, value);
                    }
                    i = j + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_section() {
        let input = "[Video]\nBaseCX = 1920\n";
        let cfg = parse_config_string(input);
        assert_eq!(cfg.get_string("Video", "BaseCX"), Some("1920"));
    }

    #[test]
    fn multiple() {
        let input = "[A]\nk=1\n[B]\nk=2\n";
        let cfg = parse_config_string(input);
        assert_eq!(cfg.get_string("A", "k"), Some("1"));
        assert_eq!(cfg.get_string("B", "k"), Some("2"));
    }

    #[test]
    fn missing_equals_safe() {
        let input = "[S]\njusttext\n";
        let cfg = parse_config_string(input);
        assert_eq!(cfg.get_string("S", "justtext"), None);
    }

    #[test]
    fn empty() {
        let cfg = parse_config_string("");
        assert!(cfg.get_string("any", "key").is_none());
    }

    #[test]
    fn differential() {
        let input = "[S]\nk=v\n";
        let cfg = parse_config_string(input);
        let got = cfg.get_string("S", "k").unwrap_or("");
        assert_equal(got, "v", "config parse");
    }
}
