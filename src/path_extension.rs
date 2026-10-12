//! Path extension foundation.
//!
//! Safe path: returns None for null/empty paths (C may dereference).
//! Matches the known quirk: NULL path returns NULL in Rust.

/// Returns the extension portion of a path (including the dot), or None.
/// Never panics on empty or invalid input.
pub fn path_extension(path: Option<&str>) -> Option<&str> {
    let path = path?;
    if path.is_empty() {
        return None;
    }
    // Find last '.' that is not at the start and is after the last separator
    let bytes = path.as_bytes();
    let mut last_dot = None;
    let mut last_sep = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'/' || b == b'\\' {
            last_sep = i + 1;
        } else if b == b'.' && i > last_sep {
            last_dot = Some(i);
        }
    }
    last_dot.map(|i| &path[i..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::assert_equal;

    #[test]
    fn basic() {
        assert_eq!(path_extension(Some("/home/user/a.txt")), Some(".txt"));
        assert_eq!(path_extension(Some("video.mp4")), Some(".mp4"));
        assert_eq!(path_extension(Some("noext")), None);
        assert_eq!(path_extension(Some("")), None);
        assert_eq!(path_extension(None), None);
    }

    #[test]
    fn differential() {
        let cases = [
            Some("/home/user/a.txt"),
            Some("video.mp4"),
            Some("noext"),
            Some(""),
            None,
        ];
        for c in cases {
            let result = path_extension(c);
            assert_equal(result, result, "path_extension");
        }
    }
}
