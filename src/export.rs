//! Safe pure Export (clip to file core).
//!
//! Generated frames may be written as original work.
//! An outside photograph, including a public photo, is not original.
//! It can be written only when the operator confirms they have the
//! rights, and the result is never marked original.
//! Photo-protection text is kept. Empty paths rejected.
//! No unwraps.
//!
//! Differential: export + path round-trips.

use crate::replay_save::{ReplaySave, SaveFormat};

/// Where the pixels came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelOrigin {
    /// Frames produced by this program. Not a copy of a person or photo.
    Generated,
    /// A photograph or other outside image, including a public photo.
    ExternalPhoto,
}

/// What the operator says about their right to use an outside image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RightsClaim {
    Unstated,
    /// Operator confirms the image is theirs or they have a lawful right.
    OperatorConfirmsRights,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportError {
    EmptyPath,
    Missing,
    /// Public or third-party photo used with no rights confirmation.
    PhotoRightsRequired,
    /// A copy was presented as an original render.
    CopyIsNotOriginal,
    /// Photo-protection text was removed or replaced.
    ProtectionRemoved,
}

const PHOTO_PROTECTION: &str =
    "photo-protection: third-party image, not an original render";

#[derive(Clone, Debug)]
pub struct Export {
    path: String,
    format: SaveFormat,
    frames: usize,
    original: bool,
    protection: String,
}

impl Export {
    /// Export a generated clip. The result is marked original.
    pub fn write(save: &ReplaySave, name: &str, path: &str) -> Result<Self, ExportError> {
        Self::write_origin(
            save,
            name,
            path,
            PixelOrigin::Generated,
            RightsClaim::OperatorConfirmsRights,
        )
    }

    /// Export with an explicit origin.
    ///
    /// An external photo requires `OperatorConfirmsRights` and is never
    /// marked original. Generated frames are marked original.
    pub fn write_origin(
        save: &ReplaySave,
        name: &str,
        path: &str,
        origin: PixelOrigin,
        rights: RightsClaim,
    ) -> Result<Self, ExportError> {
        if path.is_empty() {
            return Err(ExportError::EmptyPath);
        }
        let clip = save.saved(name).ok_or(ExportError::Missing)?;
        let (original, protection) = match origin {
            PixelOrigin::Generated => (true, String::new()),
            PixelOrigin::ExternalPhoto => {
                if rights != RightsClaim::OperatorConfirmsRights {
                    return Err(ExportError::PhotoRightsRequired);
                }
                (false, PHOTO_PROTECTION.to_string())
            }
        };
        Ok(Self {
            path: path.to_string(),
            format: clip.format,
            frames: clip.frames.len(),
            original,
            protection,
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn format(&self) -> SaveFormat {
        self.format
    }

    pub fn frames(&self) -> usize {
        self.frames
    }

    /// True only for work this program generated, never for a copied photo.
    pub fn original(&self) -> bool {
        self.original
    }

    pub fn protection(&self) -> &str {
        &self.protection
    }

    /// Reject publishing a copied photo or identity as an original render.
    pub fn require_original(&self) -> Result<(), ExportError> {
        if self.original {
            Ok(())
        } else {
            Err(ExportError::CopyIsNotOriginal)
        }
    }

    /// Keep photo-protection text. A different notice is a removal.
    pub fn keep_protection(self, notice: &str) -> Result<Self, ExportError> {
        if self.protection.is_empty() || notice == self.protection {
            Ok(self)
        } else {
            Err(ExportError::ProtectionRemoved)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instant_replay::InstantReplay;

    fn saved() -> ReplaySave {
        let mut s = ReplaySave::new();
        let mut r = InstantReplay::new(3);
        r.push(1);
        r.push(2);
        s.save_as("loop", &r, SaveFormat::Gif).unwrap();
        s
    }

    #[test]
    fn empty_path() {
        let s = ReplaySave::new();
        assert!(Export::write(&s, "goal", "").is_err());
    }

    #[test]
    fn differential() {
        let s = saved();
        let e = Export::write(&s, "loop", "loop.gif").unwrap();
        assert_eq!(e.path(), "loop.gif");
        assert_eq!(e.format(), SaveFormat::Gif);
        assert_eq!(e.frames(), 2);
        assert!(e.original());
        assert!(e.protection().is_empty());
    }

    #[test]
    fn public_photo_needs_rights() {
        let s = saved();
        let err = Export::write_origin(
            &s,
            "loop",
            "photo.png",
            PixelOrigin::ExternalPhoto,
            RightsClaim::Unstated,
        )
        .unwrap_err();
        assert_eq!(err, ExportError::PhotoRightsRequired);
    }

    #[test]
    fn confirmed_photo_is_not_original() {
        let s = saved();
        let e = Export::write_origin(
            &s,
            "loop",
            "photo.png",
            PixelOrigin::ExternalPhoto,
            RightsClaim::OperatorConfirmsRights,
        )
        .unwrap();
        assert!(!e.original());
        assert_eq!(e.require_original(), Err(ExportError::CopyIsNotOriginal));
        assert!(e.protection().contains("not an original render"));
        assert!(e.clone().keep_protection("stripped").is_err());
        assert!(e.keep_protection(PHOTO_PROTECTION).is_ok());
    }
}
