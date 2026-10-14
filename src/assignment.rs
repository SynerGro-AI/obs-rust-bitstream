//! Safe pure Assignment (member claims a resource).
//!
//! A signed-in member claims a camera or source by name.
//! Empty names rejected. No unwraps.
//!
//! Differential: claim + release round-trips.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssignmentError {
    EmptyName,
    AlreadyClaimed,
    NotClaimed,
}

#[derive(Clone, Debug)]
pub struct Assignment {
    claims: BTreeMap<String, String>, // resource -> member
}

impl Default for Assignment {
    fn default() -> Self {
        Self {
            claims: BTreeMap::new(),
        }
    }
}

impl Assignment {
    pub fn new() -> Self {
        Self::default()
    }

    /// Claim a resource for a member. Returns error if already claimed by someone else.
    pub fn claim(&mut self, resource: &str, member: &str) -> Result<(), AssignmentError> {
        if resource.is_empty() || member.is_empty() {
            return Err(AssignmentError::EmptyName);
        }
        if let Some(existing) = self.claims.get(resource) {
            if existing != member {
                return Err(AssignmentError::AlreadyClaimed);
            }
        }
        self.claims.insert(resource.to_string(), member.to_string());
        Ok(())
    }

    /// Release a resource. Only the claiming member can release it.
    pub fn release(&mut self, resource: &str, member: &str) -> Result<(), AssignmentError> {
        if resource.is_empty() || member.is_empty() {
            return Err(AssignmentError::EmptyName);
        }
        match self.claims.get(resource) {
            Some(existing) if existing == member => {
                self.claims.remove(resource);
                Ok(())
            }
            Some(_) => Err(AssignmentError::AlreadyClaimed),
            None => Err(AssignmentError::NotClaimed),
        }
    }

    pub fn owner(&self, resource: &str) -> Option<&str> {
        self.claims.get(resource).map(|s| s.as_str())
    }

    pub fn claimed_by(&self, member: &str) -> Vec<&str> {
        self.claims
            .iter()
            .filter(|(_, m)| *m == member)
            .map(|(r, _)| r.as_str())
            .collect()
    }

    pub fn count(&self) -> usize {
        self.claims.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut a = Assignment::new();
        assert!(a.claim("", "Jason").is_err());
        assert!(a.claim("cam1", "").is_err());
    }

    #[test]
    fn differential() {
        let mut a = Assignment::new();
        a.claim("cam1", "Alex").unwrap();
        assert_eq!(a.owner("cam1"), Some("Alex"));
        assert!(a.claim("cam1", "Jason").is_err());
        a.release("cam1", "Alex").unwrap();
        assert_eq!(a.owner("cam1"), None);
        assert_eq!(a.count(), 0);
    }
}
