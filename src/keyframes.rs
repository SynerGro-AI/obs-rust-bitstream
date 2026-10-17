//! Safe pure Keyframes (time-value pairs core).
//!
//! Holds named channels of (time, value) pairs.
//! Empty names rejected. No unwraps.
//!
//! Differential: add + sample at a key returns the value.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyError {
    EmptyName,
    Missing,
}

#[derive(Clone, Debug, Default)]
pub struct Keyframes {
    channels: BTreeMap<String, Vec<(u64, f32)>>,
}

impl Keyframes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, time: u64, value: f32) -> Result<(), KeyError> {
        if name.is_empty() {
            return Err(KeyError::EmptyName);
        }
        let ch = self.channels.entry(name.to_string()).or_default();
        ch.push((time, value));
        ch.sort_by_key(|k| k.0);
        Ok(())
    }

    /// Sample at time. Returns the last key at or before time.
    pub fn sample(&self, name: &str, time: u64) -> Result<f32, KeyError> {
        let ch = self.channels.get(name).ok_or(KeyError::Missing)?;
        let mut out = ch[0].1;
        for (t, v) in ch {
            if *t <= time {
                out = *v;
            } else {
                break;
            }
        }
        Ok(out)
    }

    pub fn count(&self, name: &str) -> usize {
        self.channels.get(name).map(|c| c.len()).unwrap_or(0)
    }

    pub fn channels(&self) -> Vec<&str> {
        self.channels.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut k = Keyframes::new();
        assert!(k.add("", 0, 1.0).is_err());
        assert!(k.sample("missing", 0).is_err());
    }

    #[test]
    fn differential() {
        let mut k = Keyframes::new();
        k.add("opacity", 0, 0.0).unwrap();
        k.add("opacity", 10, 1.0).unwrap();
        assert_eq!(k.sample("opacity", 0).unwrap(), 0.0);
        assert_eq!(k.sample("opacity", 10).unwrap(), 1.0);
        assert_eq!(k.sample("opacity", 5).unwrap(), 0.0);
    }
}
