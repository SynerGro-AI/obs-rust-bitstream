//! Safe pure Redo (re-apply undone action core).
//!
//! Reads the last undo from the log and returns the action
//! to re-apply. No unwraps.
//!
//! Differential: an undo of stop_record redoes to record.

use crate::log::Log;
use crate::undo::Undo;

pub struct Redo;

impl Redo {
    /// The action to re-apply, if the last entry is an undo.
    pub fn next(log: &Log) -> Option<String> {
        let last = log.last()?;
        let reverse = last.strip_prefix("undo: ")?;
        Undo::reverse(reverse).map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut l = Log::new();
        l.add("press r: record");
        l.add("undo: stop_record");
        assert_eq!(Redo::next(&l), Some("record".to_string()));
    }

    #[test]
    fn none() {
        let l = Log::new();
        assert!(Redo::next(&l).is_none());
    }
}
