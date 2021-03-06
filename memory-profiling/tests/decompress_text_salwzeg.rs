use salzweg::CodeSizeStrategy;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const LOREM_IPSUM_ENCODED: &[u8] = include_bytes!("../../test-assets/lorem_ipsum_encoded.bin");

#[test]
fn decompress_text_salzweg() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let mut decompressed = std::io::sink();

    let start_stats = dhat::HeapStats::get();

    salzweg::decoder::VariableDecoder::decode(
        LOREM_IPSUM_ENCODED,
        &mut decompressed,
        7,
        salzweg::Endianness::LittleEndian,
        CodeSizeStrategy::Default,
    )
    .unwrap();

    let stats = dhat::HeapStats::get();

    println!("{start_stats:?}");
    println!("{stats:?}");
}
