use std::{fs::File, path::Path};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{prelude::StdRng, RngCore, SeedableRng};

const LOREM_IPSUM: &[u8] = include_str!("../../test-assets/lorem_ipsum_long.txt").as_bytes();
const LOREM_IPSUM_ENCODED: &[u8] = include_bytes!("../../test-assets/lorem_ipsum_long_encoded.bin");

pub fn encoding_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding text");
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
    group.bench_function("salzweg", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            salzweg::Encoder::encode(
                LOREM_IPSUM,
                &mut compressed,
                black_box(7),
                salzweg::Endianness::LittleEndian,
            )
            .unwrap();
        })
    });
}

pub fn encoding_random_data(c: &mut Criterion) {
    let data = prepare_random_data();

    let mut group = c.benchmark_group("encoding random data");
    group.bench_function("lzw", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut encoder =
                lzw::Encoder::new(lzw::LsbWriter::new(&mut compressed), black_box(8)).unwrap();
            encoder.encode_bytes(&data).unwrap();
        })
    });
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut encoder = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, black_box(8));
            let mut stream_encoder = encoder.into_stream(&mut compressed);
            stream_encoder.encode(&data[..]).status.unwrap();
        })
    });
    group.bench_function("salzweg", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            salzweg::Encoder::encode(
                &data[..],
                &mut compressed,
                black_box(8),
                salzweg::Endianness::LittleEndian,
            )
            .unwrap();
        })
    });
}

pub fn encoding_image_data(c: &mut Criterion) {
    let data = prepare_image_data();

    let mut group = c.benchmark_group("encoding image data");
    group.bench_function("lzw", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut encoder =
                lzw::Encoder::new(lzw::LsbWriter::new(&mut compressed), black_box(7)).unwrap();
            encoder.encode_bytes(&data).unwrap();
        })
    });
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            let mut encoder = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, black_box(7));
            let mut stream_encoder = encoder.into_stream(&mut compressed);
            stream_encoder.encode(&data[..]).status.unwrap();
        })
    });
    group.bench_function("salzweg", |b| {
        b.iter(|| {
            let mut compressed = vec![];
            salzweg::Encoder::encode(
                &data[..],
                &mut compressed,
                black_box(7),
                salzweg::Endianness::LittleEndian,
            )
            .unwrap();
        })
    });
}

pub fn decoding_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("decoding text");
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, black_box(7));
            decoder.decode(LOREM_IPSUM_ENCODED).unwrap();
        })
    });
    group.bench_function("salzweg", |b| {
        b.iter(|| {
            let mut decoded = vec![];
            salzweg::Decoder::decode(
                LOREM_IPSUM_ENCODED,
                &mut decoded,
                black_box(7),
                salzweg::Endianness::LittleEndian,
            )
            .unwrap();
        })
    });
}

pub fn decoding_random_data(c: &mut Criterion) {
    let encoded_data = prepare_encoded_random_data();

    let mut group = c.benchmark_group("decoding random data");
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, black_box(8));
            decoder.decode(&encoded_data).unwrap();
        })
    });
    group.bench_function("salzweg", |b| {
        b.iter(|| {
            let mut decoded = vec![];
            salzweg::Decoder::decode(
                &encoded_data[..],
                &mut decoded,
                black_box(8),
                salzweg::Endianness::LittleEndian,
            )
            .unwrap();
        })
    });
}
pub fn decoding_image_data(c: &mut Criterion) {
    let encoded_data = prepare_encoded_image_data();

    let mut group = c.benchmark_group("decoding image data");
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, black_box(7));
            decoder.decode(&encoded_data).unwrap();
        })
    });
    group.bench_function("salzweg", |b| {
        b.iter(|| {
            let mut decoded = vec![];
            salzweg::Decoder::decode(
                &encoded_data[..],
                &mut decoded,
                black_box(7),
                salzweg::Endianness::LittleEndian,
            )
            .unwrap();
        })
    });
}

fn prepare_random_data() -> Vec<u8> {
    let mut rand = StdRng::seed_from_u64(42);
    let mut data: Vec<u8> = vec![0; 1 << 16];
    rand.fill_bytes(&mut data[..]);

    data
}

fn prepare_encoded_random_data() -> Vec<u8> {
    let data = prepare_random_data();

    let mut output = vec![];

    salzweg::Encoder::encode(&data[..], &mut output, 8, salzweg::Endianness::LittleEndian).unwrap();
    output
}

fn prepare_image_data() -> Vec<u8> {
    let image = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("test-assets/tokyo_128_colors.png");

    let png_decoder = png::Decoder::new(File::open(image).unwrap());
    let mut reader = png_decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    buf[..info.buffer_size()].to_vec()
}

fn prepare_encoded_image_data() -> Vec<u8> {
    let data = prepare_image_data();

    let mut compressed = vec![];

    salzweg::Encoder::encode(
        &data[..],
        &mut compressed,
        7,
        salzweg::Endianness::LittleEndian,
    )
    .unwrap();

    compressed
}

criterion_group!(
    benches,
    encoding_text,
    encoding_random_data,
    encoding_image_data,
    decoding_text,
    decoding_random_data,
    decoding_image_data
);
criterion_main!(benches);
