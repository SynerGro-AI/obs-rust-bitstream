//! Safe pure Marquee (rectangular and elliptical selection).
//!
//! Holds a selection rect. Drag to set. Clear to remove.
//! Zero-area selections are rejected. No unwraps.
//!
//! Differential: a different rect produces a different selection.

#[derive(Clone, Debug, PartialEq)]
pub enum MarqueeKind {
    Rect,
    Ellipse,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Marquee {
    kind: MarqueeKind,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Marquee {
    /// Create a selection. None if width or height is zero.
    pub fn new(kind: MarqueeKind, x: f32, y: f32, w: f32, h: f32) -> Option<Self> {
        if w == 0.0 || h == 0.0 {
            return None;
        }
        Some(Self { kind, x, y, w, h })
    }

    pub fn kind(&self) -> &MarqueeKind {
        &self.kind
    }

    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }

    pub fn area(&self) -> f32 {
        self.w.abs() * self.h.abs()
    }

    /// True if the point is inside the selection.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        match self.kind {
            MarqueeKind::Rect => {
                px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
            }
            MarqueeKind::Ellipse => {
                let cx = self.x + self.w / 2.0;
                let cy = self.y + self.h / 2.0;
                let rx = self.w / 2.0;
                let ry = self.h / 2.0;
                if rx == 0.0 || ry == 0.0 {
                    return false;
                }
                let dx = (px - cx) / rx;
                let dy = (py - cy) / ry;
                dx * dx + dy * dy <= 1.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains() {
        let m = Marquee::new(MarqueeKind::Rect, 0.0, 0.0, 10.0, 10.0).unwrap();
        assert!(m.contains(5.0, 5.0));
        assert!(!m.contains(15.0, 5.0));
    }

    #[test]
    fn ellipse_contains() {
        let m = Marquee::new(MarqueeKind::Ellipse, 0.0, 0.0, 10.0, 10.0).unwrap();
        assert!(m.contains(5.0, 5.0));
        assert!(!m.contains(0.0, 0.0));
    }

    #[test]
    fn zero_rejected() {
        assert!(Marquee::new(MarqueeKind::Rect, 0.0, 0.0, 0.0, 10.0).is_none());
    }

    #[test]
    fn differential() {
        let a = Marquee::new(MarqueeKind::Rect, 0.0, 0.0, 10.0, 10.0).unwrap();
        let b = Marquee::new(MarqueeKind::Rect, 0.0, 0.0, 20.0, 10.0).unwrap();
        assert_ne!(a.area(), b.area());
    }
}
