//! Safe pure Undo (reverse last action core).
//!
//! Maps an action to its reverse. No unwraps.
//!
//! Differential: record reverses to stop_record.

pub struct Undo;

impl Undo {
    /// The reverse of an action, if it has one.
    pub fn reverse(action: &str) -> Option<&'static str> {
        match action {
            "record" => Some("stop_record"),
            "stream" => Some("stop_stream"),
            "vcam" => Some("stop_vcam"),
            "stop_record" => Some("record"),
            "stop_stream" => Some("stream"),
            "stop_vcam" => Some("vcam"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        assert_eq!(Undo::reverse("record"), Some("stop_record"));
        assert_eq!(Undo::reverse("stop_record"), Some("record"));
        assert_eq!(Undo::reverse("unknown"), None);
    }
}
