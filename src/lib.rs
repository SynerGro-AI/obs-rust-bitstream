//! Public foundation for the OBS Studio rewrite in Rust.
//!
//! Work smarter: reusable patterns (Position trait, safe truncation, bug fixes)
//! cover many modules with far fewer actions.
//!
//! - bitstream: Position trait makes wrap vs no-wrap a type-level property
//! - dstr: safe path fixes infinite-loop and overflow
//! - utf8: safe truncation never splits a multibyte character

pub mod bitstream;
pub mod dstr;
pub mod utf8;

pub use bitstream::{BitstreamReader, Reader, Position, bitstream_reader};
pub use dstr::Dstr;
pub use utf8::{truncate_utf8, truncate_filename};
