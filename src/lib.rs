//! - bmem: allocation size check + 32-byte alignment contract
//! - util_uint64: exact mul-then-div with 128-bit intermediate
//! - util_uint128: safe add / mul64_64 using native u128

pub mod bitstream;
pub mod dstr;
pub mod utf8;
pub mod diff;
pub mod crc32;
pub mod ai;
pub mod array_serializer;
pub mod text_lookup;
pub mod path_extension;
pub mod bmem;
pub mod util_uint64;
pub mod util_uint128;

pub use bitstream::{BitstreamReader, Reader, Position, bitstream_reader};
pub use dstr::Dstr;
pub use utf8::{truncate_utf8, truncate_filename};
pub use crc32::crc32;
pub use ai::{AiAssistant, AiConfig, Backend};
pub use array_serializer::ArrayOutput;
pub use text_lookup::TextLookup;
pub use path_extension::path_extension;
pub use bmem::{is_valid_alloc_size, alignment};
pub use util_uint64::util_mul_div64;
pub use util_uint128::{U128, util_add128, util_mul64_64};