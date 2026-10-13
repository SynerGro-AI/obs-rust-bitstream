//! Safe pure Organization (shared media output core).
//!
//! One organization owns the output. It can be an
//! individual or a team. Members have individual
//! profiles. Empty names rejected. No unwraps.
//!
//! Differential: add member + list round-trips.
//! A team and an individual both work.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrgError {
    EmptyName,
    Duplicate,
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrgMode {
    Individual,
    Team,
}

#[derive(Clone, Debug)]
pub struct MemberProfile {
    pub name: String,
    pub role: String,
}

#[derive(Clone, Debug)]
pub struct Organization {
    name: String,
    mode: OrgMode,
    members: BTreeMap<String, MemberProfile>,
}

impl Organization {
    pub fn new(name: &str) -> Result<Self, OrgError> {
        if name.is_empty() {
            return Err(OrgError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            mode: OrgMode::Individual,
            members: BTreeMap::new(),
        })
    }

    pub fn team(name: &str) -> Result<Self, OrgError> {
        let mut o = Self::new(name)?;
        o.mode = OrgMode::Team;
        Ok(o)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mode(&self) -> &OrgMode {
        &self.mode
    }

    pub fn is_team(&self) -> bool {
        self.mode == OrgMode::Team
    }

    /// Switch between individual and team.
    pub fn set_mode(&mut self, mode: OrgMode) {
        self.mode = mode;
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

    #[test]
    fn individual_or_team() {
        let solo = Organization::new("Jason").unwrap();
        assert!(!solo.is_team());
        let team = Organization::team("SynerGro Live").unwrap();
        assert!(team.is_team());
    }
}
