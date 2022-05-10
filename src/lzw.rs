pub trait LzwCompressor {
    fn compress(bytes: &[u8], code_size: u8, possibilities: u16) -> Vec<u16>;
}

pub trait LzwDecompressor {
    fn decompress(data: &[u16], code_size: u8, possibilities: u16) -> Vec<u8>;
}
