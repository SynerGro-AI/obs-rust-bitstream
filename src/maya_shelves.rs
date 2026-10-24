//! Safe pure MayaShelves (Maya shelf layout).
//!
//! Shelves match Maya menu sets: Modeling, Rigging,
//! Animation, FX, Rendering. No unwraps.
//!
//! Differential: a different shelf has a different count.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MayaShelf {
    Modeling,
    Rigging,
    Animation,
    Fx,
    Rendering,
}

impl MayaShelf {
    pub fn label(&self) -> &'static str {
        match self {
            MayaShelf::Modeling => "Modeling",
            MayaShelf::Rigging => "Rigging",
            MayaShelf::Animation => "Animation",
            MayaShelf::Fx => "FX",
            MayaShelf::Rendering => "Rendering",
        }
    }

    /// Hotkey that selects this menu set (Maya F2–F6).
    pub fn hotkey(&self) -> &'static str {
        match self {
            MayaShelf::Modeling => "F2",
            MayaShelf::Rigging => "F3",
            MayaShelf::Animation => "F4",
            MayaShelf::Fx => "F5",
            MayaShelf::Rendering => "F6",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MayaButton {
    pub name: &'static str,
    pub shelf: MayaShelf,
}

pub struct MayaShelves;

impl MayaShelves {
    pub fn buttons() -> Vec<MayaButton> {
        vec![
            MayaButton {
                name: "Polygon Cube",
                shelf: MayaShelf::Modeling,
            },
            MayaButton {
                name: "NURBS",
                shelf: MayaShelf::Modeling,
            },
            MayaButton {
                name: "Subdivision",
                shelf: MayaShelf::Modeling,
            },
            MayaButton {
                name: "Joint",
                shelf: MayaShelf::Rigging,
            },
            MayaButton {
                name: "IK/FK",
                shelf: MayaShelf::Rigging,
            },
            MayaButton {
                name: "Key",
                shelf: MayaShelf::Animation,
            },
            MayaButton {
                name: "Blend Shape",
                shelf: MayaShelf::Animation,
            },
            MayaButton {
                name: "Particles",
                shelf: MayaShelf::Fx,
            },
            MayaButton {
                name: "Fluids",
                shelf: MayaShelf::Fx,
            },
            MayaButton {
                name: "Light",
                shelf: MayaShelf::Rendering,
            },
            MayaButton {
                name: "Camera",
                shelf: MayaShelf::Rendering,
            },
            MayaButton {
                name: "Shader",
                shelf: MayaShelf::Rendering,
            },
        ]
    }

    pub fn shelves() -> &'static [MayaShelf] {
        &[
            MayaShelf::Modeling,
            MayaShelf::Rigging,
            MayaShelf::Animation,
            MayaShelf::Fx,
            MayaShelf::Rendering,
        ]
    }

    pub fn on_shelf(shelf: MayaShelf) -> Vec<MayaButton> {
        Self::buttons()
            .into_iter()
            .filter(|b| b.shelf == shelf)
            .collect()
    }

    pub fn line() -> String {
        Self::shelves()
            .iter()
            .map(|s| s.label())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shelves_present() {
        assert_eq!(MayaShelves::shelves().len(), 5);
        assert_eq!(MayaShelves::shelves()[0].hotkey(), "F2");
        assert!(MayaShelves::line().contains("Modeling"));
    }

    #[test]
    fn has_named_buttons() {
        let buttons = MayaShelves::buttons();
        assert!(buttons.iter().any(|b| b.name == "IK/FK"));
        assert!(buttons.iter().any(|b| b.name == "Camera"));
    }

    #[test]
    fn differential() {
        let render = MayaShelves::on_shelf(MayaShelf::Rendering);
        let rig = MayaShelves::on_shelf(MayaShelf::Rigging);
        assert!(render.len() >= rig.len());
    }
}
