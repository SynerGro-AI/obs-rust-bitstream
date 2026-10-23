//! Rasterise a text layer with the bundled Ubuntu font (ab_glyph).

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use obs_rust_bitstream::paint_engine::TextSpec;
use obs_rust_bitstream::ui_canvas::Raster;

/// Render `t` into a transparent `w × h` raster. `t.y` is the top of
/// the first line. Supports multiple lines; no kerning.
pub fn render(t: &TextSpec, w: u32, h: u32) -> Raster {
    let mut r = Raster::new(w, h);
    let Ok(font) = FontRef::try_from_slice(epaint_default_fonts::UBUNTU_LIGHT) else {
        return r;
    };
    let scale = PxScale::from(t.size.max(1.0));
    let sf = font.as_scaled(scale);
    let line_h = sf.height() + sf.line_gap();
    for (li, line) in t.text.lines().enumerate() {
        let mut x = t.x;
        let baseline = t.y + sf.ascent() + li as f32 * line_h;
        let mut prev = None;
        for ch in line.chars() {
            let id = font.glyph_id(ch);
            if let Some(p) = prev {
                x += sf.kern(p, id);
            }
            let g = id.with_scale_and_position(scale, ab_glyph::point(x, baseline));
            x += sf.h_advance(id);
            prev = Some(id);
            if let Some(o) = font.outline_glyph(g) {
                let b = o.px_bounds();
                o.draw(|gx, gy, c| {
                    let px = b.min.x as i32 + gx as i32;
                    let py = b.min.y as i32 + gy as i32;
                    if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                        return;
                    }
                    let i = ((py as usize) * (w as usize) + px as usize) * 4;
                    let a = (c.clamp(0.0, 1.0) * 255.0) as u8;
                    if a > r.data[i + 3] {
                        r.data[i] = t.color[0];
                        r.data[i + 1] = t.color[1];
                        r.data[i + 2] = t.color[2];
                        r.data[i + 3] = a;
                    }
                });
            }
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_visible_pixels() {
        let t = TextSpec {
            text: "Hi\nyo".into(),
            x: 4.0,
            y: 2.0,
            size: 24.0,
            color: [255, 0, 0],
        };
        let r = render(&t, 80, 80);
        let n = r.data.chunks_exact(4).filter(|p| p[3] > 128).count();
        assert!(n > 40, "{}", n);
        assert!(r
            .data
            .chunks_exact(4)
            .filter(|p| p[3] > 0)
            .all(|p| p[0] == 255));
    }
}
