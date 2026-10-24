//! Serde helpers for `&'static str` fields (material notes, element
//! symbols, preset keys). Deserialised strings are interned: each distinct
//! string is leaked once, so loading the same project many times does not
//! grow memory.

use serde::{Deserialize, Deserializer};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// A `'static` copy of `s`, shared by every caller asking for the same text.
pub fn intern(s: &str) -> &'static str {
    static POOL: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut pool = POOL
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(v) = pool.get(s) {
        return v;
    }
    let v: &'static str = Box::leak(s.to_owned().into_boxed_str());
    pool.insert(v);
    v
}

pub fn vec<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<&'static str>, D::Error> {
    let v = Vec::<String>::deserialize(d)?;
    Ok(v.iter().map(|s| intern(s)).collect())
}

pub fn opt<'de, D: Deserializer<'de>>(d: D) -> Result<Option<&'static str>, D::Error> {
    let v = Option::<String>::deserialize(d)?;
    Ok(v.as_deref().map(intern))
}

type Nucleus = (&'static str, u32, u32, bool);
type OxState = (&'static str, i32);

pub fn nuclei<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Nucleus>, D::Error> {
    let v = Vec::<(String, u32, u32, bool)>::deserialize(d)?;
    Ok(v.iter()
        .map(|(s, p, n, st)| (intern(s), *p, *n, *st))
        .collect())
}

pub fn oxidation<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<OxState>>, D::Error> {
    let v = Option::<Vec<(String, i32)>>::deserialize(d)?;
    Ok(v.map(|v| v.iter().map(|(s, k)| (intern(s), *k)).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_shares_one_copy() {
        let a = intern(&String::from("carbon sublimes"));
        let b = intern("carbon sublimes");
        assert!(std::ptr::eq(a, b));
    }
}
