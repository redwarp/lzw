use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lzw::{lzw::Lzw, map::WithHashMap, stack::Stacked};

const BASIC_ASCII_STRING: &str =
    "Well, that is just a simple ascii string, to check compression speed";

pub fn compression_with_hashmap_benchmark(c: &mut Criterion) {
    let data = BASIC_ASCII_STRING.as_bytes();

    c.bench_function("compression with hashmap", |b| {
        b.iter(|| WithHashMap::compress(data, black_box(12), black_box(128)))
    });
}

pub fn compression_with_stack_benchmark(c: &mut Criterion) {
    let data = BASIC_ASCII_STRING.as_bytes();

    c.bench_function("compression with stacks", |b| {
        b.iter(|| Stacked::compress(data, black_box(12), black_box(128)))
    });
}

criterion_group!(
    benches,
    compression_with_hashmap_benchmark,
    compression_with_stack_benchmark
);
criterion_main!(benches);
