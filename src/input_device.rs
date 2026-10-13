//! Safe pure InputDevice (keyboard / mouse core).
//!
//! Holds the last key and the last click. No unwraps.
//!
//! Differential: press + click round-trips.

#[derive(Clone, Debug, PartialEq)]
pub struct InputDevice {
    last_key: Option<String>,
    click_x: Option<f32>,
    click_y: Option<f32>,
}

impl InputDevice {
    pub fn new() -> Self {
        Self {
            last_key: None,
            click_x: None,
            click_y: None,
        }
    }

    pub fn press(&mut self, key: &str) {
        if !key.is_empty() {
            self.last_key = Some(key.to_string());
        }
    }

    pub fn last_key(&self) -> Option<&str> {
        self.last_key.as_deref()
    }

    pub fn click(&mut self, x: f32, y: f32) {
        self.click_x = Some(x);
        self.click_y = Some(y);
    }

    pub fn last_click(&self) -> Option<(f32, f32)> {
        match (self.click_x, self.click_y) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        }
    }

    pub fn clear(&mut self) {
        self.last_key = None;
        self.click_x = None;
        self.click_y = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let d = InputDevice::new();
        assert!(d.last_key().is_none());
    }

    #[test]
    fn differential() {
        let mut d = InputDevice::new();
        d.press("space");
        d.click(10.0, 20.0);
        assert_eq!(d.last_key(), Some("space"));
        assert_eq!(d.last_click(), Some((10.0, 20.0)));
        d.clear();
        assert!(d.last_key().is_none());
    }
}
