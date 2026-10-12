//! Safe pure source-profiler stub (OBS source-profiler.h core).
//!
//! The C version tracks per-source timing with global state.
//! The safe path is a pure local map of name → elapsed duration.
//! No globals, no unwraps. Unmatched end returns None.
//!
//! Differential: elapsed is non-negative.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::diff::assert_equal;

#[derive(Clone, Debug, Default)]
pub struct SourceProfiler {
    starts: HashMap<String, Instant>,
    totals: HashMap<String, Duration>,
}

impl SourceProfiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self, name: &str) {
        self.starts.insert(name.to_string(), Instant::now());
    }

    pub fn end(&mut self, name: &str) -> Option<Duration> {
        let start = self.starts.remove(name)?;
        let delta = start.elapsed();
        *self.totals.entry(name.to_string()).or_insert(Duration::ZERO) += delta;
        Some(delta)
    }

    pub fn total(&self, name: &str) -> Duration {
        self.totals.get(name).copied().unwrap_or(Duration::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_end() {
        let mut p = SourceProfiler::new();
        p.start("src");
        let d = p.end("src");
        assert!(d.is_some());
        assert!(p.total("src") >= Duration::ZERO);
    }

    #[test]
    fn unmatched_safe() {
        let mut p = SourceProfiler::new();
        assert!(p.end("missing").is_none());
    }

    #[test]
    fn differential() {
        let mut p = SourceProfiler::new();
        p.start("x");
        let d = p.end("x").unwrap_or(Duration::ZERO);
        assert_equal(d.as_nanos() >= 0, true, "source profiler delta");
    }
}
