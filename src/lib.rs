//! Public foundation for the OBS Studio rewrite in Rust.
//!
//! - bitstream: Position trait makes wrap vs no-wrap a type-level property
//! - dstr: safe path fixes the infinite-loop and overflow bugs

pub mod bitstream;
pub mod dstr;

pub use bitstream::{BitstreamReader, Reader, Position, bitstream_reader};
pub use dstr::Dstr;
