//! Safe pure ServiceRegistry (OBS service lookup core).
//!
//! Name-to-ServiceContext map. Empty and duplicate names are
//! rejected. No unwraps.
//!
//! Differential: add + get returns the same name; settings round-trip.

use std::collections::HashMap;

use crate::service_context::{ServiceContext, ServiceError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceRegistryError {
    EmptyName,
    Duplicate,
    Missing,
}

impl From<ServiceError> for ServiceRegistryError {
    fn from(_: ServiceError) -> Self {
        ServiceRegistryError::EmptyName
    }
}

#[derive(Clone, Debug, Default)]
pub struct ServiceRegistry {
    services: HashMap<String, ServiceContext>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, id: &str) -> Result<(), ServiceRegistryError> {
        if name.is_empty() {
            return Err(ServiceRegistryError::EmptyName);
        }
        if self.services.contains_key(name) {
            return Err(ServiceRegistryError::Duplicate);
        }
        let ctx = ServiceContext::new(name, id)?;
        self.services.insert(name.to_string(), ctx);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&ServiceContext, ServiceRegistryError> {
        self.services.get(name).ok_or(ServiceRegistryError::Missing)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut ServiceContext, ServiceRegistryError> {
        self.services.get_mut(name).ok_or(ServiceRegistryError::Missing)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.services.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.services.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_get() {
        let mut r = ServiceRegistry::new();
        r.add("Twitch", "rtmp_common").unwrap();
        assert_eq!(r.get("Twitch").unwrap().id(), "rtmp_common");
    }

    #[test]
    fn duplicate_and_empty() {
        let mut r = ServiceRegistry::new();
        r.add("a", "id").unwrap();
        assert!(matches!(r.add("a", "id"), Err(ServiceRegistryError::Duplicate)));
        assert!(matches!(r.add("", "id"), Err(ServiceRegistryError::EmptyName)));
    }

    #[test]
    fn missing_and_remove() {
        let mut r = ServiceRegistry::new();
        assert!(matches!(r.get("nope"), Err(ServiceRegistryError::Missing)));
        r.add("s", "id").unwrap();
        assert!(r.remove("s"));
        assert!(!r.contains("s"));
    }

    #[test]
    fn differential() {
        let mut r = ServiceRegistry::new();
        r.add("YouTube", "rtmp_common").unwrap();
        r.get_mut("YouTube").unwrap().settings_mut().set_string("key", "abc").unwrap();
        assert_eq!(r.get("YouTube").unwrap().settings().get_string("key").unwrap(), "abc");
    }
}
