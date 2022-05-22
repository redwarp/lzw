use std::collections::HashMap;

mod decoding;
mod tree;

pub fn abcd_encode(text: &str) -> Vec<usize> {
    let mut code_stream = vec![];
    // Initialize the table, filling it with the possible values data can take
    let mut string_table: Vec<String> = vec![
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
        "D".to_string(),
    ];

    let mut current_prefix: String = "".to_string();

    for k in text.chars() {
        let mut current_string = current_prefix.clone();
        current_string.push(k);

        if string_table.contains(&current_string) {
            current_prefix = current_string;
        } else {
            string_table.push(current_string);

            let code = string_table
                .iter()
                .position(|word| word == &current_prefix)
                .unwrap();
            code_stream.push(code);
            current_prefix.clear();
            current_prefix.push(k);
        }
    }

    if !current_prefix.is_empty() {
        let code = string_table
            .iter()
            .position(|word| word == &current_prefix)
            .unwrap();
        code_stream.push(code);
    }

    code_stream
}

pub trait Encoder {
    fn new(code_size: u8) -> Self;
    fn encode(&mut self, bytes: &[u8]) -> Vec<u16>;
}

/// Basic and dumb encoder throwing everything in a vector
pub struct EncoderVersion1 {
    code_size: u8,
}

impl Encoder for EncoderVersion1 {
    fn new(code_size: u8) -> Self {
        Self { code_size }
    }

    fn encode(&mut self, bytes: &[u8]) -> Vec<u16> {
        let mut code_stream = vec![];
        // Initialize the table, filling it with the possible values data can take
        let mut string_table: Vec<Vec<u8>> =
            (0..1 << self.code_size).map(|index| vec![index]).collect();

        let mut current_prefix: Vec<u8> = vec![];

        for &k in bytes {
            let mut current_string = current_prefix.clone();
            current_string.push(k);

            if string_table.contains(&current_string) {
                current_prefix = current_string;
            } else {
                string_table.push(current_string);

                let code = string_table
                    .iter()
                    .position(|word| word == &current_prefix)
                    .unwrap() as u16;
                code_stream.push(code);
                current_prefix.clear();
                current_prefix.push(k);
            }
        }

        if !current_prefix.is_empty() {
            let code = string_table
                .iter()
                .position(|word| word == &current_prefix)
                .unwrap() as u16;
            code_stream.push(code);
        }

        code_stream
    }
}

/// Let's be less basic and use a HashMap instead.
pub struct EncoderVersion2 {
    code_size: u8,
}

impl Encoder for EncoderVersion2 {
    fn new(code_size: u8) -> Self {
        Self { code_size }
    }

    fn encode(&mut self, bytes: &[u8]) -> Vec<u16> {
        struct CompressionTable {
            entries: HashMap<Vec<u8>, u16>,
        }

        impl CompressionTable {
            fn new(code_size: u8) -> Self {
                let entries: HashMap<_, _> =
                    (0..1 << code_size).map(|i| (vec![i as u8], i)).collect();

                Self { entries }
            }

            fn code_for(&self, string: &[u8]) -> Option<&u16> {
                self.entries.get(string)
            }

            fn contains(&self, string: &[u8]) -> bool {
                self.entries.contains_key(string)
            }

            fn add(&mut self, entry: Vec<u8>) {
                self.entries.insert(entry, self.entries.len() as u16);
            }
        }

        let mut code_stream = vec![];

        let mut string_table = CompressionTable::new(self.code_size);
        let mut current_prefix: Vec<u8> = vec![];

        for &k in bytes {
            let mut current_string = current_prefix.clone();
            current_string.push(k);

            if string_table.contains(&current_string) {
                current_prefix = current_string;
            } else {
                string_table.add(current_string);
                code_stream.push(*string_table.code_for(&current_prefix).unwrap());
                current_prefix.clear();
                current_prefix.push(k);
            }
        }

        if !current_prefix.is_empty() {
            code_stream.push(*string_table.code_for(&current_prefix).unwrap())
        }

        code_stream
    }
}

/// Hashmap, but instead of storing vecs, we store Words, which are the expression
/// of [index of previous word, char]. Definitly more memory efficient, and will proceed to do less allocation.
pub struct EncoderVersion3 {
    code_size: u8,
}

impl Encoder for EncoderVersion3 {
    fn new(code_size: u8) -> Self {
        Self { code_size }
    }

    fn encode(&mut self, bytes: &[u8]) -> Vec<u16> {
        #[derive(PartialEq, Eq, Hash)]
        struct Word {
            prefix: Option<u16>,
            suffix: u8,
        }

        struct CompressionTable {
            entries: HashMap<Word, u16>,
        }

        impl CompressionTable {
            fn new(code_size: u8) -> Self {
                let entries: HashMap<_, _> = (0..1 << code_size)
                    .map(|i| {
                        (
                            Word {
                                prefix: None,
                                suffix: i as u8,
                            },
                            i as u16,
                        )
                    })
                    .collect();

                Self { entries }
            }

            fn code_for(&self, string: &Word) -> Option<u16> {
                self.entries.get(string).copied()
            }

            fn add(&mut self, entry: Word) {
                self.entries.insert(entry, self.entries.len() as u16);
            }
        }

        let mut code_stream = vec![];

        let mut string_table = CompressionTable::new(self.code_size);
        let mut current_prefix: Option<u16> = None;

        for &k in bytes {
            let current_string = Word {
                prefix: current_prefix,
                suffix: k,
            };

            if let Some(code_for_current_string) = string_table.code_for(&current_string) {
                current_prefix = Some(code_for_current_string);
            } else {
                string_table.add(current_string);
                code_stream.push(
                    current_prefix.expect(
                        "There will be a prefix, as all prefixless entries are in the table",
                    ),
                );
                current_prefix = Some(k as u16);
            }
        }

        if let Some(prefix) = current_prefix {
            code_stream.push(prefix);
        }

        code_stream
    }
}

/// Let's build a tree!
pub struct EncoderVersion4 {
    code_size: u8,
}

impl Encoder for EncoderVersion4 {
    fn new(code_size: u8) -> Self {
        Self { code_size }
    }

    fn encode(&mut self, bytes: &[u8]) -> Vec<u16> {
        #[derive(Clone)]
        struct Node {
            k: u8,
            children: Vec<u16>,
        }

        impl Node {
            fn new(k: u8) -> Self {
                let children = vec![];
                Self { k, children }
            }
        }

        struct Tree {
            nodes: Vec<Node>,
        }

        impl Tree {
            fn new(code_size: u8) -> Self {
                let mut nodes = Vec::with_capacity(1 << (code_size + 1));
                nodes.extend((0..1 << code_size).map(Node::new));

                Self { nodes }
            }

            fn find_word(&self, prefix_index: Option<u16>, next_char: u8) -> Option<u16> {
                match prefix_index {
                    Some(prefix_index) => {
                        let prefix = &self.nodes[prefix_index as usize];
                        let bob = prefix
                            .children
                            .iter()
                            .map(|id| (*id, &self.nodes[*id as usize]))
                            .find(|&(_, node)| node.k == next_char)
                            .map(|(id, _)| id);
                        bob
                    }
                    None => Some(next_char as u16),
                }
            }

            fn add(&mut self, prefix_index: u16, k: u8) {
                let new_index = self.nodes.len() as u16;
                self.nodes[prefix_index as usize].children.push(new_index);
                self.nodes.push(Node::new(k));
            }
        }
        let mut code_stream = vec![];

        let mut tree = Tree::new(self.code_size);
        let mut current_prefix: Option<u16> = None;

        for &k in bytes {
            if let Some(code_for_current_string) = tree.find_word(current_prefix, k) {
                current_prefix = Some(code_for_current_string);
            } else {
                tree.add(current_prefix.unwrap(), k);
                code_stream.push(
                    current_prefix.expect(
                        "There will be a prefix, as all prefixless entries are in the table",
                    ),
                );
                current_prefix = Some(k as u16);
            }
        }

        if let Some(prefix) = current_prefix {
            code_stream.push(prefix);
        }

        code_stream
    }
}

/// Let's build a simplified tree!
pub struct EncoderVersion5 {
    code_size: u8,
}

impl Encoder for EncoderVersion5 {
    fn new(code_size: u8) -> Self {
        Self { code_size }
    }

    fn encode(&mut self, bytes: &[u8]) -> Vec<u16> {
        #[derive(Clone)]
        struct Node {
            children: Vec<u16>,
        }

        impl Node {
            fn new(code_size: u8) -> Self {
                let children = vec![u16::MAX; 1 << code_size];
                Self { children }
            }
        }

        struct Tree {
            nodes: Vec<Node>,
            code_size: u8,
        }

        impl Tree {
            fn new(code_size: u8) -> Self {
                let mut nodes = Vec::with_capacity(1 << (code_size + 1));
                nodes.resize(1 << code_size, Node::new(code_size));

                Self { nodes, code_size }
            }

            fn find_word(&self, prefix_index: Option<u16>, next_char: u8) -> Option<u16> {
                match prefix_index {
                    Some(prefix_index) => {
                        let prefix = &self.nodes[prefix_index as usize];
                        let child_index = prefix.children[next_char as usize];
                        if child_index != u16::MAX {
                            Some(child_index)
                        } else {
                            None
                        }
                    }
                    None => Some(next_char as u16),
                }
            }

            fn add(&mut self, prefix_index: u16, k: u8) {
                self.nodes[prefix_index as usize].children[k as usize] = self.nodes.len() as u16;

                self.nodes.push(Node::new(self.code_size));
            }
        }
        let mut code_stream = vec![];

        let mut tree = Tree::new(self.code_size);
        let mut current_prefix: Option<u16> = None;

        for &k in bytes {
            if let Some(code_for_current_string) = tree.find_word(current_prefix, k) {
                current_prefix = Some(code_for_current_string);
            } else {
                tree.add(current_prefix.unwrap(), k);
                code_stream.push(
                    current_prefix.expect(
                        "There will be a prefix, as all prefixless entries are in the table",
                    ),
                );
                current_prefix = Some(k as u16);
            }
        }

        if let Some(prefix) = current_prefix {
            code_stream.push(prefix);
        }

        code_stream
    }
}

/// Let's build a simplified tree!
pub struct EncoderVersion6 {
    code_size: u8,
}

impl Encoder for EncoderVersion6 {
    fn new(code_size: u8) -> Self {
        Self { code_size }
    }

    fn encode(&mut self, bytes: &[u8]) -> Vec<u16> {
        #[derive(Clone)]
        enum Node {
            NoChildren,
            OneChild { k: u8, index: u16 },
            ManyChildren(Vec<u16>),
        }

        struct Tree {
            nodes: Vec<Node>,
            code_size: u8,
        }

        impl Tree {
            fn new(code_size: u8) -> Self {
                let mut nodes = Vec::with_capacity(1 << (code_size + 1));
                nodes.resize(1 << code_size, Node::NoChildren);

                Self { nodes, code_size }
            }

            fn find_word(&self, prefix_index: Option<u16>, next_char: u8) -> Option<u16> {
                match prefix_index {
                    Some(prefix_index) => {
                        let prefix = &self.nodes[prefix_index as usize];
                        match prefix {
                            Node::NoChildren => None,
                            Node::OneChild { k, index } => {
                                if next_char == *k {
                                    Some(*index)
                                } else {
                                    None
                                }
                            }
                            Node::ManyChildren(children) => {
                                let child_index = children[next_char as usize];
                                if child_index != u16::MAX {
                                    Some(child_index)
                                } else {
                                    None
                                }
                            }
                        }
                    }
                    None => Some(next_char as u16),
                }
            }

            fn add(&mut self, prefix_index: u16, k: u8) {
                let new_index = self.nodes.len() as u16;
                let node = &mut self.nodes[prefix_index as usize];
                match node {
                    Node::NoChildren => {
                        self.nodes[prefix_index as usize] = Node::OneChild {
                            k,
                            index: new_index,
                        };
                    }
                    Node::OneChild {
                        k: other_child_k,
                        index: other_child_index,
                    } => {
                        let mut children = vec![u16::MAX; 1 << self.code_size];
                        children[*other_child_k as usize] = *other_child_index;
                        children[k as usize] = new_index;
                        self.nodes[prefix_index as usize] = Node::ManyChildren(children);
                    }
                    Node::ManyChildren(children) => {
                        children[k as usize] = new_index;
                    }
                }

                self.nodes.push(Node::NoChildren);
            }
        }
        let mut code_stream = vec![];

        let mut tree = Tree::new(self.code_size);
        let mut current_prefix: Option<u16> = None;

        for &k in bytes {
            if let Some(code_for_current_string) = tree.find_word(current_prefix, k) {
                current_prefix = Some(code_for_current_string);
            } else {
                tree.add(current_prefix.unwrap(), k);
                code_stream.push(
                    current_prefix.expect(
                        "There will be a prefix, as all prefixless entries are in the table",
                    ),
                );
                current_prefix = Some(k as u16);
            }
        }

        if let Some(prefix) = current_prefix {
            code_stream.push(prefix);
        }

        code_stream
    }
}

pub fn compress<E: Encoder>(data: &[u8], code_size: u8) -> Vec<u16> {
    let mut encoder = E::new(code_size);

    encoder.encode(data)
}

#[cfg(test)]
mod tests {
    use crate::{
        abcd_encode, compress, EncoderVersion1, EncoderVersion2, EncoderVersion3, EncoderVersion4,
        EncoderVersion5, EncoderVersion6,
    };

    const DATA: &[u8; 40] = &[
        1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2,
        1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
    ];

    #[test]
    fn abcd_encoder() {
        let text = "ABACABADADABBBBBB";

        let compressed = abcd_encode(text);

        assert_eq!(compressed, [0, 1, 0, 2, 4, 0, 3, 9, 4, 1, 13, 13]);
    }

    #[test]
    fn encoder_version1() {
        let compressed = compress::<EncoderVersion1>(DATA, 2);

        assert_eq!(
            compressed,
            [1, 4, 4, 2, 7, 7, 5, 6, 8, 2, 10, 1, 12, 13, 4, 0, 19, 0, 8]
        );
    }

    #[test]
    fn encoder_version2() {
        let compressed = compress::<EncoderVersion2>(DATA, 2);
        assert_eq!(
            compressed,
            [1, 4, 4, 2, 7, 7, 5, 6, 8, 2, 10, 1, 12, 13, 4, 0, 19, 0, 8]
        );
    }

    #[test]
    fn encoder_version3() {
        let compressed = compress::<EncoderVersion3>(DATA, 2);
        assert_eq!(
            compressed,
            [1, 4, 4, 2, 7, 7, 5, 6, 8, 2, 10, 1, 12, 13, 4, 0, 19, 0, 8]
        );
    }

    #[test]
    fn encoder_version4() {
        let compressed = compress::<EncoderVersion4>(DATA, 2);
        assert_eq!(
            compressed,
            [1, 4, 4, 2, 7, 7, 5, 6, 8, 2, 10, 1, 12, 13, 4, 0, 19, 0, 8]
        );
    }

    #[test]
    fn encoder_version5() {
        let compressed = compress::<EncoderVersion5>(DATA, 2);
        assert_eq!(
            compressed,
            [1, 4, 4, 2, 7, 7, 5, 6, 8, 2, 10, 1, 12, 13, 4, 0, 19, 0, 8]
        );
    }

    #[test]
    fn encoder_version6() {
        let compressed = compress::<EncoderVersion6>(DATA, 2);
        assert_eq!(
            compressed,
            [1, 4, 4, 2, 7, 7, 5, 6, 8, 2, 10, 1, 12, 13, 4, 0, 19, 0, 8]
        );
    }
}
