use std::io::{Read, Write};

use crate::Endianness;

#[derive(Debug, Clone)]
enum TreeNode {
    None,
    One(u8, u16),
    Some(Vec<u16>),
}

/// Inspired by trie: https://en.wikipedia.org/wiki/Trie
/// Using this suggestion: https://dev.to/deciduously/no-more-tears-no-more-knots-arena-allocated-trees-in-rust-44k6
struct Tree {
    nodes: Vec<TreeNode>,
    code_size: u8,
}

impl Tree {
    fn new(code_size: u8) -> Self {
        let nodes = Vec::with_capacity(1 << (code_size + 1));
        Self { nodes, code_size }
    }

    fn reset(&mut self) {
        self.nodes.clear();
        self.nodes.resize((1 << self.code_size) + 2, TreeNode::None);
    }

    fn find_word(&self, prefix_index: u16, next_char: u8) -> Option<u16> {
        let prefix = &self.nodes[prefix_index as usize];
        match prefix {
            TreeNode::None => None,
            TreeNode::One(child_char, child_index) => {
                if *child_char == next_char {
                    Some(*child_index)
                } else {
                    None
                }
            }
            TreeNode::Some(child_indices) => {
                let child_index = child_indices[next_char as usize];
                if child_index != u16::MAX {
                    Some(child_index)
                } else {
                    None
                }
            }
        }
    }

    fn add(&mut self, prefix_index: u16, k: u8) -> u16 {
        let new_index = self.nodes.len() as u16;
        let prefix_index = prefix_index as usize;

        let mut old_node = &mut self.nodes[prefix_index];

        match &mut old_node {
            TreeNode::None => {
                self.nodes[prefix_index] = TreeNode::One(k, new_index);
            }
            TreeNode::One(other_k, other_index) => {
                let mut children = vec![u16::MAX; 1 << self.code_size];
                children[*other_k as usize] = *other_index;
                children[k as usize] = new_index;
                self.nodes[prefix_index] = TreeNode::Some(children);
            }
            TreeNode::Some(children) => {
                children[k as usize] = new_index;
            }
        };
        self.nodes.push(TreeNode::None);
        new_index
    }
}

pub struct Encoder {
    code_size: u8,
    string_table: Tree,
    endianness: Endianness,
}

impl Encoder {
    pub fn new(code_size: u8, endianness: Endianness) -> Self {
        let string_table = Tree::new(code_size);
        Self {
            code_size,
            string_table,
            endianness,
        }
    }

    pub fn encode<R: Read, W: Write>(&mut self, data: R, into: W) -> Result<(), std::io::Error> {
        let mut bit_writer = crate::writer::BitWriter::new(self.endianness, into);
        let mut code_size = self.code_size + 1;
        let clear_code = 1 << self.code_size;
        let end_of_information = (1 << self.code_size) + 1;

        let tree = &mut self.string_table;
        tree.reset();

        bit_writer.write(code_size, clear_code)?;

        let mut bytes = data.bytes();
        let k = bytes.next();
        if k.is_none() {
            // Well, it's an empty stream! Leaving early.
            bit_writer.write(code_size, end_of_information)?;

            bit_writer.fill()?;
            bit_writer.flush()?;

            return Ok(());
        }

        let mut current_prefix = k.unwrap()? as u16;

        for k in bytes {
            let k = k?;

            if let Some(word) = tree.find_word(current_prefix, k) {
                current_prefix = word;
            } else {
                let index_of_new_entry = tree.add(current_prefix, k);
                bit_writer.write(code_size, current_prefix)?;
                current_prefix = k as u16;

                if index_of_new_entry == 1 << code_size {
                    code_size += 1;

                    if code_size > 12 {
                        bit_writer.write(12, clear_code)?;
                        code_size = self.code_size + 1;
                        tree.reset();
                    }
                }
            }
        }

        bit_writer.write(code_size, current_prefix as u16)?;
        bit_writer.write(code_size, end_of_information)?;

        bit_writer.fill()?;
        bit_writer.flush()?;

        Ok(())
    }

    pub fn encode_to_vec<R: Read>(&mut self, data: R) -> Result<Vec<u8>, std::io::Error> {
        let mut output = vec![];
        self.encode(data, &mut output)?;
        Ok(output)
    }
}
