//! Safe pure format-conversion helpers (OBS format-conversion core).
//!
//! The C version does SIMD-accelerated packing between UYVY / I420 /
//! NV12 / I444. The safe path provides pure plane-size and simple
//! scalar conversion helpers that never overrun and return None on
//! zero or mismatched dimensions. Full SIMD can be added later.
//!
//! Differential: plane sizes match the mathematical 4:2:0 / 4:2:2 rules.

use crate::diff::assert_equal;

/// Bytes required for a Y plane of the given width and height.
pub fn y_plane_size(width: u32, height: u32) -> Option<usize> {
    if width == 0 || height == 0 {
        None
    } else {
        Some((width as usize) * (height as usize))
    }
}

/// Chroma plane size for 4:2:0 (half width, half height).
pub fn chroma_420_size(width: u32, height: u32) -> Option<usize> {
    if width == 0 || height == 0 {
        None
    } else {
        Some(((width as usize + 1) / 2) * ((height as usize + 1) / 2))
    }
}

/// Chroma plane size for 4:2:2 (half width, full height).
pub fn chroma_422_size(width: u32, height: u32) -> Option<usize> {
    if width == 0 || height == 0 {
        None
    } else {
        Some(((width as usize + 1) / 2) * (height as usize))
    }
}

/// Simple scalar UYVY → YUY2 swap for a single line.
/// Returns None if the input length is odd.
pub fn uyvy_to_yuy2_line(input: &[u8]) -> Option<Vec<u8>> {
    if input.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(input.len());
    for chunk in input.chunks_exact(4) {
        // UYVY → YUY2: U Y0 V Y1 → Y0 U Y1 V
        if chunk.len() == 4 {
            out.push(chunk[1]); // Y0
            out.push(chunk[0]); // U
            out.push(chunk[3]); // Y1
            out.push(chunk[2]); // V
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_sizes() {
        assert_eq!(y_plane_size(1920, 1080), Some(1920 * 1080));
        assert_eq!(chroma_420_size(1920, 1080), Some(960 * 540));
        assert_eq!(chroma_422_size(1920, 1080), Some(960 * 1080));
    }

    #[test]
    fn zero_safe() {
        assert_eq!(y_plane_size(0, 100), None);
        assert_eq!(chroma_420_size(100, 0), None);
    }

    #[test]
    fn uyvy_swap() {
        let input = [0x80, 0x10, 0x90, 0x20]; // U Y0 V Y1
        let out = uyvy_to_yuy2_line(&input).unwrap();
        assert_eq!(&out, &[0x10, 0x80, 0x20, 0x90]); // Y0 U Y1 V
    }

    #[test]
    fn odd_length_safe() {
        assert!(uyvy_to_yuy2_line(&[1, 2, 3]).is_none());
    }

    #[test]
    fn differential() {
        let w = 4u32;
        let h = 2u32;
        let y = y_plane_size(w, h).unwrap();
        let c420 = chroma_420_size(w, h).unwrap();
        assert_equal(y, 8, "y size");
        assert_equal(c420, 2, "chroma 420");
    }
}
