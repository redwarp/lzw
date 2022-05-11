use std::{
    collections::HashMap,
    hash::Hash,
    io::{Read, Write},
};

use crate::Endianness;

#[derive(Eq, PartialOrd, Ord)]
enum Node {
    Root(u8),
    Word { prefix: u16, suffix: u8 },
}

impl Node {
    fn from(prefix: Option<u16>, suffix: u8) -> Node {
        if let Some(prefix) = prefix {
            Node::Word { prefix, suffix }
        } else {
            Node::Root(suffix)
        }
    }

    fn as_key(&self) -> u32 {
        match *self {
            Node::Root(index) => index as u32,
            Node::Word { prefix, suffix } => ((prefix as u32) << 8) | suffix as u32,
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        let id = match *self {
            Node::Root(index) => index as usize,
            Node::Word { prefix, suffix } => ((prefix as usize) << 8) | suffix as usize,
        };
        let other_id = match *other {
            Node::Root(index) => index as usize,
            Node::Word { prefix, suffix } => ((prefix as usize) << 8) | suffix as usize,
        };
        id == other_id
    }
}

impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let id = match *self {
            Node::Root(index) => index as usize,
            Node::Word { prefix, suffix } => ((prefix as usize) << 8) | suffix as usize,
        };
        id.hash(state);
    }
}

trait EncodeTable {
    fn add(&mut self, word: Node) -> u16;

    fn index_of(&self, word: &Node) -> Option<u16>;

    fn reset(&mut self);
}

struct EncodeTable1 {
    code_size: u8,
    current_index: u16,
    words: HashMap<Node, u16>,
}

impl EncodeTable1 {
    fn new(code_size: u8) -> Self {
        let words: HashMap<Node, u16> =
            HashMap::with_capacity((1 << (code_size as usize + 2)).min(4096));

        let current_index = 0;

        Self {
            code_size,
            words,
            current_index,
        }
    }
}

impl EncodeTable for EncodeTable1 {
    fn add(&mut self, word: Node) -> u16 {
        let index = self.current_index;
        self.words.insert(word, index);
        self.current_index += 1;
        index
    }

    fn index_of(&self, word: &Node) -> Option<u16> {
        match word {
            &Node::Root(index) => Some(index as u16),
            _ => self.words.get(word).cloned(),
        }
    }

    fn reset(&mut self) {
        self.words.clear();

        self.current_index = (1 << self.code_size) + 2;
    }
}

struct EncodeTable2 {
    code_size: u8,
    current_index: u16,
    words: HashMap<u32, u16>,
}

impl EncodeTable2 {
    fn new(code_size: u8) -> Self {
        let words: HashMap<u32, u16> =
            HashMap::with_capacity((1 << (code_size as usize + 2)).min(4096));

        let current_index = 0;

        Self {
            code_size,
            words,
            current_index,
        }
    }
}

impl EncodeTable for EncodeTable2 {
    fn add(&mut self, word: Node) -> u16 {
        let index = self.current_index;
        self.words.insert(word.as_key(), index);
        self.current_index += 1;
        index
    }

    fn index_of(&self, word: &Node) -> Option<u16> {
        match word {
            &Node::Root(index) => Some(index as u16),
            _ => self.words.get(&word.as_key()).cloned(),
        }
    }

    fn reset(&mut self) {
        self.words.clear();

        self.current_index = (1 << self.code_size) + 2;
    }
}

#[derive(Clone)]
enum Entry3 {
    Blank,
    Values(Vec<u16>),
}

struct EncodeTable3 {
    code_size: u8,
    current_index: u16,
    words: Vec<Entry3>,
}

impl EncodeTable3 {
    fn new(code_size: u8) -> Self {
        let words = Vec::with_capacity(0);
        let current_index = 0;

        Self {
            code_size,
            words,
            current_index,
        }
    }
}

impl EncodeTable for EncodeTable3 {
    fn add(&mut self, word: Node) -> u16 {
        match word {
            Node::Root(_) => panic!("Shouldn't add root"),
            Node::Word { prefix, suffix } => {
                let index = self.current_index;

                match &mut self.words[prefix as usize] {
                    Entry3::Blank => {
                        let mut values = vec![0; 1 << self.code_size];
                        values[suffix as usize] = index;

                        let entry = Entry3::Values(values);
                        self.words[prefix as usize] = entry;
                    }
                    Entry3::Values(values) => values[suffix as usize] = index,
                };

                self.current_index += 1;
                if (index as usize) < self.words.len() {
                    self.words.push(Entry3::Blank);
                }
                index
            }
        }
    }

    fn index_of(&self, word: &Node) -> Option<u16> {
        match *word {
            Node::Root(index) => Some(index as u16),
            Node::Word { prefix, suffix } => {
                if prefix as usize > self.words.len() {
                    return None;
                }
                match &self.words[prefix as usize] {
                    Entry3::Blank => None,
                    Entry3::Values(values) => {
                        let value = values[suffix as usize];
                        if value == 0 {
                            None
                        } else {
                            Some(value)
                        }
                    }
                }
            }
        }
    }

    fn reset(&mut self) {
        self.words.clear();

        self.words.resize(1 << (self.code_size + 1), Entry3::Blank);
        self.current_index = (1 << self.code_size) + 2;
    }
}

#[derive(Clone)]
enum Entry4 {
    Blank,
    Single(u8, u16),
    Values(Vec<u16>),
}

struct EncodeTable4 {
    code_size: u8,
    current_index: u16,
    words: Vec<Entry4>,
}

impl EncodeTable4 {
    fn new(code_size: u8) -> Self {
        let words = Vec::with_capacity(1 << (code_size + 1));
        let current_index = 0;

        Self {
            code_size,
            words,
            current_index,
        }
    }
}

impl EncodeTable for EncodeTable4 {
    fn add(&mut self, word: Node) -> u16 {
        match word {
            Node::Root(_) => panic!("Shouldn't add root"),
            Node::Word { prefix, suffix } => {
                let index = self.current_index;

                match &mut self.words[prefix as usize] {
                    Entry4::Blank => {
                        let entry = Entry4::Single(suffix, index);
                        self.words[prefix as usize] = entry;
                    }
                    Entry4::Single(entry_suffix, entry_index) => {
                        let mut values = vec![0; 1 << self.code_size];
                        values[suffix as usize] = index;
                        values[*entry_suffix as usize] = *entry_index;

                        let entry = Entry4::Values(values);
                        self.words[prefix as usize] = entry;
                    }
                    Entry4::Values(values) => values[suffix as usize] = index,
                };

                self.current_index += 1;
                if (index as usize) >= self.words.len() {
                    self.words.push(Entry4::Blank);
                }
                index
            }
        }
    }

    fn index_of(&self, word: &Node) -> Option<u16> {
        match *word {
            Node::Root(index) => Some(index as u16),
            Node::Word { prefix, suffix } => {
                if prefix as usize > self.words.len() {
                    return None;
                }
                match &self.words[prefix as usize] {
                    Entry4::Blank => None,
                    Entry4::Single(entry_suffix, entry_index) => {
                        if *entry_suffix == suffix {
                            Some(*entry_index)
                        } else {
                            None
                        }
                    }
                    Entry4::Values(values) => {
                        let value = values[suffix as usize];
                        if value == 0 {
                            None
                        } else {
                            Some(value)
                        }
                    }
                }
            }
        }
    }

    fn reset(&mut self) {
        self.words.clear();

        self.words.resize((1 << self.code_size) + 2, Entry4::Blank);
        self.current_index = (1 << self.code_size) + 2;
    }
}

#[derive(Clone)]
enum Entry5 {
    Blank,
    Single(u8, u16),
    Values(usize),
}

struct EncodeTable5 {
    code_size: u8,
    current_index: u16,
    words: Vec<Entry5>,
    values: Vec<Vec<u16>>,
}

impl EncodeTable5 {
    fn new(code_size: u8) -> Self {
        let words = Vec::with_capacity(0);
        let values = vec![];
        let current_index = 0;

        Self {
            code_size,
            current_index,
            words,
            values,
        }
    }
}

impl EncodeTable for EncodeTable5 {
    fn add(&mut self, word: Node) -> u16 {
        match word {
            Node::Root(_) => panic!("Shouldn't add root"),
            Node::Word { prefix, suffix } => {
                let index = self.current_index;

                match &mut self.words[prefix as usize] {
                    Entry5::Blank => {
                        let entry = Entry5::Single(suffix, index);
                        self.words[prefix as usize] = entry;
                    }
                    Entry5::Single(entry_suffix, entry_index) => {
                        let values_index = self.values.len();
                        let mut values = vec![0; 1 << self.code_size];
                        values[suffix as usize] = index;
                        values[*entry_suffix as usize] = *entry_index;

                        let entry = Entry5::Values(values_index);
                        self.words[prefix as usize] = entry;
                        self.values.push(values);
                    }
                    Entry5::Values(values_index) => {
                        let values = &mut self.values[*values_index];
                        values[suffix as usize] = index
                    }
                };

                self.current_index += 1;
                if (index as usize) < self.words.len() {
                    self.words.push(Entry5::Blank);
                }
                index
            }
        }
    }

    fn index_of(&self, word: &Node) -> Option<u16> {
        match *word {
            Node::Root(index) => Some(index as u16),
            Node::Word { prefix, suffix } => {
                if prefix as usize > self.words.len() {
                    return None;
                }
                match &self.words[prefix as usize] {
                    Entry5::Blank => None,
                    Entry5::Single(entry_suffix, entry_index) => {
                        if *entry_suffix == suffix {
                            Some(*entry_index)
                        } else {
                            None
                        }
                    }
                    Entry5::Values(values_index) => {
                        let value = self.values[*values_index][suffix as usize];
                        if value == 0 {
                            None
                        } else {
                            Some(value)
                        }
                    }
                }
            }
        }
    }

    fn reset(&mut self) {
        self.words.clear();
        self.values.clear();

        self.words.resize(1 << (self.code_size + 1), Entry5::Blank);
        self.current_index = (1 << self.code_size) + 2;
    }
}

pub struct Encoder {
    code_size: u8,
    string_table: EncodeTable4,
    endianness: Endianness,
}

impl Encoder {
    pub fn new(code_size: u8, endianness: Endianness) -> Self {
        let encode_table = EncodeTable4::new(code_size);
        Self {
            code_size,
            string_table: encode_table,
            endianness,
        }
    }

    pub fn encode<R: Read, W: Write>(&mut self, data: R, into: W) -> Result<(), std::io::Error> {
        let mut bit_writer = crate::writer::BitWriter::new(self.endianness, into);
        let mut code_size = self.code_size + 1;
        let clear_code = 1 << self.code_size;
        let end_of_information = (1 << self.code_size) + 1;

        let string_table = &mut self.string_table;
        string_table.reset();

        let mut current_prefix: Option<u16> = None;

        bit_writer.write(code_size, clear_code)?;

        for k in data.bytes() {
            let k = k?;
            let word = Node::from(current_prefix, k);

            if let Some(index) = string_table.index_of(&word) {
                current_prefix = Some(index)
            } else {
                let index_of_new_entry = string_table.add(word);
                let output_code = current_prefix.expect("The current_prefix can't be none");
                bit_writer.write(code_size, output_code)?;

                if index_of_new_entry == 1 << code_size {
                    code_size += 1;

                    if code_size > 12 {
                        bit_writer.write(12, clear_code)?;
                        code_size = self.code_size + 1;
                        string_table.reset();
                    }
                }
                current_prefix = Some(k as u16);
            }
        }

        if let Some(k) = current_prefix {
            bit_writer.write(code_size, k)?;
        }
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

struct Tree {
    nodes: Vec<TreeNode>,
    code_size: u8,
    current_index: u16,
}

impl Tree {
    fn new(code_size: u8) -> Self {
        let nodes = Vec::with_capacity(1 << (code_size + 1));
        let current_index = 0;
        Self {
            nodes,
            code_size,
            current_index,
        }
    }

    fn reset(&mut self) {
        self.nodes.clear();

        self.nodes
            .extend((0..1 << self.code_size).map(|_| TreeNode::None));
        self.nodes.push(TreeNode::None);
        self.nodes.push(TreeNode::None);

        self.current_index = (1 << self.code_size) + 2;
    }

    fn find_word(&self, prefix_index: usize, next_char: u8) -> Option<usize> {
        let prefix = &self.nodes[prefix_index];
        match &prefix {
            TreeNode::None => None,
            TreeNode::One(child_index, child_char) => {
                if *child_char == next_char {
                    Some(*child_index)
                } else {
                    None
                }
            }
            TreeNode::Some(child_indices) => {
                let child_index = child_indices[next_char as usize];
                if child_index != usize::MAX {
                    Some(child_index)
                } else {
                    None
                }
            }
        }
    }

    fn add(&mut self, prefix_index: usize, k: u8) -> usize {
        let new_index = self.current_index as usize;

        let mut old_node = self
            .nodes
            .get_mut(prefix_index)
            .expect("Must be in the tree already");

        match &mut old_node {
            TreeNode::None => {
                self.nodes[prefix_index] = TreeNode::One(new_index, k);
            }
            TreeNode::One(other_index, other_k) => {
                let mut children = vec![usize::MAX; 1 << self.code_size];
                children[*other_k as usize] = *other_index;
                children[k as usize] = new_index;
                self.nodes[prefix_index] = TreeNode::Some(children);
            }
            TreeNode::Some(children) => {
                children[k as usize] = new_index;
            }
        };
        self.nodes.push(TreeNode::None);
        self.current_index += 1;
        new_index
    }
}

enum TreeNode {
    None,
    One(usize, u8),
    Some(Vec<usize>),
}

pub struct Encoder2 {
    code_size: u8,
    string_table: Tree,
    endianness: Endianness,
}

impl Encoder2 {
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
            bit_writer.write(code_size, end_of_information)?;

            bit_writer.fill()?;
            bit_writer.flush()?;

            return Ok(());
        }
        let k = k.unwrap()?;
        let mut current_prefix = k as usize;

        for k in bytes {
            let k = k?;

            if let Some(word) = tree.find_word(current_prefix, k) {
                current_prefix = word;
            } else {
                let index_of_new_entry = tree.add(current_prefix, k);
                let output_code = current_prefix as u16;
                bit_writer.write(code_size, output_code)?;

                if index_of_new_entry == 1 << code_size {
                    code_size += 1;

                    if code_size > 12 {
                        bit_writer.write(12, clear_code)?;
                        code_size = self.code_size + 1;
                        tree.reset();
                    }
                }
                current_prefix = k as usize;
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
