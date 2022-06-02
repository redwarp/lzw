use std::{
    fmt::Display,
    io::{Read, Write},
};

use crate::{
    io::{BigEndianWriter, BitWriter, LittleEndianWriter},
    Endianness,
};

#[derive(Debug)]
pub enum EncodingError {
    Io(std::io::Error),
    CodeSize(u8),
}

impl Display for EncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodingError::Io(error) => std::fmt::Display::fmt(&error, f),
            EncodingError::CodeSize(code_size) => f.write_fmt(format_args!(
                "Code size must be between 2 and 8, was {code_size}",
            )),
        }
    }
}

impl std::error::Error for EncodingError {}

impl From<std::io::Error> for EncodingError {
    fn from(error: std::io::Error) -> Self {
        EncodingError::Io(error)
    }
}

#[derive(Debug, Clone)]
enum Node {
    NoChild,
    OneChild(u8, u16),
    ManyChildren(Vec<u16>),
}

/// Inspired by trie: https://en.wikipedia.org/wiki/Trie
/// Using this suggestion: https://dev.to/deciduously/no-more-tears-no-more-knots-arena-allocated-trees-in-rust-44k6
struct Tree {
    nodes: Vec<Node>,
    code_size: u8,
}

impl Tree {
    fn new(code_size: u8) -> Self {
        let nodes = Vec::with_capacity(1 << (code_size + 1));
        Self { nodes, code_size }
    }

    fn reset(&mut self) {
        self.nodes.clear();
        self.nodes.resize((1 << self.code_size) + 2, Node::NoChild);
    }

    fn find_word(&self, prefix_index: u16, next_char: u8) -> Option<u16> {
        let prefix = &self.nodes[prefix_index as usize];
        match prefix {
            Node::NoChild => None,
            Node::OneChild(child_char, child_index) => {
                if *child_char == next_char {
                    Some(*child_index)
                } else {
                    None
                }
            }
            Node::ManyChildren(child_indices) => {
                let child_index = child_indices[next_char as usize];
                if child_index != u16::MAX {
                    Some(child_index)
                } else {
                    None
                }
            }
        }
    }

    fn add(&mut self, prefix_index: u16, k: u8) -> u16 {
        let new_index = self.nodes.len() as u16;
        let prefix_index = prefix_index as usize;

        let mut old_node = &mut self.nodes[prefix_index];

        match &mut old_node {
            Node::NoChild => {
                self.nodes[prefix_index] = Node::OneChild(k, new_index);
            }
            Node::OneChild(other_k, other_index) => {
                let mut children = vec![u16::MAX; 1 << self.code_size];
                children[*other_k as usize] = *other_index;
                children[k as usize] = new_index;
                self.nodes[prefix_index] = Node::ManyChildren(children);
            }
            Node::ManyChildren(children) => {
                children[k as usize] = new_index;
            }
        };
        self.nodes.push(Node::NoChild);
        new_index
    }
}

pub struct Encoder {}

impl Encoder {
    pub fn encode<R: Read, W: Write>(
        data: R,
        into: W,
        code_size: u8,
        endianness: Endianness,
    ) -> Result<(), EncodingError> {
        match endianness {
            Endianness::BigEndian => {
                Encoder::inner_encode(data, BigEndianWriter::new(into), code_size)
            }
            Endianness::LittleEndian => {
                Encoder::inner_encode(data, LittleEndianWriter::new(into), code_size)
            }
        }
    }

    pub fn encode_to_vec<R: Read>(
        data: R,
        code_size: u8,
        endianness: Endianness,
    ) -> Result<Vec<u8>, EncodingError> {
        let mut output = vec![];
        Encoder::encode(data, &mut output, code_size, endianness)?;
        Ok(output)
    }

    fn inner_encode<R: Read, B: BitWriter>(
        data: R,
        bit_writer: B,
        code_size: u8,
    ) -> Result<(), EncodingError> {
        if code_size < 2 && code_size > 8 {
            return Err(EncodingError::CodeSize(code_size));
        }

        let mut bit_writer = bit_writer;

        let mut write_size = code_size + 1;
        let clear_code = 1 << code_size;
        let end_of_information = (1 << code_size) + 1;

        let mut tree = Tree::new(code_size);
        tree.reset();

        bit_writer.write(write_size, clear_code)?;

        let mut bytes = data.bytes();
        let k = bytes.next();
        if k.is_none() {
            // Well, it's an empty stream! Leaving early.
            bit_writer.write(write_size, end_of_information)?;

            bit_writer.fill()?;
            bit_writer.flush()?;

            return Ok(());
        }

        let mut current_prefix = k.unwrap()? as u16;

        for k in bytes {
            let k = k?;

            if let Some(word) = tree.find_word(current_prefix, k) {
                current_prefix = word;
            } else {
                let index_of_new_entry = tree.add(current_prefix, k);
                bit_writer.write(write_size, current_prefix)?;
                current_prefix = k as u16;

                if index_of_new_entry == 1 << write_size {
                    write_size += 1;

                    if write_size > 12 {
                        bit_writer.write(12, clear_code)?;
                        write_size = code_size + 1;
                        tree.reset();
                    }
                }
            }
        }

        bit_writer.write(write_size, current_prefix as u16)?;
        bit_writer.write(write_size, end_of_information)?;

        bit_writer.fill()?;
        bit_writer.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_4color_data() {
        let data = [
            1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2,
            2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
        ];

        let mut compressed = vec![];
        Encoder::encode(&data[..], &mut compressed, 2, Endianness::LittleEndian).unwrap();

        assert_eq!(
            compressed,
            [0x8C, 0x2D, 0x99, 0x87, 0x2A, 0x1C, 0xDC, 0x33, 0xA0, 0x2, 0x55, 0x0,]
        )
    }

    #[test]
    fn encode_multiple_with_same_encoder() {
        let data = [
            1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2,
            2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
        ];

        let compression1 = Encoder::encode_to_vec(&data[..], 2, Endianness::LittleEndian).unwrap();
        let compression2 = Encoder::encode_to_vec(&data[..], 2, Endianness::LittleEndian).unwrap();

        assert_eq!(compression1, compression2);
    }

    #[test]
    fn encode_lorem_ipsum() {
        let data = include_bytes!("../../test-assets/lorem_ipsum.txt");
        let expected = include_bytes!("../../test-assets/lorem_ipsum_encoded.bin");

        let mut compressed = vec![];
        Encoder::encode(&data[..], &mut compressed, 7, Endianness::LittleEndian).unwrap();

        assert_eq!(compressed, expected);
    }
}
