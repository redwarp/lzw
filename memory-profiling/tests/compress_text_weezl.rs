#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const LOREM_IPSUM_LONG: &str = include_str!("../../test-assets/lorem_ipsum_long.txt");

#[test]
fn compress_text_weezl() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let mut compressed = Vec::with_capacity(10000);

    let start_stats = dhat::HeapStats::get();

    let mut my_encoder = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, 7);
    my_encoder
        .into_stream(&mut compressed)
        .encode(LOREM_IPSUM_LONG.as_bytes())
        .status
        .unwrap();

    let stats = dhat::HeapStats::get();

    println!("Let's profile stuff!");
    println!("Compressed size: {}", compressed.len());

    println!("{start_stats:?}");
    println!("{stats:?}");
}
