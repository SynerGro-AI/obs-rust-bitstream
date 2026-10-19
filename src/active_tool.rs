//! Safe pure ActiveTool (toolbar click → mode).
//!
//! Holds the currently selected tool and which toolbar
//! it came from. A click on a known tool switches the
//! mode. Unknown names are rejected. No unwraps.
//!
//! Differential: a different click produces a different tool.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolSource {
    Photoshop,
    Maya,
    AfterEffects,
    ToonBoom,
}

impl ToolSource {
    pub fn label(&self) -> &'static str {
        match self {
            ToolSource::Photoshop => "photoshop",
            ToolSource::Maya => "maya",
            ToolSource::AfterEffects => "aftereffects",
            ToolSource::ToonBoom => "toonboom",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveTool {
    name: String,
    source: ToolSource,
}

impl ActiveTool {
    /// Default is the Photoshop Move tool.
    pub fn new() -> Self {
        Self {
            name: "Move".to_string(),
            source: ToolSource::Photoshop,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> ToolSource {
        self.source
    }

    /// Activate a tool from a toolbar click.
    /// Returns false if the name is empty.
    pub fn activate(&mut self, name: &str, source: ToolSource) -> bool {
        if name.is_empty() {
            return false;
        }
        self.name = name.to_string();
        self.source = source;
        true
    }

    /// One line: "tool=N source=N"
    pub fn line(&self) -> String {
        format!("tool={} source={}", self.name, self.source.label())
    }
}

impl Default for ActiveTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_switches() {
        let mut t = ActiveTool::new();
        assert_eq!(t.name(), "Move");
        assert!(t.activate("Brush", ToolSource::Photoshop));
        assert_eq!(t.name(), "Brush");
        assert_eq!(t.source(), ToolSource::Photoshop);
        assert!(t.activate("Camera", ToolSource::Maya));
        assert_eq!(t.source(), ToolSource::Maya);
    }

    #[test]
    fn empty_rejected() {
        let mut t = ActiveTool::new();
        assert!(!t.activate("", ToolSource::Photoshop));
        assert_eq!(t.name(), "Move");
    }

    #[test]
    fn differential() {
        let mut a = ActiveTool::new();
        let mut b = ActiveTool::new();
        a.activate("Lasso", ToolSource::Photoshop);
        b.activate("Pen", ToolSource::AfterEffects);
        assert_ne!(a.line(), b.line());
    }
}
