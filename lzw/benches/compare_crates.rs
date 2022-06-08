use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{fs::File, path::Path};

const LOREM_IPSUM: &[u8] = include_str!("../../test-assets/lorem_ipsum_long.txt").as_bytes();

fn bench_text(c: &mut Criterion) {
    let data = LOREM_IPSUM;

    bench(c, "ASCII data", data, 7);
}

fn bench_image(c: &mut Criterion) {
    let data = prepare_image_data();

    bench(c, "Image data", data.as_slice(), 7);
}

fn bench(c: &mut Criterion, name: &str, data: &[u8], code_size: u8) {
    bench_gif_encoding(c, name, data, code_size);
    bench_gif_decoding(c, name, data, code_size);
    bench_tiff_encoding(c, name, data);
    bench_tiff_decoding(c, name, data);
    bench_fixed_size(c, name, data);
}

fn bench_gif_encoding(c: &mut Criterion, name: &str, data: &[u8], code_size: u8) {
    let mut group = c.benchmark_group("Encode GIF style");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_with_input(BenchmarkId::new(name, "Lzw"), data, |b, i| {
        let mut output = {
            let mut output = vec![];
            let mut encoder =
                lzw::Encoder::new(lzw::LsbWriter::new(&mut output), code_size).unwrap();
            encoder.encode_bytes(i).unwrap();
            drop(encoder);
            output
        };

        b.iter(|| {
            let mut encoder = lzw::Encoder::new(
                lzw::LsbWriter::new(output.as_mut_slice()),
                black_box(code_size),
            )
            .unwrap();
            encoder.encode_bytes(i).expect("Compression failed")
        })
    });

    group.bench_with_input(BenchmarkId::new(name, "Weezl"), data, |b, i| {
        let mut output = {
            weezl::encode::Encoder::new(weezl::BitOrder::Lsb, code_size)
                .encode(data)
                .expect("Compression failed")
        };

        b.iter(|| {
            let mut encoder =
                weezl::encode::Encoder::new(weezl::BitOrder::Lsb, black_box(code_size));
            let mut stream_encoder = encoder.into_stream(output.as_mut_slice());
            stream_encoder.encode(i).status.expect("Compression failed")
        })
    });

    group.bench_with_input(BenchmarkId::new(name, "Salzweg"), data, |b, i| {
        let mut output = salzweg::encoder::GifStyleEncoder::encode_to_vec(data, code_size)
            .expect("Compression failed");

        b.iter(|| {
            salzweg::encoder::GifStyleEncoder::encode(
                i,
                output.as_mut_slice(),
                black_box(code_size),
            )
            .expect("Compression failed");
        })
    });

    group.finish();
}

fn bench_gif_decoding(c: &mut Criterion, name: &str, data: &[u8], code_size: u8) {
    let compressed = salzweg::encoder::GifStyleEncoder::encode_to_vec(data, code_size)
        .expect("Compression failed");

    let mut group = c.benchmark_group("Decode GIF style");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_with_input(
        BenchmarkId::new(name, "Weezl"),
        compressed.as_slice(),
        |b, i| {
            let mut output = vec![0; data.len()];

            b.iter(|| {
                let mut decoder =
                    weezl::decode::Decoder::new(weezl::BitOrder::Lsb, black_box(code_size));
                decoder
                    .into_stream(output.as_mut_slice())
                    .decode(i)
                    .status
                    .expect("Compression failed");
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new(name, "Salzweg"),
        compressed.as_slice(),
        |b, i| {
            let mut output = vec![0; data.len()];

            b.iter(|| {
                salzweg::decoder::GifStyleDecoder::decode(
                    i,
                    output.as_mut_slice(),
                    black_box(code_size),
                )
                .expect("Compression failed");
            })
        },
    );

    group.finish();
}

fn bench_tiff_encoding(c: &mut Criterion, name: &str, data: &[u8]) {
    let mut group = c.benchmark_group("Encode TIFF style");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_with_input(BenchmarkId::new(name, "Weezl"), data, |b, i| {
        let mut output = {
            weezl::encode::Encoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8)
                .encode(data)
                .expect("Compression failed")
        };

        b.iter(|| {
            let mut encoder = weezl::encode::Encoder::new(weezl::BitOrder::Msb, black_box(8));
            let mut stream_encoder = encoder.into_stream(output.as_mut_slice());
            stream_encoder.encode(i).status.expect("Compression failed")
        })
    });

    group.bench_with_input(BenchmarkId::new(name, "Salzweg"), data, |b, i| {
        let mut output =
            salzweg::encoder::TiffStyleEncoder::encode_to_vec(data).expect("Compression failed");

        b.iter(|| {
            salzweg::encoder::TiffStyleEncoder::encode(i, output.as_mut_slice())
                .expect("Compression failed");
        })
    });

    group.finish();
}

fn bench_tiff_decoding(c: &mut Criterion, name: &str, data: &[u8]) {
    let compressed =
        salzweg::encoder::TiffStyleEncoder::encode_to_vec(data).expect("Compression failed");

    let mut group = c.benchmark_group("Decode TIFF style");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_with_input(
        BenchmarkId::new(name, "Weezl"),
        compressed.as_slice(),
        |b, i| {
            let mut output = vec![0; data.len()];

            b.iter(|| {
                let mut decoder = weezl::decode::Decoder::with_tiff_size_switch(
                    weezl::BitOrder::Msb,
                    black_box(8),
                );
                decoder
                    .into_stream(output.as_mut_slice())
                    .decode(i)
                    .status
                    .expect("Compression failed");
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new(name, "Salzweg"),
        compressed.as_slice(),
        |b, i| {
            let mut output = vec![0; data.len()];

            b.iter(|| {
                salzweg::decoder::TiffStyleDecoder::decode(i, output.as_mut_slice())
                    .expect("Compression failed");
            })
        },
    );

    group.finish();
}

fn bench_fixed_size(c: &mut Criterion, name: &str, data: &[u8]) {
    {
        let mut output =
            salzweg::encoder::FixedEncoder::encode_to_vec(data, salzweg::Endianness::LittleEndian)
                .expect("Compression failed");
        let mut group = c.benchmark_group("Encode fixed");
        group.throughput(Throughput::Bytes(data.len() as u64));

        group.bench_with_input(BenchmarkId::new(name, "Little Endian"), data, |b, i| {
            b.iter(|| {
                salzweg::encoder::FixedEncoder::encode(
                    i,
                    output.as_mut_slice(),
                    salzweg::Endianness::LittleEndian,
                )
                .expect("Compression failed");
            })
        });
        group.bench_with_input(BenchmarkId::new(name, "Big Endian"), data, |b, i| {
            b.iter(|| {
                salzweg::encoder::FixedEncoder::encode(
                    i,
                    output.as_mut_slice(),
                    salzweg::Endianness::BigEndian,
                )
                .expect("Compression failed");
            });
        });
    }

    {
        let mut output = vec![0; data.len()];
        let mut group = c.benchmark_group("Decode fixed");

        let compressed =
            salzweg::encoder::FixedEncoder::encode_to_vec(data, salzweg::Endianness::LittleEndian)
                .expect("Compression failed");
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(
            BenchmarkId::new(name, "Little Endian"),
            compressed.as_slice(),
            |b, i| {
                b.iter(|| {
                    salzweg::decoder::FixedDecoder::decode(
                        i,
                        output.as_mut_slice(),
                        salzweg::Endianness::LittleEndian,
                    )
                    .expect("Compression failed");
                })
            },
        );

        let compressed =
            salzweg::encoder::FixedEncoder::encode_to_vec(data, salzweg::Endianness::BigEndian)
                .expect("Compression failed");
        group.bench_with_input(
            BenchmarkId::new(name, "Big Endian"),
            compressed.as_slice(),
            |b, i| {
                b.iter(|| {
                    salzweg::decoder::FixedDecoder::decode(
                        i,
                        output.as_mut_slice(),
                        salzweg::Endianness::BigEndian,
                    )
                    .expect("Compression failed");
                });
            },
        );
    }
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

criterion_group!(benches, bench_text, bench_image);
criterion_main!(benches);
