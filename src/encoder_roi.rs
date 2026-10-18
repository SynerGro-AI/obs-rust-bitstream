//! Safe pure EncoderRoi (OBS obs_encoder_roi).
//!
//! Parsed from libobs/obs-encoder.h.
//! Holds a rectangle and a priority between -1 and 1.
//! Inverted rectangles are rejected. Priority is clamped.
//! No unwraps.
//!
//! Differential: a higher priority is greater than a lower one.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoiError {
    Inverted,
}

#[derive(Clone, Debug)]
pub struct EncoderRoi {
    top: u32,
    bottom: u32,
    left: u32,
    right: u32,
    priority: f32,
}

impl EncoderRoi {
    /// Create a region. bottom must be greater than top, right greater than left.
    /// Priority is clamped to [-1, 1].
    pub fn new(top: u32, bottom: u32, left: u32, right: u32, priority: f32) -> Result<Self, RoiError> {
        if bottom <= top || right <= left {
            return Err(RoiError::Inverted);
        }
        Ok(Self {
            top,
            bottom,
            left,
            right,
            priority: priority.clamp(-1.0, 1.0),
        })
    }

    pub fn top(&self) -> u32 {
        self.top
    }

    pub fn bottom(&self) -> u32 {
        self.bottom
    }

    pub fn left(&self) -> u32 {
        self.left
    }

    pub fn right(&self) -> u32 {
        self.right
    }

    pub fn priority(&self) -> f32 {
        self.priority
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.right - self.left
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.bottom - self.top
    }

    /// Area in pixels.
    pub fn area(&self) -> u64 {
        self.width() as u64 * self.height() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let r = EncoderRoi::new(10, 50, 20, 80, 0.5).unwrap();
        assert_eq!(r.width(), 60);
        assert_eq!(r.height(), 40);
        assert_eq!(r.area(), 2400);
        assert_eq!(r.priority(), 0.5);
    }

    #[test]
    fn inverted_rejected() {
        assert!(EncoderRoi::new(50, 10, 0, 10, 0.0).is_err());
        assert!(EncoderRoi::new(0, 10, 80, 20, 0.0).is_err());
    }

    #[test]
    fn priority_clamped() {
        let r = EncoderRoi::new(0, 10, 0, 10, 2.0).unwrap();
        assert_eq!(r.priority(), 1.0);
        let r2 = EncoderRoi::new(0, 10, 0, 10, -3.0).unwrap();
        assert_eq!(r2.priority(), -1.0);
    }

    #[test]
    fn differential() {
        let a = EncoderRoi::new(0, 10, 0, 10, -0.5).unwrap();
        let b = EncoderRoi::new(0, 10, 0, 10, 0.5).unwrap();
        assert!(b.priority() > a.priority());
    }
}
