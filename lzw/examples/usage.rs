use anyhow::Result;
use salzweg::CodeSizeStrategy;

const LOREM_IPSUM: &[u8] = include_str!("../../test-assets/lorem_ipsum.txt").as_bytes();
const LOREM_IPSUM_ENCODED: &[u8] = include_bytes!("../../test-assets/lorem_ipsum_encoded.bin");

fn main() -> Result<()> {
    let mut compressed = vec![];
    salzweg::encoder::VariableEncoder::encode(
        LOREM_IPSUM,
        &mut compressed,
        7,
        salzweg::Endianness::LittleEndian,
        CodeSizeStrategy::Default,
    )?;

    assert_eq!(compressed, LOREM_IPSUM_ENCODED);

    let mut decompressed = vec![];

    salzweg::decoder::VariableDecoder::decode(
        &compressed[..],
        &mut decompressed,
        7,
        salzweg::Endianness::LittleEndian,
        CodeSizeStrategy::Default,
    )?;

    assert_eq!(decompressed, LOREM_IPSUM);

    Ok(())
}
