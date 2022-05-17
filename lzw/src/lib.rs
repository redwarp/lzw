mod decoder;
mod encoder;
mod io;

pub use decoder::Decoder;
pub use encoder::Encoder;

#[derive(Debug, Clone, Copy)]
pub enum Endianness {
    BigEndian,
    LittleEndian,
}
