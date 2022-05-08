pub mod lzw;
pub mod map;
pub mod stack;

pub fn string_to_bytes(string: &str) -> Vec<u8> {
    let mut converted = string.as_bytes().to_vec();
    for bob in converted.iter_mut() {
        *bob -= 65;
    }
    converted
}

pub fn bytes_to_string(data: &[u8]) -> String {
    std::str::from_utf8(&data.iter().map(|&byte| byte + 65).collect::<Vec<_>>())
        .unwrap()
        .to_string()
}

#[cfg(test)]
mod tests {
    use crate::{lzw::Lzw, map::WithHashMap};

    #[test]
    fn test_compress_decompress_with_hashmap() {
        let original = "Just a simple ASCII string, without issues";

        let compressed = WithHashMap::compress(original.as_bytes(), 12, 128);
        let decoded = WithHashMap::decompress(&compressed, 12, 128);

        let reverted = String::from_utf8_lossy(&decoded);

        assert_eq!(original, reverted.as_ref());
    }
}
