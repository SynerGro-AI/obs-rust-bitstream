//! OBS Rust Bitstream — perfect port foundation.
//!
//! Drop-in style module for the obs-rust tree.
//! The Position trait is the foundational "opcode" that makes wrap vs no-wrap
//! a pure type-level property.

pub mod bitstream;

pub use bitstream::{BitstreamReader, Reader, Position, bitstream_reader};
