//! Safe pure KeyframeStatus (group completion report).
//!
//! One line of required tracks, done count, and complete.
//! No unwraps.
//!
//! Differential: more seen tracks produce a higher done count.

use crate::keyframe_group::KeyframeGroup;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyframeStatus {
    pub required: u32,
    pub done: usize,
    pub complete: bool,
}

impl KeyframeStatus {
    /// Build from a KeyframeGroup.
    pub fn from(group: &KeyframeGroup) -> Self {
        Self {
            required: group.required(),
            done: group.done_count(),
            complete: group.is_complete(),
        }
    }

    /// One line: "required=N done=N complete=true/false"
    pub fn one_line(&self) -> String {
        format!(
            "required={} done={} complete={}",
            self.required, self.done, self.complete
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports() {
        let mut g = KeyframeGroup::new(1, 0, 2);
        g.mark_seen(0);
        let s = KeyframeStatus::from(&g);
        assert_eq!(s.required, 2);
        assert_eq!(s.done, 1);
        assert!(!s.complete);
        assert!(s.one_line().contains("done=1"));
    }

    #[test]
    fn complete() {
        let mut g = KeyframeGroup::new(1, 0, 1);
        g.mark_seen(0);
        let s = KeyframeStatus::from(&g);
        assert!(s.complete);
    }

    #[test]
    fn differential() {
        let mut a = KeyframeGroup::new(1, 0, 2);
        let mut b = KeyframeGroup::new(2, 0, 2);
        a.mark_seen(0);
        b.mark_seen(0);
        b.mark_seen(1);
        let sa = KeyframeStatus::from(&a);
        let sb = KeyframeStatus::from(&b);
        assert!(sb.done > sa.done);
    }
}
