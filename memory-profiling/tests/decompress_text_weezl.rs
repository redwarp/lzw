#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const LOREM_IPSUM_LONG_ENCODED: &[u8] =
    include_bytes!("../../test-assets/lorem_ipsum_long_encoded.bin");

#[test]
fn decompress_text_salzweg() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let mut decompressed = std::io::sink();

    let start_stats = dhat::HeapStats::get();

    let mut my_decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, 7);
    my_decoder
        .into_stream(&mut decompressed)
        .decode(LOREM_IPSUM_LONG_ENCODED)
        .status
        .unwrap();

    let stats = dhat::HeapStats::get();

    println!("{start_stats:?}");
    println!("{stats:?}");
}
