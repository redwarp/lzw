use std::io::Write;

use crate::Endianness;

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
    use super::{BitWriter, Endianness};

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
