//! Tile-based undo: store only the 256×256 tiles that changed.
//!
//! [`TileDelta`] records the tiles of a *target* pixel plane that differ
//! from a *base* plane at the same document position, so the target can be
//! rebuilt exactly from the base. Planes carry an offset (layer position),
//! so a layer that grew, shrank or moved still diffs tile by tile.
//!
//! [`DeltaHistory`] is an undo/redo stack built on any state that can
//! encode itself relative to a neighbour ([`Delta`]). Only the two states
//! next to the live one are kept whole (and those share unchanged layers
//! with the live document); everything further away is a chain of deltas.
//! A byte budget evicts the oldest steps; undo and redo stay exact.

use std::collections::VecDeque;

/// Tile edge in pixels.
pub const TILE: u32 = 256;

/// Default undo memory budget per document (bytes).
pub const DEFAULT_BUDGET: usize = 512 * 1024 * 1024;

/// Undo budget from `SERAPHINA_UNDO_MB` (megabytes), else [`DEFAULT_BUDGET`].
pub fn budget_from_env() -> usize {
    std::env::var("SERAPHINA_UNDO_MB")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|mb| *mb > 0)
        .map(|mb| mb.saturating_mul(1024 * 1024))
        .unwrap_or(DEFAULT_BUDGET)
}

/// A borrowed pixel plane placed in document space.
#[derive(Clone, Copy, Debug)]
pub struct PlaneRef<'a> {
    pub data: &'a [u8],
    pub width: u32,
    pub height: u32,
    /// Bytes per pixel (4 for RGBA, 1 for masks).
    pub bpp: u32,
    /// Document position of the plane's top-left pixel.
    pub offset: (i32, i32),
}

impl<'a> PlaneRef<'a> {
    pub fn new(data: &'a [u8], width: u32, height: u32, bpp: u32, offset: (i32, i32)) -> Self {
        debug_assert_eq!(data.len(), width as usize * height as usize * bpp as usize);
        Self {
            data,
            width,
            height,
            bpp,
            offset,
        }
    }

    fn same_geometry(&self, o: &PlaneRef) -> bool {
        (self.width, self.height, self.bpp, self.offset) == (o.width, o.height, o.bpp, o.offset)
    }

    /// Copy the row segment at document row `dy`, columns `dx..dx+n`, into
    /// `out` (`n * bpp` bytes); pixels outside the plane read as zero.
    fn read_row(&self, dy: i32, dx: i32, out: &mut [u8]) {
        let bpp = self.bpp as i64;
        let n = out.len() as i64 / bpp;
        let sy = dy as i64 - self.offset.1 as i64;
        if sy < 0 || sy >= self.height as i64 {
            out.fill(0);
            return;
        }
        let sx0 = dx as i64 - self.offset.0 as i64;
        let lo = sx0.max(0);
        let hi = (sx0 + n).min(self.width as i64);
        if lo >= hi {
            out.fill(0);
            return;
        }
        let a = ((lo - sx0) * bpp) as usize;
        let b = ((hi - sx0) * bpp) as usize;
        out[..a].fill(0);
        out[b..].fill(0);
        let row = (sy * self.width as i64 * bpp) as usize;
        out[a..b].copy_from_slice(&self.data[row + (lo * bpp) as usize..row + (hi * bpp) as usize]);
    }
}

/// The tiles of a target plane that differ from a base plane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileDelta {
    pub width: u32,
    pub height: u32,
    pub bpp: u32,
    pub offset: (i32, i32),
    /// (tile index, row-major tile bytes) in the target's own grid.
    pub tiles: Vec<(u32, Box<[u8]>)>,
}

fn tile_rect(w: u32, h: u32, t: u32) -> (u32, u32, u32, u32) {
    let cols = w.div_ceil(TILE);
    let (tx, ty) = (t % cols, t / cols);
    let (x0, y0) = (tx * TILE, ty * TILE);
    (x0, y0, TILE.min(w - x0), TILE.min(h - y0))
}

impl TileDelta {
    /// Tiles of `target` that `base` does not reproduce.
    pub fn diff(target: PlaneRef, base: PlaneRef) -> Self {
        assert_eq!(target.bpp, base.bpp, "planes must share a pixel format");
        let (w, h, bpp) = (target.width, target.height, target.bpp as usize);
        let same = target.same_geometry(&base);
        let n_tiles = w.div_ceil(TILE) * h.div_ceil(TILE);
        let mut tiles = Vec::new();
        let mut scratch = vec![0u8; TILE as usize * bpp];
        for t in 0..n_tiles {
            let (x0, y0, tw, th) = tile_rect(w, h, t);
            let rb = tw as usize * bpp;
            let differs = (0..th).any(|ry| {
                let y = (y0 + ry) as usize;
                let start = (y * w as usize + x0 as usize) * bpp;
                let trow = &target.data[start..start + rb];
                if same {
                    trow != &base.data[start..start + rb]
                } else {
                    let s = &mut scratch[..rb];
                    base.read_row(y as i32 + target.offset.1, x0 as i32 + target.offset.0, s);
                    trow != &s[..]
                }
            });
            if differs {
                let mut buf = Vec::with_capacity(rb * th as usize);
                for ry in 0..th {
                    let start = (((y0 + ry) * w + x0) as usize) * bpp;
                    buf.extend_from_slice(&target.data[start..start + rb]);
                }
                tiles.push((t, buf.into_boxed_slice()));
            }
        }
        Self {
            width: w,
            height: h,
            bpp: target.bpp,
            offset: target.offset,
            tiles,
        }
    }

    /// Rebuild the target from the same base it was diffed against.
    pub fn apply(&self, base: PlaneRef) -> Vec<u8> {
        assert_eq!(self.bpp, base.bpp, "planes must share a pixel format");
        let (w, h, bpp) = (self.width, self.height, self.bpp as usize);
        let mut out = vec![0u8; w as usize * h as usize * bpp];
        let same = (w, h, self.offset) == (base.width, base.height, base.offset);
        if same {
            out.copy_from_slice(base.data);
        } else {
            let rb = w as usize * bpp;
            for y in 0..h as usize {
                base.read_row(
                    y as i32 + self.offset.1,
                    self.offset.0,
                    &mut out[y * rb..(y + 1) * rb],
                );
            }
        }
        for (t, buf) in &self.tiles {
            let (x0, y0, tw, th) = tile_rect(w, h, *t);
            let rb = tw as usize * bpp;
            for ry in 0..th as usize {
                let d = ((y0 as usize + ry) * w as usize + x0 as usize) * bpp;
                out[d..d + rb].copy_from_slice(&buf[ry * rb..(ry + 1) * rb]);
            }
        }
        out
    }

    /// True when the target is the base, pixel for pixel and in place.
    pub fn is_identity_for(&self, base: &PlaneRef) -> bool {
        self.tiles.is_empty()
            && (self.width, self.height, self.bpp, self.offset)
                == (base.width, base.height, base.bpp, base.offset)
    }

    /// Heap bytes held.
    pub fn bytes(&self) -> usize {
        self.tiles.iter().map(|(_, b)| b.len() + 16).sum::<usize>() + 48
    }
}

/// A state that can be stored relative to a neighbouring state.
pub trait Delta: Sized {
    type Enc;
    /// Encode `self` so that `decode(&enc, rel)` rebuilds it exactly.
    fn encode(&self, rel: &Self) -> Self::Enc;
    fn decode(enc: &Self::Enc, rel: &Self) -> Self;
    /// Heap bytes the encoding holds (for the budget).
    fn enc_bytes(enc: &Self::Enc) -> usize;
}

/// Undo/redo over delta-encoded states with a step limit and byte budget.
///
/// `u_anchor` is the next undo target, kept whole; each older step in
/// `u_enc` is encoded relative to the step after it (the newest relative to
/// `u_anchor`). Redo mirrors it. The anchors are clones of states the
/// document just held, so for `Arc`-backed layers they share every layer
/// that has not changed since.
pub struct DeltaHistory<S: Delta + Clone> {
    u_anchor: Option<S>,
    u_enc: VecDeque<(S::Enc, usize)>,
    r_anchor: Option<S>,
    r_enc: VecDeque<(S::Enc, usize)>,
    bytes: usize,
    max_steps: usize,
    budget: usize,
    evicted: u64,
}

impl<S: Delta + Clone> DeltaHistory<S> {
    pub fn new(max_steps: usize, budget: usize) -> Self {
        Self {
            u_anchor: None,
            u_enc: VecDeque::new(),
            r_anchor: None,
            r_enc: VecDeque::new(),
            bytes: 0,
            max_steps: max_steps.max(1),
            budget,
            evicted: 0,
        }
    }

    /// Record `before`, the state just before a change.
    pub fn push(&mut self, before: &S) {
        if let Some(a) = self.u_anchor.take() {
            let e = a.encode(before);
            let b = S::enc_bytes(&e);
            self.bytes += b;
            self.u_enc.push_back((e, b));
        }
        self.u_anchor = Some(before.clone());
        for (_, b) in self.r_enc.drain(..) {
            self.bytes -= b;
        }
        self.r_anchor = None;
        self.enforce();
    }

    /// Most recent undo state, without popping it.
    pub fn peek(&self) -> Option<&S> {
        self.u_anchor.as_ref()
    }

    pub fn undo(&mut self, current: S) -> Option<S> {
        let target = self.u_anchor.take()?;
        if let Some(r) = self.r_anchor.take() {
            let e = r.encode(&current);
            let b = S::enc_bytes(&e);
            self.bytes += b;
            self.r_enc.push_back((e, b));
        }
        self.r_anchor = Some(current);
        if let Some((e, b)) = self.u_enc.pop_back() {
            self.bytes -= b;
            self.u_anchor = Some(S::decode(&e, &target));
        }
        self.enforce();
        Some(target)
    }

    pub fn redo(&mut self, current: S) -> Option<S> {
        let target = self.r_anchor.take()?;
        if let Some(u) = self.u_anchor.take() {
            let e = u.encode(&current);
            let b = S::enc_bytes(&e);
            self.bytes += b;
            self.u_enc.push_back((e, b));
        }
        self.u_anchor = Some(current);
        if let Some((e, b)) = self.r_enc.pop_back() {
            self.bytes -= b;
            self.r_anchor = Some(S::decode(&e, &target));
        }
        self.enforce();
        Some(target)
    }

    fn enforce(&mut self) {
        while (self.u_enc.len() + 1 > self.max_steps || self.bytes > self.budget)
            && !self.u_enc.is_empty()
        {
            let (_, b) = self.u_enc.pop_front().unwrap();
            self.bytes -= b;
            self.evicted += 1;
        }
        while self.bytes > self.budget && !self.r_enc.is_empty() {
            let (_, b) = self.r_enc.pop_front().unwrap();
            self.bytes -= b;
            self.evicted += 1;
        }
    }

    pub fn can_undo(&self) -> bool {
        self.u_anchor.is_some()
    }

    pub fn can_redo(&self) -> bool {
        self.r_anchor.is_some()
    }

    pub fn undo_len(&self) -> usize {
        self.u_enc.len() + self.u_anchor.is_some() as usize
    }

    pub fn redo_len(&self) -> usize {
        self.r_enc.len() + self.r_anchor.is_some() as usize
    }

    /// Bytes held by the encoded steps (the two whole anchors excluded:
    /// they share their pixels with the document).
    pub fn delta_bytes(&self) -> usize {
        self.bytes
    }

    /// The whole states kept for the next undo and redo (these usually share
    /// most of their storage with the live document).
    pub fn anchors(&self) -> (Option<&S>, Option<&S>) {
        (self.u_anchor.as_ref(), self.r_anchor.as_ref())
    }

    pub fn budget(&self) -> usize {
        self.budget
    }

    pub fn set_budget(&mut self, budget: usize) {
        self.budget = budget;
        self.enforce();
    }

    /// Steps dropped so far to honour the limit or budget.
    pub fn evicted(&self) -> u64 {
        self.evicted
    }

    pub fn clear(&mut self) {
        self.u_anchor = None;
        self.r_anchor = None;
        self.u_enc.clear();
        self.r_enc.clear();
        self.bytes = 0;
    }
}

/// One frame of an encoded frame list.
#[derive(Clone, Debug)]
pub enum FrameEnc {
    /// Identical to frame `i` of the neighbouring state.
    Same(usize),
    /// Frame `i` of the neighbour plus changed tiles.
    Delta(usize, TileDelta),
    Full(crate::ui_canvas::Raster),
}

fn rgba(r: &crate::ui_canvas::Raster) -> PlaneRef<'_> {
    PlaneRef::new(&r.data, r.width, r.height, 4, (0, 0))
}

/// Animate's frame list. Frames are matched by position, or one slot
/// either side (insert/delete), so a new frame costs only itself.
impl Delta for Vec<crate::ui_canvas::Raster> {
    type Enc = Vec<FrameEnc>;

    fn encode(&self, rel: &Self) -> Vec<FrameEnc> {
        self.iter()
            .enumerate()
            .map(|(i, f)| {
                let near = [Some(i), i.checked_sub(1), Some(i + 1)];
                for j in near.into_iter().flatten() {
                    if rel.get(j).is_some_and(|r| r == f) {
                        return FrameEnc::Same(j);
                    }
                }
                match rel.get(i) {
                    Some(r) if (r.width, r.height) == (f.width, f.height) => {
                        FrameEnc::Delta(i, TileDelta::diff(rgba(f), rgba(r)))
                    }
                    _ => FrameEnc::Full(f.clone()),
                }
            })
            .collect()
    }

    fn decode(enc: &Vec<FrameEnc>, rel: &Self) -> Self {
        enc.iter()
            .map(|e| match e {
                FrameEnc::Same(j) => rel[*j].clone(),
                FrameEnc::Delta(j, d) => crate::ui_canvas::Raster {
                    width: d.width,
                    height: d.height,
                    data: d.apply(rgba(&rel[*j])),
                },
                FrameEnc::Full(r) => r.clone(),
            })
            .collect()
    }

    fn enc_bytes(enc: &Vec<FrameEnc>) -> usize {
        enc.iter()
            .map(|e| match e {
                FrameEnc::Same(_) => 16,
                FrameEnc::Delta(_, d) => d.bytes(),
                FrameEnc::Full(r) => r.data.len() + 32,
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lcg(seed: &mut u64) -> u64 {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *seed >> 33
    }

    fn plane(w: u32, h: u32, bpp: u32, seed: u64) -> Vec<u8> {
        let mut s = seed;
        (0..w * h * bpp).map(|_| lcg(&mut s) as u8).collect()
    }

    #[test]
    fn identical_planes_need_no_tiles() {
        let a = plane(600, 300, 4, 1);
        let d = TileDelta::diff(
            PlaneRef::new(&a, 600, 300, 4, (0, 0)),
            PlaneRef::new(&a, 600, 300, 4, (0, 0)),
        );
        assert!(d.tiles.is_empty());
        assert!(d.is_identity_for(&PlaneRef::new(&a, 600, 300, 4, (0, 0))));
    }

    #[test]
    fn only_changed_tiles_are_stored_and_rebuild_exactly() {
        let base = plane(700, 520, 4, 2);
        let mut target = base.clone();
        // One pixel in tile (1,0) and one in the clipped corner tile (2,2).
        target[(10 * 700 + 300) * 4] ^= 0xff;
        target[(519 * 700 + 699) * 4 + 3] ^= 1;
        let t = PlaneRef::new(&target, 700, 520, 4, (0, 0));
        let b = PlaneRef::new(&base, 700, 520, 4, (0, 0));
        let d = TileDelta::diff(t, b);
        assert_eq!(d.tiles.iter().map(|x| x.0).collect::<Vec<_>>(), vec![1, 8]);
        assert_eq!(d.apply(b), target);
    }

    #[test]
    fn diffs_across_size_and_offset_changes() {
        // A layer that was trimmed to its content and then moved.
        let base = plane(400, 300, 4, 3);
        let target = plane(150, 90, 4, 4);
        let d = TileDelta::diff(
            PlaneRef::new(&target, 150, 90, 4, (-20, 37)),
            PlaneRef::new(&base, 400, 300, 4, (5, 0)),
        );
        assert_eq!(d.apply(PlaneRef::new(&base, 400, 300, 4, (5, 0))), target);
        // Target equal to the base window: no tiles.
        let mut win = vec![0u8; 100 * 50];
        let m = plane(300, 200, 1, 5);
        for y in 0..50 {
            for x in 0..100 {
                win[y * 100 + x] = m[(y + 10) * 300 + x + 30];
            }
        }
        let d = TileDelta::diff(
            PlaneRef::new(&win, 100, 50, 1, (30, 10)),
            PlaneRef::new(&m, 300, 200, 1, (0, 0)),
        );
        assert!(d.tiles.is_empty());
        assert_eq!(d.apply(PlaneRef::new(&m, 300, 200, 1, (0, 0))), win);
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Img(Vec<u8>);
    const W: u32 = 520;
    const H: u32 = 300;
    impl Delta for Img {
        type Enc = TileDelta;
        fn encode(&self, rel: &Self) -> TileDelta {
            TileDelta::diff(
                PlaneRef::new(&self.0, W, H, 1, (0, 0)),
                PlaneRef::new(&rel.0, W, H, 1, (0, 0)),
            )
        }
        fn decode(e: &TileDelta, rel: &Self) -> Self {
            Img(e.apply(PlaneRef::new(&rel.0, W, H, 1, (0, 0))))
        }
        fn enc_bytes(e: &TileDelta) -> usize {
            e.bytes()
        }
    }

    fn edit(img: &mut Img, seed: &mut u64) {
        for _ in 0..1 + lcg(seed) % 40 {
            let i = (lcg(seed) % (W * H) as u64) as usize;
            img.0[i] = lcg(seed) as u8;
        }
    }

    /// Random edits, undos and redos against a reference full-clone history.
    #[test]
    fn undo_redo_matches_full_snapshots_exactly() {
        let mut seed = 99;
        let mut live = Img(plane(W, H, 1, 7));
        let mut h = DeltaHistory::<Img>::new(1000, usize::MAX);
        let mut ru: Vec<Img> = Vec::new();
        let mut rr: Vec<Img> = Vec::new();
        for _ in 0..600 {
            match lcg(&mut seed) % 5 {
                0 | 1 => {
                    h.push(&live);
                    ru.push(live.clone());
                    rr.clear();
                    edit(&mut live, &mut seed);
                }
                2 => {
                    let a = h.undo(live.clone());
                    let b = ru.pop().inspect(|_| rr.push(live.clone()));
                    assert_eq!(a, b);
                    if let Some(s) = a {
                        live = s;
                    }
                }
                3 => {
                    let a = h.redo(live.clone());
                    let b = rr.pop().inspect(|_| ru.push(live.clone()));
                    assert_eq!(a, b);
                    if let Some(s) = a {
                        live = s;
                    }
                }
                _ => edit(&mut live, &mut seed), // coalesced: no new step
            }
            assert_eq!(h.undo_len(), ru.len());
            assert_eq!(h.redo_len(), rr.len());
        }
    }

    #[test]
    fn budget_evicts_the_oldest_steps() {
        let mut seed = 5;
        let mut live = Img(plane(W, H, 1, 8));
        let mut h = DeltaHistory::<Img>::new(1000, 64 * 1024);
        let mut states = Vec::new();
        for _ in 0..50 {
            h.push(&live);
            states.push(live.clone());
            // Each edit dirties every tile: ~150 KB per step.
            for k in (0..live.0.len()).step_by(97) {
                live.0[k] = live.0[k].wrapping_add(1);
            }
            edit(&mut live, &mut seed);
        }
        assert!(h.delta_bytes() <= 64 * 1024, "{}", h.delta_bytes());
        assert!(h.evicted() > 0);
        let kept = h.undo_len();
        assert!(kept >= 1 && kept < 50);
        // What is kept is exact: the newest `kept` states, newest first.
        for want in states.iter().rev().take(kept) {
            live = h.undo(live).unwrap();
            assert_eq!(&live, want);
        }
        assert!(!h.can_undo());
    }

    #[test]
    fn frame_lists_round_trip_through_inserts_and_deletes() {
        use crate::ui_canvas::Raster;
        let mk = |v: u8| {
            let mut r = Raster::new(300, 200);
            r.data[(v as usize) * 37 % 240000] = v;
            r
        };
        let base: Vec<Raster> = (1..=8).map(mk).collect();
        let mut ins = base.clone();
        ins.insert(3, mk(77));
        let mut del = base.clone();
        del.remove(5);
        let mut edit = base.clone();
        edit[2].data[1000] = 5;
        for target in [ins, del, edit, Vec::new()] {
            let e = target.encode(&base);
            assert_eq!(Vec::<Raster>::decode(&e, &base), target);
            assert!(Vec::<Raster>::enc_bytes(&e) < 2 * 300 * 200 * 4);
        }
    }

    #[test]
    fn step_limit_still_applies() {
        let mut live = Img(vec![0; (W * H) as usize]);
        let mut h = DeltaHistory::<Img>::new(6, usize::MAX);
        for i in 0..20 {
            h.push(&live);
            live.0[i * 1000] = 9;
        }
        assert_eq!(h.undo_len(), 6);
    }
}
