//! Safe pure ShellSummary (full path in one line).
//!
//! Summarizes the shell path: loop status, encoded
//! size, socket payload, and sent count. No unwraps.
//!
//! Differential: a different sequence produces a different summary.

use crate::shell_loop::{ShellAction, ShellLoop};
use crate::shell_loop_frame::ShellLoopFrame;
use crate::shell_loop_socket::ShellLoopSocket;

pub struct ShellSummary;

impl ShellSummary {
    /// One line covering the full path.
    pub fn line(actions: &[ShellAction]) -> String {
        let status = ShellLoop::run(actions);
        let encoded = ShellLoopFrame::encode(actions);
        let payload = ShellLoopSocket::payload(actions).map(|p| p.len()).unwrap_or(0);
        let valid = ShellLoopSocket::payload(actions)
            .map(|p| ShellLoopSocket::valid(&p))
            .unwrap_or(false);
        format!("{} encoded={} payload={} valid={}", status, encoded, payload, valid)
    }

    /// True if the line contains every section.
    pub fn complete(line: &str) -> bool {
        line.contains("tool=") && line.contains("encoded=") && line.contains("payload=") && line.contains("valid=")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_sections() {
        let actions = [ShellAction::Tool("Brush"), ShellAction::NewLayer];
        let line = ShellSummary::line(&actions);
        assert!(ShellSummary::complete(&line));
        assert!(line.contains("tool=Brush"));
        assert!(line.contains("encoded="));
        assert!(line.contains("valid=true"));
    }

    #[test]
    fn empty_is_move() {
        let line = ShellSummary::line(&[]);
        assert!(line.contains("tool=Move"));
    }

    #[test]
    fn differential() {
        let a = ShellSummary::line(&[ShellAction::NewLayer]);
        let b = ShellSummary::line(&[ShellAction::NewLayer, ShellAction::NewLayer]);
        assert_ne!(a, b);
    }
}
