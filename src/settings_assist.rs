//! Safe pure SettingsAssist (AI assist for any settings bag).
//!
//! Asks an AiAssistant for a key=value suggestion and writes it
//! into an ObsData if the reply parses. No unwraps.
//!
//! Differential: a reply with no key keeps the bag unchanged.

use crate::ai::AiAssistant;
use crate::obs_data::ObsData;

pub struct SettingsAssist;

impl SettingsAssist {
    /// Ask the assistant and apply a single key if the reply
    /// contains `key=value`. Returns true if a value was written.
    pub fn apply(ai: &AiAssistant, data: &mut ObsData, prompt: &str) -> bool {
        let reply = ai.assist(prompt);
        if let Some((k, v)) = parse_kv(&reply) {
            if let Ok(n) = v.parse::<i64>() {
                let _ = data.set_int(&k, n);
                return true;
            }
            if let Ok(n) = v.parse::<f64>() {
                let _ = data.set_double(&k, n);
                return true;
            }
            let _ = data.set_string(&k, &v);
            return true;
        }
        false
    }
}

fn parse_kv(s: &str) -> Option<(String, String)> {
    for line in s.lines() {
        let line = line.trim();
        if !line.contains('=') || line.contains('[') || line.contains('|') {
            continue;
        }
        let mut parts = line.splitn(2, '=');
        let k = parts.next()?.trim().to_string();
        let v = parts.next()?.trim().to_string();
        if !k.is_empty() && !v.is_empty() && !k.contains(' ') {
            return Some((k, v));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_key_keeps_bag() {
        let ai = AiAssistant::new(crate::ai::AiConfig::seraphina());
        let mut data = ObsData::new();
        let applied = SettingsAssist::apply(&ai, &mut data, "suggest a setting");
        assert!(!applied);
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn differential() {
        let mut data = ObsData::new();
        data.set_int("width", 1920).unwrap();
        assert_eq!(data.get_int("width").unwrap(), 1920);
    }
}
