//! Safe pure StreamInspect (step-through view of bitstream::Reader).
//!
//! Drives the real `Reader<P: Position>` from `bitstream` and records
//! every read: width, value, byte/bit cursor before and after, and
//! whether the counter wrapped. `Safe` uses `usize` (never wraps);
//! `Wrap` uses `u8` (wraps at 256 like the original C uint8_t).
//! No unwraps.
//!
//! Differential: past byte 255 the two modes report different positions.

use crate::bitstream::{Position, Reader};

/// Which counter type drives the reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterMode {
    /// `usize` counter. Stops at the end of the buffer.
    Safe,
    /// `u8` counter. Wraps to byte 0 after byte 255.
    Wrap,
}

impl CounterMode {
    pub fn label(&self) -> &'static str {
        match self {
            CounterMode::Safe => "No-wrap (usize)",
            CounterMode::Wrap => "Wrap (u8, C ABI)",
        }
    }
}

/// One recorded read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadRecord {
    pub index: usize,
    /// Bits requested (1..=16).
    pub width: u32,
    pub value: u16,
    pub byte_before: usize,
    pub bit_before: u8,
    pub byte_after: usize,
    pub bit_after: u8,
    /// Counter wrapped during this read.
    pub wrapped: bool,
    /// Some bits came from past the end (read as 0).
    pub past_end: bool,
}

/// Cursor state shared by both counter types.
#[derive(Clone, Copy, Debug)]
struct Cursor<P: Position> {
    pos: P,
    /// Bits consumed in the current byte, 0..=7.
    bit: u8,
}

/// Step-through inspector over an owned buffer.
#[derive(Clone, Debug)]
pub struct StreamInspector {
    data: Vec<u8>,
    mode: CounterMode,
    safe: Cursor<usize>,
    wrap: Cursor<u8>,
    records: Vec<ReadRecord>,
    total_bits: u64,
}

impl StreamInspector {
    pub fn new(data: Vec<u8>, mode: CounterMode) -> Self {
        Self {
            data,
            mode,
            safe: Cursor { pos: 0, bit: 0 },
            wrap: Cursor { pos: 0, bit: 0 },
            records: Vec::new(),
            total_bits: 0,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn mode(&self) -> CounterMode {
        self.mode
    }

    /// Switch counter type and reset the cursor.
    pub fn set_mode(&mut self, mode: CounterMode) {
        self.mode = mode;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.safe = Cursor { pos: 0, bit: 0 };
        self.wrap = Cursor { pos: 0, bit: 0 };
        self.records.clear();
        self.total_bits = 0;
    }

    pub fn records(&self) -> &[ReadRecord] {
        &self.records
    }

    pub fn total_bits(&self) -> u64 {
        self.total_bits
    }

    /// Current (byte, bit) cursor.
    pub fn cursor(&self) -> (usize, u8) {
        match self.mode {
            CounterMode::Safe => (self.safe.pos, self.safe.bit),
            CounterMode::Wrap => (self.wrap.pos.index(), self.wrap.bit),
        }
    }

    /// True when further reads can only return padding zeros.
    pub fn exhausted(&self) -> bool {
        self.cursor().0 >= self.data.len()
    }

    /// Read `width` bits (clamped to 1..=16) through the real Reader.
    pub fn read(&mut self, width: u32) -> ReadRecord {
        let width = width.clamp(1, 16);
        let (byte_before, bit_before) = self.cursor();
        let mut value: u16 = 0;
        let mut past_end = false;
        let mut wrapped = false;
        for _ in 0..width {
            let (bit, advanced_past, did_wrap) = match self.mode {
                CounterMode::Safe => Self::one_bit(&self.data, &mut self.safe),
                CounterMode::Wrap => Self::one_bit(&self.data, &mut self.wrap),
            };
            value = (value << 1) | u16::from(bit);
            past_end |= advanced_past;
            wrapped |= did_wrap;
        }
        self.total_bits += u64::from(width);
        let (byte_after, bit_after) = self.cursor();
        let rec = ReadRecord {
            index: self.records.len(),
            width,
            value,
            byte_before,
            bit_before,
            byte_after,
            bit_after,
            wrapped,
            past_end,
        };
        self.records.push(rec.clone());
        rec
    }

    /// Read until exhausted (Safe) or `max_reads` reads happen.
    pub fn run(&mut self, width: u32, max_reads: usize) -> usize {
        let mut n = 0;
        while n < max_reads && !self.exhausted() {
            self.read(width);
            n += 1;
        }
        n
    }

    /// Read one bit with the real Reader positioned at the cursor.
    fn one_bit<P: Position + PartialEq>(data: &[u8], c: &mut Cursor<P>) -> (u8, bool, bool) {
        let before = c.pos.index();
        if before >= data.len() {
            // Reader returns 0 without moving at the end of the buffer.
            let mut r = Reader::new(data, c.pos);
            return (r.read_bit(), true, false);
        }
        // Rebuild a Reader at the byte, skip consumed bits, read one.
        let mut r = Reader::new(data, c.pos);
        for _ in 0..c.bit {
            r.read_bit();
        }
        let bit = r.read_bit();
        let after = r.position();
        let moved = after.index() != before;
        let wrapped = moved && after.index() < before;
        c.pos = after;
        c.bit = if moved { 0 } else { c.bit + 1 };
        (bit, false, wrapped)
    }
}

/// `width`-digit binary string of `value`.
pub fn to_binary(value: u16, width: u32) -> String {
    let w = width.clamp(1, 16);
    (0..w)
        .rev()
        .map(|i| if value >> i & 1 == 1 { '1' } else { '0' })
        .collect()
}

/// Deterministic sample longer than 256 bytes so wrap is visible.
pub fn sample_stream() -> Vec<u8> {
    let mut v = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0xC0, 0x1E];
    v.extend((0u32..312).map(|i| ((i * 37 + 11) % 256) as u8));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitstream::BitstreamReader;

    #[test]
    fn matches_bitstream_reader() {
        let data = sample_stream();
        let mut a = BitstreamReader::new(&data);
        let mut b = StreamInspector::new(data.clone(), CounterMode::Safe);
        for w in [1u32, 3, 8, 5, 7, 8, 2, 6] {
            let expected = a.read_bits(w);
            assert_eq!(b.read(w).value, u16::from(expected));
        }
        assert_eq!(a.byte_pos(), b.cursor().0);
    }

    #[test]
    fn bit_cursor_advances() {
        let mut s = StreamInspector::new(vec![0b1011_0000, 0xFF], CounterMode::Safe);
        let r = s.read(3);
        assert_eq!(r.value, 0b101);
        assert_eq!((r.byte_after, r.bit_after), (0, 3));
        let r = s.read(5);
        assert_eq!((r.byte_after, r.bit_after), (1, 0));
    }

    #[test]
    fn sixteen_bit_reads() {
        let mut s = StreamInspector::new(vec![0x12, 0x34], CounterMode::Safe);
        assert_eq!(s.read(16).value, 0x1234);
    }

    #[test]
    fn safe_stops_at_end() {
        let mut s = StreamInspector::new(vec![0xFF], CounterMode::Safe);
        s.read(8);
        assert!(s.exhausted());
        let r = s.read(4);
        assert_eq!(r.value, 0);
        assert!(r.past_end);
        assert_eq!(s.cursor(), (1, 0));
    }

    #[test]
    fn wrap_returns_to_zero() {
        let data: Vec<u8> = (0..300u32).map(|i| (i % 256) as u8).collect();
        let mut s = StreamInspector::new(data, CounterMode::Wrap);
        for _ in 0..255 {
            s.read(8);
        }
        let r = s.read(8);
        assert_eq!(r.value, 255);
        assert!(r.wrapped);
        assert_eq!(s.cursor(), (0, 0));
        assert_eq!(s.read(8).value, 0);
    }

    #[test]
    fn modes_differ_past_255() {
        let data = sample_stream();
        let mut a = StreamInspector::new(data.clone(), CounterMode::Safe);
        let mut b = StreamInspector::new(data, CounterMode::Wrap);
        for _ in 0..256 {
            a.read(8);
            b.read(8);
        }
        assert_eq!(a.cursor().0, 256);
        assert_eq!(b.cursor().0, 0);
    }

    #[test]
    fn run_is_bounded() {
        let mut s = StreamInspector::new(sample_stream(), CounterMode::Wrap);
        assert_eq!(s.run(8, 1000), 1000);
        let mut t = StreamInspector::new(vec![1, 2, 3], CounterMode::Safe);
        assert_eq!(t.run(8, 1000), 3);
    }

    #[test]
    fn empty_buffer_safe() {
        let mut s = StreamInspector::new(Vec::new(), CounterMode::Safe);
        assert!(s.exhausted());
        assert_eq!(s.read(8).value, 0);
        assert_eq!(s.run(8, 10), 0);
    }

    #[test]
    fn set_mode_resets() {
        let mut s = StreamInspector::new(vec![1, 2], CounterMode::Safe);
        s.read(8);
        s.set_mode(CounterMode::Wrap);
        assert!(s.records().is_empty());
        assert_eq!(s.cursor(), (0, 0));
    }

    #[test]
    fn binary_string() {
        assert_eq!(to_binary(0b101, 3), "101");
        assert_eq!(to_binary(1, 8), "00000001");
    }
}
