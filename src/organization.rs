//! Safe pure Organization (shared media output core).
//!
//! One organization owns the output. Members have
//! individual profiles. Empty names rejected. No unwraps.
//!
//! Differential: add member + list round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrgError {
    EmptyName,
    Duplicate,
    Missing,
}

#[derive(Clone, Debug)]
pub struct MemberProfile {
    pub name: String,
    pub role: String,
}

#[derive(Clone, Debug)]
pub struct Organization {
    name: String,
    members: BTreeMap<String, MemberProfile>,
}

impl Organization {
    pub fn new(name: &str) -> Result<Self, OrgError> {
        if name.is_empty() {
            return Err(OrgError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            members: BTreeMap::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add a member with an individual profile.
    pub fn add_member(&mut self, name: &str, role: &str) -> Result<(), OrgError> {
        if name.is_empty() {
            return Err(OrgError::EmptyName);
        }
        if self.members.contains_key(name) {
            return Err(OrgError::Duplicate);
        }
        self.members.insert(
            name.to_string(),
            MemberProfile {
                name: name.to_string(),
                role: role.to_string(),
            },
        );
        Ok(())
    }

    pub fn remove_member(&mut self, name: &str) -> Result<(), OrgError> {
        if self.members.remove(name).is_some() {
            Ok(())
        } else {
            Err(OrgError::Missing)
        }
    }

    pub fn member(&self, name: &str) -> Option<&MemberProfile> {
        self.members.get(name)
    }

    pub fn members(&self) -> Vec<&str> {
        self.members.keys().map(|s| s.as_str()).collect()
    }

    pub fn count(&self) -> usize {
        self.members.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(Organization::new("").is_err());
    }

    #[test]
    fn differential() {
        let mut org = Organization::new("SynerGro Live").unwrap();
        org.add_member("Jason", "host").unwrap();
        org.add_member("Alex", "camera").unwrap();
        assert!(org.add_member("Jason", "host").is_err());
        assert_eq!(org.count(), 2);
        assert_eq!(org.member("Alex").unwrap().role, "camera");
        org.remove_member("Alex").unwrap();
        assert_eq!(org.members(), vec!["Jason"]);
    }
}
