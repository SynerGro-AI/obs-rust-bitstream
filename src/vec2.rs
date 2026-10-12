//! Safe pure Vec2 (OBS graphics/vec2.h core).
//!
//! The C version is a float pair with add/sub/mul/div/dot/length.
//! The safe path is a Copy struct with the same operations.
//! Division by zero returns None. No unwraps.
//!
//! Differential: add and dot match the mathematical definitions.

use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    pub fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }

    pub fn div(self, other: Self) -> Option<Self> {
        if other.x == 0.0 || other.y == 0.0 {
            None
        } else {
            Some(Self::new(self.x / other.x, self.y / other.y))
        }
    }

    pub fn mulf(self, f: f32) -> Self {
        Self::new(self.x * f, self.y * f)
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }

    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a.add(b), Vec2::new(4.0, 6.0));
        assert_eq!(b.sub(a), Vec2::new(2.0, 2.0));
    }

    #[test]
    fn div_safe() {
        let a = Vec2::new(4.0, 6.0);
        assert!(a.div(Vec2::ZERO).is_none());
        assert_eq!(a.div(Vec2::new(2.0, 3.0)), Some(Vec2::new(2.0, 2.0)));
    }

    #[test]
    fn dot_length() {
        let a = Vec2::new(3.0, 4.0);
        assert_eq!(a.dot(a), 25.0);
        assert!((a.length() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn differential() {
        let a = Vec2::new(1.0, 1.0);
        let b = Vec2::new(2.0, 3.0);
        assert_equal(a.add(b).x, 3.0, "vec2 add");
    }
}
