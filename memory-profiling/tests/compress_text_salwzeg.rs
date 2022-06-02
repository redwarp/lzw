#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const LOREM_IPSUM_LONG: &str = include_str!("../../test-assets/lorem_ipsum_long.txt");

#[test]
fn compress_text_salzweg() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let mut compressed = Vec::with_capacity(10000);

    let start_stats = dhat::HeapStats::get();

    let my_encoder = salzweg::Encoder::new(7, salzweg::Endianness::LittleEndian);
    my_encoder
        .encode(LOREM_IPSUM_LONG.as_bytes(), &mut compressed)
        .unwrap();

    let stats = dhat::HeapStats::get();

    println!("Let's profile stuff!");
    println!("Compressed size: {}", compressed.len());

    println!("{start_stats:?}");
    println!("{stats:?}");
}
