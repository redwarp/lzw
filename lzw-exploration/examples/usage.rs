use std::{fs::File, path::Path};

use rand::{prelude::StdRng, RngCore, SeedableRng};
use salzweg::Encoder;

const LOREM_IPSUM: &str = include_str!("../../test-assets/lorem_ipsum.txt");
const LOREM_IPSUM_LONG: &str = include_str!("../../test-assets/lorem_ipsum_long.txt");
const LOREM_IPSUM_ENCODED: &[u8] = include_bytes!("../../test-assets/lorem_ipsum_encoded.bin");
const LOREM_IPSUM_LONG_ENCODED: &[u8] =
    include_bytes!("../../test-assets/lorem_ipsum_long_encoded.bin");

/// See https://www.eecis.udel.edu/~amer/CISC651/lzw.and.gif.explained.html
/// Use https://crates.io/crates/bitstream-io for bit packing?
/// Rust example https://rosettacode.org/wiki/LZW_compression#Rust
fn main() {
    check_string_compression(LOREM_IPSUM);
    check_string_compression(LOREM_IPSUM_LONG);
    check_string_decoding(LOREM_IPSUM_ENCODED);
    check_string_decoding(LOREM_IPSUM_LONG_ENCODED);
    decode_colors();
    calculate_max_stack();

    image_data();
    compressed_image_data();
}

fn check_string_compression(string: &str) {
    let mut my_encoder = salzweg::Encoder::new(7, salzweg::Endianness::LittleEndian);
    let mut compressed = vec![];
    my_encoder
        .encode(string.as_bytes(), &mut compressed)
        .unwrap();

    let mut decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, 7);
    let decompressed = decoder.decode(&compressed).unwrap();

    let decompressed_string = String::from_utf8_lossy(&decompressed);

    assert_eq!(decompressed_string, string);

    let mut second_compression = vec![];
    my_encoder
        .encode(string.as_bytes(), &mut second_compression)
        .unwrap();

    assert_eq!(compressed, second_compression);
}

fn check_string_decoding(data: &[u8]) {
    let mut my_decoder = salzweg::Decoder::new(7, salzweg::Endianness::LittleEndian);
    let mut my_decompressed = vec![];
    my_decoder.decode(data, &mut my_decompressed).unwrap();

    let mut weezl_decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, 7);
    let weezl_decompressed = weezl_decoder.decode(&data).unwrap();

    assert_eq!(my_decompressed, weezl_decompressed);
}

fn decode_colors() {
    let data = [
        0x8C, 0x2D, 0x99, 0x87, 0x2A, 0x1C, 0xDC, 0x33, 0xA0, 0x2, 0x55, 0x0,
    ];

    let mut weezl_decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, 2);

    let decoded = weezl_decoder.decode(&data).unwrap();

    assert_eq!(
        decoded,
        [
            1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2,
            2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
        ]
    );
}

// How big should the stack be?
fn calculate_max_stack() {
    let dict_max_size = 4096;
    let max_free = dict_max_size - ((1 << 2) + 2);

    println!("Max length of word is {}", max_free + 1);

    let max = 4096 - (1 << 2) - 1;
    println!("Max stack = {max}");
}

fn image_data() -> Vec<u8> {
    let image = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("test-assets/tokyo_128_colors.png");

    let png_decoder = png::Decoder::new(File::open(image).unwrap());
    let mut reader = png_decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    buf[..info.buffer_size()].to_vec()
}

fn compressed_image_data() -> Vec<u8> {
    let data = image_data();

    let mut encoder = Encoder::new(7, salzweg::Endianness::LittleEndian);
    let mut compressed = vec![];

    encoder.encode(&data[..], &mut compressed).unwrap();

    compressed
}
