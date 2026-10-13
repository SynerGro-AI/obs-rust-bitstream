//! Safe pure Filter (image filter core).
//!
//! A filter owns a name and a kind (blur, sharpen, grayscale).
//! It can be applied to a sample. Empty names rejected.
//! No unwraps.
//!
//! Differential: grayscale of pure red is 0.3.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterKind {
    Blur,
    Sharpen,
    Grayscale,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Filter {
    name: String,
    kind: FilterKind,
    amount: f32,
}

impl Filter {
    pub fn new(name: &str, kind: FilterKind) -> Result<Self, FilterError> {
        if name.is_empty() {
            return Err(FilterError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            kind,
            amount: 1.0,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> FilterKind {
        self.kind
    }

    pub fn set_amount(&mut self, a: f32) {
        self.amount = a.clamp(0.0, 1.0);
    }

    pub fn amount(&self) -> f32 {
        self.amount
    }

    /// Apply the filter to an RGB sample.
    pub fn apply(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        match self.kind {
            FilterKind::Grayscale => {
                let y = 0.3 * r + 0.59 * g + 0.11 * b;
                let mix = self.amount;
                (
                    r * (1.0 - mix) + y * mix,
                    g * (1.0 - mix) + y * mix,
                    b * (1.0 - mix) + y * mix,
                )
            }
            FilterKind::Blur => {
                let avg = (r + g + b) / 3.0;
                let mix = self.amount * 0.5;
                (
                    r * (1.0 - mix) + avg * mix,
                    g * (1.0 - mix) + avg * mix,
                    b * (1.0 - mix) + avg * mix,
                )
            }
            FilterKind::Sharpen => {
                let avg = (r + g + b) / 3.0;
                let mix = self.amount * 0.5;
                (
                    (r + (r - avg) * mix).clamp(0.0, 1.0),
                    (g + (g - avg) * mix).clamp(0.0, 1.0),
                    (b + (b - avg) * mix).clamp(0.0, 1.0),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let f = Filter::new("gray", FilterKind::Grayscale).unwrap();
        assert_eq!(f.kind(), FilterKind::Grayscale);
    }

    #[test]
    fn empty_name() {
        assert!(Filter::new("", FilterKind::Blur).is_err());
    }

    #[test]
    fn differential() {
        let f = Filter::new("gray", FilterKind::Grayscale).unwrap();
        let (r, g, b) = f.apply(1.0, 0.0, 0.0);
        assert!((r - 0.3).abs() < 0.001);
        assert!((g - 0.3).abs() < 0.001);
        assert!((b - 0.3).abs() < 0.001);
    }
}
