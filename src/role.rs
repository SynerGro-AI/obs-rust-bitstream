//! Safe pure Role (role list core).
//!
//! Lists the known roles and checks if a role is known.
//! No unwraps.
//!
//! Differential: host and camera are known, unknown is not.

pub struct Role;

impl Role {
    pub const HOST: &'static str = "host";
    pub const CAMERA: &'static str = "camera";
    pub const AUDIO: &'static str = "audio";
    pub const DIRECTOR: &'static str = "director";
    pub const GUEST: &'static str = "guest";

    pub fn known() -> &'static [&'static str] {
        &[Self::HOST, Self::CAMERA, Self::AUDIO, Self::DIRECTOR, Self::GUEST]
    }

    pub fn is_known(role: &str) -> bool {
        Self::known().contains(&role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        assert!(Role::is_known("host"));
        assert!(Role::is_known("camera"));
        assert!(!Role::is_known("unknown"));
        assert_eq!(Role::known().len(), 5);
    }
}
