//! Safe pure KeyframeGroup (OBS keyframe_group_data).
//!
//! Parsed from libobs/obs-internal.h.
//! Tracks which video tracks have seen a keyframe at a pts.
//! A group is complete when all required tracks are seen.
//! No unwraps.
//!
//! Differential: a group with more seen tracks is closer to complete.

/// Matching OBS keyframe_group_track_status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackStatus {
    NotSeen,
    Seen,
    Skipped,
}

/// Matching OBS MAX_OUTPUT_VIDEO_ENCODERS.
pub const MAX_VIDEO_TRACKS: usize = 10;

#[derive(Clone, Debug)]
pub struct KeyframeGroup {
    id: u64,
    pts: i64,
    required: u32,
    seen: [TrackStatus; MAX_VIDEO_TRACKS],
}

impl KeyframeGroup {
    /// Create a group. required is clamped to MAX_VIDEO_TRACKS.
    pub fn new(id: u64, pts: i64, required: u32) -> Self {
        Self {
            id,
            pts,
            required: required.min(MAX_VIDEO_TRACKS as u32),
            seen: [TrackStatus::NotSeen; MAX_VIDEO_TRACKS],
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn pts(&self) -> i64 {
        self.pts
    }

    pub fn required(&self) -> u32 {
        self.required
    }

    /// Mark a track as seen. Out-of-range index is ignored.
    pub fn mark_seen(&mut self, track: usize) {
        if track < MAX_VIDEO_TRACKS {
            self.seen[track] = TrackStatus::Seen;
        }
    }

    /// Mark a track as skipped. Out-of-range index is ignored.
    pub fn mark_skipped(&mut self, track: usize) {
        if track < MAX_VIDEO_TRACKS {
            self.seen[track] = TrackStatus::Skipped;
        }
    }

    pub fn status(&self, track: usize) -> TrackStatus {
        self.seen.get(track).copied().unwrap_or(TrackStatus::NotSeen)
    }

    /// Count of tracks that are Seen or Skipped.
    pub fn done_count(&self) -> usize {
        self.seen.iter().take(self.required as usize).filter(|s| {
            matches!(**s, TrackStatus::Seen | TrackStatus::Skipped)
        }).count()
    }

    /// True when all required tracks are Seen or Skipped.
    pub fn is_complete(&self) -> bool {
        self.done_count() == self.required as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_when_all_seen() {
        let mut g = KeyframeGroup::new(1, 100, 2);
        assert!(!g.is_complete());
        g.mark_seen(0);
        g.mark_seen(1);
        assert!(g.is_complete());
        assert_eq!(g.done_count(), 2);
    }

    #[test]
    fn skip_counts() {
        let mut g = KeyframeGroup::new(1, 100, 2);
        g.mark_seen(0);
        g.mark_skipped(1);
        assert!(g.is_complete());
    }

    #[test]
    fn out_of_range() {
        let mut g = KeyframeGroup::new(1, 0, 1);
        g.mark_seen(100);
        assert_eq!(g.status(0), TrackStatus::NotSeen);
    }

    #[test]
    fn differential() {
        let mut a = KeyframeGroup::new(1, 0, 3);
        let mut b = KeyframeGroup::new(2, 0, 3);
        a.mark_seen(0);
        b.mark_seen(0);
        b.mark_seen(1);
        assert!(b.done_count() > a.done_count());
    }
}
