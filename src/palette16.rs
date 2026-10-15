//! Safe pure Palette16 (16-color indexed core).
//!
//! Fixed VGA-like palette. Index 0..=15. No unwraps.
//!
//! Differential: index 1 is blue, index 15 is white.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Standard 16-color palette (VGA order).
pub const PALETTE16: [Rgb; 16] = [
    Rgb { r: 0, g: 0, b: 0 },       // 0 black
    Rgb { r: 0, g: 0, b: 170 },     // 1 blue
    Rgb { r: 0, g: 170, b: 0 },     // 2 green
    Rgb { r: 0, g: 170, b: 170 },   // 3 cyan
    Rgb { r: 170, g: 0, b: 0 },     // 4 red
    Rgb { r: 170, g: 0, b: 170 },   // 5 magenta
    Rgb { r: 170, g: 85, b: 0 },    // 6 brown
    Rgb { r: 170, g: 170, b: 170 }, // 7 light gray
    Rgb { r: 85, g: 85, b: 85 },    // 8 dark gray
    Rgb { r: 85, g: 85, b: 255 },   // 9 bright blue
    Rgb { r: 85, g: 255, b: 85 },   // 10 bright green
    Rgb { r: 85, g: 255, b: 255 },  // 11 bright cyan
    Rgb { r: 255, g: 85, b: 85 },   // 12 bright red
    Rgb { r: 255, g: 85, b: 255 },  // 13 bright magenta
    Rgb { r: 255, g: 255, b: 85 },  // 14 yellow
    Rgb { r: 255, g: 255, b: 255 }, // 15 white
];

pub struct Palette16;

impl Palette16 {
    pub fn get(index: u8) -> Rgb {
        PALETTE16[(index as usize) & 15]
    }

    /// BT.601 luminance of the color.
    pub fn luminance(index: u8) -> u8 {
        let c = Self::get(index);
        ((77 * c.r as u32 + 150 * c.g as u32 + 29 * c.b as u32) / 256) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        assert_eq!(Palette16::get(0).r, 0);
        assert_eq!(Palette16::get(1).b, 170);
        assert_eq!(Palette16::get(15).r, 255);
        assert_eq!(Palette16::get(15).g, 255);
        assert_eq!(Palette16::get(15).b, 255);
    }
}
