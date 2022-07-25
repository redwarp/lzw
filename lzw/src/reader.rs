use core::slice;
use std::io::Read;

use crate::encoder::Tree;

pub struct FixedEncoder<R> {
    inner: R,
    tree: Tree,
    packer: LittleEndianPacker,
    current_prefix: Option<u16>,
    wrote_last: bool,
}

impl<R: Read> FixedEncoder<R> {
    fn new(inner: R) -> Self {
        let mut tree = Tree::new(8, false);
        tree.reset();
        let packer = LittleEndianPacker::new();
        Self {
            inner,
            tree,
            packer,
            current_prefix: None,
            wrote_last: false,
        }
    }
}

impl<R: Read> Read for FixedEncoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        const WRITE_SIZE: u8 = 12;
        const MAX_TABLE_SIZE: usize = 4096;

        let mut index = 0;
        while index < buf.len() {
            if let Some(byte) = self.packer.packed() {
                buf[index] = byte;
                index += 1;
                continue;
            }

            let mut k = 0;
            match self.inner.read(slice::from_mut(&mut k))? {
                0 => {
                    // Inner read reached the end, so we wrap things up.
                    if !self.wrote_last {
                        if let Some(prefix) = self.current_prefix {
                            self.packer.pack(prefix, WRITE_SIZE);
                        }
                        self.wrote_last = true;
                        continue;
                    }
                    if let Some(byte) = self.packer.last() {
                        buf[index] = byte;
                        index += 1;
                    }
                    return Ok(index);
                }
                _ => {
                    if self.current_prefix.is_none() {
                        self.current_prefix = Some(k as u16);
                        continue;
                    }

                    let current_prefix = self.current_prefix.unwrap();

                    if let Some(word) = self.tree.find_word(current_prefix, k) {
                        self.current_prefix = Some(word);
                    } else {
                        if self.tree.len() < MAX_TABLE_SIZE {
                            self.tree.add(current_prefix, k);
                        }
                        self.packer.pack(current_prefix, WRITE_SIZE);
                        self.current_prefix = Some(k as u16);
                    }
                }
            }
        }
        Ok(index)
    }
}

struct LittleEndianPacker {
    cursor: u8,
    byte_buffer: u32,
}

impl LittleEndianPacker {
    pub fn new() -> Self {
        let byte_buffer = 0;
        let cursor = 0;
        Self {
            byte_buffer,
            cursor,
        }
    }

    fn pack(&mut self, data: u16, amount: u8) {
        let mask = (1 << amount) - 1;
        self.byte_buffer |= (data as u32 & mask) << self.cursor;
        self.cursor += amount;
    }

    fn packed(&mut self) -> Option<u8> {
        if self.cursor >= 8 {
            let byte = self.byte_buffer as u8;
            self.byte_buffer >>= 8;
            self.cursor -= 8;

            Some(byte)
        } else {
            None
        }
    }

    fn last(&mut self) -> Option<u8> {
        if self.cursor > 0 {
            let byte = self.byte_buffer as u8;
            self.byte_buffer = 0;
            self.cursor = 0;
            Some(byte)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_encoder() -> Result<(), std::io::Error> {
        let data = [0, 7, 1, 3, 5, 1];

        let mut encoder = FixedEncoder::new(data.as_slice());

        let mut compressed = vec![];

        encoder.read_to_end(&mut compressed)?;

        assert_eq!(compressed, [0, 112, 0, 1, 48, 0, 5, 16, 0]);

        Ok(())
    }

    #[test]
    fn fixed_encoder_read_2_by_2() -> Result<(), std::io::Error> {
        let data = [0, 7, 1, 3, 5, 1];

        let mut encoder = FixedEncoder::new(data.as_slice());

        let mut compressed = [0, 0, 0, 0];

        encoder.read(&mut compressed)?;
        assert_eq!(compressed, [0, 112, 0, 1]);
        encoder.read(&mut compressed)?;
        assert_eq!(compressed, [48, 0, 5, 16]);
        let read_count = encoder.read(&mut compressed)?;
        assert_eq!(read_count, 1);
        assert_eq!(compressed[0], 0);

        Ok(())
    }

    #[test]
    fn compare_fixed_encoders() -> Result<(), std::io::Error> {
        let data = include_bytes!("../../test-assets/lorem_ipsum.txt");

        let mut encoder = FixedEncoder::new(data.as_slice());

        let mut compressed = vec![];
        encoder.read_to_end(&mut compressed)?;

        let compressed2 = crate::encoder::FixedEncoder::encode_to_vec(
            data.as_slice(),
            crate::Endianness::LittleEndian,
        )
        .unwrap();

        assert_eq!(compressed, compressed2);

        Ok(())
    }
}
