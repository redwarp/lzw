use std::{
    fmt::{Debug, Display},
    io::{Read, Write},
};

use crate::{
    io::{BigEndianReader, BitReader, LittleEndianReader},
    Endianness,
};

#[derive(Debug)]
pub enum DecodingError {
    Io(std::io::Error),
    Lzw(&'static str),
    CodeSize(u8),
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodingError::Io(error) => std::fmt::Display::fmt(&error, f),
            DecodingError::Lzw(message) => f.write_str(message),
            DecodingError::CodeSize(code_size) => f.write_fmt(format_args!(
                "Code size must be between 2 and 8, was {code_size}",
            )),
        }
    }
}

impl std::error::Error for DecodingError {}

impl From<std::io::Error> for DecodingError {
    fn from(error: std::io::Error) -> Self {
        DecodingError::Io(error)
    }
}

impl PartialEq for DecodingError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(l0), Self::Io(r0)) => l0.kind() == r0.kind(),
            (Self::Lzw(l0), Self::Lzw(r0)) => l0 == r0,
            (Self::CodeSize(l0), Self::CodeSize(r0)) => l0 == r0,
            _ => false,
        }
    }
}
pub struct Decoder {}

impl Decoder {
    pub fn decode<R: Read, W: Write>(
        data: R,
        into: W,
        code_size: u8,
        endianness: Endianness,
    ) -> Result<(), DecodingError> {
        match endianness {
            Endianness::BigEndian => {
                Decoder::inner_decode(BigEndianReader::new(data), into, code_size)
            }
            Endianness::LittleEndian => {
                Decoder::inner_decode(LittleEndianReader::new(data), into, code_size)
            }
        }
    }

    pub fn decode_to_vec<R: Read>(
        data: R,
        code_size: u8,
        endianness: Endianness,
    ) -> Result<Vec<u8>, DecodingError> {
        let mut output = vec![];
        Decoder::decode(data, &mut output, code_size, endianness)?;
        Ok(output)
    }

    fn inner_decode<B: BitReader, W: Write>(
        bit_reader: B,
        into: W,
        code_size: u8,
    ) -> Result<(), DecodingError> {
        if code_size < 2 || code_size > 8 {
            return Err(DecodingError::CodeSize(code_size));
        }
        let mut into = into;

        const TABLE_MAX_SIZE: usize = 4096;
        // The stack should be as big as the longest word that the dictionnary can have.
        // The longuest word would be reached if by bad luck, each entry of the dictionnary is made of the
        // previous entry, increasing in size each time. This size would be the biggest for the minimum code size of 2,
        // as there would be more "free entry" in the table not corresponding to a single digit.
        // In effect, stack max size = 4096 - 2^2 - 2 entries for clear and EOF + 1.
        const STACK_MAX_SIZE: usize = 4091;
        // In effect, our prefix and suffix is our decoding table, as each word can be expressed by a previous
        // code (prefix), and the extra letter (suffix). We store the word length as well, it's useful
        // to recreate the word stack.
        let mut prefix: [u16; TABLE_MAX_SIZE] = [0; TABLE_MAX_SIZE];
        let mut suffix: [u8; TABLE_MAX_SIZE] = [0; TABLE_MAX_SIZE];
        let mut length: [usize; TABLE_MAX_SIZE] = [0; TABLE_MAX_SIZE];
        // We will use this stack to decode each string.
        let mut decoding_stack: [u8; STACK_MAX_SIZE] = [0; STACK_MAX_SIZE];
        // We prefill our dictionnary with all the known values;
        for code in 0..1 << code_size {
            suffix[code as usize] = code as u8;
            length[code as usize] = 1;
        }

        let mut read_size = code_size + 1;

        let clear_code = 1 << code_size;
        let end_of_information = clear_code + 1;

        let mut mask = (1 << read_size) - 1;
        let mut next_index = clear_code + 2;
        let mut previous_code: Option<u16> = None;
        let mut bit_reader = bit_reader;
        let mut word_length = 0;

        loop {
            let mut code = bit_reader.read(read_size)?;

            if code == clear_code {
                read_size = code_size + 1;
                mask = (1 << read_size) - 1;
                next_index = clear_code + 2;
                previous_code = None;
                continue;
            } else if code == end_of_information {
                break;
            } else if previous_code == None {
                into.write_all(&[suffix[code as usize]])?;
                previous_code = Some(code);
                decoding_stack[0] = code as u8;
                word_length = 1;
                continue;
            }

            let initial_code = code;

            if code > next_index {
                return Err(DecodingError::Lzw("Unexpected code while decoding."));
            } else if code == next_index {
                // New word! It correspond to the last decoded word,
                // plus the first char of the previously decoded word.
                decoding_stack[word_length] = decoding_stack[0];
                // The word length is the length of the previous word, plus one.
                word_length = word_length + 1;
            } else {
                word_length = length[code as usize];
                let mut stack_top = word_length;

                // We assemble the string char by char.
                while code >= clear_code {
                    stack_top -= 1;
                    decoding_stack[stack_top] = suffix[code as usize];
                    code = prefix[code as usize]
                }

                decoding_stack[0] = code as u8;
            }

            into.write_all(&decoding_stack[0..word_length])?;

            if next_index < TABLE_MAX_SIZE as u16 {
                prefix[next_index as usize] = previous_code.unwrap();
                suffix[next_index as usize] = decoding_stack[0];
                length[next_index as usize] = length[previous_code.unwrap() as usize] + 1;
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

        into.flush()?;

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

        let mut decoded = vec![];
        Decoder::decode(&data[..], &mut decoded, 2, Endianness::LittleEndian).unwrap();

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

        let mut decoded1 = vec![];
        let mut decoded2 = vec![];
        Decoder::decode(&data[..], &mut decoded1, 2, Endianness::LittleEndian).unwrap();
        Decoder::decode(&data[..], &mut decoded2, 2, Endianness::LittleEndian).unwrap();

        assert_eq!(decoded1, decoded2);
    }

    #[test]
    fn decode_lorem_ipsum() {
        let data = include_bytes!("../../test-assets/lorem_ipsum_encoded.bin");
        let expected = include_bytes!("../../test-assets/lorem_ipsum.txt");

        let mut decoded = vec![];
        Decoder::decode(&data[..], &mut decoded, 7, Endianness::LittleEndian).unwrap();

        assert_eq!(decoded, expected);
    }

    #[test]
    fn unsupported_code_size() {
        let data = [0];
        let into = vec![];

        let result = Decoder::decode(&data[..], into, 10, Endianness::LittleEndian).err();
        let expected = Some(DecodingError::CodeSize(10));

        assert_eq!(expected, result);
    }
}
