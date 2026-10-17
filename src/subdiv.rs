//! Safe pure Subdiv (mesh subdivision core).
//!
//! Splits each edge by adding a midpoint vertex.
//! No unwraps.
//!
//! Differential: one subdivision increases the vertex count.

use crate::live_weights::LiveWeights;
use crate::mesh::Mesh;

pub struct Subdiv;

impl Subdiv {
    /// Add a midpoint for each consecutive pair of vertices.
    pub fn once(mesh: &mut Mesh) {
        let verts = mesh.verts().to_vec();
        if verts.len() < 2 {
            return;
        }
        for i in 0..verts.len() - 1 {
            let a = verts[i];
            let b = verts[i + 1];
            mesh.add_vert(
                (a.x + b.x) * 0.5,
                (a.y + b.y) * 0.5,
                (a.z + b.z) * 0.5,
            );
        }
    }

    /// Read the number of passes from live weights.
    pub fn from_weights(mesh: &mut Mesh, weights: &LiveWeights) {
        let n = weights.get("subdiv.passes").unwrap_or(1.0) as u32;
        for _ in 0..n.max(1).min(4) {
            Self::once(mesh);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut m = Mesh::new();
        m.add_vert(0.0, 0.0, 0.0);
        m.add_vert(2.0, 0.0, 0.0);
        let before = m.vert_count();
        Subdiv::once(&mut m);
        assert!(m.vert_count() > before);
        assert_eq!(m.verts()[2].x, 1.0);
    }
}
