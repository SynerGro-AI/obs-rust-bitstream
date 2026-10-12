//! Safe pure profiler (OBS profiler.h core).
//!
//! The C version uses global state, threads, and snapshots.
//! The safe path is a pure local Profiler that records
//! start/end pairs and returns elapsed nanoseconds.
//! No globals, no unwraps, no panics on mismatched end.
//!
//! Differential: elapsed time is non-negative and ordered.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::diff::assert_equal;

#[derive(Clone, Debug, Default)]
pub struct Profiler {
    starts: HashMap<String, Instant>,
    totals: HashMap<String, Duration>,
    counts: HashMap<String, u64>,
}

impl Profiler {
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
        *self.counts.entry(name.to_string()).or_insert(0) += 1;
        Some(delta)
    }

    pub fn total(&self, name: &str) -> Duration {
        self.totals.get(name).copied().unwrap_or(Duration::ZERO)
    }

    pub fn count(&self, name: &str) -> u64 {
        self.counts.get(name).copied().unwrap_or(0)
    }

    pub fn average_ns(&self, name: &str) -> u64 {
        let c = self.count(name);
        if c == 0 {
            0
        } else {
            self.total(name).as_nanos() as u64 / c
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn start_end() {
        let mut p = Profiler::new();
        p.start("work");
        thread::sleep(StdDuration::from_millis(1));
        let d = p.end("work");
        assert!(d.is_some());
        assert!(p.total("work") >= StdDuration::from_millis(1));
        assert_eq!(p.count("work"), 1);
    }

    #[test]
    fn unmatched_end_safe() {
        let mut p = Profiler::new();
        assert!(p.end("missing").is_none());
        assert_eq!(p.count("missing"), 0);
    }

    #[test]
    fn average() {
        let mut p = Profiler::new();
        p.start("a");
        p.end("a");
        p.start("a");
        p.end("a");
        assert_eq!(p.count("a"), 2);
        assert!(p.average_ns("a") > 0);
    }

    #[test]
    fn differential() {
        let mut p = Profiler::new();
        p.start("x");
        let d = p.end("x").unwrap_or(Duration::ZERO);
        assert_equal(d.as_nanos() > 0, true, "profiler positive delta");
    }
}
