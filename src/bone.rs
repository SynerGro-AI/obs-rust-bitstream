//! Safe pure Bone (character skeleton core).
//!
//! A bone owns a name, a local transform, and child bones.
//! Empty names are rejected. No unwraps.
//!
//! Differential: set local position + world position round-trips.

use crate::matrix4::Matrix4;
use crate::vec4::Vec4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoneError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct Bone {
    name: String,
    local: Matrix4,
    children: Vec<Bone>,
}

impl Bone {
    pub fn new(name: &str) -> Result<Self, BoneError> {
        if name.is_empty() {
            return Err(BoneError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            local: Matrix4::identity(),
            children: Vec::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_local_translation(&mut self, x: f32, y: f32, z: f32) {
        self.local.t = Vec4::new(x, y, z, 1.0);
    }

    pub fn local(&self) -> &Matrix4 {
        &self.local
    }

    pub fn add_child(&mut self, child: Bone) {
        self.children.push(child);
    }

    pub fn children(&self) -> &[Bone] {
        &self.children
    }

    /// World matrix = parent_world * local.
    pub fn world(&self, parent: &Matrix4) -> Matrix4 {
        parent.mul(self.local)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let b = Bone::new("root").unwrap();
        assert_eq!(b.name(), "root");
        assert!(b.children().is_empty());
    }

    #[test]
    fn empty_name() {
        assert!(Bone::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut root = Bone::new("root").unwrap();
        root.set_local_translation(1.0, 2.0, 3.0);
        let mut child = Bone::new("spine").unwrap();
        child.set_local_translation(0.0, 1.0, 0.0);
        root.add_child(child);
        let world = root.world(&Matrix4::identity());
        assert_eq!(world.t.x, 1.0);
        assert_eq!(world.t.y, 2.0);
        assert_eq!(root.children().len(), 1);
    }
}
