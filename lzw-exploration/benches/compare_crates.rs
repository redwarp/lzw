use criterion::{black_box, criterion_group, criterion_main, Criterion};

const LOREM_IPSUM: &[u8] = include_str!("../../test-assets/lorem_ipsum_long.txt").as_bytes();
const LOREM_IPSUM_ENCODED: &[u8] = include_bytes!("../../test-assets/lorem_ipsum_long_encoded.bin");

pub fn encoding_all_crates(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding crates comparison");
    group.bench_function("lzw", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut encoder =
                lzw::Encoder::new(lzw::LsbWriter::new(&mut compressed), black_box(7)).unwrap();
            encoder.encode_bytes(LOREM_IPSUM).unwrap();
        })
    });
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut encoder = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, black_box(7));
            let mut stream_encoder = encoder.into_stream(&mut compressed);
            stream_encoder.encode(LOREM_IPSUM).status.unwrap();
        })
    });
    group.bench_function("fast-lzw", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut encoder =
                fast_lzw::Encoder::new(black_box(7), fast_lzw::Endianness::LittleEndian);
            encoder.encode(LOREM_IPSUM, &mut compressed).unwrap();
        })
    });
}

pub fn decoding_all_crates(c: &mut Criterion) {
    let mut group = c.benchmark_group("decoding crates comparison");
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, black_box(7));
            decoder.decode(LOREM_IPSUM_ENCODED).unwrap();
        })
    });
    group.bench_function("fast-lzw", |b| {
        b.iter(|| {
            let mut decoded = vec![];
            let mut decoder =
                fast_lzw::Decoder::new(black_box(7), fast_lzw::Endianness::LittleEndian);
            decoder.decode(LOREM_IPSUM_ENCODED, &mut decoded).unwrap();
        })
    });
    group.bench_function("fast-lzw-2", |b| {
        b.iter(|| {
            let mut decoded = vec![];
            let mut decoder =
                fast_lzw::Decoder::new(black_box(7), fast_lzw::Endianness::LittleEndian);
            decoder.decode2(LOREM_IPSUM_ENCODED, &mut decoded).unwrap();
        })
    });
}

criterion_group!(benches, encoding_all_crates, decoding_all_crates);
criterion_main!(benches);
