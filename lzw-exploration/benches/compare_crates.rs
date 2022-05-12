use criterion::{black_box, criterion_group, criterion_main, Criterion};

const LOREM_IPSUM: &str = include_str!("../../test-assets/lorem_ipsum.txt");

fn load_data() -> &'static [u8] {
    LOREM_IPSUM.as_bytes()
}

pub fn encoding_all_crates(c: &mut Criterion) {
    let data = load_data();

    let mut group = c.benchmark_group("encoding crates comparison");
    group.bench_function("lzw", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut enc =
                lzw::Encoder::new(lzw::LsbWriter::new(&mut compressed), black_box(7)).unwrap();
            enc.encode_bytes(data).unwrap();
        })
    });
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut encoder = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, black_box(7));
            let mut stream_encoder = encoder.into_stream(&mut compressed);
            stream_encoder.encode(data).status.unwrap();
        })
    });
    group.bench_function("fast-lzw", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut encoder =
                fast_lzw::Encoder::new(black_box(7), fast_lzw::Endianness::LittleEndian);
            encoder.encode(data, &mut compressed).unwrap();
        })
    });
}

criterion_group!(benches, encoding_all_crates);
criterion_main!(benches);
