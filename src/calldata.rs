//! Safe pure Calldata (OBS callback/calldata.h core).
//!
//! The C version is a named parameter bag. Gets return true only
//! when the key exists AND the stored type matches the requested type.
//! The safe path enforces both conditions: missing key or wrong type
//! is an error. Empty names are rejected on set. No unwraps.
//!
//! Differential: set + get of the matching type round-trips the value.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum CallValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallType {
    Int,
    Float,
    Bool,
    Str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallError {
    /// Key was not present.
    MissingKey,
    /// Key was present but held a different type.
    TypeMismatch { expected: CallType, found: CallType },
    /// Empty name rejected on set.
    EmptyName,
}

#[derive(Clone, Debug, Default)]
pub struct Calldata {
    params: HashMap<String, CallValue>,
}

impl CallValue {
    pub fn call_type(&self) -> CallType {
        match self {
            CallValue::Int(_) => CallType::Int,
            CallValue::Float(_) => CallType::Float,
            CallValue::Bool(_) => CallType::Bool,
            CallValue::Str(_) => CallType::Str,
        }
    }
}

impl Calldata {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set an integer. Rejects empty names.
    pub fn set_int(&mut self, name: &str, v: i64) -> Result<(), CallError> {
        self.set(name, CallValue::Int(v))
    }

    pub fn set_float(&mut self, name: &str, v: f64) -> Result<(), CallError> {
        self.set(name, CallValue::Float(v))
    }

    pub fn set_bool(&mut self, name: &str, v: bool) -> Result<(), CallError> {
        self.set(name, CallValue::Bool(v))
    }

    pub fn set_str(&mut self, name: &str, v: &str) -> Result<(), CallError> {
        self.set(name, CallValue::Str(v.to_string()))
    }

    fn set(&mut self, name: &str, value: CallValue) -> Result<(), CallError> {
        if name.is_empty() {
            return Err(CallError::EmptyName);
        }
        self.params.insert(name.to_string(), value);
        Ok(())
    }

    /// Get an integer. Fails if the key is missing or the type is not Int.
    pub fn get_int(&self, name: &str) -> Result<i64, CallError> {
        match self.get_typed(name, CallType::Int)? {
            CallValue::Int(v) => Ok(*v),
            other => Err(CallError::TypeMismatch {
                expected: CallType::Int,
                found: other.call_type(),
            }),
        }
    }

    pub fn get_float(&self, name: &str) -> Result<f64, CallError> {
        match self.get_typed(name, CallType::Float)? {
            CallValue::Float(v) => Ok(*v),
            other => Err(CallError::TypeMismatch {
                expected: CallType::Float,
                found: other.call_type(),
            }),
        }
    }

    pub fn get_bool(&self, name: &str) -> Result<bool, CallError> {
        match self.get_typed(name, CallType::Bool)? {
            CallValue::Bool(v) => Ok(*v),
            other => Err(CallError::TypeMismatch {
                expected: CallType::Bool,
                found: other.call_type(),
            }),
        }
    }

    pub fn get_str(&self, name: &str) -> Result<&str, CallError> {
        match self.get_typed(name, CallType::Str)? {
            CallValue::Str(v) => Ok(v.as_str()),
            other => Err(CallError::TypeMismatch {
                expected: CallType::Str,
                found: other.call_type(),
            }),
        }
    }

    /// Require both key presence and exact type match.
    fn get_typed(&self, name: &str, expected: CallType) -> Result<&CallValue, CallError> {
        match self.params.get(name) {
            None => Err(CallError::MissingKey),
            Some(v) if v.call_type() == expected => Ok(v),
            Some(v) => Err(CallError::TypeMismatch {
                expected,
                found: v.call_type(),
            }),
        }
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.params.contains_key(name)
    }

    pub fn type_of(&self, name: &str) -> Option<CallType> {
        self.params.get(name).map(|v| v.call_type())
    }

    pub fn clear(&mut self) {
        self.params.clear();
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_requires_key_and_type() {
        let mut cd = Calldata::new();
        cd.set_int("count", 42).unwrap();
        cd.set_str("name", "test").unwrap();
        assert_eq!(cd.get_int("count").unwrap(), 42);
        assert_eq!(cd.get_str("name").unwrap(), "test");
    }

    #[test]
    fn missing_key_is_error() {
        let cd = Calldata::new();
        assert_eq!(cd.get_int("missing"), Err(CallError::MissingKey));
        assert_eq!(cd.get_bool("nope"), Err(CallError::MissingKey));
    }

    #[test]
    fn type_mismatch_is_error() {
        let mut cd = Calldata::new();
        cd.set_int("x", 1).unwrap();
        assert_eq!(
            cd.get_float("x"),
            Err(CallError::TypeMismatch {
                expected: CallType::Float,
                found: CallType::Int,
            })
        );
        assert_eq!(
            cd.get_str("x"),
            Err(CallError::TypeMismatch {
                expected: CallType::Str,
                found: CallType::Int,
            })
        );
    }

    #[test]
    fn empty_name_rejected() {
        let mut cd = Calldata::new();
        assert_eq!(cd.set_int("", 1), Err(CallError::EmptyName));
        assert!(!cd.contains_key(""));
    }

    #[test]
    fn differential() {
        let mut cd = Calldata::new();
        cd.set_bool("flag", true).unwrap();
        assert_eq!(cd.get_bool("flag").unwrap(), true);
        assert_eq!(cd.type_of("flag"), Some(CallType::Bool));
    }
}
