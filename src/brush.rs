//! Safe pure Brush (paint core).
//!
//! A brush owns a name, a radius, and a hardness. Empty names
//! rejected. No unwraps.
//!
//! Differential: set radius + falloff at center is 1.0.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrushError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Brush {
    name: String,
    radius: f32,
    hardness: f32,
}

impl Brush {
    pub fn new(name: &str) -> Result<Self, BrushError> {
        if name.is_empty() {
            return Err(BrushError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            radius: 10.0,
            hardness: 0.5,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_radius(&mut self, r: f32) {
        self.radius = r.max(0.1);
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn set_hardness(&mut self, h: f32) {
        self.hardness = h.clamp(0.0, 1.0);
    }

    pub fn hardness(&self) -> f32 {
        self.hardness
    }

    /// Falloff at a distance from the brush center. 1.0 at center.
    pub fn falloff(&self, distance: f32) -> f32 {
        if distance >= self.radius {
            return 0.0;
        }
        let t = distance / self.radius;
        let soft = 1.0 - t;
        soft * (1.0 - self.hardness) + self.hardness
    }

    /// Paint a stamp at (cx, cy) using radius and hardness.
    pub fn paint(
        &self,
        frame: &mut crate::video_frame::VideoFrame,
        cx: u32,
        cy: u32,
        color: u8,
    ) -> Result<(), BrushError> {
        if frame.format != crate::video_fourcc::VideoFormat::Y800 {
            return Err(BrushError::EmptyName);
        }
        let r = self.radius as i32;
        let w = frame.width as i32;
        let h = frame.height as i32;
        if let Some(plane) = frame.planes.get_mut(0) {
            for dy in -r..=r {
                for dx in -r..=r {
                    let x = cx as i32 + dx;
                    let y = cy as i32 + dy;
                    if x < 0 || y < 0 || x >= w || y >= h {
                        continue;
                    }
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    let fall = self.falloff(dist);
                    if fall <= 0.0 {
                        continue;
                    }
                    let i = (y as u32 * frame.width + x as u32) as usize;
                    if i < plane.len() {
                        let blended = (plane[i] as f32 * (1.0 - fall) + color as f32 * fall) as u8;
                        plane[i] = blended;
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let b = Brush::new("round").unwrap();
        assert_eq!(b.radius(), 10.0);
    }

    #[test]
    fn empty_name() {
        assert!(Brush::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut b = Brush::new("round").unwrap();
        b.set_radius(20.0);
        b.set_hardness(0.0);
        assert_eq!(b.falloff(0.0), 1.0);
        assert_eq!(b.falloff(20.0), 0.0);
        assert!(b.falloff(10.0) > 0.0);
    }

    #[test]
    fn paint() {
        use crate::video_fourcc::VideoFormat;
        use crate::video_frame::VideoFrame;
        let mut f = VideoFrame::new(VideoFormat::Y800, 8, 8).unwrap();
        let b = Brush::new("soft").unwrap();
        b.paint(&mut f, 4, 4, 200).unwrap();
        assert!(f.planes[0][4 * 8 + 4] > 0);
    }
}
