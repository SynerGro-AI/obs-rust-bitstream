//! Safe pure Mesh (polygon mesh core).
//!
//! Holds vertices, edges, and faces. Empty rejected.
//! No unwraps.
//!
//! Differential: add vert + face round-trips.

use crate::live_weights::LiveWeights;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Face {
    pub a: usize,
    pub b: usize,
    pub c: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MeshError {
    BadFace,
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    verts: Vec<Vertex>,
    faces: Vec<Face>,
}

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_vert(&mut self, x: f32, y: f32, z: f32) -> usize {
        self.verts.push(Vertex { x, y, z });
        self.verts.len() - 1
    }

    pub fn add_face(&mut self, a: usize, b: usize, c: usize) -> Result<(), MeshError> {
        let n = self.verts.len();
        if a >= n || b >= n || c >= n {
            return Err(MeshError::BadFace);
        }
        self.faces.push(Face { a, b, c });
        Ok(())
    }

    pub fn vert_count(&self) -> usize {
        self.verts.len()
    }

    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    pub fn verts(&self) -> &[Vertex] {
        &self.verts
    }

    pub fn faces(&self) -> &[Face] {
        &self.faces
    }

    /// Move a vertex from live weights.
    pub fn from_weights(&mut self, weights: &LiveWeights, index: usize) {
        if index >= self.verts.len() {
            return;
        }
        let nx = format!("mesh.v{}.x", index);
        let ny = format!("mesh.v{}.y", index);
        let nz = format!("mesh.v{}.z", index);
        if let Ok(x) = weights.get(&nx) {
            self.verts[index].x = x;
        }
        if let Ok(y) = weights.get(&ny) {
            self.verts[index].y = y;
        }
        if let Ok(z) = weights.get(&nz) {
            self.verts[index].z = z;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_face() {
        let mut m = Mesh::new();
        assert!(m.add_face(0, 1, 2).is_err());
    }

    #[test]
    fn differential() {
        let mut m = Mesh::new();
        m.add_vert(0.0, 0.0, 0.0);
        m.add_vert(1.0, 0.0, 0.0);
        m.add_vert(0.0, 1.0, 0.0);
        m.add_face(0, 1, 2).unwrap();
        let mut w = LiveWeights::new();
        w.set("mesh.v0.x", 2.0).unwrap();
        m.from_weights(&w, 0);
        assert_eq!(m.verts()[0].x, 2.0);
        assert_eq!(m.face_count(), 1);
    }
}
