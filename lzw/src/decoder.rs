use std::{
    array,
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

const BUFFER_SIZE: usize = 8096;

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

    fn write(&mut self, data: u8) -> Result<(), std::io::Error> {
        if self.remaining() != 0 {
            self.buffer[self.count] = data;
            self.count += 1;
        } else {
            self.into.write_all(&self.buffer)?;
            self.buffer[0] = data;
            self.count = 1;
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

    #[inline]
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
        const TABLE_MAX_SIZE: usize = 4096;
        // The stack should be as big as the longest word that the dictionnary can have.
        // The longuest word would be reached if by bad luck, each entry of the dictionnary is made of the
        // previous entry, increasing in size each time. This size would be the biggest for the minimum code size of 2,
        // as there would be more "free entry" in the table not corresponding to a single digit.
        // In effect, stack max size = 4096 - 2^2 - 2 entries for clear and EOF + 1.
        const STACK_MAX_SIZE: usize = 4091;
        // In effect, our prefix and suffix is our decoding table, as each word can be expressed by a previous
        // code (prefix), and the extra letter (suffix).
        let mut prefix: [u16; TABLE_MAX_SIZE] = [0; TABLE_MAX_SIZE];
        let mut suffix: [u8; TABLE_MAX_SIZE] = [0; TABLE_MAX_SIZE];
        // We will use this stack to decode each string.
        let mut decoding_stack: [u8; STACK_MAX_SIZE] = [0; STACK_MAX_SIZE];
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
        let mut first_char = 0;

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
                buffer.write_all(array::from_ref(&suffix[code as usize]))?;
                previous_code = Some(code);
                first_char = code as u8;
                continue;
            }

            let initial_code = code;

            if code >= next_index {
                // New word!
                decoding_stack[stack_top] = first_char;
                stack_top += 1;
                code = previous_code.unwrap();
            }

            while code >= clear_code {
                decoding_stack[stack_top] = suffix[code as usize];
                stack_top += 1;
                code = prefix[code as usize]
            }

            first_char = code as u8;
            buffer.write(first_char)?;

            while stack_top > 0 {
                stack_top -= 1;
                buffer.write_all(array::from_ref(&decoding_stack[stack_top]))?;
            }

            if next_index < TABLE_MAX_SIZE as u16 {
                prefix[next_index as usize] = previous_code.unwrap();
                suffix[next_index as usize] = first_char;
                next_index += 1;
                if next_index & mask == 0 && next_index < TABLE_MAX_SIZE as u16 {
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
        decoder.decode(&data[..], &mut decoded).unwrap();

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
        decoder.decode(&data[..], &mut decoded2).unwrap();

        assert_eq!(decoded1, decoded2);
    }

    #[test]
    fn decode_lorem_ipsum() {
        let data = include_bytes!("../../test-assets/lorem_ipsum_encoded.bin");
        let expected = include_bytes!("../../test-assets/lorem_ipsum.txt");

        let mut decoder = Decoder::new(7, Endianness::LittleEndian);
        let mut decoded = vec![];
        decoder.decode(&data[..], &mut decoded).unwrap();

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
