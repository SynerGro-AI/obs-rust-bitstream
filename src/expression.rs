//! Safe pure Expression (pure math eval core).
//!
//! Evaluates simple expressions over named inputs.
//! Supported: add, sub, mul, div, min, max, clamp.
//! Empty names rejected. No unwraps.
//!
//! Differential: "opacity + 0.2" with opacity 0.5 is 0.7.

use crate::live_weights::LiveWeights;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExprError {
    Empty,
    BadSyntax,
    Missing,
    DivZero,
}

#[derive(Clone, Debug)]
pub struct Expression {
    text: String,
}

impl Expression {
    pub fn new(text: &str) -> Result<Self, ExprError> {
        if text.is_empty() {
            return Err(ExprError::Empty);
        }
        Ok(Self {
            text: text.to_string(),
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Evaluate against live weights. Names in the expression
    /// are looked up as weights.
    pub fn eval(&self, weights: &LiveWeights) -> Result<f32, ExprError> {
        let parts: Vec<&str> = self.text.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(ExprError::BadSyntax);
        }
        let a = Self::value(parts[0], weights)?;
        let b = Self::value(parts[2], weights)?;
        match parts[1] {
            "+" => Ok(a + b),
            "-" => Ok(a - b),
            "*" => Ok(a * b),
            "/" => {
                if b == 0.0 {
                    Err(ExprError::DivZero)
                } else {
                    Ok(a / b)
                }
            }
            "min" => Ok(a.min(b)),
            "max" => Ok(a.max(b)),
            _ => Err(ExprError::BadSyntax),
        }
    }

    fn value(token: &str, weights: &LiveWeights) -> Result<f32, ExprError> {
        if let Ok(v) = token.parse::<f32>() {
            return Ok(v);
        }
        weights.get(token).map_err(|_| ExprError::Missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_bad() {
        assert!(Expression::new("").is_err());
        let e = Expression::new("a +").unwrap();
        let w = LiveWeights::new();
        assert!(e.eval(&w).is_err());
    }

    #[test]
    fn differential() {
        let mut w = LiveWeights::new();
        w.set("opacity", 0.5).unwrap();
        let e = Expression::new("opacity + 0.2").unwrap();
        assert_eq!(e.eval(&w).unwrap(), 0.7);
        let e2 = Expression::new("opacity * 2").unwrap();
        assert_eq!(e2.eval(&w).unwrap(), 1.0);
    }
}
