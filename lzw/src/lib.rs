mod encoder;
mod writer;

pub use encoder::Encoder;
pub use encoder::Encoder2;

#[derive(Debug, Clone, Copy)]
pub enum Endianness {
    BigEndian,
    LittleEndian,
}

#[cfg(test)]
mod tests {
    use crate::{encoder::Encoder2, Encoder, Endianness};

    #[test]
    fn test_4color_data() {
        let data = [
            1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2,
            2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
        ];

        let mut encoder = Encoder::new(2, Endianness::LittleEndian);

        let mut compressed = vec![];
        encoder.encode(&data[..], &mut compressed).unwrap();

        assert_eq!(
            compressed,
            [0x8C, 0x2D, 0x99, 0x87, 0x2A, 0x1C, 0xDC, 0x33, 0xA0, 0x2, 0x55, 0x0,]
        )
    }

    #[test]
    fn encode_multiple_with_same_encoder() {
        let data = [
            1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2,
            2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
        ];

        let mut encoder = Encoder::new(2, Endianness::LittleEndian);

        let compression1 = encoder.encode_to_vec(&data[..]).unwrap();
        let compression2 = encoder.encode_to_vec(&data[..]).unwrap();

        assert_eq!(compression1, compression2);
    }

    #[test]
    fn test_4color_data_encoder2() {
        let data = [
            1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2,
            2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
        ];

        let mut encoder = Encoder2::new(2, Endianness::LittleEndian);

        let mut compressed = vec![];
        encoder.encode(&data[..], &mut compressed).unwrap();

        assert_eq!(
            compressed,
            [0x8C, 0x2D, 0x99, 0x87, 0x2A, 0x1C, 0xDC, 0x33, 0xA0, 0x2, 0x55, 0x0,]
        )
    }

    #[test]
    fn encode_multiple_with_same_encoder2() {
        let data = [
            1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2,
            2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
        ];

        let mut encoder = Encoder2::new(2, Endianness::LittleEndian);

        let compression1 = encoder.encode_to_vec(&data[..]).unwrap();
        let compression2 = encoder.encode_to_vec(&data[..]).unwrap();

        assert_eq!(compression1, compression2);
    }
}
