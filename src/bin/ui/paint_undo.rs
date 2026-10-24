//! Paint undo steps as tile deltas (A4).
//!
//! A [`PaintStep`] is a document with its pixels stripped out plus, per
//! layer, either "same pixels as layer N of the neighbouring state", the
//! tiles that differ from it, or (for a brand-new layer) the whole raster.
//! Masks and the selection are encoded the same way at one byte per pixel.

use crate::paint::PaintDoc;
use obs_rust_bitstream::paint_engine::Selection;
use obs_rust_bitstream::tile_undo::{Delta, PlaneRef, TileDelta};
use obs_rust_bitstream::ui_canvas::Raster;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

enum Px {
    Same(usize),
    Delta(usize, TileDelta),
    Full(Arc<Raster>),
}

enum Mk {
    Same(usize),
    Delta(usize, TileDelta),
    Full(Arc<Vec<u8>>),
}

enum Sel {
    None,
    Same,
    Delta(TileDelta),
    Full(Selection),
}

pub struct PaintStep {
    /// Layer properties, groups and ids; pixel planes are empty.
    meta: PaintDoc,
    px: Vec<Px>,
    mk: Vec<Option<Mk>>,
    sel: Sel,
}

fn empty_raster() -> Arc<Raster> {
    static E: OnceLock<Arc<Raster>> = OnceLock::new();
    E.get_or_init(|| Arc::new(Raster::new(0, 0))).clone()
}

fn empty_plane() -> Arc<Vec<u8>> {
    static E: OnceLock<Arc<Vec<u8>>> = OnceLock::new();
    E.get_or_init(|| Arc::new(Vec::new())).clone()
}

fn rgba(r: &Raster, off: (i32, i32)) -> PlaneRef<'_> {
    PlaneRef::new(&r.data, r.width, r.height, 4, off)
}

fn mono(d: &[u8], w: u32, h: u32) -> PlaneRef<'_> {
    PlaneRef::new(d, w, h, 1, (0, 0))
}

impl Delta for PaintDoc {
    type Enc = PaintStep;

    fn encode(&self, rel: &PaintDoc) -> PaintStep {
        let by_id: HashMap<u32, usize> = rel
            .layers
            .iter()
            .enumerate()
            .map(|(i, l)| (l.id, i))
            .collect();
        let mut px = Vec::with_capacity(self.layers.len());
        let mut mk = Vec::with_capacity(self.layers.len());
        for l in &self.layers {
            let r = by_id.get(&l.id).map(|&j| (j, &rel.layers[j]));
            px.push(match r {
                Some((j, b)) if Arc::ptr_eq(&l.raster, &b.raster) && l.offset == b.offset => {
                    Px::Same(j)
                }
                Some((j, b)) => {
                    let base = rgba(&b.raster, b.offset);
                    let d = TileDelta::diff(rgba(&l.raster, l.offset), base);
                    if d.is_identity_for(&base) {
                        Px::Same(j)
                    } else {
                        Px::Delta(j, d)
                    }
                }
                None => Px::Full(l.raster.clone()),
            });
            mk.push(l.mask.as_ref().map(|m| {
                let rm = r.and_then(|(j, b)| b.mask.as_ref().map(|bm| (j, bm)));
                match rm {
                    Some((j, bm)) if Arc::ptr_eq(&m.data, &bm.data) => Mk::Same(j),
                    Some((j, bm)) if (bm.width, bm.height) == (m.width, m.height) => {
                        let base = mono(&bm.data, bm.width, bm.height);
                        let d = TileDelta::diff(mono(&m.data, m.width, m.height), base);
                        if d.is_identity_for(&base) {
                            Mk::Same(j)
                        } else {
                            Mk::Delta(j, d)
                        }
                    }
                    _ => Mk::Full(m.data.clone()),
                }
            }));
        }
        let sel = match (&self.selection, &rel.selection) {
            (None, _) => Sel::None,
            (Some(a), Some(b)) if a == b => Sel::Same,
            (Some(a), Some(b)) if (a.width, a.height) == (b.width, b.height) => {
                Sel::Delta(TileDelta::diff(
                    mono(&a.mask, a.width, a.height),
                    mono(&b.mask, b.width, b.height),
                ))
            }
            (Some(a), _) => Sel::Full(a.clone()),
        };
        let mut meta = self.clone();
        meta.selection = None;
        for l in &mut meta.layers {
            l.raster = empty_raster();
            if let Some(m) = &mut l.mask {
                m.data = empty_plane();
                m.invalidate();
            }
        }
        PaintStep { meta, px, mk, sel }
    }

    fn decode(e: &PaintStep, rel: &PaintDoc) -> PaintDoc {
        let mut d = e.meta.clone();
        for (i, l) in d.layers.iter_mut().enumerate() {
            l.raster = match &e.px[i] {
                Px::Same(j) => rel.layers[*j].raster.clone(),
                Px::Delta(j, t) => {
                    let b = &rel.layers[*j];
                    Arc::new(Raster {
                        width: t.width,
                        height: t.height,
                        data: t.apply(rgba(&b.raster, b.offset)),
                    })
                }
                Px::Full(r) => r.clone(),
            };
            if let (Some(m), Some(enc)) = (&mut l.mask, &e.mk[i]) {
                match enc {
                    Mk::Same(j) => {
                        let mut bm = rel.layers[*j].mask.clone().expect("mask in base");
                        bm.density = m.density;
                        bm.feather = m.feather;
                        bm.invert = m.invert;
                        bm.enabled = m.enabled;
                        *m = bm;
                    }
                    Mk::Delta(j, t) => {
                        let bm = rel.layers[*j].mask.as_ref().expect("mask in base");
                        m.data = Arc::new(t.apply(mono(&bm.data, bm.width, bm.height)));
                    }
                    Mk::Full(p) => m.data = p.clone(),
                }
                m.prepare();
            }
        }
        d.selection = match &e.sel {
            Sel::None => None,
            Sel::Same => rel.selection.clone(),
            Sel::Delta(t) => {
                let b = rel.selection.as_ref().expect("selection in base");
                Some(Selection {
                    width: t.width,
                    height: t.height,
                    mask: t.apply(mono(&b.mask, b.width, b.height)),
                })
            }
            Sel::Full(s) => Some(s.clone()),
        };
        d
    }

    fn enc_bytes(e: &PaintStep) -> usize {
        let px: usize =
            e.px.iter()
                .map(|p| match p {
                    Px::Same(_) => 16,
                    Px::Delta(_, t) => t.bytes(),
                    Px::Full(r) => r.data.len() + 32,
                })
                .sum();
        let mk: usize =
            e.mk.iter()
                .flatten()
                .map(|m| match m {
                    Mk::Same(_) => 16,
                    Mk::Delta(_, t) => t.bytes(),
                    Mk::Full(p) => p.len() + 32,
                })
                .sum();
        let sel = match &e.sel {
            Sel::Delta(t) => t.bytes(),
            Sel::Full(s) => s.mask.len(),
            _ => 0,
        };
        px + mk + sel + 256 * (e.meta.layers.len() + 1)
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::AppCommand as C;
    use crate::paint::{PaintDoc, PaintState};
    use crate::Toasts;
    use obs_rust_bitstream::paint_engine::PaintLayer;
    use obs_rust_bitstream::ui_canvas::Raster;

    /// Everything visible about a document, independent of how layers are
    /// stored (trimmed or not): props plus each layer baked to doc size.
    fn fingerprint(d: &PaintDoc) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(format!("{}x{} sel{} ", d.width, d.height, d.selected).as_bytes());
        for g in &d.groups {
            out.extend_from_slice(format!("{:?}", g).as_bytes());
        }
        for l in &d.layers {
            out.extend_from_slice(
                format!(
                    "|{} {} {} {} {} {:?} {:?} {} {:?}",
                    l.id, l.name, l.visible, l.opacity, l.fill, l.blend, l.group, l.locked, l.text
                )
                .as_bytes(),
            );
            let mut b = l.clone();
            b.bake_to(d.width, d.height);
            out.extend_from_slice(&b.raster.data);
            if let Some(m) = &l.mask {
                out.extend_from_slice(
                    format!(
                        "m{}x{} {} {} {} {}",
                        m.width, m.height, m.density, m.feather, m.invert, m.enabled
                    )
                    .as_bytes(),
                );
                out.extend_from_slice(&m.data);
            }
        }
        if let Some(s) = &d.selection {
            out.extend_from_slice(format!("S{}x{}", s.width, s.height).as_bytes());
            out.extend_from_slice(&s.mask);
        }
        out
    }

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn f(&mut self, hi: f32) -> f32 {
            (self.next() % 10_000) as f32 / 10_000.0 * hi
        }
        fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
            &v[(self.next() % v.len() as u64) as usize]
        }
    }

    fn random_op(p: &PaintState, r: &mut Rng) -> C {
        let (w, h) = (p.width() as f32, p.height() as f32);
        let names = p
            .doc
            .layers
            .iter()
            .map(|l| l.name.clone())
            .collect::<Vec<_>>();
        let layer = Some(r.pick(&names).clone());
        match r.next() % 16 {
            0..=3 => C::BrushStroke {
                points: (0..3).map(|_| [r.f(w), r.f(h)]).collect(),
            },
            4 => C::AddLayer {
                name: Some(format!("L{}", r.next() % 1000)),
            },
            5 => C::DuplicateLayer { layer },
            6 if names.len() > 2 => C::RemoveLayer { layer },
            7 => C::MoveLayer {
                layer,
                direction: r.pick(&["up", "down"]).to_string(),
            },
            8 => C::SetLayerOpacity {
                layer,
                opacity: r.f(100.0),
            },
            9 => C::OffsetLayer {
                layer,
                dx: (r.next() % 80) as i32 - 40,
                dy: (r.next() % 80) as i32 - 40,
            },
            10 => C::AddMask { layer },
            11 => C::SetMask {
                layer,
                density: Some(r.f(1.0)),
                feather: Some(r.f(8.0)),
                invert: Some(r.next().is_multiple_of(2)),
                enabled: None,
            },
            12 => C::SelectRect {
                x: r.f(w),
                y: r.f(h),
                w: r.f(w / 2.0) + 1.0,
                h: r.f(h / 2.0) + 1.0,
            },
            13 => r
                .pick(&[C::Deselect, C::InvertSelection, C::SelectAll])
                .clone(),
            14 => C::ApplyFilter {
                filter: r.pick(&["invert", "grayscale", "blur"]).to_string(),
                strength: Some(2.0),
            },
            _ => C::Gradient {
                x0: r.f(w),
                y0: r.f(h),
                x1: r.f(w),
                y1: r.f(h),
            },
        }
    }

    /// Random edits interleaved with undo and redo, checked step by step
    /// against a model that keeps every whole state.
    #[test]
    fn delta_undo_redo_is_exact_against_whole_snapshots() {
        let mut toasts = Toasts::default();
        let mut p = PaintState::new(&mut toasts);
        let mut r = Rng(0x9e37_79b9_7f4a_7c15);
        let mut past: Vec<Vec<u8>> = Vec::new();
        let mut future: Vec<Vec<u8>> = Vec::new();
        let mut cur = fingerprint(&p.doc);
        let (mut undos, mut redos) = (0, 0);
        for step in 0..160 {
            match r.next() % 10 {
                0..=1 => {
                    let ok = p.undo();
                    assert_eq!(ok, !past.is_empty(), "step {step}");
                    if ok {
                        future.push(std::mem::replace(&mut cur, past.pop().unwrap()));
                        undos += 1;
                    }
                }
                2 => {
                    let ok = p.redo();
                    assert_eq!(ok, !future.is_empty(), "step {step}");
                    if ok {
                        past.push(std::mem::replace(&mut cur, future.pop().unwrap()));
                        redos += 1;
                    }
                }
                _ => {
                    let op = random_op(&p, &mut r);
                    let n0 = p.undo_stats().0;
                    let _ = p.apply(&op);
                    p.end_coalesce();
                    let n1 = p.undo_stats().0;
                    assert!(n1 == n0 || n1 == n0 + 1, "{op:?} made {} steps", n1 - n0);
                    if n1 == n0 + 1 {
                        past.push(cur);
                        future.clear();
                    }
                    cur = fingerprint(&p.doc);
                    continue;
                }
            }
            assert!(
                fingerprint(&p.doc) == cur,
                "state differs after step {step}"
            );
            assert_eq!(p.undo_stats().1, future.len(), "redo depth at {step}");
        }
        // unwind everything that is left, then replay it
        while p.undo() {
            future.push(std::mem::replace(&mut cur, past.pop().unwrap()));
            assert!(fingerprint(&p.doc) == cur);
        }
        assert!(past.is_empty());
        while p.redo() {
            past.push(std::mem::replace(&mut cur, future.pop().unwrap()));
            assert!(fingerprint(&p.doc) == cur);
        }
        assert!(future.is_empty());
        assert!(undos > 10 && redos > 3, "{undos} undos {redos} redos");
    }

    fn layer(id: u32, w: u32, h: u32, off: (i32, i32), seed: u8) -> PaintLayer {
        let mut r = Raster::new(w, h);
        for (k, px) in r.data.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            *px = [(k as u8).wrapping_mul(seed), seed, (k >> 8) as u8, 255];
        }
        let mut l = PaintLayer::new(id, &format!("Layer {id}"), r);
        l.offset = off;
        l
    }

    /// The M2 acceptance case: an 8K document with 20 layers and 40 undo
    /// steps. Three layers cover the whole canvas (a photo, a paper texture,
    /// a colour wash); the other 17 hold artwork of 1-2K pixels a side, as
    /// real layered documents do. Old cost of the history alone: 40 whole
    /// document clones of the edited layers (~132.7 MB per full 8K layer).
    #[test]
    fn an_8k_20_layer_document_with_40_undo_steps_stays_under_2_gb() {
        const W: u32 = 7680;
        const H: u32 = 4320;
        let mut toasts = Toasts::default();
        let mut p = PaintState::new(&mut toasts);
        let mut layers = Vec::new();
        for i in 0..3 {
            layers.push(layer(i + 1, W, H, (0, 0), 3 + i as u8));
        }
        for i in 3..20u32 {
            let (w, h) = (1000 + 60 * i, 800 + 50 * i);
            let off = ((i * 331 % (W - w)) as i32, (i * 197 % (H - h)) as i32);
            layers.push(layer(i + 1, w, h, off, 7 + i as u8));
        }
        p.set_doc(PaintDoc {
            layers,
            groups: Vec::new(),
            width: W,
            height: H,
            selection: None,
            selected: 0,
            next_id: 21,
        });
        let _ = p.apply(&C::SetBrush {
            size: Some(120.0),
            hardness: None,
            flow: None,
            color: Some("#ff3366".into()),
            exposure: None,
        });
        let names: Vec<String> = p.doc.layers.iter().map(|l| l.name.clone()).collect();
        let mut r = Rng(42);
        for s in 0..40 {
            // strokes alternate between the full layers and the partial ones
            let li = if s % 2 == 0 { s % 3 } else { 3 + s % 17 };
            p.apply(&C::SelectLayer {
                layer: names[li].clone(),
            })
            .unwrap()
            .unwrap();
            // touch-ups land on the layer's own artwork
            let l = &p.doc.layers[p.find_layer(Some(&names[li])).unwrap()];
            let (ox, oy) = (l.offset.0 as f32, l.offset.1 as f32);
            let (lw, lh) = (l.raster.width as f32, l.raster.height as f32);
            let (cx, cy) = (ox + r.f(lw * 0.6), oy + r.f(lh * 0.6));
            let pts = (0..4)
                .map(|_| [cx + r.f(lw * 0.4), cy + r.f(lh * 0.4)])
                .collect::<Vec<_>>();
            p.apply(&C::BrushStroke { points: pts }).unwrap().unwrap();
            p.end_coalesce();
        }
        let (undo, _, deltas, budget) = p.undo_stats();
        assert!(undo >= 40, "only {undo} steps kept");
        let held = p.retained_bytes();
        let full_layer = (W * H * 4) as usize;
        eprintln!(
            "8K x20 layers, {undo} steps: retained {:.1} MB (document {:.1} MB, deltas {:.1} MB, \
             budget {} MB); old model: every layer canvas-sized ({:.1} MB) plus one whole \
             layer copy per step ({:.1} MB) = {:.1} MB",
            held as f64 / 1e6,
            p.pixel_bytes() as f64 / 1e6,
            deltas as f64 / 1e6,
            budget >> 20,
            20.0 * full_layer as f64 / 1e6,
            40.0 * full_layer as f64 / 1e6,
            60.0 * full_layer as f64 / 1e6,
        );
        assert!(deltas <= budget);
        assert!(held < 2_000_000_000, "retained {held} bytes");
        // and the newest steps still undo exactly
        p.settle();
        let top: Vec<Vec<u8>> = p.doc.layers.iter().map(|l| l.raster.data.clone()).collect();
        assert!(p.undo() && p.undo());
        assert!(p.redo() && p.redo());
        for (l, t) in p.doc.layers.iter().zip(&top) {
            assert!(l.raster.data == *t, "{} changed", l.name);
        }
    }
}
