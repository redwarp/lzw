use std::io::{Read, Write};

pub trait BitReader: Sized {
    fn read_one(&mut self, amount: u8) -> Result<u16, std::io::Error>;
    fn read(&mut self, amount: u8, buf: &mut [u16]) -> Result<usize, std::io::Error>;
    fn iter(&mut self, amount: u8) -> BitReaderIterator<Self> {
        BitReaderIterator::new(self, amount)
    }
}

pub struct LittleEndianReader<R>
where
    R: Read,
{
    read: R,
    cursor: u8,
    byte_buffer: u32,
    read_buffer: [u8; 1],
}

impl<R> LittleEndianReader<R>
where
    R: Read,
{
    pub fn new(read: R) -> Self {
        let cursor = 0;
        let byte_buffer = 0;
        let read_buffer = [0; 1];
        Self {
            read,
            cursor,
            byte_buffer,
            read_buffer,
        }
    }
}

impl<R> BitReader for LittleEndianReader<R>
where
    R: Read,
{
    #[inline]
    fn read_one(&mut self, amount: u8) -> Result<u16, std::io::Error> {
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

    fn read(&mut self, amount: u8, buf: &mut [u16]) -> Result<usize, std::io::Error> {
        let mut done = 0;
        while done < buf.len() {
            while self.cursor < amount {
                if self.read.read(&mut self.read_buffer[..])? == 0 {
                    return Ok(done);
                }

                self.byte_buffer |= (self.read_buffer[0] as u32) << self.cursor;
                self.cursor += 8;
            }

            let mask = (1 << amount) - 1;
            buf[done] = (self.byte_buffer & mask) as u16;
            self.byte_buffer >>= amount;
            self.cursor -= amount;
            done += 1;
        }

        Ok(done)
    }
}

pub struct BigEndianReader<R>
where
    R: Read,
{
    read: R,
    cursor: u8,
    byte_buffer: u32,
    read_buffer: [u8; 1],
}

impl<R> BigEndianReader<R>
where
    R: Read,
{
    pub fn new(read: R) -> Self {
        let cursor = 0;
        let byte_buffer = 0;
        let read_buffer = [0; 1];
        Self {
            read,
            cursor,
            byte_buffer,
            read_buffer,
        }
    }
}

impl<R> BitReader for BigEndianReader<R>
where
    R: Read,
{
    #[inline]
    fn read_one(&mut self, amount: u8) -> Result<u16, std::io::Error> {
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

    fn read(&mut self, amount: u8, buf: &mut [u16]) -> Result<usize, std::io::Error> {
        let mut done = 0;
        while done < buf.len() {
            while self.cursor < amount {
                self.read.read_exact(&mut self.read_buffer[..])?;
                let shift = 24 - self.cursor;
                self.byte_buffer |= (self.read_buffer[0] as u32) << shift;
                self.cursor += 8;
            }

            let mask = (1 << amount) - 1;
            let shift = 32 - amount;
            buf[done] = ((self.byte_buffer >> shift) & mask) as u16;
            self.byte_buffer <<= amount;
            self.cursor -= amount;
            done += 1;
        }

        Ok(done)
    }
}

pub struct BitReaderIterator<'a, B>
where
    B: BitReader,
{
    reader: &'a mut B,
    amount: u8,
    buf: [u16; 1],
}

impl<'a, B> BitReaderIterator<'a, B>
where
    B: BitReader,
{
    fn new(reader: &'a mut B, amount: u8) -> Self {
        Self {
            reader,
            amount,
            buf: [0],
        }
    }
}

impl<'a, B> Iterator for BitReaderIterator<'a, B>
where
    B: BitReader,
{
    type Item = Result<u16, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read(self.amount, &mut self.buf[..]) {
            Ok(usize) => {
                if usize == 0 {
                    None
                } else {
                    Some(Ok(self.buf[0]))
                }
            }
            Err(err) => Some(Err(err)),
        }
    }
}

pub trait BitWriter {
    fn write(&mut self, data: u16, amount: u8) -> Result<(), std::io::Error>;

    fn fill(&mut self) -> Result<(), std::io::Error>;

    fn flush(&mut self) -> Result<(), std::io::Error>;
}

pub struct LittleEndianWriter<W>
where
    W: Write,
{
    write: W,
    cursor: u8,
    byte_buffer: u32,
}

impl<W> LittleEndianWriter<W>
where
    W: Write,
{
    pub fn new(write: W) -> Self {
        let byte_buffer = 0;
        let cursor = 0;
        Self {
            write,
            byte_buffer,
            cursor,
        }
    }
}

impl<W> BitWriter for LittleEndianWriter<W>
where
    W: Write,
{
    #[inline]
    fn write(&mut self, data: u16, amount: u8) -> Result<(), std::io::Error> {
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

    #[inline]
    fn fill(&mut self) -> Result<(), std::io::Error> {
        if self.cursor > 0 {
            self.write.write_all(&[self.byte_buffer as u8])?;
            self.byte_buffer = 0;
            self.cursor = 0;
        }

        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.write.flush()
    }
}

pub struct BigEndianWriter<W>
where
    W: Write,
{
    write: W,
    cursor: u8,
    byte_buffer: u32,
}

impl<W> BigEndianWriter<W>
where
    W: Write,
{
    pub fn new(write: W) -> Self {
        let byte_buffer = 0;
        let cursor = 0;
        Self {
            write,
            byte_buffer,
            cursor,
        }
    }
}

impl<W> BitWriter for BigEndianWriter<W>
where
    W: Write,
{
    #[inline]
    fn write(&mut self, data: u16, amount: u8) -> Result<(), std::io::Error> {
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

    #[inline]
    fn fill(&mut self) -> Result<(), std::io::Error> {
        if self.cursor > 0 {
            self.write.write_all(&[(self.byte_buffer >> 24) as u8])?;
            self.byte_buffer = 0;
            self.cursor = 0;
        }

        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.write.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_1_little_endian() {
        let input = [0x01];

        let mut reader = LittleEndianReader::new(&input[..]);

        assert_eq!(1, reader.read_one(1).unwrap());
    }

    #[test]
    fn read_colors_little_endian() {
        let input = [0x8C, 0x2D];

        let mut reader = LittleEndianReader::new(&input[..]);
        let mut output = vec![];

        output.push(reader.read_one(3).unwrap());
        output.push(reader.read_one(3).unwrap());
        output.push(reader.read_one(3).unwrap());
        output.push(reader.read_one(3).unwrap());
        output.push(reader.read_one(4).unwrap());

        assert_eq!(output, [4, 1, 6, 6, 2]);
    }

    #[test]
    fn read_12_bits_little_endian() {
        let input = [0xff, 0x0f];

        let mut reader = LittleEndianReader::new(&input[..]);

        assert_eq!(reader.read_one(12).unwrap(), 0xfff);
    }

    #[test]
    fn read_0xfffa_little_endian() {
        let input = [0xfa, 0xff];

        let mut reader = LittleEndianReader::new(&input[..]);

        assert_eq!(reader.read_one(16).unwrap(), 0xfffa);
    }

    #[test]
    fn read_1_big_endian() {
        let input = [0x80];

        let mut reader = BigEndianReader::new(&input[..]);

        assert_eq!(1, reader.read_one(1).unwrap());
    }

    #[test]
    fn read_colors_big_endian() {
        let input = [0x87, 0x62];

        let mut reader = BigEndianReader::new(&input[..]);
        let mut output = vec![];

        output.push(reader.read_one(3).unwrap());
        output.push(reader.read_one(3).unwrap());
        output.push(reader.read_one(3).unwrap());
        output.push(reader.read_one(3).unwrap());
        output.push(reader.read_one(4).unwrap());

        assert_eq!(output, [4, 1, 6, 6, 2]);
    }

    #[test]
    fn read_12_bits_big_endian() {
        let input = [0xff, 0xf0];

        let mut reader = BigEndianReader::new(&input[..]);

        assert_eq!(reader.read_one(12).unwrap(), 0xfff);
    }

    #[test]
    fn read_0xfffa_big_endian() {
        let input = [0xff, 0xfa];

        let mut reader = BigEndianReader::new(&input[..]);

        assert_eq!(reader.read_one(16).unwrap(), 0xfffa);
    }

    #[test]
    fn write_1_little_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = LittleEndianWriter::new(&mut output);
        writer.write(0x1, 1)?;
        writer.fill()?;

        assert_eq!(output, [0x01]);

        Ok(())
    }

    #[test]
    fn write_colors_little_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = LittleEndianWriter::new(&mut output);
        writer.write(4, 3)?;
        writer.write(1, 3)?;
        writer.write(6, 3)?;
        writer.write(6, 3)?;
        writer.write(2, 4)?;
        writer.fill()?;

        assert_eq!(output, [0x8C, 0x2D]);

        Ok(())
    }

    #[test]
    fn write_12bits_little_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = LittleEndianWriter::new(&mut output);
        writer.write(0xfff, 12)?;
        writer.fill()?;

        assert_eq!(output, [0xff, 0x0f]);

        Ok(())
    }

    #[test]
    fn write_0xfffa_little_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = LittleEndianWriter::new(&mut output);
        writer.write(0xfffa, 16)?;
        writer.fill()?;

        assert_eq!(output, [0xfa, 0xff]);

        Ok(())
    }

    #[test]
    fn write_1_big_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BigEndianWriter::new(&mut output);
        writer.write(0x1, 1)?;
        writer.fill()?;

        assert_eq!(output, [0x80]);

        Ok(())
    }

    #[test]
    fn write_colors_big_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BigEndianWriter::new(&mut output);
        writer.write(4, 3)?;
        writer.write(1, 3)?;
        writer.write(6, 3)?;
        writer.write(6, 3)?;
        writer.write(2, 4)?;
        writer.fill()?;

        assert_eq!(output, [0x87, 0x62]);

        Ok(())
    }

    #[test]
    fn write_12bits_big_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BigEndianWriter::new(&mut output);
        writer.write(0xfff, 12)?;
        writer.fill()?;

        assert_eq!(output, [0xff, 0xf0]);

        Ok(())
    }

    #[test]
    fn write_0xfffa_big_endian() -> Result<(), std::io::Error> {
        let mut output = vec![];

        let mut writer = BigEndianWriter::new(&mut output);

        writer.write(0xfffa, 16)?;
        writer.fill()?;

        assert_eq!(output, [0xff, 0xfa]);

        Ok(())
    }

    #[test]
    fn read_full() -> Result<(), std::io::Error> {
        let mut output = vec![];
        let mut writer = LittleEndianWriter::new(&mut output);
        writer.write(0, 12)?;
        writer.write(1, 12)?;
        writer.write(0, 12)?;
        writer.write(2, 12)?;
        writer.fill()?;
        writer.flush()?;
        drop(writer);

        let mut reader = LittleEndianReader::new(&output[..]);
        let result: Result<Vec<u16>, _> = reader.iter(12).collect();

        assert_eq!(result?, [0, 1, 0, 2]);

        Ok(())
    }
}
