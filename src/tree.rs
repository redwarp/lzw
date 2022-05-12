#![allow(dead_code)]
use std::collections::HashMap;

#[derive(Clone)]
struct Node {
    childrens: Vec<u16>,
}

impl Node {
    fn new(code_size: u8) -> Self {
        let children = vec![u16::MAX; 1 << code_size];
        Self {
            childrens: children,
        }
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
                let child_index = prefix.childrens[next_char as usize];
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
        self.nodes[prefix_index as usize].childrens[k as usize] = self.nodes.len() as u16;

        self.nodes.push(Node::new(self.code_size));
    }

    fn print_content(&self) {
        let mut node_counts: HashMap<usize, u32> = HashMap::new();
        for node in self.nodes.iter() {
            let node_count = node.childrens.iter().filter(|&&i| i != u16::MAX).count();

            let entry = node_counts.entry(node_count).or_insert(0);
            *entry += 1;
        }

        let total_node_count: u32 = node_counts.values().sum();
        let mut keys: Vec<_> = node_counts.keys().collect();
        keys.sort();

        println!("Total nodes: {total_node_count}");
        for key in keys {
            let value = node_counts[key];
            println!(
                "Nodes with {key} children: {value} ({percent}% of total nodes)",
                percent = (value as f32 / total_node_count as f32 * 100.0)
            );
        }
    }
}

pub struct TreeEncoder {
    code_size: u8,
    tree: Tree,
}

impl TreeEncoder {
    fn new(code_size: u8) -> Self {
        let tree = Tree::new(code_size);
        Self { code_size, tree }
    }

    fn encode(&mut self, bytes: &[u8]) -> Vec<u16> {
        let mut code_stream = vec![];

        let tree = &mut self.tree;
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

    fn print_tree_content(&self) {
        self.tree.print_content();
    }
}

#[cfg(test)]
mod tests {
    use super::TreeEncoder;

    #[test]
    fn debug_tree_lorem() {
        let data = include_str!("../lorem_ipsum.txt").as_bytes();

        let mut encoder = TreeEncoder::new(7);
        encoder.encode(data);

        encoder.print_tree_content();
    }

    #[test]
    fn debug_tree_sunflower() {
        let data = include_bytes!("../sunflower.bmp");

        let mut encoder = TreeEncoder::new(8);
        encoder.encode(data);

        encoder.print_tree_content();
    }
}
