use std::{
    fmt::{Debug, Display},
    io::{Read, Write},
};

use crate::{io::BitReader, Endianness};

#[derive(Debug)]
pub enum DecodingError {
    Io(std::io::Error),
    Lzw(&'static str),
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodingError::Io(error) => std::fmt::Display::fmt(&error, f),
            DecodingError::Lzw(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for DecodingError {}

impl From<std::io::Error> for DecodingError {
    fn from(error: std::io::Error) -> Self {
        DecodingError::Io(error)
    }
}

// Idea: Have one big vec of [u8]. Store the strings there, and keep a vec of [start, length] for each word.
#[derive(Debug, Clone)]
struct Word {
    start: usize,
    end: usize,
}

struct Tree {
    code_size: u8,
    strings: Vec<u8>,
    words: Vec<Word>,
}

impl Tree {
    fn new(code_size: u8) -> Self {
        let strings = (0..1 << code_size).collect();
        let mut words = Vec::with_capacity(1 << 12);
        words.extend((0..1 << code_size).map(|i| Word {
            start: i,
            end: i + 1,
        }));
        words.push(Word { start: 0, end: 0 });
        words.push(Word { start: 0, end: 0 });

        Self {
            code_size,
            strings,
            words,
        }
    }

    fn clear(&mut self) {
        self.strings.resize(1 << self.code_size, 0);
        self.words
            .resize((1 << self.code_size) + 2, Word { start: 0, end: 0 });
    }

    fn find_word(&self, code: u16) -> Option<&[u8]> {
        if let Some(word) = self.words.get(code as usize) {
            Some(&self.strings[word.start..word.end])
        } else {
            None
        }
    }

    fn add(&mut self, prefix: u16, k: u8) -> u16 {
        let prefix_word = &self.words[prefix as usize];
        let new_word = if prefix_word.end == self.strings.len() {
            // Adding to last inserted word.
            self.strings.push(k);
            Word {
                start: prefix_word.start,
                end: prefix_word.end + 1,
            }
        } else {
            let start = self.strings.len();
            self.strings
                .extend_from_within(prefix_word.start..prefix_word.end);
            self.strings.push(k);
            let end = self.strings.len();
            Word { start, end }
        };

        let new_index = self.words.len();
        self.words.push(new_word);
        new_index as u16
    }
}

impl Debug for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, word) in self.words.iter().enumerate() {
            f.write_fmt(format_args!(
                "{i} - {:?}\n",
                &self.strings[word.start..word.end]
            ))?;
        }
        Ok(())
    }
}

const BUFFER_SIZE: usize = 8192;

struct Buffer<W: Write> {
    buffer: [u8; BUFFER_SIZE],
    into: W,
    count: usize,
}

impl<W: Write> Buffer<W> {
    #[inline]
    fn new(into: W) -> Buffer<W> {
        Self {
            buffer: [0; BUFFER_SIZE],
            into,
            count: 0,
        }
    }

    fn write_all(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        let remaining = self.remaining();
        if remaining >= data.len() {
            self.buffer[self.count..self.count + data.len()].copy_from_slice(data);
            self.count += data.len();
        } else {
            self.buffer[self.count..].copy_from_slice(&data[..remaining]);
            self.into.write_all(&self.buffer)?;
            self.buffer[..data.len() - remaining].copy_from_slice(&data[remaining..]);
            self.count = data.len() - remaining
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        if self.count == 0 {
            return Ok(());
        }

        self.into.write_all(&self.buffer[..self.count])?;
        self.count = 0;
        Ok(())
    }

    fn remaining(&self) -> usize {
        BUFFER_SIZE - self.count
    }
}

pub struct Decoder {
    code_size: u8,
    endianness: Endianness,
}

impl Decoder {
    pub fn new(code_size: u8, endianness: Endianness) -> Self {
        Self {
            code_size,
            endianness,
        }
    }

    pub fn decode<R: Read, W: Write>(&mut self, data: R, into: W) -> Result<(), DecodingError> {
        let mut bit_reader = BitReader::new(self.endianness, data);
        let mut read_size = self.code_size + 1;
        let mut into = into;

        let clear_code = 1 << self.code_size;
        let end_of_information = (1 << self.code_size) + 1;
        let mut tree = Tree::new(self.code_size);

        let expected_clear_code = bit_reader.read(read_size)?;
        if expected_clear_code != clear_code {
            return Err(DecodingError::Lzw("Missing clear code at stream start"));
        }
        tree.clear();

        let mut current_prefix = bit_reader.read(read_size)?;
        if current_prefix == end_of_information {
            return Ok(());
        }
        into.write_all(&[current_prefix as u8])?;

        'read_loop: loop {
            let k = bit_reader.read(read_size)?;

            if k == clear_code {
                tree.clear();
                read_size = self.code_size + 1;
                current_prefix = bit_reader.read(read_size)?;
                if current_prefix == end_of_information {
                    return Ok(());
                }
                into.write_all(&[current_prefix as u8])?;
            } else if k == end_of_information {
                break 'read_loop;
            } else {
                let extra_char = if let Some(string) = tree.find_word(k) {
                    into.write_all(string)?;
                    string[0]
                } else {
                    let word = tree.find_word(current_prefix).expect("Should be set");

                    let extra_char = word[0];
                    into.write_all(word)?;
                    into.write_all(&[extra_char])?;
                    extra_char
                };
                let index_of_new_entry = tree.add(current_prefix, extra_char);

                if index_of_new_entry == (1 << read_size) - 1 {
                    read_size += 1;
                }

                current_prefix = k;
            }
        }

        into.flush()?;

        Ok(())
    }

    pub fn decode2<R: Read, W: Write>(&mut self, data: R, into: W) -> Result<(), DecodingError> {
        const MAX_STACK_SIZE: usize = 4096;
        let mut prefix: [u16; MAX_STACK_SIZE] = [0; MAX_STACK_SIZE];
        let mut suffix: [u8; MAX_STACK_SIZE] = [0; MAX_STACK_SIZE];
        let mut prefix_stack: [u8; MAX_STACK_SIZE + 1] = [0; MAX_STACK_SIZE + 1];
        for code in 0..1 << self.code_size {
            suffix[code as usize] = code as u8;
        }

        let mut bit_reader = BitReader::new(self.endianness, data);
        let mut buffer = Buffer::new(into);
        let mut read_size = self.code_size + 1;

        let clear_code = 1 << self.code_size;
        let end_of_information = clear_code + 1;

        let mut mask = (1 << read_size) - 1;
        let mut next_index = clear_code + 2;
        let mut stack_top = 0;
        let mut first = 0;

        let mut previous_code: Option<u16> = None;

        'read_loop: loop {
            let mut code = bit_reader.read(read_size)?;

            if code == clear_code {
                read_size = self.code_size + 1;
                mask = (1 << read_size) - 1;
                next_index = clear_code + 2;
                previous_code = None;
                continue;
            } else if code == end_of_information {
                break 'read_loop;
            } else if previous_code == None {
                buffer.write_all(&[suffix[code as usize]])?;
                previous_code = Some(code);
                first = code as u8;
                continue;
            }

            let initial_code = code;

            if code >= next_index {
                prefix_stack[stack_top] = first;
                stack_top += 1;
                code = previous_code.unwrap();
            }

            while code >= clear_code {
                prefix_stack[stack_top] = suffix[code as usize];
                stack_top += 1;
                code = prefix[code as usize]
            }

            first = suffix[code as usize];
            buffer.write_all(&[first])?;

            while stack_top > 0 {
                stack_top -= 1;
                buffer.write_all(&[prefix_stack[stack_top]])?;
            }

            if next_index < MAX_STACK_SIZE as u16 {
                prefix[next_index as usize] = previous_code.unwrap();
                suffix[next_index as usize] = first;
                next_index += 1;
                if next_index & mask == 0 && next_index < MAX_STACK_SIZE as u16 {
                    read_size += 1;
                    mask += next_index;
                }
            } else {
                return Err(DecodingError::Lzw(
                    "Dictionnary growing past 4096, expected CLEAR_CODE missing",
                ));
            }
            previous_code = Some(initial_code);
        }

        buffer.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_4color_data() {
        let data = [
            0x8C, 0x2D, 0x99, 0x87, 0x2A, 0x1C, 0xDC, 0x33, 0xA0, 0x2, 0x55, 0x0,
        ];

        let mut decoder = Decoder::new(2, Endianness::LittleEndian);

        let mut decoded = vec![];
        decoder.decode2(&data[..], &mut decoded).unwrap();

        assert_eq!(
            decoded,
            [
                1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2,
                2, 2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
            ]
        );
    }

    #[test]
    fn decode_multiple_time_same_decoder() {
        let data = [
            0x8C, 0x2D, 0x99, 0x87, 0x2A, 0x1C, 0xDC, 0x33, 0xA0, 0x2, 0x55, 0x0,
        ];

        let mut decoder = Decoder::new(2, Endianness::LittleEndian);

        let mut decoded1 = vec![];
        let mut decoded2 = vec![];
        decoder.decode(&data[..], &mut decoded1).unwrap();
        decoder.decode2(&data[..], &mut decoded2).unwrap();

        assert_eq!(decoded1, decoded2);
    }

    #[test]
    fn decode_lorem_ipsum() {
        let data = include_bytes!("../../test-assets/lorem_ipsum_encoded.bin");
        let expected = include_bytes!("../../test-assets/lorem_ipsum.txt");

        let mut decoder = Decoder::new(7, Endianness::LittleEndian);
        let mut decoded = vec![];
        decoder.decode2(&data[..], &mut decoded).unwrap();

        assert_eq!(decoded, expected);
    }

    #[test]
    fn buffer_write_more_than_can_chew() {
        let mut into = vec![];
        let mut buffer = Buffer::new(&mut into);

        let data = vec![10; 10000];
        buffer.write_all(&data).unwrap();
        buffer.flush().unwrap();

        assert_eq!(into.len(), 10000);
    }

    #[test]
    fn buffer_overflow_with_two_slices() {
        let mut into = vec![];
        let mut buffer = Buffer::new(&mut into);

        let data = vec![10; 10000];
        buffer.write_all(&data[..5000]).unwrap();
        buffer.write_all(&data[5000..]).unwrap();
        buffer.flush().unwrap();

        assert_eq!(into.len(), 10000);
    }

    #[test]
    fn buffer_write_one_by_one() {
        let mut into = vec![];
        let mut buffer = Buffer::new(&mut into);

        let data = vec![0; 10000];

        for i in 0..10000 {
            buffer.write_all(&data[i..i + 1]).unwrap();
        }

        buffer.flush().unwrap();

        assert_eq!(into.len(), 10000);
    }

    #[test]
    fn buffer_flush() {
        let mut into = vec![];
        let data = vec![0; 50];

        let mut buffer = Buffer::new(&mut into);
        buffer.write_all(&data).unwrap();
        assert_eq!(into.len(), 0);

        let mut buffer = Buffer::new(&mut into);
        buffer.write_all(&data).unwrap();
        buffer.flush().unwrap();
        assert_eq!(into.len(), 50);
    }

    #[test]
    fn buffer_flush_multiple() {
        let mut into = vec![];
        let data = vec![0; 50];

        let mut buffer = Buffer::new(&mut into);
        buffer.write_all(&data).unwrap();
        buffer.flush().unwrap();
        buffer.write_all(&data).unwrap();
        buffer.flush().unwrap();
        assert_eq!(into.len(), 100);
    }
}
