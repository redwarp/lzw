use crate::lzw::Lzw;

pub struct Stacked;

struct StringTable {
    words: Vec<Word>,
    stack: Vec<u8>,
}

impl StringTable {
    fn new(code_size: u8, possibilities: u16) -> Self {
        let mut words = Vec::with_capacity(4096);
        for i in 0..possibilities {
            words.push(Word::new(None, i as u8));
        }
        let stack = Vec::with_capacity(4097);

        Self { words, stack }
    }

    fn contains(&self, word: &Word) -> bool {
        self.words.contains(word)
    }

    fn add(&mut self, word: Word) -> u16 {
        let index = self.words.len() as u16;
        self.words.push(word);
        index
    }

    fn entry_of(&self, word: &Word) -> Option<u16> {
        self.words
            .iter()
            .position(|a| a == word)
            .map(|index| index as u16)
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

#[derive(PartialEq)]
struct Word {
    prefix: Option<u16>,
    suffix: u8,
}

impl Word {
    fn new(prefix: Option<u16>, suffix: u8) -> Self {
        Self { prefix, suffix }
    }
}

impl Lzw for Stacked {
    fn compress(bytes: &[u8], code_size: u8, possibilities: u16) -> Vec<u16> {
        let mut string_table = StringTable::new(code_size, possibilities);

        let mut code_stream = vec![];
        let mut current_prefix: Option<u16> = None;

        for &k in bytes {
            let word = Word::new(current_prefix, k);

            if let Some(index) = string_table.entry_of(&word) {
                current_prefix = Some(index)
            } else {
                string_table.add(word);
                // string_table.push_to_stream(current_prefix.unwrap(), &mut code_stream);
                code_stream.push(current_prefix.unwrap());
                current_prefix = Some(k as u16);
            }
        }

        if let Some(k) = current_prefix {
            code_stream.push(k);
        }

        code_stream
    }

    fn decompress(data: &[u16], code_size: u8, possibilities: u16) -> Vec<u8> {
        todo!()
    }
}
