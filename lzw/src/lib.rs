//! LZW encoder and decoder.
//!
//! This crate provides a Encoder and Decoder to compress and decompress LZW data.
//! This particular implementation provides the GIF and TIFF variation, as well
//! as the original fixed 12 bit LZW variation.
//!
//! It's fast, and use limited memory to do so: the decoder only uses the stack.
//!
//! It works with any [std::io::Read] and [std::io::Write].
//!
//! # Examples
//!
//! ## Encoding GIF data
//! ```
//! use salzweg::{
//!     decoder::{DecodingError, GifStyleDecoder},
//!     encoder::{EncodingError, GifStyleEncoder},
//!     Endianness,
//! };
//!
//! let data = [0, 0, 1, 3];
//! let mut compressed = vec![];
//! let mut decompressed = vec![];
//!
//! GifStyleEncoder::encode(&data[..], &mut compressed, 2).expect("Compression failed");
//!
//! assert_eq!(compressed, [0x04, 0x32, 0x05]);
//!
//! GifStyleDecoder::decode(&compressed[..], &mut decompressed, 2).expect("Decompression failed");
//!
//! assert_eq!(decompressed, data);
//! ```
//!
//! ## Compressing a file using the TIFF variation
//! ```
//! use salzweg::encoder::TiffStyleEncoder;
//! use std::{fs::File, io::BufReader};
//!
//! let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
//!     .parent()
//!     .expect("Couldn't find parent folder")
//!     .join("test-assets/lorem_ipsum.txt");
//!
//! let output_file = std::io::sink(); // Let's pretend this is a file.
//!
//! let data = BufReader::new(File::open(path).expect("Couldn't open the file"));
//!
//! TiffStyleEncoder::encode(data, output_file).expect("Compression failed");
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

/// Code size increase strategy.
///
/// For variable code size encoding, there is a difference between the strategy used
/// by TIFF compared to GIF or other variable code LZW.
#[derive(Debug)]
pub enum CodeSizeStrategy {
    /// Default code size increase.
    ///
    /// The read and write size increase when the dictionary's size is equal to 2.pow2(code-size).
    Default,
    /// Code size increase strategy for TIFF.
    ///
    /// The read and write size increase when the dictionary's size is equal
    /// to 2.pow2(code-size) - 1.
    Tiff,
}

impl CodeSizeStrategy {
    pub(crate) const fn increment(&self) -> u16 {
        match self {
            CodeSizeStrategy::Default => 0,
            CodeSizeStrategy::Tiff => 1,
        }
    }
}
