use lzw::{lzw::Lzw, map::basic_string, stack::Stacked, string_to_bytes};

/// See https://www.eecis.udel.edu/~amer/CISC651/lzw.and.gif.explained.html
/// Use https://crates.io/crates/bitstream-io for bit packing?
/// Rust example https://rosettacode.org/wiki/LZW_compression#Rust
fn main() {
    basic_string();
    with_stack();
    // ascii_string();
}

fn with_stack() {
    let original = "ABACABADADABBBBBB";
    println!("Original: {original}");

    let converted = string_to_bytes(original);
    println!("Converted: {converted:?}");

    let compressed = Stacked::compress(&converted, 4, 4);
    println!("Stacked stream: {compressed:?}");
}
