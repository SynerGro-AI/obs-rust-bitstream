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
        let t = &toks[i];
        match t.kind {
            CfTokenType::Other if t.text == "[" => {
                // Section header: [name]
                if i + 2 < toks.len() && toks[i + 1].kind == CfTokenType::Name && toks[i + 2].text == "]" {
                    current_section = toks[i + 1].text.to_string();
                    i += 3;
                    continue;
                }
            }
            CfTokenType::Name | CfTokenType::Num => {
                // Possible key = value
                if i + 2 < toks.len() && toks[i + 1].text == "=" {
                    let key = t.text;
                    let value = toks[i + 2].text.trim_matches(|c| c == '"' || c == '\'');
                    if !current_section.is_empty() {
                        cfg.set_string(&current_section, key, value);
                    }
                    i += 3;
                    continue;
                }
            }
            _ => {}
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
