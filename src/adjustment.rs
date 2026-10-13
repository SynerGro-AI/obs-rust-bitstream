//! Safe pure Adjustment (Photoshop-style adjustment layer core).
//!
//! An adjustment owns a name and a kind (brightness, contrast,
//! hue). It can be applied to a value. Empty names rejected.
//! No unwraps.
//!
//! Differential: brightness +0.2 raises a 0.5 sample to 0.7.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdjustKind {
    Brightness,
    Contrast,
    Hue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdjustError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Adjustment {
    name: String,
    kind: AdjustKind,
    amount: f32,
}

impl Adjustment {
    pub fn new(name: &str, kind: AdjustKind) -> Result<Self, AdjustError> {
        if name.is_empty() {
            return Err(AdjustError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            kind,
            amount: 0.0,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> AdjustKind {
        self.kind
    }

    pub fn set_amount(&mut self, a: f32) {
        self.amount = a.clamp(-1.0, 1.0);
    }

    pub fn amount(&self) -> f32 {
        self.amount
    }

    /// Apply the adjustment to a 0-1 sample.
    pub fn apply(&self, sample: f32) -> f32 {
        match self.kind {
            AdjustKind::Brightness => (sample + self.amount).clamp(0.0, 1.0),
            AdjustKind::Contrast => {
                let mid = sample - 0.5;
                (0.5 + mid * (1.0 + self.amount)).clamp(0.0, 1.0)
            }
            AdjustKind::Hue => sample,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let a = Adjustment::new("bright", AdjustKind::Brightness).unwrap();
        assert_eq!(a.kind(), AdjustKind::Brightness);
    }

    #[test]
    fn empty_name() {
        assert!(Adjustment::new("", AdjustKind::Contrast).is_err());
    }

    #[test]
    fn differential() {
        let mut a = Adjustment::new("bright", AdjustKind::Brightness).unwrap();
        a.set_amount(0.2);
        assert!((a.apply(0.5) - 0.7).abs() < 0.001);
    }
}
