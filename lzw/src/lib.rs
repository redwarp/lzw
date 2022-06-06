//! LZW encoder and decoder, GIF flavored.

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
    BigEndian,
    LittleEndian,
}
