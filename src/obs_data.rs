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
    BadJson,
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

    /// Merge another bag into this one. Keys in `other` overwrite.
    /// Matches OBS obs_data_apply. Defaults are not touched.
    pub fn apply(&mut self, other: &ObsData) {
        for (k, v) in &other.values {
            self.values.insert(k.clone(), v.clone());
        }
    }

    /// Serialize values to a compact JSON object string.
    /// Nested objects and arrays are included. No unwraps.
    pub fn to_json(&self) -> String {
        let mut parts = Vec::new();
        for (k, v) in &self.values {
            parts.push(format!("\"{}\":{}", escape_json(k), value_to_json(v)));
        }
        format!("{{{}}}", parts.join(","))
    }

    /// Parse a compact JSON object into values.
    /// Supports strings, ints, doubles, bools, nested objects, and arrays.
    /// Returns an error on malformed input. No unwraps.
    pub fn from_json(s: &str) -> Result<Self, DataError> {
        let s = s.trim();
        if !s.starts_with('{') || !s.ends_with('}') {
            return Err(DataError::BadJson);
        }
        let inner = &s[1..s.len() - 1];
        let mut data = ObsData::new();
        if inner.trim().is_empty() {
            return Ok(data);
        }
        for pair in split_top(inner, ',') {
            let (k, v) = split_once_top(pair, ':').ok_or(DataError::BadJson)?;
            let key = unquote(k.trim()).ok_or(DataError::BadJson)?;
            let value = parse_value(v.trim()).ok_or(DataError::BadJson)?;
            data.set(&key, value)?;
        }
        Ok(data)
    }
}

fn unquote(s: &str) -> Option<String> {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        Some(s[1..s.len() - 1].replace("\\\"", "\"").replace("\\\\", "\\"))
    } else {
        None
    }
}

fn parse_value(s: &str) -> Option<DataValue> {
    if s == "true" {
        Some(DataValue::Bool(true))
    } else if s == "false" {
        Some(DataValue::Bool(false))
    } else if s.starts_with('"') {
        unquote(s).map(DataValue::String)
    } else if s.starts_with('{') {
        ObsData::from_json(s).ok().map(|o| DataValue::Object(Box::new(o)))
    } else if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        if inner.trim().is_empty() {
            return Some(DataValue::Array(Vec::new()));
        }
        let mut items = Vec::new();
        for part in split_top(inner, ',') {
            items.push(parse_value(part.trim())?);
        }
        Some(DataValue::Array(items))
    } else if s.contains('.') {
        s.parse::<f64>().ok().map(DataValue::Double)
    } else {
        s.parse::<i64>().ok().map(DataValue::Int)
    }
}

/// Split on a separator only at the top level (not inside strings, objects, or arrays).
fn split_top(s: &str, sep: char) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    let mut in_str = false;
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        let c = b as char;
        if c == '"' && (i == 0 || bytes[i - 1] != b'\\') {
            in_str = !in_str;
        }
        if !in_str {
            if c == '{' || c == '[' {
                depth += 1;
            } else if c == '}' || c == ']' {
                depth -= 1;
            } else if c == sep && depth == 0 {
                out.push(&s[start..i]);
                start = i + 1;
            }
        }
    }
    out.push(&s[start..]);
    out
}

fn split_once_top(s: &str, sep: char) -> Option<(&str, &str)> {
    let parts = split_top(s, sep);
    if parts.len() < 2 {
        return None;
    }
    Some((parts[0], &s[parts[0].len() + 1..]))
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn value_to_json(v: &DataValue) -> String {
    match v {
        DataValue::String(s) => format!("\"{}\"", escape_json(s)),
        DataValue::Int(n) => n.to_string(),
        DataValue::Double(n) => n.to_string(),
        DataValue::Bool(b) => if *b { "true".into() } else { "false".into() },
        DataValue::Object(o) => o.to_json(),
        DataValue::Array(a) => {
            let items: Vec<String> = a.iter().map(value_to_json).collect();
            format!("[{}]", items.join(","))
        }
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

    #[test]
    fn json_round_trip_shape() {
        let mut d = ObsData::new();
        d.set_string("name", "cam").unwrap();
        d.set_int("width", 1920).unwrap();
        d.set_bool("on", true).unwrap();
        let json = d.to_json();
        assert!(json.contains("\"name\":\"cam\""));
        assert!(json.contains("\"width\":1920"));
        assert!(json.contains("\"on\":true"));
    }

    #[test]
    fn json_nested() {
        let mut child = ObsData::new();
        child.set_int("h", 720).unwrap();
        let mut parent = ObsData::new();
        parent.set_obj("video", child).unwrap();
        parent.set_array("ids", vec![DataValue::Int(1), DataValue::Int(2)]).unwrap();
        let json = parent.to_json();
        assert!(json.contains("\"video\":{"));
        assert!(json.contains("\"ids\":[1,2]"));
    }

    #[test]
    fn json_parse_round_trip() {
        let mut d = ObsData::new();
        d.set_string("name", "cam").unwrap();
        d.set_int("w", 1280).unwrap();
        d.set_bool("on", false).unwrap();
        let parsed = ObsData::from_json(&d.to_json()).unwrap();
        assert_eq!(parsed.get_string("name").unwrap(), "cam");
        assert_eq!(parsed.get_int("w").unwrap(), 1280);
        assert_eq!(parsed.get_bool("on").unwrap(), false);
    }

    #[test]
    fn json_bad() {
        assert_eq!(ObsData::from_json("not json"), Err(DataError::BadJson));
        assert_eq!(ObsData::from_json("{"), Err(DataError::BadJson));
    }

    #[test]
    fn apply_overwrites() {
        let mut base = ObsData::new();
        base.set_int("w", 1280).unwrap();
        base.set_string("name", "old").unwrap();
        let mut patch = ObsData::new();
        patch.set_string("name", "new").unwrap();
        patch.set_bool("on", true).unwrap();
        base.apply(&patch);
        assert_eq!(base.get_string("name").unwrap(), "new");
        assert_eq!(base.get_int("w").unwrap(), 1280);
        assert_eq!(base.get_bool("on").unwrap(), true);
    }
}
