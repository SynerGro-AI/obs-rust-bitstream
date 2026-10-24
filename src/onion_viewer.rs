//! Safe pure OnionViewer (onion-skin edit viewer core).
//!
//! Shows the current frame plus faded previous and next frames
//! so an editor can see motion. No unwraps.
//!
//! Differential: a 3-frame clip at frame 1 returns prev, current,
//! and next with the expected opacities.

#[derive(Clone, Debug)]
pub struct OnionFrame {
    pub index: i32,
    pub opacity: f32,
}

#[derive(Clone, Debug)]
pub struct OnionViewer {
    skin_opacity: f32,
}

impl OnionViewer {
    pub fn new() -> Self {
        Self { skin_opacity: 0.3 }
    }

    pub fn set_skin_opacity(&mut self, o: f32) {
        self.skin_opacity = o.clamp(0.0, 1.0);
    }

    pub fn skin_opacity(&self) -> f32 {
        self.skin_opacity
    }

    /// Build the onion stack for `current` inside a clip of `count` frames.
    /// Previous and next are included when they exist, at skin opacity.
    pub fn view(&self, current: i32, count: i32) -> Vec<OnionFrame> {
        let mut frames = Vec::new();
        if current > 0 {
            frames.push(OnionFrame {
                index: current - 1,
                opacity: self.skin_opacity,
            });
        }
        if current >= 0 && current < count {
            frames.push(OnionFrame {
                index: current,
                opacity: 1.0,
            });
        }
        if current + 1 < count {
            frames.push(OnionFrame {
                index: current + 1,
                opacity: self.skin_opacity,
            });
        }
        frames
    }
}

impl Default for OnionViewer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let v = OnionViewer::new();
        assert_eq!(v.skin_opacity(), 0.3);
    }

    #[test]
    fn differential() {
        let v = OnionViewer::new();
        let frames = v.view(1, 3);
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].index, 0);
        assert_eq!(frames[0].opacity, 0.3);
        assert_eq!(frames[1].index, 1);
        assert_eq!(frames[1].opacity, 1.0);
        assert_eq!(frames[2].index, 2);
    }

    #[test]
    fn edges() {
        let v = OnionViewer::new();
        assert_eq!(v.view(0, 3).len(), 2);
        assert_eq!(v.view(2, 3).len(), 2);
    }
}
