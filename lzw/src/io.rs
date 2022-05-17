use std::io::{Read, Write};

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
    pub fn new(endianness: Endianness, read: R) -> Self {
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

    pub fn read(&mut self, amount: u8) -> Result<u16, std::io::Error> {
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

pub struct BitWriter<W>
where
    W: Write,
{
    endianness: Endianness,
    write: W,
    cursor: u8,
    byte_buffer: u32,
}

impl<W> BitWriter<W>
where
    W: Write,
{
    pub fn new(endianness: Endianness, write: W) -> Self {
        let byte_buffer = 0;
        let cursor = 0;
        Self {
            endianness,
            write,
            byte_buffer,
            cursor,
        }
    }

    pub fn write(&mut self, amount: u8, data: u16) -> Result<(), std::io::Error> {
        match self.endianness {
            Endianness::BigEndian => self.write_big_endian2(amount, data),
            Endianness::LittleEndian => self.write_little_endian(amount, data),
        }
    }

    pub fn fill(&mut self) -> Result<(), std::io::Error> {
        if self.cursor > 0 {
            match self.endianness {
                Endianness::BigEndian => self.write.write_all(&[(self.byte_buffer >> 24) as u8])?,
                Endianness::LittleEndian => self.write.write_all(&[self.byte_buffer as u8])?,
            }

            self.byte_buffer = 0;
            self.cursor = 0;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.write.flush()
    }

    fn write_little_endian(&mut self, amount: u8, data: u16) -> Result<(), std::io::Error> {
        let mask = (1 << amount) - 1;
        self.byte_buffer |= (data as u32 & mask) << self.cursor;
        self.cursor += amount;

        while self.cursor >= 8 {
            let byte = self.byte_buffer as u8;
            self.byte_buffer >>= 8;
            self.cursor -= 8;

            self.write.write_all(&[byte])?;
        }

        Ok(())
    }

    fn write_big_endian2(&mut self, amount: u8, data: u16) -> Result<(), std::io::Error> {
        let mask = (1 << amount) - 1;
        let shift = 32 - amount - self.cursor;
        self.byte_buffer |= (data as u32 & mask) << shift;
        self.cursor += amount;

        while self.cursor >= 8 {
            let byte = (self.byte_buffer >> 24) as u8;
            self.byte_buffer <<= 8;
            self.cursor -= 8;

            self.write.write_all(&[byte])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::{BitReader, BitWriter, Endianness};

    #[test]
    fn read_1_little_endian() {
        let input = [0x01];

        let mut reader = BitReader::new(Endianness::LittleEndian, &input[..]);

        assert_eq!(1, reader.read(1).unwrap());
    }

    #[test]
    fn read_colors_little_endian() {
        let input = [0x8C, 0x2D];

        let mut reader = BitReader::new(Endianness::LittleEndian, &input[..]);
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

        let mut reader = BitReader::new(Endianness::LittleEndian, &input[..]);

        assert_eq!(reader.read(12).unwrap(), 0xfff);
    }

    #[test]
    fn read_0xfffa_little_endian() {
        let input = [0xfa, 0xff];

        let mut reader = BitReader::new(Endianness::LittleEndian, &input[..]);

        assert_eq!(reader.read(16).unwrap(), 0xfffa);
    }

    #[test]
    fn read_1_big_endian() {
        let input = [0x80];

        let mut reader = BitReader::new(Endianness::BigEndian, &input[..]);

        assert_eq!(1, reader.read(1).unwrap());
    }

    #[test]
    fn read_colors_big_endian() {
        let input = [0x87, 0x62];

        let mut reader = BitReader::new(Endianness::BigEndian, &input[..]);
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

        let mut reader = BitReader::new(Endianness::BigEndian, &input[..]);

        assert_eq!(reader.read(12).unwrap(), 0xfff);
    }

    #[test]
    fn read_0xfffa_big_endian() {
        let input = [0xff, 0xfa];

        let mut reader = BitReader::new(Endianness::BigEndian, &input[..]);

        assert_eq!(reader.read(16).unwrap(), 0xfffa);
    }

    #[test]
    fn write_1_little_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BitWriter::new(Endianness::LittleEndian, &mut output);
        writer.write(1, 0x1)?;
        writer.fill()?;

        assert_eq!(output, [0x01]);

        Ok(())
    }

    #[test]
    fn write_colors_little_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BitWriter::new(Endianness::LittleEndian, &mut output);
        writer.write(3, 4)?;
        writer.write(3, 1)?;
        writer.write(3, 6)?;
        writer.write(3, 6)?;
        writer.write(4, 2)?;
        writer.fill()?;

        assert_eq!(output, [0x8C, 0x2D]);

        Ok(())
    }

    #[test]
    fn write_12bits_little_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BitWriter::new(Endianness::LittleEndian, &mut output);
        writer.write(12, 0xfff)?;
        writer.fill()?;

        assert_eq!(output, [0xff, 0x0f]);

        Ok(())
    }

    #[test]
    fn write_0xfffa_little_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BitWriter::new(Endianness::LittleEndian, &mut output);

        writer.write(16, 0xfffa)?;
        writer.fill()?;

        assert_eq!(output, [0xfa, 0xff]);

        Ok(())
    }

    #[test]
    fn write_1_big_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BitWriter::new(Endianness::BigEndian, &mut output);
        writer.write(1, 0x1)?;
        writer.fill()?;

        assert_eq!(output, [0x80]);

        Ok(())
    }

    #[test]
    fn write_colors_big_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BitWriter::new(Endianness::BigEndian, &mut output);
        writer.write(3, 4)?;
        writer.write(3, 1)?;
        writer.write(3, 6)?;
        writer.write(3, 6)?;
        writer.write(4, 2)?;
        writer.fill()?;

        assert_eq!(output, [0x87, 0x62]);

        Ok(())
    }

    #[test]
    fn write_12bits_big_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BitWriter::new(Endianness::BigEndian, &mut output);
        writer.write(12, 0xfff)?;
        writer.fill()?;

        assert_eq!(output, [0xff, 0xf0]);

        Ok(())
    }

    #[test]
    fn write_0xfffa_big_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BitWriter::new(Endianness::BigEndian, &mut output);

        writer.write(16, 0xfffa)?;
        writer.fill()?;

        assert_eq!(output, [0xff, 0xfa]);

        Ok(())
    }
}
