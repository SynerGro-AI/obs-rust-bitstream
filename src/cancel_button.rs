//! Safe pure CancelButton (UI control core).
//!
//! Sets an EncodeCancel when pressed. No unwraps.
//!
//! Differential: press cancels.

use crate::encode_cancel::EncodeCancel;

#[derive(Clone, Debug, Default)]
pub struct CancelButton {
    pressed: bool,
}

impl CancelButton {
    pub fn new() -> Self {
        Self { pressed: false }
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// Press the button and cancel the encode.
    pub fn press(&mut self, cancel: &mut EncodeCancel) {
        self.pressed = true;
        cancel.cancel();
    }

    /// Release the button and reset the cancel.
    pub fn release(&mut self, cancel: &mut EncodeCancel) {
        self.pressed = false;
        cancel.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let mut button = CancelButton::new();
        let mut cancel = EncodeCancel::new();
        assert!(!button.is_pressed());
        button.press(&mut cancel);
        assert!(button.is_pressed());
        assert!(cancel.is_cancelled());
        button.release(&mut cancel);
        assert!(!button.is_pressed());
        assert!(!cancel.is_cancelled());
    }
}
