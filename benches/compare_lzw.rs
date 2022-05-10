use bitstream_io::LittleEndian;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const LOREM_IPSUM: &str = include_str!("../lorem_ipsum.txt");

fn load_data() -> &'static [u8] {
    LOREM_IPSUM.as_bytes()
}

pub fn compress_lzw_crate(c: &mut Criterion) {
    let data = load_data();

    c.bench_function("compression with lzw crate", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut enc =
                lzw::Encoder::new(lzw::LsbWriter::new(&mut compressed), black_box(7)).unwrap();
            enc.encode_bytes(data).unwrap();
        })
    });
}

pub fn compress_weezl_crate(c: &mut Criterion) {
    let data = load_data();

    c.bench_function("compression with weezl crate", |b| {
        b.iter(|| {
            let mut encoder = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, black_box(7));
            encoder.encode(data).unwrap();
        })
    });
}

pub fn compress_mylzw_crate(c: &mut Criterion) {
    let data = load_data();

    c.bench_function("compression with my implementation", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut encoder = my_lzw::Encoder::new(7, LittleEndian);
            encoder.encode(data, &mut compressed);
        })
    });
}

criterion_group!(
    benches,
    compress_lzw_crate,
    compress_weezl_crate,
    compress_mylzw_crate
);
criterion_main!(benches);
