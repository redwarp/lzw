use lzw_test::{
    lzw::LzwCompressor,
    map::basic_string,
    stack::{Stacked, WithBigVec},
    string_to_bytes,
};

const LOREM_IPSUM: &str = include_str!("../lorem_ipsum.txt");
const LOREM_IPSUM_LONG: &str = include_str!("../lorem_ipsum_long.txt");

/// See https://www.eecis.udel.edu/~amer/CISC651/lzw.and.gif.explained.html
/// Use https://crates.io/crates/bitstream-io for bit packing?
/// Rust example https://rosettacode.org/wiki/LZW_compression#Rust
fn main() {
    basic_string();
    with_stack();

    check_string_compression(LOREM_IPSUM);
    check_string_compression(LOREM_IPSUM_LONG);
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

fn check_string_compression(string: &str) {
    let mut my_encoder = my_lzw::Encoder::new(7, my_lzw::Endianness::LittleEndian);
    let mut compressed = vec![];
    my_encoder.encode(string.as_bytes(), &mut compressed);

    let mut decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, 7);
    let decompressed = decoder.decode(&compressed).unwrap();

    let decompressed_string = String::from_utf8_lossy(&decompressed);

    assert_eq!(decompressed_string, string);

    let mut second_compression = vec![];
    my_encoder.encode(string.as_bytes(), &mut second_compression);

    assert_eq!(compressed, second_compression);
}
