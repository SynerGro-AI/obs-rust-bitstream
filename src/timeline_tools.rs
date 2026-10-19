//! Safe pure TimelineTools (AE timeline tool set).
//!
//! The tools After Effects puts on the timeline:
//! Selection, Razor, Trim, Slip, Slide, Ripple, Roll,
//! Marker, Work Area, Time Remap, Keyframe nav.
//! Each tool has a hotkey and a 💭 thought.
//! No unwraps.
//!
//! Differential: a different tool has a different hotkey.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimelineTool {
    pub name: &'static str,
    pub hotkey: &'static str,
    pub thought: &'static str,
}

pub struct TimelineTools;

impl TimelineTools {
    /// Every AE timeline tool.
    pub fn tools() -> Vec<TimelineTool> {
        vec![
            TimelineTool {
                name: "Selection",
                hotkey: "V",
                thought: "💭 Selection moves and trims clips. Drag the body to move. Drag an edge to trim.",
            },
            TimelineTool {
                name: "Razor",
                hotkey: "C",
                thought: "💭 Razor cuts a clip at the playhead. Click to split into two.",
            },
            TimelineTool {
                name: "Trim",
                hotkey: "[",
                thought: "💭 Trim sets the in or out point. [ sets the in. ] sets the out.",
            },
            TimelineTool {
                name: "Slip",
                hotkey: "Y",
                thought: "💭 Slip changes the source in and out without moving the clip on the timeline.",
            },
            TimelineTool {
                name: "Slide",
                hotkey: "U",
                thought: "💭 Slide moves a clip and trims its neighbors so the gap stays filled.",
            },
            TimelineTool {
                name: "Ripple",
                hotkey: "B",
                thought: "💭 Ripple trims a clip and shifts everything after it to close the gap.",
            },
            TimelineTool {
                name: "Roll",
                hotkey: "N",
                thought: "💭 Roll trims the out of one clip and the in of the next at the same time.",
            },
            TimelineTool {
                name: "Marker",
                hotkey: "M",
                thought: "💭 Marker drops a named point on the timeline. Press M to add. Shift+M to go to the next.",
            },
            TimelineTool {
                name: "Work Area",
                hotkey: "W",
                thought: "💭 Work Area sets the range that renders or previews. Drag the ends to set it.",
            },
            TimelineTool {
                name: "Time Remap",
                hotkey: "Ctrl+Alt+T",
                thought: "💭 Time Remap adds keyframes that control playback speed. 100% is normal. 0% is frozen.",
            },
            TimelineTool {
                name: "Keyframe Nav",
                hotkey: "K",
                thought: "💭 Keyframe Nav jumps between keys. K goes to the next. J goes to the previous.",
            },
        ]
    }

    /// One line of tool names.
    pub fn line() -> String {
        Self::tools()
            .iter()
            .map(|t| format!("{} ({})", t.name, t.hotkey))
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /// Thought for a tool by name.
    pub fn thought(name: &str) -> &'static str {
        Self::tools()
            .into_iter()
            .find(|t| t.name == name)
            .map(|t| t.thought)
            .unwrap_or("💭 Select a timeline tool.")
    }

    /// Tool by hotkey.
    pub fn by_hotkey(key: &str) -> Option<TimelineTool> {
        Self::tools().into_iter().find(|t| t.hotkey == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_razor() {
        let tools = TimelineTools::tools();
        assert!(tools.iter().any(|t| t.name == "Razor"));
        assert!(tools.iter().any(|t| t.name == "Trim"));
        assert!(tools.iter().any(|t| t.name == "Slip"));
        assert!(tools.iter().any(|t| t.name == "Slide"));
        assert_eq!(tools.len(), 11);
    }

    #[test]
    fn thought() {
        let t = TimelineTools::thought("Razor");
        assert!(t.contains("💭"));
        assert!(t.contains("cuts"));
    }

    #[test]
    fn by_hotkey() {
        let t = TimelineTools::by_hotkey("C").unwrap();
        assert_eq!(t.name, "Razor");
    }

    #[test]
    fn differential() {
        let a = TimelineTools::by_hotkey("C").unwrap();
        let b = TimelineTools::by_hotkey("V").unwrap();
        assert_ne!(a.name, b.name);
    }
}
