//! Safe pure Shader (Lambert, Phong, PBR core).
//!
//! Computes a luminance from light and material.
//! No unwraps.
//!
//! Differential: Lambert is brighter with a facing light.

use crate::live_weights::LiveWeights;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderKind {
    Lambert,
    Phong,
    Pbr,
}

#[derive(Clone, Debug)]
pub struct Shader {
    kind: ShaderKind,
    shininess: f32,
}

impl Shader {
    pub fn new(kind: ShaderKind) -> Self {
        Self {
            kind,
            shininess: 32.0,
        }
    }

    pub fn kind(&self) -> ShaderKind {
        self.kind
    }

    pub fn set_shininess(&mut self, s: f32) {
        self.shininess = s.max(1.0);
    }

    /// light_dir and normal are unit vectors. Returns 0.0..=1.0.
    pub fn shade(
        &self,
        light_dir: (f32, f32, f32),
        normal: (f32, f32, f32),
        roughness: f32,
    ) -> f32 {
        let ndotl =
            (light_dir.0 * normal.0 + light_dir.1 * normal.1 + light_dir.2 * normal.2).max(0.0);
        match self.kind {
            ShaderKind::Lambert => ndotl,
            ShaderKind::Phong => {
                let spec = ndotl.powf(self.shininess);
                (ndotl * 0.7 + spec * 0.3).min(1.0)
            }
            ShaderKind::Pbr => {
                let rough = roughness.clamp(0.0, 1.0);
                (ndotl * (1.0 - rough * 0.5)).min(1.0)
            }
        }
    }

    /// Read shininess from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        if let Ok(s) = weights.get("shader.shininess") {
            self.set_shininess(s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let s = Shader::new(ShaderKind::Lambert);
        let facing = s.shade((0.0, 0.0, 1.0), (0.0, 0.0, 1.0), 0.5);
        let away = s.shade((0.0, 0.0, -1.0), (0.0, 0.0, 1.0), 0.5);
        assert!(facing > away);
        let mut p = Shader::new(ShaderKind::Phong);
        let mut w = LiveWeights::new();
        w.set("shader.shininess", 16.0).unwrap();
        p.from_weights(&w);
        assert_eq!(p.shininess, 16.0);
    }
}
