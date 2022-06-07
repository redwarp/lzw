use anyhow::{Ok, Result};
use salzweg::encoder::GifStyleEncoder;
use std::{fs::File, path::Path};

fn main() -> Result<()> {
    // This actually prepare a vec of values in 0..128.
    // It works because the image, a png with 128 colors,
    // has been reduced with oxipng, and is now a png with indexed colors from 0..128.
    let image_data = {
        let image = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("We should arrive in the root folder")
            .join("test-assets/tokyo_128_colors.png");

        let png_decoder = png::Decoder::new(File::open(image)?);
        let mut reader = png_decoder.read_info()?;
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        Ok(buf[..info.buffer_size()].to_vec())
    }?;

    let output = std::io::sink(); // Let's use a sink as the output.

    GifStyleEncoder::encode(&image_data[..], output, 7)?;

    Ok(())
}
