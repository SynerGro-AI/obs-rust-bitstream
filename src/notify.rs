//! Safe pure Notify (share confirmation core).
//!
//! Builds a confirmation message for a share. Empty
//! targets rejected. No unwraps.
//!
//! Differential: a share produces a message naming the target.

use crate::share::Share;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotifyError {
    EmptyTarget,
}

pub struct Notify;

impl Notify {
    /// Confirmation message for a share.
    pub fn confirm(share: &Share) -> Result<String, NotifyError> {
        if share.target().is_empty() {
            return Err(NotifyError::EmptyTarget);
        }
        Ok(format!(
            "shared {} bytes to {}",
            share.bytes(),
            share.target()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let s = Share::post("x.com/live", b"GIF|loop.gif|2").unwrap();
        let msg = Notify::confirm(&s).unwrap();
        assert!(msg.contains("x.com/live"));
        assert!(msg.contains("14"));
    }
}
