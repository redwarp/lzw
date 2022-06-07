#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const LOREM_IPSUM_LONG: &str = include_str!("../../test-assets/lorem_ipsum_long.txt");

#[test]
fn compress_text_salzweg() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let mut compressed = std::io::sink();

    let start_stats = dhat::HeapStats::get();

    salzweg::encoder::VariableEncoder::encode(
        LOREM_IPSUM_LONG.as_bytes(),
        &mut compressed,
        7,
        salzweg::Endianness::LittleEndian,
        salzweg::CodeSizeStrategy::Default,
    )
    .unwrap();

    let stats = dhat::HeapStats::get();

    println!("{start_stats:?}");
    println!("{stats:?}");
}
