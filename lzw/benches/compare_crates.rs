use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{prelude::StdRng, RngCore, SeedableRng};
use salzweg::CodeSizeIncrease;
use std::{fs::File, io::Write, path::Path};

const LOREM_IPSUM: &[u8] = include_str!("../../test-assets/lorem_ipsum_long.txt").as_bytes();
const LOREM_IPSUM_ENCODED: &[u8] = include_bytes!("../../test-assets/lorem_ipsum_long_encoded.bin");
const LOREM_IPSUM_ENCODED_BE: &[u8] =
    include_bytes!("../../test-assets/lorem_ipsum_long_encoded_be.bin");

pub fn encoding_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding text");
    group.bench_function("lzw", |b| {
        b.iter(|| {
            let mut encoder =
                lzw::Encoder::new(lzw::LsbWriter::new(std::io::sink()), black_box(7)).unwrap();
            encoder.encode_bytes(LOREM_IPSUM).unwrap();
        })
    });
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut encoder = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, black_box(7));
            let mut stream_encoder = encoder.into_stream(std::io::sink());
            stream_encoder.encode(LOREM_IPSUM).status.unwrap();
        })
    });
    group.bench_function("salzweg", |b| {
        b.iter(|| {
            salzweg::encoder::VariableEncoder::encode(
                LOREM_IPSUM,
                std::io::sink(),
                black_box(7),
                salzweg::Endianness::LittleEndian,
                CodeSizeIncrease::Default,
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
            let mut encoder =
                lzw::Encoder::new(lzw::LsbWriter::new(std::io::sink()), black_box(8)).unwrap();
            encoder.encode_bytes(&data).unwrap();
        })
    });
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut encoder = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, black_box(8));
            let mut stream_encoder = encoder.into_stream(std::io::sink());
            stream_encoder.encode(&data[..]).status.unwrap();
        })
    });
    group.bench_function("salzweg", |b| {
        b.iter(|| {
            salzweg::encoder::VariableEncoder::encode(
                &data[..],
                std::io::sink(),
                black_box(8),
                salzweg::Endianness::LittleEndian,
                CodeSizeIncrease::Default,
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
            let mut encoder =
                lzw::Encoder::new(lzw::LsbWriter::new(std::io::sink()), black_box(7)).unwrap();
            encoder.encode_bytes(&data).unwrap();
        })
    });
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut encoder = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, black_box(7));
            let mut stream_encoder = encoder.into_stream(std::io::sink());
            stream_encoder.encode(&data[..]).status.unwrap();
        })
    });
    group.bench_function("salzweg", |b| {
        b.iter(|| {
            salzweg::encoder::VariableEncoder::encode(
                &data[..],
                std::io::sink(),
                black_box(7),
                salzweg::Endianness::LittleEndian,
                CodeSizeIncrease::Default,
            )
            .unwrap();
        })
    });
}

pub fn decoding_text(c: &mut Criterion) {
    decoding_bench(c, "decoding text", LOREM_IPSUM_ENCODED, 7, || {
        std::io::sink()
    });
}

pub fn decoding_random_data(c: &mut Criterion) {
    let encoded_data = prepare_encoded_random_data();

    decoding_bench(c, "decoding random data", &encoded_data, 8, || {
        std::io::sink()
    });
}
pub fn decoding_image_data(c: &mut Criterion) {
    let encoded_data = prepare_encoded_image_data();

    decoding_bench(c, "decoding image data", &encoded_data, 7, || {
        std::io::sink()
    });
}

pub fn decoding_image_to_vec(c: &mut Criterion) {
    let encoded_data = prepare_encoded_image_data();

    decoding_bench(c, "decoding image to vec", &encoded_data, 7, || vec![]);
}

pub fn decoding_text_to_vec_be(c: &mut Criterion) {
    let mut group = c.benchmark_group("decoding text to vec be");
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut decoder = weezl::decode::Decoder::new(weezl::BitOrder::Msb, black_box(7));
            decoder
                .into_stream(vec![])
                .decode(LOREM_IPSUM_ENCODED_BE)
                .status
                .unwrap();
        })
    });
    group.bench_function("salzweg", |b| {
        b.iter(|| {
            salzweg::decoder::VariableDecoder::decode(
                LOREM_IPSUM_ENCODED_BE,
                vec![],
                black_box(7),
                salzweg::Endianness::BigEndian,
                CodeSizeIncrease::Default,
            )
            .unwrap();
        })
    });
}

fn decoding_bench<F, W>(c: &mut Criterion, name: &str, data: &[u8], code_size: u8, into: F)
where
    F: 'static + FnOnce() -> W + Copy,
    W: Write,
{
    let mut group = c.benchmark_group(name);
    group.bench_function("weezl", |b| {
        b.iter(|| {
            let mut decoder =
                weezl::decode::Decoder::new(weezl::BitOrder::Lsb, black_box(code_size));
            decoder.into_stream(into()).decode(data).status.unwrap();
        })
    });
    group.bench_function("salzweg", |b| {
        b.iter(|| {
            salzweg::decoder::VariableDecoder::decode(
                data,
                into(),
                black_box(code_size),
                salzweg::Endianness::LittleEndian,
                CodeSizeIncrease::Default,
            )
            .unwrap();
        })
    });
}

fn prepare_random_data() -> Vec<u8> {
    let mut rand = StdRng::seed_from_u64(42);
    let mut data: Vec<u8> = vec![0; 1 << 20];
    rand.fill_bytes(&mut data[..]);

    data
}

fn prepare_encoded_random_data() -> Vec<u8> {
    let data = prepare_random_data();

    let mut output = vec![];

    salzweg::encoder::VariableEncoder::encode(
        &data[..],
        &mut output,
        8,
        salzweg::Endianness::LittleEndian,
        CodeSizeIncrease::Default,
    )
    .unwrap();
    output
}

/// This actually prepare a vec of values in 0..128. It works because the image, a png with 128 colors,
/// has been reduced with oxipng, and is now a png with indexed colors.
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

    let mut output = vec![];

    salzweg::encoder::VariableEncoder::encode(
        &data[..],
        &mut output,
        7,
        salzweg::Endianness::LittleEndian,
        CodeSizeIncrease::Default,
    )
    .unwrap();

    output
}

criterion_group!(
    benches,
    encoding_text,
    encoding_random_data,
    encoding_image_data,
    decoding_text,
    decoding_random_data,
    decoding_image_data,
    decoding_image_to_vec,
    decoding_text_to_vec_be
);
criterion_main!(benches);
