use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lzw_test::{
    compress, EncoderVersion1, EncoderVersion2, EncoderVersion3, EncoderVersion4, EncoderVersion5,
    EncoderVersion6,
};

const LOREM_IPSUM: &[u8] = include_str!("../lorem_ipsum.txt").as_bytes();

pub fn compression_evolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression evolution");
    group.bench_function("version 1: A giant vec of vecs", |b| {
        b.iter(|| compress::<EncoderVersion1>(LOREM_IPSUM, black_box(7)));
    });
    group.bench_function("version 2: A hash maps of vecs", |b| {
        b.iter(|| compress::<EncoderVersion2>(LOREM_IPSUM, black_box(7)));
    });
    group.bench_function("version 3: A hash maps containins words", |b| {
        b.iter(|| compress::<EncoderVersion3>(LOREM_IPSUM, black_box(7)));
    });
    group.bench_function("version 4: Using a tree", |b| {
        b.iter(|| compress::<EncoderVersion4>(LOREM_IPSUM, black_box(7)));
    });
    group.bench_function("version 5: Using a simplified tree", |b| {
        b.iter(|| compress::<EncoderVersion5>(LOREM_IPSUM, black_box(7)));
    });
    group.bench_function("version 6: Tree with optimized leaves", |b| {
        b.iter(|| compress::<EncoderVersion6>(LOREM_IPSUM, black_box(7)));
    });
}

criterion_group!(benches, compression_evolution);
criterion_main!(benches);
