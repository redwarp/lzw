pub trait Lzw {
    fn compress(bytes: &[u8], code_size: u8, possibilities: u16) -> Vec<u16>;
    fn decompress(data: &[u16], code_size: u8, possibilities: u16) -> Vec<u8>;
}
