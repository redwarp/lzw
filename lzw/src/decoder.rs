use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    io::{Read, Write},
};

use crate::{
    io::{BigEndianReader, BitReader, LittleEndianReader},
    Endianness,
};

/// The error type for decoding operations.
#[derive(Debug)]
pub enum DecodingError {
    /// An I/O error happend when reading or writing data.
    Io(std::io::Error),
    /// Code size out of bounds. It should be between 2 and 8 included.
    CodeSize(u8),
    /// Unexpected code read in the data.
    UnexpectedCode(u16),
    /// If the dictionnary grows past size 4096, an expected clear code is missing.
    MissingClearCode,
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodingError::Io(error) => std::fmt::Display::fmt(&error, f),
            DecodingError::CodeSize(code_size) => f.write_fmt(format_args!(
                "Code size must be between 2 and 8, was {code_size}",
            )),
            DecodingError::UnexpectedCode(code) => {
                f.write_fmt(format_args!("Unexpected code while decompressing: {code}"))
            }
            DecodingError::MissingClearCode => {
                f.write_str("Dictionnary growing past 4096, expected CLEAR_CODE missing")
            }
        }
    }
}

impl std::error::Error for DecodingError {}

impl From<std::io::Error> for DecodingError {
    fn from(error: std::io::Error) -> Self {
        DecodingError::Io(error)
    }
}

/// LZW decoder.
pub struct Decoder;

impl Decoder {
    /// Decode lzw using variable code size.
    ///
    /// # Arguments
    ///
    /// * `data` - The source data to be decoded.
    /// * `into` - The output where decoded data will be written.
    /// * `code_size` - Between 2 and 8, the initial code size to use.
    ///   Initial code size correspond to the range of expected data.
    ///   For example, let's say we are compressing an ASCII string.
    ///   An ASCII string consist of bytes with values between 0 and 127, so 128 possibilities.
    ///   A code size of 7 means that we expect 2.pow(7) == 128 possibilities. It would then provide the best compression.
    /// * `endianess` - Bit ordering when reading compressed data.
    ///
    /// # Errors
    ///
    /// This function can fail on an [std::io::Error] or for unexpected codes or code sizes.
    ///
    /// # Examples
    /// ```
    /// use salzweg::{
    ///     decoder::{Decoder, DecodingError},
    ///     Endianness,
    /// };
    ///
    /// fn main() -> Result<(), DecodingError> {
    ///     let data = [0x04, 0x32, 0x05];
    ///     let mut output = vec![];
    ///
    ///     Decoder::decode(&data[..], &mut output, 2, Endianness::LittleEndian)?;
    ///
    ///     assert_eq!(output, [0, 0, 1, 3]);
    ///     Ok(())
    /// }
    /// ```
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

    /// Decode lzw using variable code size. Convenient wrapper that creates a [Vec<u8>] under the hood.
    ///
    /// # Arguments
    ///
    /// * `data` - The source data to be decoded.
    /// * `code_size` - Between 2 and 8, the initial code size to use.
    ///   Initial code size correspond to the range of expected data.
    ///   For example, let's say we are compressing an ASCII string.
    ///   An ASCII string consist of bytes with values between 0 and 127, so 128 possibilities.
    ///   A code size of 7 means that we expect 2.pow(7) == 128 possibilities. It would then provide the best compression.
    /// * `endianess` - Bit ordering when reading compressed data.
    ///
    /// # Errors
    ///
    /// This function can fail on an [std::io::Error] or for unexpected codes or code sizes.
    ///
    /// # Examples
    /// ```
    /// use salzweg::{
    ///     decoder::{Decoder, DecodingError},
    ///     Endianness,
    /// };
    ///
    /// fn main() -> Result<(), DecodingError> {
    ///     let data = [0x04, 0x32, 0x05];
    ///
    ///     let output = Decoder::decode_to_vec(&data[..], 2, Endianness::LittleEndian)?;
    ///
    ///     assert_eq!(output, [0, 0, 1, 3]);
    ///     Ok(())
    /// }
    /// ```
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
        if !(2..=8).contains(&code_size) {
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
            let mut code = bit_reader.read_one(read_size)?;

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

            match code.cmp(&next_index) {
                Ordering::Greater => {
                    return Err(DecodingError::UnexpectedCode(code));
                }
                Ordering::Equal => {
                    // New word! It correspond to the last decoded word,
                    // plus the first char of the previously decoded word.
                    decoding_stack[word_length] = decoding_stack[0];
                    // The word length is the length of the previous word, plus one.
                    word_length += 1;
                }
                Ordering::Less => {
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
                return Err(DecodingError::MissingClearCode);
            }
            previous_code = Some(initial_code);
        }

        into.flush()?;

        Ok(())
    }
}

pub struct FixedDecoder;

impl FixedDecoder {
    pub fn decode<R: Read, W: Write>(
        data: R,
        into: W,
        endianness: Endianness,
    ) -> Result<(), DecodingError> {
        match endianness {
            Endianness::BigEndian => FixedDecoder::inner_decode(BigEndianReader::new(data), into),
            Endianness::LittleEndian => {
                FixedDecoder::inner_decode(LittleEndianReader::new(data), into)
            }
        }
    }

    fn inner_decode<B: BitReader, W: Write>(bit_reader: B, into: W) -> Result<(), DecodingError> {
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
        const READ_SIZE: u8 = 12;

        let mut prefix: [u16; TABLE_MAX_SIZE] = [0; TABLE_MAX_SIZE];
        let mut suffix: [u8; TABLE_MAX_SIZE] = [0; TABLE_MAX_SIZE];
        let mut length: [usize; TABLE_MAX_SIZE] = [0; TABLE_MAX_SIZE];
        // We will use this stack to decode each string.
        let mut decoding_stack: [u8; STACK_MAX_SIZE] = [0; STACK_MAX_SIZE];
        // We prefill our dictionnary with all the known values;
        for code in 0..256 {
            suffix[code as usize] = code as u8;
            length[code as usize] = 1;
        }

        let mut next_index = 256;
        let mut previous_code: Option<u16> = None;
        let mut bit_reader = bit_reader;
        let mut word_length = 0;

        for code in bit_reader.iter(READ_SIZE) {
            let mut code = code?;

            if previous_code == None {
                into.write_all(&[suffix[code as usize]])?;
                previous_code = Some(code);
                decoding_stack[0] = code as u8;
                word_length = 1;
                continue;
            }

            let initial_code = code;

            match code.cmp(&next_index) {
                Ordering::Greater => {
                    return Err(DecodingError::UnexpectedCode(code));
                }
                Ordering::Equal => {
                    // New word! It correspond to the last decoded word,
                    // plus the first char of the previously decoded word.
                    decoding_stack[word_length] = decoding_stack[0];
                    // The word length is the length of the previous word, plus one.
                    word_length += 1;
                }
                Ordering::Less => {
                    word_length = length[code as usize];
                    let mut stack_top = word_length;

                    // We assemble the string char by char.
                    while code >= 256 {
                        stack_top -= 1;
                        decoding_stack[stack_top] = suffix[code as usize];
                        code = prefix[code as usize]
                    }

                    decoding_stack[0] = code as u8;
                }
            }

            into.write_all(&decoding_stack[0..word_length])?;

            if next_index < TABLE_MAX_SIZE as u16 {
                prefix[next_index as usize] = previous_code.unwrap();
                suffix[next_index as usize] = decoding_stack[0];
                length[next_index as usize] = length[previous_code.unwrap() as usize] + 1;
                next_index += 1;
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

        let result = Decoder::decode(&data[..], into, 10, Endianness::LittleEndian)
            .err()
            .unwrap();
        let expected = DecodingError::CodeSize(10);

        assert_eq!(expected.to_string(), result.to_string());
    }

    #[test]
    fn decode_4color_data_fix() {
        let data = [
            0x1, 0x0, 0x10, 0x0, 0x21, 0x0, 0x3, 0x31, 0x10, 0x1, 0x21, 0x10, 0x4, 0x21, 0x0, 0x6,
            0x11, 0x0, 0x8, 0x91, 0x10, 0x0, 0x1, 0x0, 0xF, 0x1, 0x0, 0x4, 0x1,
        ];

        let mut decoded = vec![];
        FixedDecoder::decode(&data[..], &mut decoded, Endianness::LittleEndian).unwrap();

        assert_eq!(
            decoded,
            [
                1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2,
                2, 2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
            ]
        );
    }
}
