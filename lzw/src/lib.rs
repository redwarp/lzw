//! LZW encoder and decoder, GIF flavored.
//!
//! This crate provides a Encoder and Decoder to compress and decompress LZW data.
//! This particular implementation provides the gif variation of LZW, using variable code size.
//!
//! It's fast, and use limited memory to do so: the decoder only uses the stack.
//!
//! It also work with any [std::io::Read] and [std::io::Write].
//!
//! # Examples
//!
//! ```
//! use salzweg::{Encoder, Endianness, EncodingError, Decoder, DecodingError};
//!
//! let data = [0, 0, 1, 3];
//! let mut compressed = vec![];
//! let mut decompressed = vec![];
//!
//! Encoder::encode(&data[..], &mut compressed, 2, Endianness::LittleEndian).unwrap();
//!
//! assert_eq!(compressed, [0x04, 0x32, 0x05]);
//!
//! Decoder::decode(&compressed[..], &mut decompressed, 2, Endianness::LittleEndian).unwrap();
//!
//! assert_eq!(decompressed, data);
//!
//! ```

mod decoder;
mod encoder;
mod io;

pub use decoder::Decoder;
pub use decoder::DecodingError;
pub use encoder::Encoder;
pub use encoder::EncodingError;

/// The bit ordering when encoding or decoding LZW.
///
/// This crate currently only supports the GIF variation and GIF typically use little endian,
/// but big endian still works.
#[derive(Debug, Clone, Copy)]
pub enum Endianness {
    /// Most significant order.
    BigEndian,
    /// Least significant order.
    LittleEndian,
}
