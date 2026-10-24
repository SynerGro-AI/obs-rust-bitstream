//! Safe pure FlightReplay (recorded flight playback core).
//!
//! Plays a FlightSession altitude trace back onto a bone so
//! the flight can be rendered. No unwraps.
//!
//! Differential: replay step 0 matches the first recorded altitude.

use crate::bone::Bone;
use crate::flight_session::FlightSession;

#[derive(Clone, Debug)]
pub struct FlightReplay {
    index: usize,
}

impl FlightReplay {
    pub fn new() -> Self {
        Self { index: 0 }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    /// Apply the altitude at the current index to the bone and
    /// advance. Returns false when the trace is exhausted.
    pub fn step(&mut self, session: &FlightSession, bone: &mut Bone) -> bool {
        if self.index >= session.len() {
            return false;
        }
        let y = session.altitudes()[self.index];
        bone.set_local_translation(bone.local().t.x, y, bone.local().t.z);
        self.index += 1;
        true
    }

    /// True when every recorded altitude has been applied.
    pub fn finished(&self, session: &FlightSession) -> bool {
        self.index >= session.len()
    }
}

impl Default for FlightReplay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;
    use crate::pilot::Pilot;

    #[test]
    fn replay_matches() {
        let mut session = FlightSession::new();
        let env = Environment::new("earth").unwrap();
        let pilot = Pilot::new(15.0);
        let mut plane = Bone::new("plane").unwrap();
        session.run(&env, &pilot, &mut plane, 3, 0.1);
        let mut replay = FlightReplay::new();
        let mut out = Bone::new("plane").unwrap();
        assert!(replay.step(&session, &mut out));
        assert_eq!(out.local().t.y, session.altitudes()[0]);
    }

    #[test]
    fn differential() {
        let mut session = FlightSession::new();
        let env = Environment::new("earth").unwrap();
        let pilot = Pilot::new(15.0);
        let mut plane = Bone::new("plane").unwrap();
        session.run(&env, &pilot, &mut plane, 2, 0.1);
        let mut replay = FlightReplay::new();
        let mut out = Bone::new("plane").unwrap();
        replay.step(&session, &mut out);
        replay.step(&session, &mut out);
        assert!(replay.finished(&session));
        assert_eq!(out.local().t.y, session.altitudes()[1]);
    }
}
