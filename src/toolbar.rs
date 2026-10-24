//! Safe pure Toolbar (Photoshop-style expandable toolbar).
//!
//! Holds a list of tool groups. Each group can expand or
//! retract. The active tool is tracked. No unwraps.
//!
//! Differential: expanding a group shows more tools.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolGroup {
    pub name: &'static str,
    pub tools: Vec<&'static str>,
    pub expanded: bool,
}

pub struct Toolbar {
    groups: Vec<ToolGroup>,
    active: String,
}

impl Toolbar {
    /// Photoshop-style toolbar with expandable groups.
    pub fn photoshop() -> Self {
        Self {
            groups: vec![
                ToolGroup {
                    name: "Select",
                    tools: vec!["Move", "Marquee", "Lasso", "Quick Selection"],
                    expanded: false,
                },
                ToolGroup {
                    name: "Crop",
                    tools: vec!["Crop", "Eyedropper"],
                    expanded: false,
                },
                ToolGroup {
                    name: "Paint",
                    tools: vec!["Brush", "Pencil", "Eraser", "Gradient", "Paint Bucket"],
                    expanded: false,
                },
                ToolGroup {
                    name: "Retouch",
                    tools: vec!["Clone Stamp", "Healing Brush", "Dodge", "Burn", "Sponge"],
                    expanded: false,
                },
                ToolGroup {
                    name: "Draw",
                    tools: vec!["Pen", "Shape", "Text"],
                    expanded: false,
                },
                ToolGroup {
                    name: "View",
                    tools: vec!["Hand", "Zoom"],
                    expanded: false,
                },
            ],
            active: "Move".to_string(),
        }
    }

    /// After Effects toolbar.
    pub fn after_effects() -> Self {
        Self {
            groups: vec![
                ToolGroup {
                    name: "Select",
                    tools: vec!["Selection", "Hand", "Zoom"],
                    expanded: false,
                },
                ToolGroup {
                    name: "Transform",
                    tools: vec!["Rotation", "Pan Behind"],
                    expanded: false,
                },
                ToolGroup {
                    name: "Create",
                    tools: vec!["Shape", "Pen", "Text", "Camera"],
                    expanded: false,
                },
                ToolGroup {
                    name: "Paint",
                    tools: vec!["Brush", "Clone Stamp", "Eraser", "Roto Brush"],
                    expanded: false,
                },
                ToolGroup {
                    name: "Puppet",
                    tools: vec!["Puppet Pin"],
                    expanded: false,
                },
            ],
            active: "Selection".to_string(),
        }
    }

    /// Expand a group by name. Returns true if found.
    pub fn expand(&mut self, name: &str) -> bool {
        if let Some(g) = self.groups.iter_mut().find(|g| g.name == name) {
            g.expanded = true;
            true
        } else {
            false
        }
    }

    /// Retract a group by name. Returns true if found.
    pub fn retract(&mut self, name: &str) -> bool {
        if let Some(g) = self.groups.iter_mut().find(|g| g.name == name) {
            g.expanded = false;
            true
        } else {
            false
        }
    }

    /// Visible tools: group name plus tools if expanded.
    pub fn visible(&self) -> Vec<String> {
        let mut out = Vec::new();
        for g in &self.groups {
            out.push(g.name.to_string());
            if g.expanded {
                for t in &g.tools {
                    out.push(format!("  {}", t));
                }
            }
        }
        out
    }

    /// Select a tool. Returns true if the tool exists in any group.
    pub fn select(&mut self, name: &str) -> bool {
        let exists = self.groups.iter().any(|g| g.tools.contains(&name));
        if exists {
            self.active = name.to_string();
        }
        exists
    }

    pub fn active(&self) -> &str {
        &self.active
    }

    pub fn group_count(&self) -> usize {
        self.groups.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_shows_tools() {
        let mut tb = Toolbar::photoshop();
        let before = tb.visible().len();
        tb.expand("Paint");
        let after = tb.visible().len();
        assert!(after > before);
        assert!(tb.visible().iter().any(|s| s.contains("Brush")));
    }

    #[test]
    fn retract_hides() {
        let mut tb = Toolbar::photoshop();
        tb.expand("Paint");
        tb.retract("Paint");
        assert!(!tb.visible().iter().any(|s| s.contains("Brush")));
    }

    #[test]
    fn select() {
        let mut tb = Toolbar::photoshop();
        assert!(tb.select("Brush"));
        assert_eq!(tb.active(), "Brush");
        assert!(!tb.select("Nope"));
    }

    #[test]
    fn differential() {
        let mut a = Toolbar::photoshop();
        let mut b = Toolbar::photoshop();
        a.expand("Paint");
        b.expand("Draw");
        assert_ne!(a.visible(), b.visible());
    }
}
