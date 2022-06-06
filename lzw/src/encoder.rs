use std::{
    fmt::Display,
    io::{Read, Write},
};

use crate::{
    io::{BigEndianWriter, BitWriter, LittleEndianWriter},
    Endianness,
};

/// The error type for encoding operations.
///
/// Encapsulate [std::io::Error] and expose LZW code size or unexpected data issues.
#[derive(Debug)]
pub enum EncodingError {
    /// An I/O error happened when reading or writing data.
    Io(std::io::Error),
    /// Code size out of bounds. It should be between 2 and 8 included.
    CodeSize(u8),
    /// An unexpected code was read.
    ///
    /// For a code size of 4 for example,
    /// we expect the data to be between 0 and 2.pow(4) = 16.
    /// If in the data, we would then try to encode 42, it would not be correct and we return this unexpected code error.
    UnexpectedCode { code: u8, code_size: u8 },
}

impl Display for EncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodingError::Io(error) => std::fmt::Display::fmt(&error, f),
            EncodingError::CodeSize(code_size) => f.write_fmt(format_args!(
                "Code size must be between 2 and 8, was {code_size}.",
            )),
            EncodingError::UnexpectedCode { code, code_size } => f.write_fmt(format_args!(
                "Unexpected code {code}. For code size {code_size}, data should be < {}.",
                (1 << code_size)
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
    with_clear_code: bool,
}

impl Tree {
    fn new(code_size: u8, with_clear_code: bool) -> Self {
        let nodes = Vec::with_capacity(1 << (code_size + 1));
        Self {
            nodes,
            code_size,
            with_clear_code,
        }
    }

    fn reset(&mut self) {
        self.nodes.clear();
        if self.with_clear_code {
            self.nodes.resize((1 << self.code_size) + 2, Node::NoChild);
        } else {
            self.nodes.resize(1 << self.code_size, Node::NoChild);
        }
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

    fn len(&self) -> usize {
        self.nodes.len()
    }
}

/// LZW encoder.
pub struct Encoder;

impl Encoder {
    /// Encode lzw, with variable code size.
    ///
    /// # Arguments
    ///
    /// * `data` - The source data to be compressed.
    /// * `into` - The output where compressed data should be written.
    /// * `code_size` - Between 2 and 8, the initial code size to use.
    ///   Initial code size correspond to the range of expected data.
    ///   For example, let's say we are compressing an ASCII string.
    ///   An ASCII string consist of bytes with values between 0 and 127, so 128 possibilities.
    ///   A code size of 7 means that we expect 2.pow(7) == 128 possibilities. It would then provide the best compression.
    /// * `endianess` - Bit ordering when writing compressed data.
    ///
    /// # Errors
    ///
    /// This function can fail on an [std::io::Error] or for unexpected codes or code sizes.
    ///
    /// # Examples
    /// ```
    /// use salzweg::{
    ///     encoder::{Encoder, EncodingError},
    ///     Endianness,
    /// };
    ///
    /// fn main() -> Result<(), EncodingError> {
    ///     let data = [0, 0, 1, 3];
    ///     let mut output = vec![];
    ///
    ///     Encoder::encode(&data[..], &mut output, 2, Endianness::LittleEndian)?;
    ///
    ///     assert_eq!(output, [0x04, 0x32, 0x05]);
    ///     Ok(())
    /// }
    /// ```
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

    /// Encode lzw, with variable code size. Convenient wrapper that creates a [Vec<u8>] under the hood.
    ///
    /// # Arguments
    ///
    /// * `data` - The source data to be compressed.
    /// * `code_size` - Between 2 and 8, the initial code size to use.
    ///   Initial code size correspond to the range of expected data.
    ///   For example, let's say we are compressing an ASCII string.
    ///   An ASCII string consist of bytes with values between 0 and 127, so 128 possibilities.
    ///   A code size of 7 means that we expect 2.pow(7) == 128 possibilities. It would then provide the best compression.
    /// * `endianess` - Bit ordering when writing compressed data.
    ///
    /// # Errors
    ///
    /// This function can fail on an [std::io::Error], unexpected codes or code sizes.
    ///
    /// # Examples
    /// ```
    /// use salzweg::{
    ///     encoder::{Encoder, EncodingError},
    ///     Endianness,
    /// };
    ///
    /// fn main() -> Result<(), EncodingError> {
    ///     let data = [0, 0, 1, 3];
    ///
    ///     let output = Encoder::encode_to_vec(&data[..], 2, Endianness::LittleEndian)?;
    ///
    ///     assert_eq!(output, [0x04, 0x32, 0x05]);
    ///     Ok(())
    /// }
    /// ```
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
        if !(2..=8).contains(&code_size) {
            return Err(EncodingError::CodeSize(code_size));
        }

        let max_code: u8 = ((1u32 << code_size) - 1) as u8;

        let mut bit_writer = bit_writer;

        let mut write_size = code_size + 1;
        let clear_code = 1 << code_size;
        let end_of_information = (1 << code_size) + 1;

        let mut tree = Tree::new(code_size, true);
        tree.reset();

        bit_writer.write(clear_code, write_size)?;

        let mut bytes = data.bytes();
        let k = bytes.next();
        if k.is_none() {
            // Well, it's an empty stream! Leaving early.
            bit_writer.write(end_of_information, write_size)?;

            bit_writer.fill()?;
            bit_writer.flush()?;

            return Ok(());
        }

        let mut current_prefix = k.unwrap()? as u16;

        for k in bytes {
            let k = k?;
            if k > max_code {
                return Err(EncodingError::UnexpectedCode { code: k, code_size });
            }

            if let Some(word) = tree.find_word(current_prefix, k) {
                current_prefix = word;
            } else {
                let index_of_new_entry = tree.add(current_prefix, k);
                bit_writer.write(current_prefix, write_size)?;
                current_prefix = k as u16;

                if index_of_new_entry == 1 << write_size {
                    write_size += 1;

                    if write_size > 12 {
                        bit_writer.write(clear_code, 12)?;
                        write_size = code_size + 1;
                        tree.reset();
                    }
                }
            }
        }

        bit_writer.write(current_prefix as u16, write_size)?;
        bit_writer.write(end_of_information, write_size)?;

        bit_writer.fill()?;
        bit_writer.flush()?;

        Ok(())
    }
}

pub struct FixedEncoder;

impl FixedEncoder {
    pub fn encode<R: Read, W: Write>(
        data: R,
        into: W,
        endianness: Endianness,
    ) -> Result<(), EncodingError> {
        match endianness {
            Endianness::BigEndian => FixedEncoder::inner_encode(data, BigEndianWriter::new(into)),
            Endianness::LittleEndian => {
                FixedEncoder::inner_encode(data, LittleEndianWriter::new(into))
            }
        }
    }

    pub fn encode_to_vec<R: Read>(
        data: R,
        endianness: Endianness,
    ) -> Result<Vec<u8>, EncodingError> {
        let mut output = vec![];
        FixedEncoder::encode(data, &mut output, endianness)?;
        Ok(output)
    }

    fn inner_encode<R: Read, B: BitWriter>(data: R, bit_writer: B) -> Result<(), EncodingError> {
        const WRITE_SIZE: u8 = 12;

        let mut bit_writer = bit_writer;

        let mut tree = Tree::new(8, false);
        tree.reset();

        let mut bytes = data.bytes();
        let k = bytes.next();
        if k.is_none() {
            // Well, it's an empty stream! Leaving early.
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
                if tree.len() < 4096 {
                    tree.add(current_prefix, k);
                }
                bit_writer.write(current_prefix, WRITE_SIZE)?;
                current_prefix = k as u16;
            }
        }

        bit_writer.write(current_prefix as u16, WRITE_SIZE)?;
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
    fn encode_few_bytes() {
        let data = [0, 0, 1, 3];

        let mut compressed = vec![];
        Encoder::encode(&data[..], &mut compressed, 2, Endianness::LittleEndian).unwrap();
        assert_eq!(compressed, [0x04, 0x32, 0x05,])
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

    #[test]
    fn unsupported_code_size() {
        let data = [0];
        let into = vec![];

        let result = Encoder::encode(&data[..], into, 10, Endianness::LittleEndian)
            .err()
            .unwrap();
        let expected = EncodingError::CodeSize(10);

        assert_eq!(expected.to_string(), result.to_string());
    }

    #[test]
    fn wrong_data_for_code_size() {
        let data = [0, 1, 8, 3];

        let result = Encoder::encode_to_vec(&data[..], 2, Endianness::BigEndian)
            .err()
            .unwrap();
        let expected = EncodingError::UnexpectedCode {
            code: 8,
            code_size: 2,
        };

        println!("{expected}");
        assert_eq!(expected.to_string(), result.to_string());
    }

    #[test]
    fn encode_4color_data_fix() {
        let data = [
            1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2,
            2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
        ];

        let mut compressed = vec![];
        FixedEncoder::encode(&data[..], &mut compressed, Endianness::LittleEndian).unwrap();
        println!("{compressed:#02X?}");

        let expected = [
            0x1, 0x0, 0x10, 0x0, 0x21, 0x0, 0x3, 0x31, 0x10, 0x1, 0x21, 0x10, 0x4, 0x21, 0x0, 0x6,
            0x11, 0x0, 0x8, 0x91, 0x10, 0x0, 0x1, 0x0, 0xF, 0x1, 0x0, 0x4, 0x1,
        ];
        assert_eq!(compressed, expected)
    }
}
