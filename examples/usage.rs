use bitstream_io::LittleEndian;
use lzw_test::{
    lzw::LzwCompressor,
    map::basic_string,
    stack::{Stacked, WithBigVec},
    string_to_bytes,
};

const LOREM_IPSUM: &str = include_str!("../lorem_ipsum.txt");

/// See https://www.eecis.udel.edu/~amer/CISC651/lzw.and.gif.explained.html
/// Use https://crates.io/crates/bitstream-io for bit packing?
/// Rust example https://rosettacode.org/wiki/LZW_compression#Rust
fn main() {
    basic_string();
    with_stack();
    // ascii_string();

    let mut my_encoder = my_lzw::Encoder::new(7, LittleEndian);
    let mut compressed = vec![];
    my_encoder.encode(LOREM_IPSUM.as_bytes(), &mut compressed);

    let mut decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, 7);
    let decompressed = decoder.decode(&compressed).unwrap();

    let decompressed_string = String::from_utf8_lossy(&decompressed);

    assert_eq!(decompressed_string, LOREM_IPSUM);

    let mut second_compression = vec![];
    my_encoder.encode(LOREM_IPSUM.as_bytes(), &mut second_compression);

    assert_eq!(compressed, second_compression);
}

fn with_stack() {
    let original = "ABACABADADABBBBBB";
    println!("Original: {original}");

    let converted = string_to_bytes(original);
    println!("Converted: {converted:?}");

    let compressed = Stacked::compress(&converted, 4, 4);
    println!("Stacked stream: {compressed:?}");
    let table_compressed = WithBigVec::compress(&converted, 4, 4);
    println!("Tabled stream: {table_compressed:?}");
}
