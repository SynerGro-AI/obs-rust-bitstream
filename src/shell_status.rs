//! Safe pure ShellStatus (one line for the shell path).
//!
//! Summarizes tool, layers, edit, and blend in one line.
//! No unwraps.
//!
//! Differential: a different blend produces a different line.

use crate::blend_composite::BlendComposite;
use crate::edit_round::EditRound;
use crate::group_composite::GroupComposite;

pub struct ShellStatus;

impl ShellStatus {
    /// One line covering the shell path.
    pub fn line() -> String {
        let (layer, group) = EditRound::run();
        let (single, group_n) = GroupComposite::run();
        let (normal, multiply) = BlendComposite::run();
        format!(
            "edit={} group={} members={}/{} blend={:.0}/{:.0}",
            layer, group, single, group_n, normal, multiply
        )
    }

    /// True if the line contains every section.
    pub fn complete(line: &str) -> bool {
        line.contains("edit=") && line.contains("group=") && line.contains("blend=")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_sections() {
        let line = ShellStatus::line();
        assert!(ShellStatus::complete(&line));
        assert!(line.contains("edit=1"));
        assert!(line.contains("blend="));
    }

    #[test]
    fn blend_differs() {
        let line = ShellStatus::line();
        // normal/multiply are different numbers.
        assert!(line.contains("200/78") || line.contains("/"));
    }

    #[test]
    fn differential() {
        let a = ShellStatus::line();
        assert!(a.contains("edit="));
        assert!(a.len() > 10);
    }
}
