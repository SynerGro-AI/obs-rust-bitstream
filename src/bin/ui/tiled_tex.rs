//! GPU textures for images bigger than one texture (A9).
//!
//! A GPU caps each texture side (`max_texture_side`: often 8192 or 16384,
//! 2048 on old hardware), and egui's renderers refuse anything larger, so
//! one `load_texture` of a 20000 px document fails. A [`TiledTexture`]
//! splits the image into tiles no bigger than that (at most [`MAX_TILE`]),
//! uploads dirty rectangles only to the tiles they touch, and draws each
//! tile into its share of the on-screen rectangle.
//!
//! Tiles overlap by one texel on each inner edge and draw only their core,
//! so bilinear filtering samples real neighbours across tile seams instead
//! of clamping to the tile edge.

use eframe::egui::{self, Color32, Pos2, Rect};
use obs_rust_bitstream::ui_canvas::{PixelRect, Raster};

/// Largest tile side used even when the GPU allows more: smaller tiles keep
/// partial uploads cheap and spread over the texture cache.
pub const MAX_TILE: u32 = 4096;

/// The tile side for this context: the GPU's limit, capped at [`MAX_TILE`].
pub fn tile_side(ctx: &egui::Context) -> u32 {
    (ctx.input(|i| i.max_texture_side) as u32).clamp(64, MAX_TILE)
}

/// One tile: the image region it draws (`core`) and the region its texture
/// holds (`tex`, the core plus a one-texel apron inside the image).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub core: PixelRect,
    pub tex: PixelRect,
}

/// Split a `w`×`h` image into tiles whose textures are at most `side`.
pub fn plan(w: u32, h: u32, side: u32) -> Vec<Tile> {
    let full = PixelRect { x: 0, y: 0, w, h };
    if w <= side && h <= side {
        return vec![Tile {
            core: full,
            tex: full,
        }];
    }
    let stride = side.saturating_sub(2).max(1);
    let mut out = Vec::new();
    for y0 in (0..h).step_by(stride as usize) {
        for x0 in (0..w).step_by(stride as usize) {
            let core = PixelRect {
                x: x0,
                y: y0,
                w: stride.min(w - x0),
                h: stride.min(h - y0),
            };
            let (tx, ty) = (x0.saturating_sub(1), y0.saturating_sub(1));
            let (tx1, ty1) = ((x0 + core.w + 1).min(w), (y0 + core.h + 1).min(h));
            out.push(Tile {
                core,
                tex: PixelRect {
                    x: tx,
                    y: ty,
                    w: tx1 - tx,
                    h: ty1 - ty,
                },
            });
        }
    }
    out
}

fn intersect(a: PixelRect, b: PixelRect) -> Option<PixelRect> {
    let (x0, y0) = (a.x.max(b.x), a.y.max(b.y));
    let (x1, y1) = ((a.x + a.w).min(b.x + b.w), (a.y + a.h).min(b.y + b.h));
    (x1 > x0 && y1 > y0).then(|| PixelRect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    })
}

fn image(r: &Raster, rect: PixelRect) -> egui::ColorImage {
    let full = rect.x == 0 && rect.y == 0 && rect.w == r.width && rect.h == r.height;
    let sub;
    let src = if full {
        r
    } else {
        sub = r
            .cropped(rect)
            .unwrap_or_else(|| Raster::new(rect.w, rect.h));
        &sub
    };
    egui::ColorImage::from_rgba_unmultiplied([src.width as usize, src.height as usize], &src.data)
}

/// An image on the GPU as a grid of textures.
pub struct TiledTexture {
    w: u32,
    h: u32,
    opts: egui::TextureOptions,
    tiles: Vec<(Tile, egui::TextureHandle)>,
}

impl TiledTexture {
    pub fn new(
        ctx: &egui::Context,
        name: &str,
        r: &Raster,
        side: u32,
        opts: egui::TextureOptions,
    ) -> Self {
        let tiles = plan(r.width, r.height, side)
            .into_iter()
            .enumerate()
            .map(|(i, t)| {
                let tex = ctx.load_texture(format!("{name}#{i}"), image(r, t.tex), opts);
                (t, tex)
            })
            .collect();
        Self {
            w: r.width,
            h: r.height,
            opts,
            tiles,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Re-upload `dirty` (image pixels) from `r` to the tiles it touches.
    pub fn update(&mut self, r: &Raster, dirty: PixelRect) {
        for (t, tex) in &mut self.tiles {
            let Some(i) = intersect(t.tex, dirty) else {
                continue;
            };
            if i == t.tex {
                tex.set(image(r, i), self.opts);
            } else {
                tex.set_partial(
                    [(i.x - t.tex.x) as usize, (i.y - t.tex.y) as usize],
                    image(r, i),
                    self.opts,
                );
            }
        }
    }

    /// Draw the whole image into `rect` (screen space).
    pub fn paint(&self, painter: &egui::Painter, rect: Rect, tint: Color32) {
        let (sx, sy) = (
            rect.width() / self.w.max(1) as f32,
            rect.height() / self.h.max(1) as f32,
        );
        for (t, tex) in &self.tiles {
            let c = t.core;
            let screen = Rect::from_min_max(
                Pos2::new(rect.min.x + c.x as f32 * sx, rect.min.y + c.y as f32 * sy),
                Pos2::new(
                    rect.min.x + (c.x + c.w) as f32 * sx,
                    rect.min.y + (c.y + c.h) as f32 * sy,
                ),
            );
            let (tw, th) = (t.tex.w as f32, t.tex.h as f32);
            let uv = Rect::from_min_max(
                Pos2::new((c.x - t.tex.x) as f32 / tw, (c.y - t.tex.y) as f32 / th),
                Pos2::new(
                    (c.x + c.w - t.tex.x) as f32 / tw,
                    (c.y + c.h - t.tex.y) as f32 / th,
                ),
            );
            painter.image(tex.id(), screen, uv, tint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn tiles_cover_every_pixel_once_and_fit_the_texture_limit() {
        for (w, h, side) in [
            (100, 80, 128),
            (300, 200, 64),
            (129, 64, 64),
            (65, 65, 64),
            (20000, 9000, 4096),
            (16384, 16384, 8192),
            (1, 5000, 4096),
        ] {
            let tiles = plan(w, h, side);
            let mut cover = vec![0u8; (w as usize) * (h as usize)];
            for t in &tiles {
                assert!(t.tex.w <= side && t.tex.h <= side, "{:?}", t);
                assert!(t.tex.x + t.tex.w <= w && t.tex.y + t.tex.h <= h);
                assert!(t.tex.x <= t.core.x && t.core.x + t.core.w <= t.tex.x + t.tex.w);
                assert!(t.tex.y <= t.core.y && t.core.y + t.core.h <= t.tex.y + t.tex.h);
                // an apron on every inner edge
                if t.core.x > 0 {
                    assert_eq!(t.tex.x + 1, t.core.x);
                }
                if t.core.x + t.core.w < w {
                    assert_eq!(t.tex.x + t.tex.w, t.core.x + t.core.w + 1);
                }
                if w * h <= 10_000_000 {
                    for y in t.core.y..t.core.y + t.core.h {
                        for x in t.core.x..t.core.x + t.core.w {
                            cover[(y * w + x) as usize] += 1;
                        }
                    }
                }
            }
            if w * h <= 10_000_000 {
                assert!(cover.iter().all(|c| *c == 1), "{w}x{h} side {side}");
            } else {
                let area: u64 = tiles
                    .iter()
                    .map(|t| t.core.w as u64 * t.core.h as u64)
                    .sum();
                assert_eq!(area, w as u64 * h as u64);
            }
        }
        assert_eq!(plan(4096, 4096, 4096).len(), 1);
        assert_eq!(plan(20000, 9000, 4096).len(), 5 * 3);
    }

    fn pattern(w: u32, h: u32, seed: u32) -> Raster {
        let mut r = Raster::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let v = x.wrapping_mul(2654435761) ^ y.wrapping_mul(40503) ^ seed;
                r.data[i..i + 4].copy_from_slice(&[v as u8, (v >> 8) as u8, (v >> 16) as u8, 255]);
            }
        }
        r
    }

    /// The GPU side of egui, on the CPU: apply texture deltas, then
    /// rasterise the frame's meshes with nearest sampling at pixel centres.
    struct SoftGpu {
        tex: HashMap<egui::TextureId, egui::ColorImage>,
    }

    impl SoftGpu {
        fn apply(&mut self, d: &egui::TexturesDelta) {
            for (id, delta) in &d.set {
                let egui::ImageData::Color(img) = &delta.image;
                match delta.pos {
                    None => {
                        self.tex.insert(*id, (**img).clone());
                    }
                    Some([px, py]) => {
                        let t = self.tex.get_mut(id).expect("partial before full");
                        for y in 0..img.size[1] {
                            for x in 0..img.size[0] {
                                t.pixels[(py + y) * t.size[0] + px + x] =
                                    img.pixels[y * img.size[0] + x];
                            }
                        }
                    }
                }
            }
        }

        /// Colour at each pixel of `area`, plus how many textured triangles
        /// covered it (so gaps and overlaps show up).
        fn render(
            &self,
            ctx: &egui::Context,
            out: egui::FullOutput,
            area: PixelRect,
        ) -> (Vec<Color32>, Vec<u8>) {
            let n = (area.w * area.h) as usize;
            let (mut px, mut hits) = (vec![Color32::TRANSPARENT; n], vec![0u8; n]);
            for p in ctx.tessellate(out.shapes, out.pixels_per_point) {
                let egui::epaint::Primitive::Mesh(m) = p.primitive else {
                    continue;
                };
                let Some(t) = self.tex.get(&m.texture_id) else {
                    continue;
                };
                if m.texture_id == egui::TextureId::default() {
                    continue; // font atlas
                }
                for tri in m.indices.chunks(3) {
                    let v = [
                        m.vertices[tri[0] as usize],
                        m.vertices[tri[1] as usize],
                        m.vertices[tri[2] as usize],
                    ];
                    for y in area.y..area.y + area.h {
                        for x in area.x..area.x + area.w {
                            let p = Pos2::new(x as f32 + 0.5, y as f32 + 0.5);
                            let Some(b) = bary(p, v[0].pos, v[1].pos, v[2].pos) else {
                                continue;
                            };
                            let u = b[0] * v[0].uv.x + b[1] * v[1].uv.x + b[2] * v[2].uv.x;
                            let w = b[0] * v[0].uv.y + b[1] * v[1].uv.y + b[2] * v[2].uv.y;
                            let tx = ((u * t.size[0] as f32) as usize).min(t.size[0] - 1);
                            let ty = ((w * t.size[1] as f32) as usize).min(t.size[1] - 1);
                            let k = ((y - area.y) * area.w + x - area.x) as usize;
                            px[k] = t.pixels[ty * t.size[0] + tx];
                            hits[k] += 1;
                        }
                    }
                }
            }
            (px, hits)
        }
    }

    /// Barycentric weights of `p` in a triangle, half-open on shared edges
    /// (top-left rule) so a pixel on a seam belongs to one triangle.
    fn bary(p: Pos2, a: Pos2, b: Pos2, c: Pos2) -> Option<[f32; 3]> {
        let area = (b - a).x * (c - a).y - (b - a).y * (c - a).x;
        if area.abs() < 1e-9 {
            return None;
        }
        let edge =
            |p0: Pos2, p1: Pos2| ((p1 - p0).x * (p - p0).y - (p1 - p0).y * (p - p0).x) / area;
        let w = [edge(b, c), edge(c, a), edge(a, b)];
        let top_left = |p0: Pos2, p1: Pos2| {
            let d = if area > 0.0 { p1 - p0 } else { p0 - p1 };
            (d.y == 0.0 && d.x > 0.0) || d.y < 0.0
        };
        let tl = [top_left(b, c), top_left(c, a), top_left(a, b)];
        for k in 0..3 {
            if w[k] < 0.0 || (w[k] == 0.0 && !tl[k]) {
                return None;
            }
        }
        Some(w)
    }

    fn input(w: u32, h: u32, max_side: usize) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(w as f32 + 20.0, h as f32 + 20.0),
            )),
            max_texture_side: Some(max_side),
            ..Default::default()
        }
    }

    #[test]
    fn the_tile_side_follows_the_gpu_limit_up_to_4096() {
        let ctx = egui::Context::default();
        for (gpu, want) in [(2048, 2048), (8192, 4096), (16384, 4096)] {
            let _ = ctx.run(input(100, 100, gpu), |ctx| {
                assert_eq!(tile_side(ctx), want);
            });
        }
    }

    #[test]
    fn a_tiled_image_renders_pixel_exact_and_partial_updates_land() {
        let (w, h) = (300u32, 170u32);
        let ctx = egui::Context::default();
        ctx.set_pixels_per_point(1.0);
        let mut img = pattern(w, h, 0);
        let mut tt: Option<TiledTexture> = None;
        let mut gpu = SoftGpu {
            tex: HashMap::new(),
        };
        let area = PixelRect { x: 10, y: 10, w, h };
        let mut frame = |img: &Raster, tt: &mut Option<TiledTexture>, dirty: Option<PixelRect>| {
            let out = ctx.run(input(w, h, 2048), |ctx| {
                let side = 64;
                match (tt.as_mut(), dirty) {
                    (None, _) => {
                        *tt = Some(TiledTexture::new(
                            ctx,
                            "t",
                            img,
                            side,
                            egui::TextureOptions::NEAREST,
                        ))
                    }
                    (Some(t), Some(d)) => t.update(img, d),
                    _ => {}
                }
                let painter = ctx.layer_painter(egui::LayerId::background());
                let r = Rect::from_min_size(Pos2::new(10.0, 10.0), egui::vec2(w as f32, h as f32));
                tt.as_ref().unwrap().paint(&painter, r, Color32::WHITE);
            });
            gpu.apply(&out.textures_delta);
            for (id, t) in &gpu.tex {
                if *id != egui::TextureId::default() {
                    assert!(t.size[0] <= 64 && t.size[1] <= 64, "{:?}", t.size);
                }
            }
            gpu.render(&ctx, out, area)
        };
        let expect = |img: &Raster| -> Vec<Color32> {
            img.data
                .chunks(4)
                .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                .collect()
        };
        let (px, hits) = frame(&img, &mut tt, None);
        let n_tiles = tt.as_ref().unwrap().tile_count();
        assert_eq!(n_tiles, plan(w, h, 64).len());
        assert!(n_tiles > 1);
        assert!(
            hits.iter().all(|h| *h == 1),
            "every pixel drawn exactly once"
        );
        assert!(px == expect(&img), "tiles reassemble the image");
        // paint across a tile seam and upload only that rectangle
        let d = PixelRect {
            x: 50,
            y: 40,
            w: 40,
            h: 50,
        };
        for y in d.y..d.y + d.h {
            for x in d.x..d.x + d.w {
                let i = ((y * w + x) * 4) as usize;
                img.data[i..i + 4].copy_from_slice(&[255, 0, (x + y) as u8, 255]);
            }
        }
        let (px, hits) = frame(&img, &mut tt, Some(d));
        assert!(hits.iter().all(|h| *h == 1));
        assert!(
            px == expect(&img),
            "the edit shows in every tile it touches"
        );
    }
}
