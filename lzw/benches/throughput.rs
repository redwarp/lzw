use std::{fs::File, path::Path};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{prelude::StdRng, RngCore, SeedableRng};
use salzweg::{
    decoder::{GifStyleDecoder, TiffStyleDecoder},
    encoder::{GifStyleEncoder, TiffStyleEncoder},
};

trait Encoded {
    fn gif_encoded(&self, code_size: u8) -> Vec<u8>;
    fn tiff_encoded(&self) -> Vec<u8>;
}

impl Encoded for &[u8] {
    fn gif_encoded(&self, code_size: u8) -> Vec<u8> {
        GifStyleEncoder::encode_to_vec(&self[..], code_size).expect("Couldn't compress")
    }

    fn tiff_encoded(&self) -> Vec<u8> {
        TiffStyleEncoder::encode_to_vec(&self[..]).expect("Couldn't compress")
    }
}

impl Encoded for Vec<u8> {
    fn gif_encoded(&self, code_size: u8) -> Vec<u8> {
        (&self[..]).gif_encoded(code_size)
    }

    fn tiff_encoded(&self) -> Vec<u8> {
        (&self[..]).tiff_encoded()
    }
}

fn bench_text(c: &mut Criterion) {
    let data = load_file("lorem_ipsum_long.txt");

    bench(c, "ASCII data", data.as_slice(), 7);
}

fn bench_image(c: &mut Criterion) {
    let data = prepare_image_data();

    bench(c, "Image data", data.as_slice(), 7);
}

fn bench_random(c: &mut Criterion) {
    let data = prepare_random_data();

    bench(c, "Random data", data.as_slice(), 8);
}

fn bench(c: &mut Criterion, name: &str, data: &[u8], code_size: u8) {
    encoding(c, name, data, code_size);
    decoding(c, name, data, code_size);
}

fn encoding(c: &mut Criterion, name: &str, data: &[u8], code_size: u8) {
    gif_encoding(c, name, data, code_size);
    tiff_encoding(c, name, data);
}

fn decoding(c: &mut Criterion, name: &str, data: &[u8], code_size: u8) {
    gif_decoding(c, name, data.gif_encoded(code_size).as_slice(), code_size);
    tiff_decoding(c, name, data.tiff_encoded().as_slice());
}

fn gif_encoding(c: &mut Criterion, name: &str, data: &[u8], code_size: u8) {
    let mut group = c.benchmark_group(format!("Throughput"));

    let mut encoded = GifStyleEncoder::encode_to_vec(&data[..], code_size).expect("Error");

    let id = BenchmarkId::new(name, "Encode GIF");
    group.throughput(criterion::Throughput::Bytes(data.len() as u64));
    group.bench_with_input(id, &data[..], |b, data| {
        b.iter(|| GifStyleEncoder::encode(data, encoded.as_mut_slice(), black_box(code_size)))
    });
    group.finish();
}

fn tiff_encoding(c: &mut Criterion, name: &str, data: &[u8]) {
    let mut group = c.benchmark_group(format!("Throughput"));

    let mut encoded = TiffStyleEncoder::encode_to_vec(&data[..]).expect("Error");

    let id = BenchmarkId::new(name, "Encode TIFF");
    group.throughput(criterion::Throughput::Bytes(data.len() as u64));
    group.bench_with_input(id, &data[..], |b, data| {
        b.iter(|| TiffStyleEncoder::encode(data, encoded.as_mut_slice()))
    });
    group.finish();
}

fn gif_decoding(c: &mut Criterion, name: &str, data: &[u8], code_size: u8) {
    let mut group = c.benchmark_group(format!("Throughput"));

    let mut decoded = GifStyleDecoder::decode_to_vec(&data[..], code_size).expect("Error");

    let id = BenchmarkId::new(name, "Decode GIF");
    group.throughput(criterion::Throughput::Bytes(data.len() as u64));
    group.bench_with_input(id, &data[..], |b, data| {
        b.iter(|| GifStyleDecoder::decode(data, decoded.as_mut_slice(), black_box(code_size)))
    });
    group.finish();
}

fn tiff_decoding(c: &mut Criterion, name: &str, data: &[u8]) {
    let mut group = c.benchmark_group(format!("Throughput"));

    let mut decoded = TiffStyleDecoder::decode_to_vec(&data[..]).expect("Error");

    let id = BenchmarkId::new(name, "Decode TIFF");
    group.throughput(criterion::Throughput::Bytes(data.len() as u64));
    group.bench_with_input(id, &data[..], |b, data| {
        b.iter(|| TiffStyleDecoder::decode(data, decoded.as_mut_slice()))
    });
    group.finish();
}

fn load_file(file_name: &str) -> Vec<u8> {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Root folder not found")
        .join("test-assets")
        .join(file_name);
    let data = std::fs::read(file).expect(format!("File {file_name} not found").as_str());
    data
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

fn prepare_random_data() -> Vec<u8> {
    let mut rand = StdRng::seed_from_u64(42);
    let mut data: Vec<u8> = vec![0; 1 << 20];
    rand.fill_bytes(&mut data[..]);

    data
}

criterion_group!(benches, bench_text, bench_image, bench_random);

criterion_main!(benches);
