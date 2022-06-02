#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const LOREM_IPSUM_LONG_ENCODED: &[u8] =
    include_bytes!("../../test-assets/lorem_ipsum_long_encoded.bin");

#[test]
fn decompress_text_salzweg() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let mut decompressed = Vec::with_capacity(25000);

    let start_stats = dhat::HeapStats::get();

    salzweg::Decoder::decode(
        LOREM_IPSUM_LONG_ENCODED,
        &mut decompressed,
        7,
        salzweg::Endianness::LittleEndian,
    )
    .unwrap();

    let stats = dhat::HeapStats::get();

    println!("Let's profile stuff!");
    println!("Compressed size: {}", decompressed.len());

    println!("{start_stats:?}");
    println!("{stats:?}");
}
