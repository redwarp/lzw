use std::collections::HashMap;

pub mod lzw;
pub mod stack;

pub fn abcd_encode(text: &str) -> Vec<usize> {
    let mut code_stream = vec![];
    // Initialize the table, filling it with the possible values data can take
    let mut string_table: Vec<String> = vec![
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
        "D".to_string(),
    ];

    let mut current_prefix: String = "".to_string();

    for k in text.chars() {
        let mut current_string = current_prefix.clone();
        current_string.push(k);

        if string_table.contains(&current_string) {
            current_prefix = current_string;
        } else {
            string_table.push(current_string);

            let code = string_table
                .iter()
                .position(|word| word == &current_prefix)
                .unwrap();
            code_stream.push(code);
            current_prefix.clear();
            current_prefix.push(k);
        }
    }

    if !current_prefix.is_empty() {
        let code = string_table
            .iter()
            .position(|word| word == &current_prefix)
            .unwrap();
        code_stream.push(code);
    }

    code_stream
}

pub trait Encoder {
    fn new(code_size: u8) -> Self;
    fn encode(&mut self, bytes: &[u8]) -> Vec<u16>;
}

/// Basic and dumb encoder throwing everything in a vector
pub struct EncoderVersion1 {
    code_size: u8,
}

impl Encoder for EncoderVersion1 {
    fn new(code_size: u8) -> Self {
        Self { code_size }
    }

    fn encode(&mut self, bytes: &[u8]) -> Vec<u16> {
        let mut code_stream = vec![];
        // Initialize the table, filling it with the possible values data can take
        let mut string_table: Vec<Vec<u8>> =
            (0..1 << self.code_size).map(|index| vec![index]).collect();

        let mut current_prefix: Vec<u8> = vec![];

        for &k in bytes {
            let mut current_string = current_prefix.clone();
            current_string.push(k);

            if string_table.contains(&current_string) {
                current_prefix = current_string;
            } else {
                string_table.push(current_string);

                let code = string_table
                    .iter()
                    .position(|word| word == &current_prefix)
                    .unwrap() as u16;
                code_stream.push(code);
                current_prefix.clear();
                current_prefix.push(k);
            }
        }

        if !current_prefix.is_empty() {
            let code = string_table
                .iter()
                .position(|word| word == &current_prefix)
                .unwrap() as u16;
            code_stream.push(code);
        }

        code_stream
    }
}

// Let's be less basic and use a hashmap instead
pub struct EncoderVersion2 {
    code_size: u8,
}

impl Encoder for EncoderVersion2 {
    fn new(code_size: u8) -> Self {
        Self { code_size }
    }

    fn encode(&mut self, bytes: &[u8]) -> Vec<u16> {
        struct CompressionTable {
            entries: HashMap<Vec<u8>, u16>,
        }

        impl CompressionTable {
            fn new(code_size: u8) -> Self {
                let entries: HashMap<_, _> =
                    (0..1 << code_size).map(|i| (vec![i as u8], i)).collect();

                Self { entries }
            }

            fn code_for(&self, string: &[u8]) -> Option<&u16> {
                self.entries.get(string)
            }

            fn contains(&self, string: &[u8]) -> bool {
                self.entries.contains_key(string)
            }

            fn add(&mut self, entry: Vec<u8>) {
                self.entries.insert(entry, self.entries.len() as u16);
            }
        }

        let mut code_stream = vec![];

        let mut string_table = CompressionTable::new(self.code_size);
        let mut current_prefix: Vec<u8> = vec![];

        for &k in bytes {
            let mut current_string = current_prefix.clone();
            current_string.push(k);

            if string_table.contains(&current_string) {
                current_prefix = current_string;
            } else {
                string_table.add(current_string);
                code_stream.push(*string_table.code_for(&current_prefix).unwrap());
                current_prefix.clear();
                current_prefix.push(k);
            }
        }

        if !current_prefix.is_empty() {
            code_stream.push(*string_table.code_for(&current_prefix).unwrap())
        }

        code_stream
    }
}

pub fn compress<E: Encoder>(data: &[u8], code_size: u8) -> Vec<u16> {
    let mut encoder = E::new(code_size);

    encoder.encode(data)
}

#[cfg(test)]
mod tests {
    use crate::{abcd_encode, compress, EncoderVersion1, EncoderVersion2};
    const DATA: &[u8; 40] = &[
        1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2,
        1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
    ];

    #[test]
    fn abcd_encoder() {
        let text = "ABACABADADABBBBBB";

        let compressed = abcd_encode(text);

        assert_eq!(compressed, [0, 0, 1, 1, 5, 2, 4, 3, 3, 4, 1]);
    }

    #[test]
    fn encoder_version1() {
        let compressed = compress::<EncoderVersion1>(DATA, 2);

        assert_eq!(
            compressed,
            [1, 4, 4, 2, 7, 7, 5, 6, 8, 2, 10, 1, 12, 13, 4, 0, 19, 0, 8]
        )
    }

    #[test]
    fn encoder_version2() {
        let compressed = compress::<EncoderVersion2>(DATA, 2);
        assert_eq!(
            compressed,
            [1, 4, 4, 2, 7, 7, 5, 6, 8, 2, 10, 1, 12, 13, 4, 0, 19, 0, 8]
        )
    }
}
