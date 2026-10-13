//! Safe pure Invite (team invite core).
//!
//! Creates an invite for a name and role. Empty names
//! rejected. No unwraps.
//!
//! Differential: create + accept round-trips.

use crate::organization::{Organization, OrgError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InviteError {
    EmptyName,
    AlreadyMember,
}

#[derive(Clone, Debug)]
pub struct Invite {
    pub name: String,
    pub role: String,
}

impl Invite {
    pub fn new(name: &str, role: &str) -> Result<Self, InviteError> {
        if name.is_empty() {
            return Err(InviteError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            role: role.to_string(),
        })
    }

    /// Accept the invite into an organization.
    pub fn accept(&self, org: &mut Organization) -> Result<(), OrgError> {
        org.add_member(&self.name, &self.role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(Invite::new("", "camera").is_err());
    }

    #[test]
    fn differential() {
        let invite = Invite::new("Alex", "camera").unwrap();
        let mut org = Organization::team("SynerGro Live").unwrap();
        invite.accept(&mut org).unwrap();
        assert_eq!(org.member("Alex").unwrap().role, "camera");
    }
}
