// #![allow(dead_code)]
// use std::{
//     fmt::{Debug, Display},
//     io::{Read, Write},
// };

// use fast_lzw::{io::BitReader, Endianness};

// #[derive(Debug)]
// pub enum DecodingError {
//     Io(std::io::Error),
//     Lzw(&'static str),
// }

// impl Display for DecodingError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             DecodingError::Io(error) => std::fmt::Display::fmt(&error, f),
//             DecodingError::Lzw(message) => f.write_str(message),
//         }
//     }
// }

// impl std::error::Error for DecodingError {}

// impl From<std::io::Error> for DecodingError {
//     fn from(error: std::io::Error) -> Self {
//         DecodingError::Io(error)
//     }
// }

// // Idea: Have one big vec of [u8]. Store the strings there, and keep a vec of [start, length] for each word.
// #[derive(Debug, Clone)]
// struct Word {
//     start: usize,
//     end: usize,
// }

// struct Tree {
//     code_size: u8,
//     strings: Vec<u8>,
//     words: Vec<Word>,
// }

// impl Tree {
//     fn new(code_size: u8) -> Self {
//         let strings = (0..1 << code_size).collect();
//         let mut words = Vec::with_capacity(1 << 12);
//         words.extend((0..1 << code_size).map(|i| Word {
//             start: i,
//             end: i + 1,
//         }));
//         words.push(Word { start: 0, end: 0 });
//         words.push(Word { start: 0, end: 0 });

//         Self {
//             code_size,
//             strings,
//             words,
//         }
//     }

//     fn clear(&mut self) {
//         self.strings.resize(1 << self.code_size, 0);
//         self.words
//             .resize((1 << self.code_size) + 2, Word { start: 0, end: 0 });
//     }

//     fn find_word(&self, code: u16) -> Option<&[u8]> {
//         if let Some(word) = self.words.get(code as usize) {
//             Some(&self.strings[word.start..word.end])
//         } else {
//             None
//         }
//     }

//     fn add(&mut self, prefix: u16, k: u8) -> u16 {
//         let prefix_word = &self.words[prefix as usize];
//         let new_word = if prefix_word.end == self.strings.len() {
//             // Adding to last inserted word.
//             self.strings.push(k);
//             Word {
//                 start: prefix_word.start,
//                 end: prefix_word.end + 1,
//             }
//         } else {
//             let start = self.strings.len();
//             self.strings
//                 .extend_from_within(prefix_word.start..prefix_word.end);
//             self.strings.push(k);
//             let end = self.strings.len();
//             Word { start, end }
//         };

//         let new_index = self.words.len();
//         self.words.push(new_word);
//         new_index as u16
//     }
// }

// impl Debug for Tree {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for (i, word) in self.words.iter().enumerate() {
//             f.write_fmt(format_args!(
//                 "{i} - {:?}\n",
//                 &self.strings[word.start..word.end]
//             ))?;
//         }
//         Ok(())
//     }
// }

// pub struct Decoder {
//     code_size: u8,
//     endianness: Endianness,
// }

// impl Decoder {
//     pub fn new(code_size: u8, endianness: Endianness) -> Self {
//         Self {
//             code_size,
//             endianness,
//         }
//     }

//     pub fn decode<R: Read, W: Write>(&mut self, data: R, into: W) -> Result<(), DecodingError> {
//         const MAX_READ_SIZE: u8 = 12;

//         let mut bit_reader = BitReader::new(self.endianness, data);
//         let mut read_size = self.code_size + 1;
//         let mut into = into;

//         let clear_code = 1 << self.code_size;
//         let end_of_information = (1 << self.code_size) + 1;
//         let mut tree = Tree::new(self.code_size);

//         let expected_clear_code = bit_reader.read(read_size)?;
//         if expected_clear_code != clear_code {
//             return Err(DecodingError::Lzw("Missing clear code at stream start"));
//         }
//         tree.clear();

//         let mut current_prefix: Option<u16> = None;

//         'read_loop: loop {
//             let k = bit_reader.read(read_size)?;

//             if k == clear_code {
//                 tree.clear();
//                 read_size = self.code_size + 1;
//                 current_prefix = None;
//                 continue 'read_loop;
//             } else if k == end_of_information {
//                 break 'read_loop;
//             } else if current_prefix == None {
//                 into.write_all(&[k as u8])?;
//                 current_prefix = Some(k);
//                 continue 'read_loop;
//             }

//             let prefix = current_prefix.unwrap();
//             let extra_char = if let Some(string) = tree.find_word(k) {
//                 into.write_all(string)?;
//                 string[0]
//             } else {
//                 let word = tree.find_word(prefix).expect("Should be set");

//                 let extra_char = word[0];
//                 into.write_all(word)?;
//                 into.write_all(&[extra_char])?;
//                 extra_char
//             };
//             let index_of_new_entry = tree.add(prefix, extra_char);

//             if index_of_new_entry == (1 << read_size) - 1 && read_size < MAX_READ_SIZE {
//                 read_size += 1;
//             }

//             current_prefix = Some(k);
//         }

//         into.flush()?;

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn decode_4color_data() {
//         let data = [
//             0x8C, 0x2D, 0x99, 0x87, 0x2A, 0x1C, 0xDC, 0x33, 0xA0, 0x2, 0x55, 0x0,
//         ];

//         let mut decoder = Decoder::new(2, Endianness::LittleEndian);

//         let mut decoded = vec![];
//         decoder.decode(&data[..], &mut decoded).unwrap();

//         assert_eq!(
//             decoded,
//             [
//                 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2,
//                 2, 2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
//             ]
//         );
//     }

//     #[test]
//     fn decode_multiple_time_same_decoder() {
//         let data = [
//             0x8C, 0x2D, 0x99, 0x87, 0x2A, 0x1C, 0xDC, 0x33, 0xA0, 0x2, 0x55, 0x0,
//         ];

//         let mut decoder = Decoder::new(2, Endianness::LittleEndian);

//         let mut decoded1 = vec![];
//         let mut decoded2 = vec![];
//         decoder.decode(&data[..], &mut decoded1).unwrap();
//         decoder.decode(&data[..], &mut decoded2).unwrap();

//         assert_eq!(decoded1, decoded2);
//     }

//     #[test]
//     fn decode_lorem_ipsum() {
//         let data = include_bytes!("../../test-assets/lorem_ipsum_encoded.bin");
//         let expected = include_bytes!("../../test-assets/lorem_ipsum.txt");

//         let mut decoder = Decoder::new(7, Endianness::LittleEndian);
//         let mut decoded = vec![];
//         decoder.decode(&data[..], &mut decoded).unwrap();

//         assert_eq!(decoded, expected);
//     }
// }
