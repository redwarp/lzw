use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lzw_test::{
    compress,
    lzw::LzwCompressor,
    stack::{Stacked, WithBigVec, WithMixHashVec},
    EncoderVersion1, EncoderVersion2,
};

const LOREM_IPSUM: &[u8] = include_str!("../lorem_ipsum.txt").as_bytes();

pub fn compression_with_stack_benchmark(c: &mut Criterion) {
    let data = LOREM_IPSUM;

    c.bench_function("compression with stacks", |b| {
        b.iter(|| Stacked::compress(data, black_box(12), black_box(128)))
    });
}

pub fn compression_with_table_benchmark(c: &mut Criterion) {
    let data = LOREM_IPSUM;

    c.bench_function("compression with table", |b| {
        b.iter(|| WithBigVec::compress(data, black_box(12), black_box(128)))
    });
}

pub fn compression_with_mix_benchmark(c: &mut Criterion) {
    let data = LOREM_IPSUM;

    c.bench_function("compression with mix", |b| {
        b.iter(|| WithMixHashVec::compress(data, black_box(12), black_box(128)))
    });
}

pub fn compression_evolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression evolution");
    group.bench_function("version 1: A giant vec of vecs", |b| {
        b.iter(|| compress::<EncoderVersion1>(LOREM_IPSUM, 7));
    });
    group.bench_function("version 2: A hash maps of vecs", |b| {
        b.iter(|| compress::<EncoderVersion2>(LOREM_IPSUM, 7));
    });
}

criterion_group!(
    benches,
    compression_with_stack_benchmark,
    compression_with_table_benchmark,
    compression_with_mix_benchmark,
    compression_evolution
);
criterion_main!(benches);
