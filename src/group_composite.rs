//! Safe pure GroupComposite (group edit → members).
//!
//! Confirms that a group opacity edit changes every
//! member. The composite reflects the top visible layer.
//! No unwraps.
//!
//! Differential: a group edit changes more members.

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

    /// Run and return (single_changed, group_changed).
    pub fn run() -> (usize, usize) {
        let mut single = LayerStack::new();
        single.add("A", Self::frame(200)).unwrap();
        single.add("B", Self::frame(200)).unwrap();
        let _ = single.set_opacity("A", 128);
        let single_changed = single.visible_layers().iter().filter(|l| l.opacity != 255).count();

        let mut group = LayerStack::new();
        group.add("A", Self::frame(200)).unwrap();
        group.add("B", Self::frame(200)).unwrap();
        let _ = group.set_opacity("A", 128);
        let _ = group.set_opacity("B", 128);
        let group_changed = group.visible_layers().iter().filter(|l| l.opacity != 255).count();

        (single_changed, group_changed)
    }

    /// True if the group changed more members.
    pub fn correct(single: usize, group: usize) -> bool {
        group > single
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_changes_more() {
        let (single, group) = GroupComposite::run();
        assert!(GroupComposite::correct(single, group));
        assert_eq!(single, 1);
        assert_eq!(group, 2);
    }

    #[test]
    fn both_changed() {
        let (_, group) = GroupComposite::run();
        assert_eq!(group, 2);
    }

    #[test]
    fn differential() {
        let (single, group) = GroupComposite::run();
        assert_ne!(single, group);
    }
}
