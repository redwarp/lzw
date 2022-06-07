//! LZW encoder and decoder.
//!
//! This crate provides a Encoder and Decoder to compress and decompress LZW data.
//! This particular implementation provides the gif variation of LZW, using variable code size,
//!  as well as the TIFF variation, or the original 12 bit fixed sized LZW variation.
//!
//! It's fast, and use limited memory to do so: the decoder only uses the stack.
//!
//! It works with any [std::io::Read] and [std::io::Write].
//!
//! # Examples
//!
//! ```
//! use salzweg::{
//!     decoder::{DecodingError, GifDecoder},
//!     encoder::{EncodingError, GifEncoder},
//!     Endianness,
//! };
//!
//! let data = [0, 0, 1, 3];
//! let mut compressed = vec![];
//! let mut decompressed = vec![];
//!
//! GifEncoder::encode(&data[..], &mut compressed, 2).unwrap();
//!
//! assert_eq!(compressed, [0x04, 0x32, 0x05]);
//!
//! GifDecoder::decode(&compressed[..], &mut decompressed, 2).unwrap();
//!
//! assert_eq!(decompressed, data);
//! ```

pub mod decoder;
pub mod encoder;
mod io;

/// The bit ordering when encoding or decoding LZW.
///
/// This crate currently only supports the GIF variation and GIF typically use little endian,
/// but big endian still works.
#[derive(Debug)]
pub enum Endianness {
    /// Most significant order.
    BigEndian,
    /// Least significant order.
    LittleEndian,
}

#[derive(Debug)]
pub enum CodeSizeIncrease {
    Default,
    Tiff,
}

impl CodeSizeIncrease {
    pub(crate) const fn increment(&self) -> u16 {
        match self {
            CodeSizeIncrease::Default => 0,
            CodeSizeIncrease::Tiff => 1,
        }
    }
}
