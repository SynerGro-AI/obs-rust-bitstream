//! Safe pure declaration-string parser (OBS callback/decl.c core).
//!
//! Parses strings like:
//!   "void activate(ptr source)"
//!   "void mute(ptr source, bool muted)"
//!   "void volume(ptr source, in out float volume)"
//!
//! Returns a DeclInfo with name and typed params, or an error.
//! Duplicate names, reserved names, and unknown types are rejected.
//! No unwraps.
//!
//! Differential: a known source signal parses to the expected name and params.

use crate::calldata::CallType;

/// Input / output flags. Default is In when neither is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParamFlags {
    pub in_: bool,
    pub out: bool,
}

impl Default for ParamFlags {
    fn default() -> Self {
        Self { in_: true, out: false }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeclParam {
    pub name: String,
    pub ty: CallType,
    pub flags: ParamFlags,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeclInfo {
    pub name: String,
    pub params: Vec<DeclParam>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclError {
    Empty,
    MissingName,
    MissingParen,
    UnknownType(String),
    DuplicateName(String),
    ReservedName(String),
    UnexpectedToken(String),
}

const RESERVED: &[&str] = &["int", "float", "bool", "ptr", "string", "void", "return", "in", "out"];

fn is_reserved(name: &str) -> bool {
    RESERVED.iter().any(|r| *r == name)
}

fn parse_type(tok: &str) -> Option<CallType> {
    match tok {
        "int" => Some(CallType::Int),
        "float" => Some(CallType::Float),
        "bool" => Some(CallType::Bool),
        "ptr" => Some(CallType::Ptr),
        "string" => Some(CallType::Str),
        _ => None,
    }
}

/// Parse a declaration string into a DeclInfo.
///
/// Grammar (simplified from OBS decl.c):
///   [void] name ( [param [, param]*] )
///   param := [in | out]* type name
pub fn parse_decl(s: &str) -> Result<DeclInfo, DeclError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(DeclError::Empty);
    }

    // Split on the first '(' 
    let open = s.find('(').ok_or(DeclError::MissingParen)?;
    let close = s.rfind(')').ok_or(DeclError::MissingParen)?;
    if close < open {
        return Err(DeclError::MissingParen);
    }

    let head = s[..open].trim();
    let body = s[open + 1..close].trim();

    // Head is either "void name" or just "name"
    let mut parts = head.split_whitespace();
    let first = parts.next().ok_or(DeclError::MissingName)?;
    let name = if first == "void" {
        parts.next().ok_or(DeclError::MissingName)?.to_string()
    } else {
        first.to_string()
    };
    if name.is_empty() {
        return Err(DeclError::MissingName);
    }

    let mut params = Vec::new();
    if !body.is_empty() {
        for raw in body.split(',') {
            let param = parse_param(raw.trim())?;
            if params.iter().any(|p: &DeclParam| p.name == param.name) {
                return Err(DeclError::DuplicateName(param.name));
            }
            if is_reserved(&param.name) {
                return Err(DeclError::ReservedName(param.name));
            }
            params.push(param);
        }
    }

    Ok(DeclInfo { name, params })
}

fn parse_param(raw: &str) -> Result<DeclParam, DeclError> {
    let tokens: Vec<&str> = raw.split_whitespace().collect();
    if tokens.is_empty() {
        return Err(DeclError::UnexpectedToken(raw.to_string()));
    }

    let mut flags = ParamFlags { in_: false, out: false };
    let mut i = 0;

    // Leading in/out specifiers
    while i < tokens.len() {
        match tokens[i] {
            "in" => {
                flags.in_ = true;
                i += 1;
            }
            "out" => {
                flags.out = true;
                i += 1;
            }
            _ => break,
        }
    }

    // Default to in when no specifier was given
    if !flags.in_ && !flags.out {
        flags.in_ = true;
    }

    if i >= tokens.len() {
        return Err(DeclError::UnknownType(raw.to_string()));
    }
    let ty = parse_type(tokens[i]).ok_or_else(|| DeclError::UnknownType(tokens[i].to_string()))?;
    i += 1;

    if i >= tokens.len() {
        return Err(DeclError::UnexpectedToken("missing name".into()));
    }
    let name = tokens[i].to_string();
    i += 1;

    if i != tokens.len() {
        return Err(DeclError::UnexpectedToken(tokens[i].to_string()));
    }

    Ok(DeclParam { name, ty, flags })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_source_signal() {
        let d = parse_decl("void activate(ptr source)").unwrap();
        assert_eq!(d.name, "activate");
        assert_eq!(d.params.len(), 1);
        assert_eq!(d.params[0].name, "source");
        assert_eq!(d.params[0].ty, CallType::Ptr);
        assert!(d.params[0].flags.in_);
    }

    #[test]
    fn multi_params() {
        let d = parse_decl("void mute(ptr source, bool muted)").unwrap();
        assert_eq!(d.params.len(), 2);
        assert_eq!(d.params[1].ty, CallType::Bool);
    }

    #[test]
    fn in_out() {
        let d = parse_decl("void volume(ptr source, in out float volume)").unwrap();
        assert!(d.params[1].flags.in_);
        assert!(d.params[1].flags.out);
        assert_eq!(d.params[1].ty, CallType::Float);
    }

    #[test]
    fn no_void() {
        let d = parse_decl("get_name(string name)").unwrap();
        assert_eq!(d.name, "get_name");
    }

    #[test]
    fn empty_params() {
        let d = parse_decl("void tick()").unwrap();
        assert!(d.params.is_empty());
    }

    #[test]
    fn duplicate_rejected() {
        assert!(matches!(
            parse_decl("void bad(int a, int a)"),
            Err(DeclError::DuplicateName(_))
        ));
    }

    #[test]
    fn reserved_rejected() {
        assert!(matches!(
            parse_decl("void bad(int void)"),
            Err(DeclError::ReservedName(_))
        ));
    }

    #[test]
    fn unknown_type() {
        assert!(matches!(
            parse_decl("void bad(foo bar)"),
            Err(DeclError::UnknownType(_))
        ));
    }

    #[test]
    fn differential() {
        let d = parse_decl("void rename(ptr source, string new_name, string prev_name)").unwrap();
        assert_eq!(d.name, "rename");
        assert_eq!(d.params.len(), 3);
        assert_eq!(d.params[1].ty, CallType::Str);
    }
}
