//! Safe pure Projection (perspective and ortho core).
//!
//! Builds a simple projection matrix. No unwraps.
//!
//! Differential: perspective and ortho differ.

use crate::live_weights::LiveWeights;
use crate::matrix4::Matrix4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionKind {
    Perspective,
    Orthographic,
}

#[derive(Clone, Debug)]
pub struct Projection {
    kind: ProjectionKind,
    fov: f32,
    near: f32,
    far: f32,
    size: f32,
}

impl Projection {
    pub fn perspective(fov: f32, near: f32, far: f32) -> Self {
        Self {
            kind: ProjectionKind::Perspective,
            fov,
            near,
            far,
            size: 1.0,
        }
    }

    pub fn orthographic(size: f32, near: f32, far: f32) -> Self {
        Self {
            kind: ProjectionKind::Orthographic,
            fov: 0.0,
            near,
            far,
            size,
        }
    }

    pub fn kind(&self) -> ProjectionKind {
        self.kind
    }

    /// Build a simple projection matrix.
    pub fn matrix(&self) -> Matrix4 {
        match self.kind {
            ProjectionKind::Perspective => {
                let f = 1.0 / (self.fov * 0.5).tan();
                Matrix4::from_columns(
                    crate::vec4::Vec4::new(f, 0.0, 0.0, 0.0),
                    crate::vec4::Vec4::new(0.0, f, 0.0, 0.0),
                    crate::vec4::Vec4::new(0.0, 0.0, (self.far + self.near) / (self.near - self.far), -1.0),
                    crate::vec4::Vec4::new(0.0, 0.0, (2.0 * self.far * self.near) / (self.near - self.far), 0.0),
                )
            }
            ProjectionKind::Orthographic => {
                let s = 1.0 / self.size.max(0.001);
                Matrix4::from_columns(
                    crate::vec4::Vec4::new(s, 0.0, 0.0, 0.0),
                    crate::vec4::Vec4::new(0.0, s, 0.0, 0.0),
                    crate::vec4::Vec4::new(0.0, 0.0, -2.0 / (self.far - self.near), 0.0),
                    crate::vec4::Vec4::new(0.0, 0.0, -(self.far + self.near) / (self.far - self.near), 1.0),
                )
            }
        }
    }

    /// Read fov or size from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights) {
        match self.kind {
            ProjectionKind::Perspective => {
                if let Ok(f) = weights.get("camera.fov") {
                    self.fov = f.max(0.01);
                }
            }
            ProjectionKind::Orthographic => {
                if let Ok(s) = weights.get("camera.size") {
                    self.size = s.max(0.01);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut persp = Projection::perspective(1.0, 0.1, 100.0);
        let ortho = Projection::orthographic(2.0, 0.1, 100.0);
        assert_ne!(persp.kind(), ortho.kind());
        let mut w = LiveWeights::new();
        w.set("camera.fov", 1.5).unwrap();
        persp.from_weights(&w);
        assert_eq!(persp.fov, 1.5);
    }
}
