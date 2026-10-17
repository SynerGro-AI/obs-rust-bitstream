//! Safe pure CurveTangent (interpolation type core).
//!
//! Holds a tangent type per key. Sample uses the type
//! to pick a value between keys. No unwraps.
//!
//! Differential: linear interpolates, stepped holds.

use crate::live_weights::LiveWeights;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tangent {
    Linear,
    Stepped,
    Flat,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TangentError {
    Empty,
}

#[derive(Clone, Debug, Default)]
pub struct CurveTangent {
    keys: Vec<(f32, f32, Tangent)>,
}

impl CurveTangent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, time: f32, value: f32, tangent: Tangent) {
        self.keys.push((time, value, tangent));
        self.keys.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Sample at time using the tangent of the previous key.
    pub fn sample(&self, time: f32) -> Result<f32, TangentError> {
        if self.keys.is_empty() {
            return Err(TangentError::Empty);
        }
        let mut prev = &self.keys[0];
        for k in &self.keys {
            if k.0 <= time {
                prev = k;
            } else {
                break;
            }
        }
        // Find next key.
        let next = self.keys.iter().find(|k| k.0 > prev.0);
        match (prev.2, next) {
            (Tangent::Stepped, _) => Ok(prev.1),
            (Tangent::Flat, _) => Ok(prev.1),
            (Tangent::Linear, Some(n)) => {
                let span = n.0 - prev.0;
                if span <= 0.0 {
                    return Ok(prev.1);
                }
                let t = (time - prev.0) / span;
                Ok(prev.1 + (n.1 - prev.1) * t.clamp(0.0, 1.0))
            }
            (Tangent::Linear, None) => Ok(prev.1),
        }
    }

    /// Read a tangent type from live weights (0=linear, 1=stepped, 2=flat).
    pub fn from_weights(
        &mut self,
        index: usize,
        weights: &LiveWeights,
    ) -> Result<(), TangentError> {
        if index >= self.keys.len() {
            return Err(TangentError::Empty);
        }
        let key = format!("tangent.{}", index);
        let v = weights.get(&key).map_err(|_| TangentError::Empty)?;
        self.keys[index].2 = match v as u8 {
            1 => Tangent::Stepped,
            2 => Tangent::Flat,
            _ => Tangent::Linear,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let c = CurveTangent::new();
        assert!(c.sample(0.5).is_err());
    }

    #[test]
    fn differential() {
        let mut c = CurveTangent::new();
        c.add(0.0, 0.0, Tangent::Linear);
        c.add(1.0, 10.0, Tangent::Linear);
        let mid = c.sample(0.5).unwrap();
        assert!((mid - 5.0).abs() < 0.01);
        c.add(2.0, 20.0, Tangent::Stepped);
        // stepped from key at 1.0 holds 10 until 2.0
        assert_eq!(c.sample(1.5).unwrap(), 10.0);
        let mut w = LiveWeights::new();
        w.set("tangent.0", 1.0).unwrap();
        c.from_weights(0, &w).unwrap();
        assert_eq!(c.sample(0.5).unwrap(), 0.0);
    }
}
