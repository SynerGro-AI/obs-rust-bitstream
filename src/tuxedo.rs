//! Safe pure Tuxedo (UI theme).
//!
//! Black, white, and gold. Elegant spacing. Refined labels.
//! No unwraps.
//!
//! Differential: a different accent produces a different theme.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tuxedo {
    pub bg: &'static str,
    pub fg: &'static str,
    pub accent: &'static str,
    pub muted: &'static str,
}

impl Tuxedo {
    /// The tuxedo theme: black background, white text, gold accent.
    pub fn theme() -> Self {
        Self {
            bg: "#0a0a0a",
            fg: "#f5f5f5",
            accent: "#c9a84c",
            muted: "#6b6b6b",
        }
    }

    /// One line of CSS variables.
    pub fn css(&self) -> String {
        format!(
            "--bg:{}; --fg:{}; --accent:{}; --muted:{};",
            self.bg, self.fg, self.accent, self.muted
        )
    }

    /// A refined label: "◆ NAME"
    pub fn label(name: &str) -> String {
        format!("◆ {}", name.to_uppercase())
    }

    /// A status line in tuxedo style.
    pub fn status(tool: &str, zoom: f32, w: u32, h: u32) -> String {
        format!("◆ {}  ·  {:.0}%  ·  {}×{}", tool.to_uppercase(), zoom, w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme() {
        let t = Tuxedo::theme();
        assert!(t.css().contains("#c9a84c"));
        assert!(t.css().contains("#0a0a0a"));
    }

    #[test]
    fn label() {
        let l = Tuxedo::label("Brush");
        assert!(l.contains("◆"));
        assert!(l.contains("BRUSH"));
    }

    #[test]
    fn status() {
        let s = Tuxedo::status("Brush", 100.0, 3840, 2160);
        assert!(s.contains("BRUSH"));
        assert!(s.contains("3840×2160"));
    }

    #[test]
    fn differential() {
        let a = Tuxedo::label("Brush");
        let b = Tuxedo::label("Crop");
        assert_ne!(a, b);
    }
}
