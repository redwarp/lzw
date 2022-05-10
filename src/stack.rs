use std::{collections::HashMap, ops::Shl};

use indexmap::IndexSet;

use crate::lzw::LzwCompressor;

pub struct Stacked;

#[derive(PartialEq, Eq, Hash)]
struct Word {
    prefix: Option<u16>,
    suffix: u8,
}

impl Word {
    fn new(prefix: Option<u16>, suffix: u8) -> Self {
        Self { prefix, suffix }
    }
}

impl LzwCompressor for Stacked {
    fn compress(bytes: &[u8], code_size: u8, possibilities: u16) -> Vec<u16> {
        struct StringTable {
            words: IndexSet<Word>,
            stack: Vec<u8>,
        }

        impl StringTable {
            fn new(code_size: u8, possibilities: u16) -> Self {
                let mut words = IndexSet::with_capacity(4096);
                words.extend((0..possibilities).map(|i| Word::new(None, i as u8)));
                let stack = Vec::with_capacity(4097);

                Self { words, stack }
            }

            fn contains(&self, word: &Word) -> bool {
                self.words.contains(word)
            }

            fn add(&mut self, word: Word) -> u16 {
                let index = self.words.len() as u16;
                self.words.insert(word);
                index
            }

            fn entry_of(&self, word: &Word) -> Option<u16> {
                self.words.get_index_of(word).map(|index| index as u16)
            }

            fn push_to_stream(&mut self, code: u16, stream: &mut Vec<u16>) {
                self.stack.clear();

                let mut code = code;
                loop {
                    let word = &self.words[code as usize];
                    self.stack.push(word.suffix);
                    if let Some(prefix) = word.prefix {
                        code = prefix;
                    } else {
                        break;
                    }
                }

                for index in (0..self.stack.len()).rev() {}
            }
        }

        let mut string_table = StringTable::new(code_size, possibilities);

        let mut code_stream = vec![];
        let mut current_prefix: Option<u16> = None;

        for &k in bytes {
            let word = Word::new(current_prefix, k);

            if let Some(index) = string_table.entry_of(&word) {
                current_prefix = Some(index)
            } else {
                string_table.add(word);
                code_stream.push(current_prefix.unwrap());
                current_prefix = Some(k as u16);
            }
        }

        if let Some(k) = current_prefix {
            code_stream.push(k);
        }

        code_stream
    }
}

pub struct WithBigVec;

impl LzwCompressor for WithBigVec {
    fn compress(bytes: &[u8], code_size: u8, possibilities: u16) -> Vec<u16> {
        struct StringTable {
            table: Vec<Vec<Option<u16>>>,
            index: usize,
        }

        impl StringTable {
            fn new(code_size: u8, possibilities: u16) -> Self {
                // let table = vec![None; possibilities as usize * 1usize.shl(code_size)];
                let table = vec![vec![None; possibilities as usize]; 1usize.shl(code_size)];

                Self {
                    table,
                    index: possibilities as usize,
                }
            }

            fn add(&mut self, word: Word) {
                if let Some(prefix) = word.prefix {
                    self.table[prefix as usize][word.suffix as usize] = Some(self.index as u16);
                }
                self.index += 1;
            }

            fn entry_of(&self, word: &Word) -> Option<u16> {
                if let Some(prefix) = word.prefix {
                    self.table[prefix as usize][word.suffix as usize]
                } else {
                    Some(word.suffix as u16)
                }
            }
        }

        let mut string_table = StringTable::new(code_size, possibilities);

        let mut code_stream = vec![];
        let mut current_prefix: Option<u16> = None;

        for &k in bytes {
            let word = Word::new(current_prefix, k);

            if let Some(index) = string_table.entry_of(&word) {
                current_prefix = Some(index)
            } else {
                string_table.add(word);
                code_stream.push(current_prefix.unwrap());
                current_prefix = Some(k as u16);
            }
        }

        if let Some(k) = current_prefix {
            code_stream.push(k);
        }

        code_stream
    }
}

pub struct WithMixHashVec;

impl LzwCompressor for WithMixHashVec {
    fn compress(bytes: &[u8], code_size: u8, possibilities: u16) -> Vec<u16> {
        struct StringTable {
            table: HashMap<u16, Vec<Option<u16>>>,
            index: usize,
        }

        impl StringTable {
            fn new(code_size: u8, possibilities: u16) -> Self {
                // let table = vec![None; possibilities as usize * 1usize.shl(code_size)];
                let table = HashMap::with_capacity(1usize.shl(code_size));

                Self {
                    table,
                    index: possibilities as usize,
                }
            }

            fn add(&mut self, word: Word) {
                if let Some(prefix) = word.prefix {
                    let values = self.table.entry(prefix).or_insert_with(|| vec![None; 256]);
                    values[word.suffix as usize] = Some(self.index as u16);
                }
                self.index += 1;
            }

            fn entry_of(&self, word: &Word) -> Option<u16> {
                if let Some(prefix) = word.prefix {
                    if let Some(values) = self.table.get(&prefix) {
                        values[word.suffix as usize]
                    } else {
                        None
                    }
                } else {
                    Some(word.suffix as u16)
                }
            }
        }

        let mut string_table = StringTable::new(code_size, possibilities);

        let mut code_stream = vec![];
        let mut current_prefix: Option<u16> = None;

        for &k in bytes {
            let word = Word::new(current_prefix, k);

            if let Some(index) = string_table.entry_of(&word) {
                current_prefix = Some(index)
            } else {
                string_table.add(word);
                code_stream.push(current_prefix.unwrap());
                current_prefix = Some(k as u16);
            }
        }

        if let Some(k) = current_prefix {
            code_stream.push(k);
        }

        code_stream
    }
}
