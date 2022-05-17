use std::{
    fmt::Display,
    io::{Read, Write},
};

use crate::{io::BitReader, Endianness};

#[derive(Debug)]
pub enum DecodingError {
    Io(std::io::Error),
    Lzw(&'static str),
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodingError::Io(error) => error.fmt(f),
            DecodingError::Lzw(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for DecodingError {}

impl From<std::io::Error> for DecodingError {
    fn from(error: std::io::Error) -> Self {
        DecodingError::Io(error)
    }
}

#[derive(Debug, Clone)]
enum Depth {
    Root,
    Child { depth: u16, parent: u16 },
}

impl Depth {
    fn get_depth(&self) -> u16 {
        match self {
            Depth::Root => 0,
            Depth::Child { depth, parent: _ } => *depth,
        }
    }

    fn get_parent(&self) -> Option<u16> {
        match self {
            Depth::Root => None,
            Depth::Child { depth: _, parent } => Some(*parent),
        }
    }
}

#[derive(Debug, Clone)]
enum Children {
    Zero,
    One(u8, u16),
    Many(Vec<u16>),
}

struct TreeNode {
    k: u8,
    root: u8,
    depth: Depth,
    children: Children,
}

impl TreeNode {
    fn root(k: u8) -> Self {
        Self {
            k,
            root: k,
            depth: Depth::Root,
            children: Children::Zero,
        }
    }
}

struct Tree {
    code_size: u8,
    buffer: [u8; 4086],
    nodes: Vec<TreeNode>,
}

struct TreeIterator<'a> {
    depth: usize,
    size: usize,
    next_word: Option<u16>,
    nodes: &'a Vec<TreeNode>,
}

impl<'a> TreeIterator<'a> {
    fn new(nodes: &'a Vec<TreeNode>, word: u16) -> Self {
        let depth = nodes[word as usize].depth.get_depth() as usize;
        let next_word = Some(word);
        Self {
            depth,
            size: depth + 1,
            next_word,
            nodes,
        }
    }
}

impl<'a> Iterator for TreeIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_word {
            Some(next_word) => {
                let tree_node = &self.nodes[next_word as usize];
                self.next_word = tree_node.depth.get_parent();

                self.size -= 1;

                Some(tree_node.k)
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.size;
        (size, Some(size))
    }
}

impl<'a> ExactSizeIterator for TreeIterator<'a> {
    fn len(&self) -> usize {
        self.size
    }
}

impl Tree {
    fn new(code_size: u8) -> Self {
        let buffer = [0; 4086];
        let nodes = Vec::with_capacity(1 << (code_size + 1));

        Self {
            code_size,
            buffer,
            nodes,
        }
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.nodes
            .extend((0..1 << self.code_size).map(TreeNode::root));
        self.nodes.push(TreeNode::root(0));
        self.nodes.push(TreeNode::root(0));
    }

    fn write_word_if_found<W: Write>(
        &mut self,
        word: u16,
        write: &mut W,
    ) -> Result<Option<u8>, std::io::Error> {
        if (word as usize) < self.nodes.len() {
            let iterator = TreeIterator::new(&self.nodes, word);
            let depth = iterator.depth;
            for (i, k) in iterator.enumerate() {
                self.buffer[depth - i] = k;
            }
            write.write_all(&self.buffer[..=depth])?;

            Ok(Some(self.buffer[0]))
        } else {
            Ok(None)
        }
    }

    fn add(&mut self, prefix: u16, k: u8) -> u16 {
        let new_index = self.nodes.len() as u16;
        let prefix_index = prefix as usize;

        let mut parent_node = &mut self.nodes[prefix_index];

        match &mut parent_node.children {
            Children::Zero => {
                parent_node.children = Children::One(k, new_index);
            }
            Children::One(other_k, other_index) => {
                let mut children = vec![u16::MAX; 1 << self.code_size];
                children[*other_k as usize] = *other_index;
                children[k as usize] = new_index;
                parent_node.children = Children::Many(children);
            }
            Children::Many(children) => {
                children[k as usize] = new_index;
            }
        };
        let root = parent_node.root;
        let depth = Depth::Child {
            depth: parent_node.depth.get_depth() + 1,
            parent: prefix,
        };

        self.nodes.push(TreeNode {
            k,
            root,
            depth,
            children: Children::Zero,
        });
        new_index
    }

    fn root_for(&self, prefix: u16) -> u8 {
        self.nodes[prefix as usize].root
    }
}

pub struct Decoder {
    code_size: u8,
    endianness: Endianness,
}

impl Decoder {
    pub fn new(code_size: u8, endianness: Endianness) -> Self {
        Self {
            code_size,
            endianness,
        }
    }

    pub fn decode<R: Read, W: Write>(&mut self, data: R, into: W) -> Result<(), DecodingError> {
        let mut bit_reader = BitReader::new(self.endianness, data);
        let mut read_size = self.code_size + 1;
        let mut into = into;

        let clear_code = 1 << self.code_size;
        let end_of_information = (1 << self.code_size) + 1;
        let mut tree = Tree::new(self.code_size);

        let expected_clear_code = bit_reader.read(read_size)?;
        if expected_clear_code != clear_code {
            return Err(DecodingError::Lzw("Missing clear code at stream start"));
        }
        tree.clear();

        let mut current_prefix = bit_reader.read(read_size)?;
        if current_prefix == end_of_information {
            return Ok(());
        }
        into.write_all(&[current_prefix as u8])?;

        'read_loop: loop {
            let k = bit_reader.read(read_size)?;

            if k == clear_code {
                tree.clear();
                read_size = self.code_size + 1;
                current_prefix = bit_reader.read(read_size)?;
                if current_prefix == end_of_information {
                    return Ok(());
                }
                into.write_all(&[current_prefix as u8])?;
            } else if k == end_of_information {
                break 'read_loop;
            } else {
                let index_of_new_entry =
                    if let Some(first_k) = tree.write_word_if_found(k, &mut into)? {
                        tree.add(current_prefix, first_k)
                    } else {
                        let first_k = tree.root_for(current_prefix);
                        tree.write_word_if_found(current_prefix, &mut into)?;
                        into.write_all(&[first_k])?;
                        tree.add(current_prefix, first_k)
                    };

                if index_of_new_entry == (1 << read_size) - 1 {
                    read_size += 1;
                }

                current_prefix = k;
            }
        }

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

        let mut decoder = Decoder::new(2, Endianness::LittleEndian);

        let mut decoded = vec![];
        decoder.decode(&data[..], &mut decoded).unwrap();

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

        let mut decoder = Decoder::new(2, Endianness::LittleEndian);

        let mut decoded1 = vec![];
        let mut decoded2 = vec![];
        decoder.decode(&data[..], &mut decoded1).unwrap();
        decoder.decode(&data[..], &mut decoded2).unwrap();

        assert_eq!(decoded1, decoded2);
    }

    #[test]
    fn decode_lorem_ipsum() {
        let data = include_bytes!("../../test-assets/lorem_ipsum_encoded.bin");
        let expected = include_str!("../../test-assets/lorem_ipsum.txt").as_bytes();

        let mut decoder = Decoder::new(7, Endianness::LittleEndian);
        let mut decoded = vec![];
        decoder.decode(&data[..], &mut decoded).unwrap();

        assert_eq!(decoded, expected);
    }
}
