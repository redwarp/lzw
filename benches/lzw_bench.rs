use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lzw_test::{
    lzw::LzwCompressor,
    map::WithHashMap,
    stack::{Stacked, WithBigVec, WithMixHashVec},
};

const LOREM_IPSUM: &str = include_str!("../lorem_ipsum.txt");

pub fn compression_with_hashmap_benchmark(c: &mut Criterion) {
    let data = LOREM_IPSUM.as_bytes();

    c.bench_function("compression with hashmap", |b| {
        b.iter(|| WithHashMap::compress(data, black_box(12), black_box(128)))
    });
}

pub fn compression_with_stack_benchmark(c: &mut Criterion) {
    let data = LOREM_IPSUM.as_bytes();

    c.bench_function("compression with stacks", |b| {
        b.iter(|| Stacked::compress(data, black_box(12), black_box(128)))
    });
}

pub fn compression_with_table_benchmark(c: &mut Criterion) {
    let data = LOREM_IPSUM.as_bytes();

    c.bench_function("compression with table", |b| {
        b.iter(|| WithBigVec::compress(data, black_box(12), black_box(128)))
    });
}

pub fn compression_with_mix_benchmark(c: &mut Criterion) {
    let data = LOREM_IPSUM.as_bytes();

    c.bench_function("compression with mix", |b| {
        b.iter(|| WithMixHashVec::compress(data, black_box(12), black_box(128)))
    });
}

criterion_group!(
    benches,
    compression_with_hashmap_benchmark,
    compression_with_stack_benchmark,
    compression_with_table_benchmark,
    compression_with_mix_benchmark
);
criterion_main!(benches);
