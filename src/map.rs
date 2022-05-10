use std::collections::HashMap;

use crate::{
    bytes_to_string,
    lzw::{LzwCompressor, LzwDecompressor},
    string_to_bytes,
};

struct CompressionTable {
    entries: HashMap<Vec<u8>, u16>,
}

impl CompressionTable {
    fn new(code_size: u8, possibilities: u16) -> Self {
        if !(1..=12).contains(&code_size) {
            panic!("Code size is {code_size}, should be between 1 and 12");
        }

        let entries: HashMap<_, _> = (0..possibilities).map(|i| (vec![i as u8], i)).collect();

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

struct DecompressionTable {
    entries: HashMap<u16, Vec<u8>>,
}

impl DecompressionTable {
    fn new(code_size: u8, possibilities: u16) -> Self {
        if !(1..=12).contains(&code_size) {
            panic!("Code size is {code_size}, should be between 1 and 12");
        }

        let entries: HashMap<_, _> = (0..possibilities).map(|i| (i, vec![i as u8])).collect();

        Self { entries }
    }

    fn string_for(&self, code: u16) -> Option<&Vec<u8>> {
        self.entries.get(&code)
    }

    fn add(&mut self, entry: Vec<u8>) {
        self.entries.insert(self.entries.len() as u16, entry);
    }
}

pub struct WithHashMap;

impl LzwCompressor for WithHashMap {
    fn compress(bytes: &[u8], code_size: u8, possibilities: u16) -> Vec<u16> {
        let mut code_stream = vec![];

        let mut string_table = CompressionTable::new(code_size, possibilities);
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

impl LzwDecompressor for WithHashMap {
    fn decompress(data: &[u16], code_size: u8, possibilities: u16) -> Vec<u8> {
        let mut char_stream: Vec<u8> = vec![];
        let mut string_table = DecompressionTable::new(code_size, possibilities);

        let current_code = data[0];
        char_stream.extend_from_slice(
            string_table
                .string_for(current_code)
                .expect("First entry should be in the table boundaries"),
        );

        let mut previous_entry = string_table
            .string_for(current_code)
            .expect("We should have an entry for the previous code by now")
            .to_vec();

        for &current_code in &data[1..] {
            let entry = if let Some(string) = string_table.string_for(current_code) {
                string.clone()
            } else {
                let mut entry = previous_entry.clone();
                entry.push(previous_entry[0]);
                entry
            };

            char_stream.extend_from_slice(&entry);

            previous_entry.push(entry[0]);
            string_table.add(previous_entry);
            previous_entry = entry;
        }

        char_stream
    }
}

pub fn basic_string() {
    let original = "ABACABADADABBBBBB";
    println!("Original: {original}");

    let converted = string_to_bytes(original);
    println!("Converted: {converted:?}");

    let stream = WithHashMap::compress(&converted, 4, 4);
    println!("Code stream {stream:?}");

    let decompressed_stream = WithHashMap::decompress(&stream, 4, 4);
    println!("Decoded stream {decompressed_stream:?}");

    let reverted = bytes_to_string(&decompressed_stream);
    println!("Reverted: {reverted}");
}

pub fn ascii_string() {
    let original = "TOBEORNOTTOBEORTOBEORNOTTOBEORNOTTOBEORTOBEORNOT";
    println!("Original: {original}");

    let converted = original.as_bytes();
    println!("Converted: {converted:?}");

    let stream = WithHashMap::compress(&converted, 12, 128);
    println!("Code stream {stream:?}");

    let decompressed_stream = WithHashMap::decompress(&stream, 12, 128);
    println!("Decoded stream {decompressed_stream:?}");

    let reverted = String::from_utf8_lossy(&decompressed_stream);
    println!("Reverted: {reverted}");

    println!(
        "Original size: {}, code stream size: {}",
        original.len(),
        stream.len()
    );
}
