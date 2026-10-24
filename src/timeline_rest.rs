//! Safe pure TimelineRest (Marker, Work Area, Time Remap, Keyframe Nav).
//!
//! Marker drops a named point. Work Area sets the render range.
//! Time Remap stores a speed key. Keyframe Nav finds the next
//! and previous key. No unwraps.
//!
//! Differential: a different time produces a different result.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub time: f32,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct WorkArea {
    pub start: f32,
    pub end: f32,
}

#[derive(Clone, Debug)]
pub struct TimeRemap {
    keys: BTreeMap<i32, f32>,
}

impl TimeRemap {
    pub fn new() -> Self {
        Self {
            keys: BTreeMap::new(),
        }
    }

    /// Add a speed key at time. 1.0 is normal. 0.0 is frozen.
    pub fn add_key(&mut self, time: f32, speed: f32) {
        let k = (time * 1000.0) as i32;
        self.keys.insert(k, speed.max(0.0));
    }

    /// Speed at time. Returns the nearest key at or before time, or 1.0.
    pub fn speed_at(&self, time: f32) -> f32 {
        let k = (time * 1000.0) as i32;
        self.keys
            .range(..=k)
            .next_back()
            .map(|(_, s)| *s)
            .unwrap_or(1.0)
    }

    pub fn key_count(&self) -> usize {
        self.keys.len()
    }
}

impl Default for TimeRemap {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TimelineRest;

impl TimelineRest {
    /// Drop a marker. Returns the marker.
    pub fn marker(time: f32, label: &str) -> Marker {
        Marker {
            time,
            label: label.to_string(),
        }
    }

    /// Set the work area. Returns it. End is clamped above start.
    pub fn work_area(start: f32, end: f32) -> WorkArea {
        WorkArea {
            start,
            end: end.max(start),
        }
    }

    /// True if time is inside the work area.
    pub fn in_work_area(area: &WorkArea, time: f32) -> bool {
        time >= area.start && time <= area.end
    }

    /// Next key at or after time. None if none.
    pub fn next_key(keys: &[f32], time: f32) -> Option<f32> {
        keys.iter().copied().find(|&k| k >= time)
    }

    /// Previous key at or before time. None if none.
    pub fn prev_key(keys: &[f32], time: f32) -> Option<f32> {
        keys.iter().copied().rev().find(|&k| k <= time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker() {
        let m = TimelineRest::marker(3.5, "beat");
        assert!((m.time - 3.5).abs() < 0.01);
        assert_eq!(m.label, "beat");
    }

    #[test]
    fn work_area() {
        let a = TimelineRest::work_area(1.0, 5.0);
        assert!(TimelineRest::in_work_area(&a, 3.0));
        assert!(!TimelineRest::in_work_area(&a, 6.0));
    }

    #[test]
    fn time_remap() {
        let mut r = TimeRemap::new();
        r.add_key(0.0, 1.0);
        r.add_key(2.0, 0.5);
        assert!((r.speed_at(1.0) - 1.0).abs() < 0.01);
        assert!((r.speed_at(3.0) - 0.5).abs() < 0.01);
        assert_eq!(r.key_count(), 2);
    }

    #[test]
    fn key_nav() {
        let keys = [1.0, 3.0, 5.0];
        assert_eq!(TimelineRest::next_key(&keys, 2.0), Some(3.0));
        assert_eq!(TimelineRest::prev_key(&keys, 2.0), Some(1.0));
        assert_eq!(TimelineRest::next_key(&keys, 6.0), None);
    }

    #[test]
    fn differential() {
        let mut a = TimeRemap::new();
        a.add_key(1.0, 2.0);
        let mut b = TimeRemap::new();
        b.add_key(1.0, 0.5);
        assert_ne!(a.speed_at(1.0), b.speed_at(1.0));
    }
}
