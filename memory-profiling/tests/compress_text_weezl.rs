#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const LOREM_IPSUM: &str = include_str!("../../test-assets/lorem_ipsum.txt");

#[test]
fn compress_text_weezl() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let mut compressed = std::io::sink();

    let start_stats = dhat::HeapStats::get();

    let mut my_encoder = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, 7);
    my_encoder
        .into_stream(&mut compressed)
        .encode(LOREM_IPSUM.as_bytes())
        .status
        .unwrap();

    let stats = dhat::HeapStats::get();

    println!("{start_stats:?}");
    println!("{stats:?}");
}
