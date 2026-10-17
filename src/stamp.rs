//! Safe pure Brush (paint core).
//!
//! A brush owns a name, a radius, and a hardness. Empty names
//! rejected. No unwraps.
//!
//! Differential: set radius + falloff at center is 1.0.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StampError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Stamp {
    name: String,
    radius: f32,
    hardness: f32,
}

impl Stamp {
    pub fn new(name: &str) -> Result<Self, StampError> {
        if name.is_empty() {
            return Err(StampError::EmptyName);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let b = Stamp::new("round").unwrap();
        assert_eq!(b.radius(), 10.0);
    }

    #[test]
    fn empty_name() {
        assert!(Stamp::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut b = Stamp::new("round").unwrap();
        b.set_radius(20.0);
        b.set_hardness(0.0);
        assert_eq!(b.falloff(0.0), 1.0);
        assert_eq!(b.falloff(20.0), 0.0);
        assert!(b.falloff(10.0) > 0.0);
    }
}
