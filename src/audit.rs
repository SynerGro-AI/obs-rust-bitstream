//! Safe pure Audit (who-did-what core).
//!
//! Records the name, role, and action. Empty names
//! rejected. No unwraps.
//!
//! Differential: two entries round-trip.

#[derive(Clone, Debug)]
pub struct AuditEntry {
    pub name: String,
    pub role: String,
    pub action: String,
}

#[derive(Clone, Debug, Default)]
pub struct Audit {
    entries: Vec<AuditEntry>,
}

impl Audit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, name: &str, role: &str, action: &str) {
        if name.is_empty() || action.is_empty() {
            return;
        }
        self.entries.push(AuditEntry {
            name: name.to_string(),
            role: role.to_string(),
            action: action.to_string(),
        });
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn last(&self) -> Option<&AuditEntry> {
        self.entries.last()
    }

    pub fn by(&self, name: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.name == name).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let a = Audit::new();
        assert_eq!(a.len(), 0);
    }

    #[test]
    fn differential() {
        let mut a = Audit::new();
        a.record("Jason", "host", "record");
        a.record("Alex", "camera", "switch");
        assert_eq!(a.len(), 2);
        assert_eq!(a.last().unwrap().name, "Alex");
        assert_eq!(a.by("Jason").len(), 1);
    }
}
