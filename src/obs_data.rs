//! Safe pure ObsData (OBS obs-data.h core).
//!
//! The C version is a named settings bag (string/int/double/bool/obj/array)
//! with defaults. The safe path is a HashMap. Missing keys return the
//! default if one was set, otherwise a type-zero / None. Empty names are
//! rejected. No unwraps.
//!
//! Differential: set + get round-trips; default fills a missing key.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DataValue {
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    /// Nested object. Matches OBS obs_data_set_obj.
    Object(Box<ObsData>),
    /// Array of values. Matches OBS obs_data_set_array.
    Array(Vec<DataValue>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataError {
    EmptyName,
    MissingKey,
    TypeMismatch,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObsData {
    values: HashMap<String, DataValue>,
    defaults: HashMap<String, DataValue>,
}

impl ObsData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_string(&mut self, name: &str, v: &str) -> Result<(), DataError> {
        self.set(name, DataValue::String(v.to_string()))
    }

    pub fn set_int(&mut self, name: &str, v: i64) -> Result<(), DataError> {
        self.set(name, DataValue::Int(v))
    }

    pub fn set_double(&mut self, name: &str, v: f64) -> Result<(), DataError> {
        self.set(name, DataValue::Double(v))
    }

    pub fn set_bool(&mut self, name: &str, v: bool) -> Result<(), DataError> {
        self.set(name, DataValue::Bool(v))
    }

    /// Store a nested object under a name.
    pub fn set_obj(&mut self, name: &str, obj: ObsData) -> Result<(), DataError> {
        self.set(name, DataValue::Object(Box::new(obj)))
    }

    /// Store an array under a name.
    pub fn set_array(&mut self, name: &str, items: Vec<DataValue>) -> Result<(), DataError> {
        self.set(name, DataValue::Array(items))
    }

    pub fn set_default_string(&mut self, name: &str, v: &str) -> Result<(), DataError> {
        self.set_default(name, DataValue::String(v.to_string()))
    }

    pub fn set_default_int(&mut self, name: &str, v: i64) -> Result<(), DataError> {
        self.set_default(name, DataValue::Int(v))
    }

    pub fn set_default_bool(&mut self, name: &str, v: bool) -> Result<(), DataError> {
        self.set_default(name, DataValue::Bool(v))
    }

    fn set(&mut self, name: &str, value: DataValue) -> Result<(), DataError> {
        if name.is_empty() {
            return Err(DataError::EmptyName);
        }
        self.values.insert(name.to_string(), value);
        Ok(())
    }

    fn set_default(&mut self, name: &str, value: DataValue) -> Result<(), DataError> {
        if name.is_empty() {
            return Err(DataError::EmptyName);
        }
        self.defaults.insert(name.to_string(), value);
        Ok(())
    }

    /// Get a string. Uses the default if the key is missing.
    pub fn get_string(&self, name: &str) -> Result<&str, DataError> {
        match self.lookup(name) {
            Some(DataValue::String(s)) => Ok(s.as_str()),
            Some(_) => Err(DataError::TypeMismatch),
            None => Err(DataError::MissingKey),
        }
    }

    pub fn get_int(&self, name: &str) -> Result<i64, DataError> {
        match self.lookup(name) {
            Some(DataValue::Int(v)) => Ok(*v),
            Some(_) => Err(DataError::TypeMismatch),
            None => Err(DataError::MissingKey),
        }
    }

    pub fn get_double(&self, name: &str) -> Result<f64, DataError> {
        match self.lookup(name) {
            Some(DataValue::Double(v)) => Ok(*v),
            Some(_) => Err(DataError::TypeMismatch),
            None => Err(DataError::MissingKey),
        }
    }

    pub fn get_bool(&self, name: &str) -> Result<bool, DataError> {
        match self.lookup(name) {
            Some(DataValue::Bool(v)) => Ok(*v),
            Some(_) => Err(DataError::TypeMismatch),
            None => Err(DataError::MissingKey),
        }
    }

    /// Get a nested object. Fails if the key is missing or not an object.
    pub fn get_obj(&self, name: &str) -> Result<&ObsData, DataError> {
        match self.lookup(name) {
            Some(DataValue::Object(o)) => Ok(o.as_ref()),
            Some(_) => Err(DataError::TypeMismatch),
            None => Err(DataError::MissingKey),
        }
    }

    /// Get an array. Fails if the key is missing or not an array.
    pub fn get_array(&self, name: &str) -> Result<&[DataValue], DataError> {
        match self.lookup(name) {
            Some(DataValue::Array(a)) => Ok(a.as_slice()),
            Some(_) => Err(DataError::TypeMismatch),
            None => Err(DataError::MissingKey),
        }
    }

    fn lookup(&self, name: &str) -> Option<&DataValue> {
        self.values.get(name).or_else(|| self.defaults.get(name))
    }

    pub fn erase(&mut self, name: &str) -> bool {
        self.values.remove(name).is_some()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn has(&self, name: &str) -> bool {
        self.values.contains_key(name) || self.defaults.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut d = ObsData::new();
        d.set_string("name", "cam").unwrap();
        d.set_int("width", 1920).unwrap();
        d.set_bool("visible", true).unwrap();
        assert_eq!(d.get_string("name").unwrap(), "cam");
        assert_eq!(d.get_int("width").unwrap(), 1920);
        assert_eq!(d.get_bool("visible").unwrap(), true);
    }

    #[test]
    fn default_fills_missing() {
        let mut d = ObsData::new();
        d.set_default_int("fps", 30).unwrap();
        assert_eq!(d.get_int("fps").unwrap(), 30);
        d.set_int("fps", 60).unwrap();
        assert_eq!(d.get_int("fps").unwrap(), 60);
    }

    #[test]
    fn missing_and_mismatch() {
        let d = ObsData::new();
        assert_eq!(d.get_int("nope"), Err(DataError::MissingKey));
        let mut d = ObsData::new();
        d.set_string("x", "a").unwrap();
        assert_eq!(d.get_int("x"), Err(DataError::TypeMismatch));
    }

    #[test]
    fn empty_name() {
        let mut d = ObsData::new();
        assert_eq!(d.set_int("", 1), Err(DataError::EmptyName));
    }

    #[test]
    fn differential() {
        let mut d = ObsData::new();
        d.set_double("gain", 1.5).unwrap();
        assert_eq!(d.get_double("gain").unwrap(), 1.5);
    }

    #[test]
    fn nested_object() {
        let mut child = ObsData::new();
        child.set_int("width", 1280).unwrap();
        let mut parent = ObsData::new();
        parent.set_obj("video", child).unwrap();
        let got = parent.get_obj("video").unwrap();
        assert_eq!(got.get_int("width").unwrap(), 1280);
    }

    #[test]
    fn nested_mismatch() {
        let mut d = ObsData::new();
        d.set_int("x", 1).unwrap();
        assert_eq!(d.get_obj("x"), Err(DataError::TypeMismatch));
    }

    #[test]
    fn array_round_trip() {
        let mut d = ObsData::new();
        d.set_array(
            "items",
            vec![DataValue::Int(1), DataValue::Int(2), DataValue::String("a".into())],
        )
        .unwrap();
        let a = d.get_array("items").unwrap();
        assert_eq!(a.len(), 3);
        assert_eq!(a[0], DataValue::Int(1));
    }

    #[test]
    fn array_mismatch() {
        let mut d = ObsData::new();
        d.set_bool("flag", true).unwrap();
        assert_eq!(d.get_array("flag"), Err(DataError::TypeMismatch));
    }
}
