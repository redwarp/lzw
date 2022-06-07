use salzweg::{decoder::TiffStyleDecoder, encoder::TiffStyleEncoder, CodeSizeStrategy};

const LOREM_IPSUM: &str = include_str!("../../test-assets/lorem_ipsum.txt");
const LOREM_IPSUM_LONG: &str = include_str!("../../test-assets/lorem_ipsum_long.txt");
const LOREM_IPSUM_ENCODED: &[u8] = include_bytes!("../../test-assets/lorem_ipsum_encoded.bin");
const LOREM_IPSUM_LONG_ENCODED: &[u8] =
    include_bytes!("../../test-assets/lorem_ipsum_long_encoded.bin");

/// See https://www.eecis.udel.edu/~amer/CISC651/lzw.and.gif.explained.html
/// Rust example https://rosettacode.org/wiki/LZW_compression#Rust
fn main() {
    check_string_compression(LOREM_IPSUM);
    check_string_compression(LOREM_IPSUM_LONG);
    check_string_decoding(LOREM_IPSUM_ENCODED);
    check_string_decoding(LOREM_IPSUM_LONG_ENCODED);
    check_fixed_string_encoding(LOREM_IPSUM_ENCODED);
    decode_colors();
    check_tiff_encoding(LOREM_IPSUM);
}

fn check_string_compression(string: &str) {
    let mut compressed = vec![];
    salzweg::encoder::VariableEncoder::encode(
        string.as_bytes(),
        &mut compressed,
        7,
        salzweg::Endianness::LittleEndian,
        CodeSizeStrategy::Default,
    )
    .unwrap();

    let mut decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, 7);
    let decompressed = decoder.decode(&compressed).unwrap();

    let decompressed_string = String::from_utf8_lossy(&decompressed);

    assert_eq!(decompressed_string, string);
}

fn check_string_decoding(data: &[u8]) {
    let mut my_decompressed = vec![];
    salzweg::decoder::VariableDecoder::decode(
        data,
        &mut my_decompressed,
        7,
        salzweg::Endianness::LittleEndian,
        CodeSizeStrategy::Default,
    )
    .unwrap();

    let mut weezl_decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, 7);
    let weezl_decompressed = weezl_decoder.decode(&data).unwrap();

    assert_eq!(my_decompressed, weezl_decompressed);
}

fn check_fixed_string_encoding(data: &[u8]) {
    let compressed =
        salzweg::encoder::FixedEncoder::encode_to_vec(data, salzweg::Endianness::LittleEndian)
            .unwrap();

    let decompressed = salzweg::decoder::FixedDecoder::decode_to_vec(
        &compressed[..],
        salzweg::Endianness::LittleEndian,
    )
    .unwrap();

    assert_eq!(decompressed, data);
}

fn decode_colors() {
    let data = [
        0x8C, 0x2D, 0x99, 0x87, 0x2A, 0x1C, 0xDC, 0x33, 0xA0, 0x2, 0x55, 0x0,
    ];

    let decoded = salzweg::decoder::VariableDecoder::decode_to_vec(
        &data[..],
        2,
        salzweg::Endianness::LittleEndian,
        CodeSizeStrategy::Default,
    )
    .unwrap();

    assert_eq!(
        decoded,
        [
            1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2,
            2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
        ]
    );
}

fn check_tiff_encoding(string: &str) {
    let data = string.as_bytes();
    let compressed = TiffStyleEncoder::encode_to_vec(data).unwrap();

    let decompressed = TiffStyleDecoder::decode_to_vec(&compressed[..]).unwrap();

    assert_eq!(data, decompressed);
}
