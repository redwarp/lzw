use std::io::Read;

use crate::Endianness;

pub struct BitReader<R>
where
    R: Read,
{
    endianness: Endianness,
    read: R,
    cursor: u8,
    byte_buffer: u32,
    read_buffer: [u8; 1],
}

impl<R> BitReader<R>
where
    R: Read,
{
    fn new(endianness: Endianness, read: R) -> Self {
        let cursor = 0;
        let byte_buffer = 0;
        let read_buffer = [0; 1];
        Self {
            endianness,
            read,
            cursor,
            byte_buffer,
            read_buffer,
        }
    }

    fn read(&mut self, amount: u8) -> Result<u16, std::io::Error> {
        match self.endianness {
            Endianness::BigEndian => self.read_big_endian(amount),
            Endianness::LittleEndian => self.read_little_endian(amount),
        }
    }

    fn read_little_endian(&mut self, amount: u8) -> Result<u16, std::io::Error> {
        while self.cursor < amount {
            self.read.read_exact(&mut self.read_buffer[..])?;
            self.byte_buffer |= (self.read_buffer[0] as u32) << self.cursor;
            self.cursor += 8;
        }

        let mask = (1 << amount) - 1;
        let data = (self.byte_buffer & mask) as u16;
        self.byte_buffer >>= amount;
        self.cursor -= amount;
        Ok(data)
    }

    fn read_big_endian(&mut self, amount: u8) -> Result<u16, std::io::Error> {
        while self.cursor < amount {
            self.read.read_exact(&mut self.read_buffer[..])?;
            let shift = 24 - self.cursor;
            self.byte_buffer |= (self.read_buffer[0] as u32) << shift;
            self.cursor += 8;
        }

        let mask = (1 << amount) - 1;
        let shift = 32 - amount;
        let data = ((self.byte_buffer >> shift) & mask) as u16;
        self.byte_buffer <<= amount;
        self.cursor -= amount;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use crate::Endianness::{BigEndian, LittleEndian};

    use super::BitReader;

    #[test]
    fn read_1_little_endian() {
        let input = [0x01];

        let mut reader = BitReader::new(LittleEndian, &input[..]);

        assert_eq!(1, reader.read(1).unwrap());
    }

    #[test]
    fn read_colors_little_endian() {
        let input = [0x8C, 0x2D];

        let mut reader = BitReader::new(LittleEndian, &input[..]);
        let mut output = vec![];

        output.push(reader.read(3).unwrap());
        output.push(reader.read(3).unwrap());
        output.push(reader.read(3).unwrap());
        output.push(reader.read(3).unwrap());
        output.push(reader.read(4).unwrap());

        assert_eq!(output, [4, 1, 6, 6, 2]);
    }

    #[test]
    fn read_12_bits_little_endian() {
        let input = [0xff, 0x0f];

        let mut reader = BitReader::new(LittleEndian, &input[..]);

        assert_eq!(reader.read(12).unwrap(), 0xfff);
    }

    #[test]
    fn read_0xfffa_little_endian() {
        let input = [0xfa, 0xff];

        let mut reader = BitReader::new(LittleEndian, &input[..]);

        assert_eq!(reader.read(16).unwrap(), 0xfffa);
    }

    #[test]
    fn read_1_big_endian() {
        let input = [0x80];

        let mut reader = BitReader::new(BigEndian, &input[..]);

        assert_eq!(1, reader.read(1).unwrap());
    }

    #[test]
    fn read_colors_big_endian() {
        let input = [0x87, 0x62];

        let mut reader = BitReader::new(BigEndian, &input[..]);
        let mut output = vec![];

        output.push(reader.read(3).unwrap());
        output.push(reader.read(3).unwrap());
        output.push(reader.read(3).unwrap());
        output.push(reader.read(3).unwrap());
        output.push(reader.read(4).unwrap());

        assert_eq!(output, [4, 1, 6, 6, 2]);
    }

    #[test]
    fn read_12_bits_big_endian() {
        let input = [0xff, 0xf0];

        let mut reader = BitReader::new(BigEndian, &input[..]);

        assert_eq!(reader.read(12).unwrap(), 0xfff);
    }

    #[test]
    fn read_0xfffa_big_endian() {
        let input = [0xff, 0xfa];

        let mut reader = BitReader::new(BigEndian, &input[..]);

        assert_eq!(reader.read(16).unwrap(), 0xfffa);
    }
}
