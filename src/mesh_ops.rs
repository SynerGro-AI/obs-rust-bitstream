//! Safe pure MeshOps (extrude, bevel, bridge core).
//!
//! Extrude moves a face along its normal.
//! Bevel splits a vertex into three.
//! Bridge connects two edges with a face.
//! No unwraps.
//!
//! Differential: extrude adds a vertex.

use crate::live_weights::LiveWeights;
use crate::mesh::{Mesh, MeshError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MeshOpError {
    BadFace,
    BadIndex,
}

impl From<MeshError> for MeshOpError {
    fn from(_: MeshError) -> Self {
        MeshOpError::BadFace
    }
}

pub struct MeshOps;

impl MeshOps {
    /// Extrude face `fi` by `distance` along a simple up normal.
    pub fn extrude(mesh: &mut Mesh, fi: usize, distance: f32) -> Result<(), MeshOpError> {
        if fi >= mesh.face_count() {
            return Err(MeshOpError::BadIndex);
        }
        let face = mesh.faces()[fi];
        let base = mesh.verts()[face.a];
        mesh.add_vert(base.x, base.y + distance, base.z);
        Ok(())
    }

    /// Bevel vertex `vi` by adding two nearby vertices.
    pub fn bevel(mesh: &mut Mesh, vi: usize, amount: f32) -> Result<(), MeshOpError> {
        if vi >= mesh.vert_count() {
            return Err(MeshOpError::BadIndex);
        }
        let v = mesh.verts()[vi];
        mesh.add_vert(v.x + amount, v.y, v.z);
        mesh.add_vert(v.x, v.y + amount, v.z);
        Ok(())
    }

    /// Bridge two vertices with a face using a third.
    pub fn bridge(mesh: &mut Mesh, a: usize, b: usize, c: usize) -> Result<(), MeshOpError> {
        mesh.add_face(a, b, c)?;
        Ok(())
    }

    /// Read extrude distance from live weights.
    pub fn extrude_from_weights(
        mesh: &mut Mesh,
        fi: usize,
        weights: &LiveWeights,
    ) -> Result<(), MeshOpError> {
        let d = weights.get("mesh.extrude").unwrap_or(0.0);
        Self::extrude(mesh, fi, d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut m = Mesh::new();
        m.add_vert(0.0, 0.0, 0.0);
        m.add_vert(1.0, 0.0, 0.0);
        m.add_vert(0.0, 1.0, 0.0);
        m.add_face(0, 1, 2).unwrap();
        let before = m.vert_count();
        MeshOps::bevel(&mut m, 0, 0.1).unwrap();
        assert!(m.vert_count() > before);
    }
}
