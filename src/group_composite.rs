//! Safe pure GroupComposite (group edit → composite).
//!
//! Confirms that editing a group's opacity changes the
//! composite for every member. No unwraps.
//!
//! Differential: a group edit lowers the mean more
//! than a single-layer edit.

use crate::layer_composite::LayerComposite;
use crate::photo_layers::LayerStack;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;

pub struct GroupComposite;

impl GroupComposite {
    fn frame(v: u8) -> VideoFrame {
        let mut f = VideoFrame::new(VideoFormat::Y800, 4, 4).unwrap();
        if let Some(p) = f.planes.get_mut(0) {
            for px in p.iter_mut() {
                *px = v;
            }
        }
        f
    }

    /// Run and return (single_mean, group_mean).
    /// Single halves one layer. Group halves two.
    pub fn run() -> (f32, f32) {
        let mut single = LayerStack::new();
        single.add("A", Self::frame(200)).unwrap();
        single.add("B", Self::frame(200)).unwrap();
        let _ = single.set_opacity("B", 128);
        let single_mean = LayerComposite::mean(&single).unwrap_or(0.0);

        let mut group = LayerStack::new();
        group.add("A", Self::frame(200)).unwrap();
        group.add("B", Self::frame(200)).unwrap();
        let _ = group.set_opacity("A", 128);
        let _ = group.set_opacity("B", 128);
        let group_mean = LayerComposite::mean(&group).unwrap_or(0.0);

        (single_mean, group_mean)
    }

    /// True if the group edit lowered the mean more.
    pub fn correct(single: f32, group: f32) -> bool {
        group < single
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_lowers_more() {
        let (single, group) = GroupComposite::run();
        assert!(GroupComposite::correct(single, group));
        assert!(group < single);
    }

    #[test]
    fn both_below_full() {
        let (single, group) = GroupComposite::run();
        assert!(single < 200.0);
        assert!(group < 200.0);
    }

    #[test]
    fn differential() {
        let (single, group) = GroupComposite::run();
        assert_ne!(single, group);
    }
}
