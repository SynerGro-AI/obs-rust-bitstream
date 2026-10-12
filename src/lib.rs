//! Public foundation for the OBS Studio rewrite in Rust.
//!
//! Work smarter: reusable patterns + AI assistance cover many modules
//! with far fewer actions.
//!
//! - bitstream: Position trait (wrap vs no-wrap is type-level)
//! - dstr: safe path fixes infinite-loop and overflow
//! - utf8: safe truncation never splits a multibyte character
//! - diff: reusable differential test helper (wired into modules)
//! - crc32: standard CRC32 with known test vectors
//! - ai: simple config to switch Grok / Claude / Seraphina for automated help
//! - array_serializer: Vec-based output serializer + LE helpers
//! - text_lookup: HashMap store with escape expansion

pub mod bitstream;
pub mod dstr;
pub mod utf8;
pub mod diff;
pub mod crc32;
pub mod ai;
pub mod array_serializer;
pub mod text_lookup;

pub use bitstream::{BitstreamReader, Reader, Position, bitstream_reader};
pub use dstr::Dstr;
pub use utf8::{truncate_utf8, truncate_filename};
pub use crc32::crc32;
pub use ai::{AiAssistant, AiConfig, Backend};
pub use array_serializer::ArrayOutput;
pub use text_lookup::TextLookup;