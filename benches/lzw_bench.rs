use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lzw::{lzw::Lzw, map::WithHashMap, stack::Stacked};

const BASIC_ASCII_STRING: &str = r#"
Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium,
totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo.
Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos
qui ratione voluptatem sequi nesciunt. Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur,
adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem.
Ut enim ad minima veniam, quis nostrum exercitationem ullam corporis suscipit laboriosam, nisi ut aliquid ex ea commodi consequatur?
Quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur,
vel illum qui dolorem eum fugiat quo voluptas nulla pariatur?
"#;

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
