mod encoder;

pub use encoder::Encoder;

pub enum Endianness {
    BigEndian,
    SmallEndian,
}
