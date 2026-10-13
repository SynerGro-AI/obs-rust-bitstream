//! Safe pure GameCapture (game window holder core).
//!
//! Holds a game title and an anti-cheat flag. Empty titles
//! rejected. No unwraps.
//!
//! Differential: set + read round-trips.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameError {
    EmptyTitle,
}

#[derive(Clone, Debug)]
pub struct GameCapture {
    title: String,
    anti_cheat: bool,
}

impl GameCapture {
    pub fn new(title: &str, anti_cheat: bool) -> Result<Self, GameError> {
        if title.is_empty() {
            return Err(GameError::EmptyTitle);
        }
        Ok(Self {
            title: title.to_string(),
            anti_cheat,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn anti_cheat(&self) -> bool {
        self.anti_cheat
    }

    pub fn set_title(&mut self, title: &str) -> Result<(), GameError> {
        if title.is_empty() {
            return Err(GameError::EmptyTitle);
        }
        self.title = title.to_string();
        Ok(())
    }

    pub fn set_anti_cheat(&mut self, anti_cheat: bool) {
        self.anti_cheat = anti_cheat;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(GameCapture::new("", true).is_err());
    }

    #[test]
    fn differential() {
        let mut g = GameCapture::new("Minecraft", false).unwrap();
        assert_eq!(g.title(), "Minecraft");
        assert!(!g.anti_cheat());
        g.set_title("Valorant").unwrap();
        g.set_anti_cheat(true);
        assert_eq!(g.title(), "Valorant");
        assert!(g.anti_cheat());
    }
}
