use std::{io::Write, marker::PhantomData};

use bitstream_io::{BitWrite, BitWriter, Endianness};
use indexmap::IndexSet;

pub struct Encoder<E>
where
    E: Endianness,
{
    code_size: u8,
    encode_table: EncodeTable,
    phantom: PhantomData<E>,
}

#[derive(PartialEq, Eq, Hash)]
enum Node {
    Root(u8),
    Word { prefix: usize, suffix: u8 },
    ClearCode,
    EndOfInformation,
}

impl Node {
    fn from(prefix: Option<usize>, suffix: u8) -> Node {
        if let Some(prefix) = prefix {
            Node::Word { prefix, suffix }
        } else {
            Node::Root(suffix)
        }
    }
}

struct EncodeTable {
    code_size: u8,
    words: IndexSet<Node>,
}

impl EncodeTable {
    fn new(code_size: u8) -> Self {
        let mut words = IndexSet::with_capacity(1 << (code_size as usize + 1));
        words.extend((0..1 << code_size).map(|i| Node::Root(i)));
        words.insert(Node::ClearCode);
        words.insert(Node::EndOfInformation);

        Self { code_size, words }
    }

    fn add(&mut self, word: Node) -> usize {
        let index = self.words.len();
        self.words.insert(word);
        index
    }

    fn entry_of(&self, word: &Node) -> Option<usize> {
        self.words.get_index_of(word).map(|index| index)
    }

    fn reset(&mut self) {
        self.words.truncate((1 << self.code_size) + 2);
    }

    fn clear_code(&self) -> usize {
        1 << self.code_size
    }

    fn end_of_information(&self) -> usize {
        (1 << self.code_size) + 1
    }
}

impl<E> Encoder<E>
where
    E: Endianness,
{
    pub fn new(code_size: u8, _endianness: E) -> Self {
        let encode_table = EncodeTable::new(code_size);
        Self {
            code_size,
            encode_table,
            phantom: PhantomData,
        }
    }

    pub fn encode<W: Write>(&mut self, data: &[u8], into: W) {
        let mut bit_writer: BitWriter<W, E> = BitWriter::new(into);

        let mut code_size = self.code_size as u32 + 1;

        let string_table = &mut self.encode_table;
        string_table.reset();

        let mut current_prefix: Option<usize> = None;

        bit_writer
            .write(code_size, string_table.clear_code() as u32)
            .unwrap();

        for &k in data {
            let word = Node::from(current_prefix, k);

            if let Some(index) = string_table.entry_of(&word) {
                current_prefix = Some(index)
            } else {
                let new_entry = string_table.add(word);
                let output_code = current_prefix.unwrap();
                bit_writer.write(code_size, output_code as u32).unwrap();

                if new_entry == 1 << code_size {
                    code_size += 1;

                    if code_size > 12 {
                        bit_writer
                            .write(12, string_table.clear_code() as u32)
                            .unwrap();
                        code_size = self.code_size as u32 + 1;
                        string_table.reset();
                    }
                }
                current_prefix = Some(k as usize);
            }
        }

        if let Some(k) = current_prefix {
            bit_writer.write(code_size, k as u32).unwrap();
        }
        bit_writer
            .write(code_size, string_table.end_of_information() as u32)
            .unwrap();

        bit_writer.byte_align().unwrap();
        bit_writer.flush().unwrap();
    }
}
