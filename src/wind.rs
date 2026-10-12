//! Safe pure Wind (environment force core).
//!
//! Applies an environment's wind to a bone's horizontal position.
//! No unwraps.
//!
//! Differential: a positive wind_x moves the bone right.

use crate::bone::Bone;
use crate::environment::Environment;

#[derive(Clone, Debug)]
pub struct Wind;

impl Wind {
    /// Apply the environment's wind_x to the bone for one step.
    pub fn apply(env: &Environment, bone: &mut Bone, dt: f32) {
        let x = bone.local().t.x + env.wind_x() * dt;
        bone.set_local_translation(x, bone.local().t.y, bone.local().t.z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushes_right() {
        let mut env = Environment::new("earth").unwrap();
        env.set_wind_x(10.0);
        let mut bone = Bone::new("plane").unwrap();
        Wind::apply(&env, &mut bone, 0.1);
        assert!((bone.local().t.x - 1.0).abs() < 0.001);
    }

    #[test]
    fn differential() {
        let mut env = Environment::new("earth").unwrap();
        env.set_wind_x(-5.0);
        let mut bone = Bone::new("plane").unwrap();
        Wind::apply(&env, &mut bone, 0.2);
        assert!((bone.local().t.x + 1.0).abs() < 0.001);
    }
}
