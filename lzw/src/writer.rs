use std::io::Write;

use crate::Endianness;

pub struct BitWriter<W>
where
    W: Write,
{
    endianness: Endianness,
    write: W,
    cursor: u8,
    current_byte: u8,
}

impl<W> BitWriter<W>
where
    W: Write,
{
    pub fn new(endianness: Endianness, write: W) -> Self {
        let cursor = 0;
        let current_byte = 0;
        Self {
            endianness,
            write,
            cursor,
            current_byte,
        }
    }

    pub fn write(&mut self, amount: u8, data: u16) -> Result<(), std::io::Error> {
        match self.endianness {
            Endianness::LittleEndian => self.write_little_endian(amount, data),
            Endianness::BigEndian => self.write_big_endian(amount, data),
        }
    }

    pub fn fill(&mut self) -> Result<(), std::io::Error> {
        if self.cursor > 0 {
            self.write.write_all(&[self.current_byte])?;
            self.current_byte = 0;
            self.cursor = 0;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.write.flush()
    }

    fn write_little_endian(&mut self, amount: u8, data: u16) -> Result<(), std::io::Error> {
        let mut left = amount;
        let mut data = data;
        while left > 0 {
            let free: u8 = 8 - self.cursor;

            if free >= left {
                let mask = (1 << left) - 1;
                let bits = (data & mask) << self.cursor;
                self.current_byte |= bits as u8;
                self.cursor += left as u8;
                left = 0;
            } else {
                let mask = (1 << free) - 1;
                let bits = (data & mask) << self.cursor;
                self.current_byte |= bits as u8;
                self.cursor += free;
                data >>= free;
                left -= free;
            }

            if self.cursor == 8 {
                self.write.write_all(&[self.current_byte])?;
                self.current_byte = 0;
                self.cursor = 0;
            }
        }
        Ok(())
    }

    fn write_big_endian(&mut self, amount: u8, data: u16) -> Result<(), std::io::Error> {
        let mut left = amount;
        while left > 0 {
            let free: u8 = 8 - self.cursor;

            if free >= left {
                let mask = (1 << left) - 1;
                let bits = (data & mask) << (free - left);
                self.current_byte |= bits as u8;
                self.cursor += left as u8;
                left = 0;
            } else {
                let mask = ((1 << free) - 1) << (left - free);
                let bits = (data & mask) >> (left - free);
                self.current_byte |= bits as u8;
                self.cursor += free;
                left -= free;
            }

            if self.cursor == 8 {
                self.write.write_all(&[self.current_byte])?;
                self.current_byte = 0;
                self.cursor = 0;
            }
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
}
