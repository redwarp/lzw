#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const LOREM_IPSUM_LONG: &[u8] = include_str!("../../test-assets/lorem_ipsum_long.txt").as_bytes();

#[test]
fn compress_text_lzw() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let mut compressed = std::io::sink();

    let start_stats = dhat::HeapStats::get();

    let mut encoder = lzw::Encoder::new(lzw::LsbWriter::new(&mut compressed), 7).unwrap();
    encoder.encode_bytes(LOREM_IPSUM_LONG).unwrap();

    let stats = dhat::HeapStats::get();

    println!("{start_stats:?}");
    println!("{stats:?}");
}
