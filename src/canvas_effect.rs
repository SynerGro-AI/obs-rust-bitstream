//! Safe pure CanvasEffect (optional LiveWeights brightness).
//!
//! Applies a brightness offset to a canvas only when the
//! weight "effect.brightness" is set. Missing key is a no-op.
//! Offset is in the 0.0..=1.0 canvas range and clamped.
//! No unwraps.
//!
//! Differential: a positive offset raises the mean luma.

use crate::live_weights::LiveWeights;
use crate::pixel_canvas::PixelCanvas;

pub struct CanvasEffect;

impl CanvasEffect {
    /// Apply brightness if the weight is present.
    /// Returns true if the effect ran.
    pub fn apply(canvas: &mut PixelCanvas, weights: &LiveWeights) -> bool {
        let offset = match weights.get("effect.brightness") {
            Ok(v) => v,
            Err(_) => return false,
        };
        let shift = offset.clamp(-1.0, 1.0);
        if shift == 0.0 {
            return true;
        }
        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                if let Ok((r, g, b)) = canvas.get(x, y) {
                    let _ = canvas.set(
                        x,
                        y,
                        (r + shift).clamp(0.0, 1.0),
                        (g + shift).clamp(0.0, 1.0),
                        (b + shift).clamp(0.0, 1.0),
                    );
                }
            }
        }
        true
    }

    /// Mean luma of the canvas (0.0 if empty).
    pub fn mean(canvas: &PixelCanvas) -> f64 {
        let n = canvas.width() as usize * canvas.height() as usize;
        if n == 0 {
            return 0.0;
        }
        let mut sum = 0.0f64;
        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                if let Ok((r, _, _)) = canvas.get(x, y) {
                    sum += r as f64;
                }
            }
        }
        sum / n as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_is_noop() {
        let mut c = PixelCanvas::new(2, 2).unwrap();
        c.fill(0.2, 0.2, 0.2);
        let w = LiveWeights::new();
        assert!(!CanvasEffect::apply(&mut c, &w));
        assert!((CanvasEffect::mean(&c) - 0.2).abs() < 0.01);
    }

    #[test]
    fn raises() {
        let mut c = PixelCanvas::new(2, 2).unwrap();
        c.fill(0.2, 0.2, 0.2);
        let mut w = LiveWeights::new();
        w.set("effect.brightness", 0.3).unwrap();
        assert!(CanvasEffect::apply(&mut c, &w));
        assert!(CanvasEffect::mean(&c) > 0.2);
    }

    #[test]
    fn differential() {
        let mut a = PixelCanvas::new(2, 2).unwrap();
        let mut b = PixelCanvas::new(2, 2).unwrap();
        a.fill(0.2, 0.2, 0.2);
        b.fill(0.2, 0.2, 0.2);
        let mut wa = LiveWeights::new();
        let mut wb = LiveWeights::new();
        wa.set("effect.brightness", 0.1).unwrap();
        wb.set("effect.brightness", 0.4).unwrap();
        CanvasEffect::apply(&mut a, &wa);
        CanvasEffect::apply(&mut b, &wb);
        assert!(CanvasEffect::mean(&b) > CanvasEffect::mean(&a));
    }
}
