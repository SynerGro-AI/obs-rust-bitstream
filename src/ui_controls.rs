//! Safe pure UiControls (buttons and menus).
//!
//! The controls an OBS-shaped UI needs.
//! Buttons: start/stop stream, start/stop record, settings.
//! Menus: file, edit, view, tools, help.
//! No unwraps.
//!
//! Differential: streaming flips the stream button label.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiButton {
    StartStream,
    StopStream,
    StartRecord,
    StopRecord,
    Settings,
    StudioMode,
}

impl UiButton {
    pub fn label(&self, streaming: bool, recording: bool) -> &'static str {
        match self {
            UiButton::StartStream | UiButton::StopStream => {
                if streaming {
                    "Stop Streaming"
                } else {
                    "Start Streaming"
                }
            }
            UiButton::StartRecord | UiButton::StopRecord => {
                if recording {
                    "Stop Recording"
                } else {
                    "Start Recording"
                }
            }
            UiButton::Settings => "Settings",
            UiButton::StudioMode => "Studio Mode",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiMenu {
    File,
    Edit,
    View,
    Tools,
    Help,
}

impl UiMenu {
    pub fn label(&self) -> &'static str {
        match self {
            UiMenu::File => "File",
            UiMenu::Edit => "Edit",
            UiMenu::View => "View",
            UiMenu::Tools => "Tools",
            UiMenu::Help => "Help",
        }
    }

    pub fn items(&self) -> &'static [&'static str] {
        match self {
            UiMenu::File => &["New", "Open", "Open Recent", "Save", "Settings", "Exit"],
            UiMenu::Edit => &["Undo", "Redo", "Copy", "Paste"],
            UiMenu::View => &["Fullscreen", "Studio Mode", "Stats"],
            UiMenu::Tools => &["Auto-Configuration", "Scripts"],
            UiMenu::Help => &["About", "Check for Updates"],
        }
    }
}

pub struct UiControls;

impl UiControls {
    /// The buttons the main window shows.
    pub fn buttons() -> &'static [UiButton] {
        &[
            UiButton::StartStream,
            UiButton::StartRecord,
            UiButton::Settings,
            UiButton::StudioMode,
        ]
    }

    /// The menus the main window shows.
    pub fn menus() -> &'static [UiMenu] {
        &[
            UiMenu::File,
            UiMenu::Edit,
            UiMenu::View,
            UiMenu::Tools,
            UiMenu::Help,
        ]
    }

    /// One line of button labels for the current state.
    pub fn button_line(streaming: bool, recording: bool) -> String {
        Self::buttons()
            .iter()
            .map(|b| b.label(streaming, recording))
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /// One line of menu labels.
    pub fn menu_line() -> String {
        Self::menus()
            .iter()
            .map(|m| m.label())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_present() {
        assert_eq!(UiControls::buttons().len(), 4);
        assert!(UiControls::button_line(false, false).contains("Start Streaming"));
    }

    #[test]
    fn stream_flips() {
        let idle = UiControls::button_line(false, false);
        let live = UiControls::button_line(true, false);
        assert!(idle.contains("Start Streaming"));
        assert!(live.contains("Stop Streaming"));
    }

    #[test]
    fn menus_present() {
        assert_eq!(UiControls::menus().len(), 5);
        assert!(UiControls::menu_line().contains("File"));
        assert!(UiControls::menu_line().contains("Help"));
        assert_eq!(UiMenu::File.items().len(), 6);
    }

    #[test]
    fn differential() {
        let a = UiControls::button_line(false, false);
        let b = UiControls::button_line(true, true);
        assert_ne!(a, b);
    }
}
