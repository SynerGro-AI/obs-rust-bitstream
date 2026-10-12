//! Safe pure Calldata (OBS callback/calldata.h core).
//!
//! The C version is a named parameter bag (int/float/bool/ptr/string).
//! The safe path is a HashMap of names to typed values.
//! Missing keys return None. No unwraps.
//!
//! Differential: set + get round-trips the value.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum CallValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

#[derive(Clone, Debug, Default)]
pub struct Calldata {
    params: HashMap<String, CallValue>,
}

impl Calldata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_int(&mut self, name: &str, v: i64) {
        self.params.insert(name.to_string(), CallValue::Int(v));
    }

    pub fn set_float(&mut self, name: &str, v: f64) {
        self.params.insert(name.to_string(), CallValue::Float(v));
    }

    pub fn set_bool(&mut self, name: &str, v: bool) {
        self.params.insert(name.to_string(), CallValue::Bool(v));
    }

    pub fn set_str(&mut self, name: &str, v: &str) {
        self.params.insert(name.to_string(), CallValue::Str(v.to_string()));
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        match self.params.get(name) {
            Some(CallValue::Int(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_float(&self, name: &str) -> Option<f64> {
        match self.params.get(name) {
            Some(CallValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.params.get(name) {
            Some(CallValue::Bool(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_str(&self, name: &str) -> Option<&str> {
        match self.params.get(name) {
            Some(CallValue::Str(v)) => Some(v.as_str()),
            _ => None,
        }
    }

    pub fn clear(&mut self) {
        self.params.clear();
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut cd = Calldata::new();
        cd.set_int("count", 42);
        cd.set_str("name", "test");
        assert_eq!(cd.get_int("count"), Some(42));
        assert_eq!(cd.get_str("name"), Some("test"));
    }

    #[test]
    fn missing_safe() {
        let cd = Calldata::new();
        assert!(cd.get_int("missing").is_none());
        assert!(cd.get_bool("nope").is_none());
    }

    #[test]
    fn type_mismatch() {
        let mut cd = Calldata::new();
        cd.set_int("x", 1);
        assert!(cd.get_float("x").is_none());
    }

    #[test]
    fn differential() {
        let mut cd = Calldata::new();
        cd.set_bool("flag", true);
        assert_eq!(cd.get_bool("flag"), Some(true));
    }
}
