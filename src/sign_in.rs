//! Safe pure Session (signed-in member core).
//!
//! Tracks who is signed in. Empty names rejected.
//! No unwraps.
//!
//! Differential: sign in + sign out round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionError {
    EmptyName,
}

#[derive(Clone, Debug, Default)]
pub struct Session {
    name: Option<String>,
    role: Option<String>,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sign_in(&mut self, name: &str, role: &str) -> Result<(), SessionError> {
        if name.is_empty() {
            return Err(SessionError::EmptyName);
        }
        self.name = Some(name.to_string());
        self.role = Some(role.to_string());
        Ok(())
    }

    pub fn sign_out(&mut self) {
        self.name = None;
        self.role = None;
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }

    pub fn signed_in(&self) -> bool {
        self.name.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut s = Session::new();
        assert!(s.sign_in("", "host").is_err());
    }

    #[test]
    fn differential() {
        let mut s = Session::new();
        assert!(!s.signed_in());
        s.sign_in("Jason", "host").unwrap();
        assert!(s.signed_in());
        assert_eq!(s.role(), Some("host"));
        s.sign_out();
        assert!(!s.signed_in());
    }
}
