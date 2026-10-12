//! Safe pure PlaneRenderer (flight-to-skeleton core).
//!
//! Builds a simple plane skeleton and drives its root from a
//! FlightReplay so the digital plane can be rendered.
//! No unwraps.
//!
//! Differential: after one replay step the root translation
//! matches the first recorded altitude.

use crate::bone::Bone;
use crate::character_renderer::CharacterRenderer;
use crate::flight_replay::FlightReplay;
use crate::flight_session::FlightSession;
use crate::render_item::RenderItem;

#[derive(Clone, Debug)]
pub struct PlaneRenderer {
    root: Bone,
}

impl PlaneRenderer {
    pub fn new() -> Self {
        let mut root = Bone::new("plane").expect("static name");
        if let Ok(wing) = Bone::new("wing") {
            root.add_child(wing);
        }
        if let Ok(tail) = Bone::new("tail") {
            root.add_child(tail);
        }
        Self { root }
    }

    pub fn root(&self) -> &Bone {
        &self.root
    }

    /// Apply one replay step to the plane root and return the
    /// render list of the connected skeleton.
    pub fn render_step(
        &mut self,
        session: &FlightSession,
        replay: &mut FlightReplay,
    ) -> Vec<RenderItem> {
        replay.step(session, &mut self.root);
        CharacterRenderer::new().render(&self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;
    use crate::pilot::Pilot;

    #[test]
    fn skeleton_has_children() {
        let r = PlaneRenderer::new();
        assert_eq!(r.root().children().len(), 2);
    }

    #[test]
    fn differential() {
        let mut session = FlightSession::new();
        let env = Environment::new("earth").unwrap();
        let pilot = Pilot::new(15.0);
        let mut plane = Bone::new("plane").unwrap();
        session.run(&env, &pilot, &mut plane, 2, 0.1);
        let mut replay = FlightReplay::new();
        let mut renderer = PlaneRenderer::new();
        let items = renderer.render_step(&session, &mut replay);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].translation().1, session.altitudes()[0]);
    }
}
