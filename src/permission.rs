//! Safe pure Permission (role action core).
//!
//! Maps a role to the actions it can do. No unwraps.
//!
//! Differential: host can record, guest cannot.

use crate::role::Role;

pub struct Permission;

impl Permission {
    /// Can this role do this action.
    pub fn can(role: &str, action: &str) -> bool {
        match role {
            Role::HOST => matches!(action, "record" | "stream" | "vcam" | "switch" | "invite"),
            Role::DIRECTOR => matches!(action, "switch" | "invite"),
            Role::CAMERA => matches!(action, "switch"),
            Role::AUDIO => matches!(action, "mute"),
            Role::GUEST => false,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        assert!(Permission::can("host", "record"));
        assert!(Permission::can("camera", "switch"));
        assert!(!Permission::can("guest", "record"));
        assert!(!Permission::can("unknown", "record"));
    }
}
