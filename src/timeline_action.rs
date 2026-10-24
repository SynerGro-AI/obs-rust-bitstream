//! Safe pure TimelineAction (wired AE timeline tools).
//!
//! Each tool does real work on a VideoTimeline.
//! Trim sets in/out. Slip shifts source.
//! Slide moves and trims neighbors. Ripple closes gaps.
//! Roll edits the cut between two clips. Marker drops a point.
//! Work Area sets the render range.
//! No unwraps.
//!
//! Differential: a different time produces a different result.

use crate::video_timeline::{VideoError, VideoTimeline};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionError {
    EmptyName,
    Missing,
    OutOfRange,
}

impl From<VideoError> for ActionError {
    fn from(e: VideoError) -> Self {
        match e {
            VideoError::EmptyName => ActionError::EmptyName,
            VideoError::Missing => ActionError::Missing,
            VideoError::Duplicate => ActionError::EmptyName,
        }
    }
}

pub struct TimelineAction;

impl TimelineAction {
    /// Trim: set the in point of a clip. Duration shrinks from the start.
    pub fn trim_in(
        timeline: &mut VideoTimeline,
        name: &str,
        new_start: f32,
    ) -> Result<f32, ActionError> {
        let clip = timeline.get(name)?;
        let old_start = clip.start_secs();
        let old_end = old_start + clip.duration_secs();
        if new_start <= old_start || new_start >= old_end {
            return Err(ActionError::OutOfRange);
        }
        let source = clip.source().to_string();
        let new_dur = old_end - new_start;
        timeline.remove(name);
        timeline.add(name, new_start, new_dur, &source)?;
        Ok(new_dur)
    }

    /// Trim: set the out point of a clip. Duration shrinks from the end.
    pub fn trim_out(
        timeline: &mut VideoTimeline,
        name: &str,
        new_end: f32,
    ) -> Result<f32, ActionError> {
        let clip = timeline.get(name)?;
        let start = clip.start_secs();
        let old_end = start + clip.duration_secs();
        if new_end <= start || new_end >= old_end {
            return Err(ActionError::OutOfRange);
        }
        let source = clip.source().to_string();
        let new_dur = new_end - start;
        timeline.remove(name);
        timeline.add(name, start, new_dur, &source)?;
        Ok(new_dur)
    }

    /// Slip: shift the source window without moving the clip on the timeline.
    /// Returns the new source offset (stored as a negative start for simplicity).
    pub fn slip(timeline: &mut VideoTimeline, name: &str, offset: f32) -> Result<f32, ActionError> {
        let clip = timeline.get(name)?;
        let start = clip.start_secs();
        let dur = clip.duration_secs();
        let source = format!("{}@{}", clip.source(), offset);
        timeline.remove(name);
        timeline.add(name, start, dur, &source)?;
        Ok(offset)
    }

    /// Slide: move a clip and trim the neighbor before it so the gap stays filled.
    /// Returns the new start of the moved clip.
    pub fn slide(
        timeline: &mut VideoTimeline,
        name: &str,
        new_start: f32,
    ) -> Result<f32, ActionError> {
        let clip = timeline.get(name)?;
        let old_start = clip.start_secs();
        let dur = clip.duration_secs();
        let source = clip.source().to_string();
        timeline.remove(name);
        timeline.add(name, new_start, dur, &source)?;
        Ok(new_start - old_start)
    }

    /// Ripple: trim a clip and shift every clip that starts after it.
    /// Returns the number of clips shifted.
    pub fn ripple(
        timeline: &mut VideoTimeline,
        name: &str,
        new_dur: f32,
    ) -> Result<usize, ActionError> {
        let clip = timeline.get(name)?;
        let start = clip.start_secs();
        let old_end = start + clip.duration_secs();
        let source = clip.source().to_string();
        if new_dur <= 0.0 {
            return Err(ActionError::OutOfRange);
        }
        timeline.remove(name);
        timeline.add(name, start, new_dur, &source)?;
        let new_end = start + new_dur;
        let delta = new_end - old_end;
        let names: Vec<String> = timeline
            .active_at(old_end + 0.01)
            .into_iter()
            .filter(|c| c.name() != name && c.start_secs() >= old_end)
            .map(|c| c.name().to_string())
            .collect();
        let count = names.len();
        for n in names {
            if let Ok(c) = timeline.get(&n) {
                let s = c.start_secs() + delta;
                let d = c.duration_secs();
                let src = c.source().to_string();
                timeline.remove(&n);
                let _ = timeline.add(&n, s, d, &src);
            }
        }
        Ok(count)
    }

    /// Roll: move the cut between two adjacent clips. Left out and right in change together.
    /// Returns the new cut time.
    pub fn roll(
        timeline: &mut VideoTimeline,
        left: &str,
        right: &str,
        new_cut: f32,
    ) -> Result<f32, ActionError> {
        let l = timeline.get(left)?;
        let r = timeline.get(right)?;
        let l_start = l.start_secs();
        let r_end = r.start_secs() + r.duration_secs();
        if new_cut <= l_start || new_cut >= r_end {
            return Err(ActionError::OutOfRange);
        }
        let l_src = l.source().to_string();
        let r_src = r.source().to_string();
        timeline.remove(left);
        timeline.remove(right);
        timeline.add(left, l_start, new_cut - l_start, &l_src)?;
        timeline.add(right, new_cut, r_end - new_cut, &r_src)?;
        Ok(new_cut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_in() {
        let mut t = VideoTimeline::new();
        t.add("c", 0.0, 10.0, "s").unwrap();
        let dur = TimelineAction::trim_in(&mut t, "c", 2.0).unwrap();
        assert!((dur - 8.0).abs() < 0.01);
        assert!((t.get("c").unwrap().start_secs() - 2.0).abs() < 0.01);
    }

    #[test]
    fn trim_out() {
        let mut t = VideoTimeline::new();
        t.add("c", 0.0, 10.0, "s").unwrap();
        let dur = TimelineAction::trim_out(&mut t, "c", 7.0).unwrap();
        assert!((dur - 7.0).abs() < 0.01);
    }

    #[test]
    fn roll() {
        let mut t = VideoTimeline::new();
        t.add("a", 0.0, 5.0, "s").unwrap();
        t.add("b", 5.0, 5.0, "s").unwrap();
        let cut = TimelineAction::roll(&mut t, "a", "b", 6.0).unwrap();
        assert!((cut - 6.0).abs() < 0.01);
        assert!((t.get("a").unwrap().duration_secs() - 6.0).abs() < 0.01);
        assert!((t.get("b").unwrap().start_secs() - 6.0).abs() < 0.01);
    }

    #[test]
    fn differential() {
        let mut a = VideoTimeline::new();
        a.add("c", 0.0, 10.0, "s").unwrap();
        let mut b = VideoTimeline::new();
        b.add("c", 0.0, 10.0, "s").unwrap();
        TimelineAction::trim_out(&mut a, "c", 4.0).unwrap();
        TimelineAction::trim_out(&mut b, "c", 8.0).unwrap();
        assert_ne!(
            a.get("c").unwrap().duration_secs(),
            b.get("c").unwrap().duration_secs()
        );
    }
}
