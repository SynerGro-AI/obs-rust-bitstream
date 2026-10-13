//! Safe pure AiDirector (scene picker core).
//!
//! Takes a line of speech or a detected action and returns a
//! scene. No unwraps.
//!
//! Differential: "sunset" picks sunset, "shootout" picks western.

use crate::ai::AiAssistant;
use crate::effect_list::EffectList;
use crate::storyboard::Storyboard;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneKind {
    WeatherMap,
    Sunset,
    Western,
    LiveCamera,
    Default,
}

pub struct AiDirector;

impl AiDirector {
    /// Pick a scene from a line of speech or an action.
    pub fn pick(ai: &AiAssistant, line: &str) -> SceneKind {
        let r = ai.assist(line).to_lowercase();
        let l = line.to_lowercase();
        let text = format!("{r} {l}");
        if text.contains("weather") {
            SceneKind::WeatherMap
        } else if text.contains("sunset") {
            SceneKind::Sunset
        } else if text.contains("shoot") || text.contains("western") {
            SceneKind::Western
        } else if text.contains("trump") || text.contains("live camera") {
            SceneKind::LiveCamera
        } else {
            SceneKind::Default
        }
    }

    /// Background color for a scene kind.
    pub fn color(kind: SceneKind) -> (f32, f32, f32) {
        match kind {
            SceneKind::WeatherMap => (0.3, 0.6, 0.9),
            SceneKind::Sunset => (0.9, 0.4, 0.2),
            SceneKind::Western => (0.6, 0.4, 0.2),
            SceneKind::LiveCamera => (0.2, 0.2, 0.2),
            SceneKind::Default => (0.1, 0.1, 0.15),
        }
    }

    /// Pick a scene from the current storyboard beat.
    pub fn from_board(ai: &AiAssistant, board: &Storyboard) -> SceneKind {
        match board.current() {
            Some(line) => Self::pick(ai, line),
            None => SceneKind::Default,
        }
    }

    /// Trigger the effect that matches a scene kind.
    pub fn trigger(kind: SceneKind, list: &mut EffectList) {
        let name = match kind {
            SceneKind::WeatherMap => "weather",
            SceneKind::Sunset => "sunset",
            SceneKind::Western => "western",
            SceneKind::LiveCamera => "live",
            SceneKind::Default => return,
        };
        let _ = list.add(name);
        let _ = list.trigger(name, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::AiConfig;

    #[test]
    fn pick() {
        let ai = AiAssistant::new(AiConfig::seraphina());
        assert_eq!(AiDirector::pick(&ai, "sunset"), SceneKind::Sunset);
        assert_eq!(AiDirector::pick(&ai, "shootout"), SceneKind::Western);
    }

    #[test]
    fn differential() {
        let ai = AiAssistant::new(AiConfig::seraphina());
        let w = AiDirector::pick(&ai, "weather in Calgary");
        let t = AiDirector::pick(&ai, "Trump live");
        assert_eq!(w, SceneKind::WeatherMap);
        assert_eq!(t, SceneKind::LiveCamera);
        assert_ne!(AiDirector::color(w), AiDirector::color(t));
    }

    #[test]
    fn from_board() {
        let ai = AiAssistant::new(AiConfig::seraphina());
        let mut board = Storyboard::new();
        board.add("sunset");
        assert_eq!(AiDirector::from_board(&ai, &board), SceneKind::Sunset);
    }

    #[test]
    fn trigger() {
        let mut list = EffectList::new();
        AiDirector::trigger(SceneKind::Western, &mut list);
        assert!(list.get("western").unwrap().active());
    }
}
