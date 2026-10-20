//! Safe pure video fourcc mapping (OBS video-fourcc.c).
//!
//! The C version maps a uint32_t fourcc to an enum video_format.
//! The safe path uses a match on a u32 fourcc and returns Option.
//! Unknown codes return None. No unwraps.
//!
//! Differential: known fourccs map to the expected format.

#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoFormat {
    None,
    Uyvy,
    Yuy2,
    Yvyu,
    Y800,
    // Extend as needed
}

fn make_fourcc(a: u8, b: u8, c: u8, d: u8) -> u32 {
    (d as u32) << 24 | (c as u32) << 16 | (b as u32) << 8 | (a as u32)
}

pub fn video_format_from_fourcc(fourcc: u32) -> Option<VideoFormat> {
    match fourcc {
        f if f == make_fourcc(b'U', b'Y', b'V', b'Y')
            || f == make_fourcc(b'H', b'D', b'Y', b'C')
            || f == make_fourcc(b'U', b'Y', b'N', b'V')
            || f == make_fourcc(b'U', b'Y', b'N', b'Y')
            || f == make_fourcc(b'u', b'y', b'v', b'1')
            || f == make_fourcc(b'2', b'v', b'u', b'y')
            || f == make_fourcc(b'2', b'V', b'u', b'y') =>
        {
            Some(VideoFormat::Uyvy)
        }
        f if f == make_fourcc(b'Y', b'U', b'Y', b'2')
            || f == make_fourcc(b'Y', b'4', b'2', b'2')
            || f == make_fourcc(b'V', b'4', b'2', b'2')
            || f == make_fourcc(b'V', b'Y', b'U', b'Y')
            || f == make_fourcc(b'Y', b'U', b'N', b'V')
            || f == make_fourcc(b'y', b'u', b'v', b'2')
            || f == make_fourcc(b'y', b'u', b'v', b's') =>
        {
            Some(VideoFormat::Yuy2)
        }
        f if f == make_fourcc(b'Y', b'V', b'Y', b'U') => Some(VideoFormat::Yvyu),
        f if f == make_fourcc(b'Y', b'8', b'0', b'0') => Some(VideoFormat::Y800),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uyvy() {
        let fourcc = make_fourcc(b'U', b'Y', b'V', b'Y');
        assert_eq!(video_format_from_fourcc(fourcc), Some(VideoFormat::Uyvy));
    }

    #[test]
    fn yuy2() {
        let fourcc = make_fourcc(b'Y', b'U', b'Y', b'2');
        assert_eq!(video_format_from_fourcc(fourcc), Some(VideoFormat::Yuy2));
    }

    #[test]
    fn unknown_safe() {
        assert_eq!(video_format_from_fourcc(0), None);
        assert_eq!(video_format_from_fourcc(0xDEAD_BEEF), None);
    }

    #[test]
    fn differential() {
        let fourcc = make_fourcc(b'Y', b'8', b'0', b'0');
        let got = video_format_from_fourcc(fourcc);
        assert_equal(got, Some(VideoFormat::Y800), "fourcc y800");
    }
}
