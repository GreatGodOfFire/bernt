use std::mem::size_of;

use crate::movegen::Move;

const DEFAULT_HASH_SIZE: usize = 16000000;

fn tt_len(size: usize) -> usize {
    size / size_of::<TTIndex>()
}

pub struct TranspositionTable(Vec<TTIndex>);

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        Self(vec![TTIndex::default(); tt_len(size)])
    }

    pub fn new_default() -> Self {
        Self(vec![TTIndex::default(); tt_len(DEFAULT_HASH_SIZE)])
    }

    pub fn insert(&mut self, index: TTIndex) {
        let i = index.zobrist as usize % self.0.len();
        let old_index = &mut self.0[i];
        if old_index.age < index.age || old_index.depth < index.depth {
            *old_index = index;
        }
    }

    pub fn lookup(&mut self, zobrist: u64, age: u16) -> Option<(Move, i32, u8, TTIndexType)> {
        let i = zobrist as usize % self.0.len();
        let index = &self.0[i];
        if index.zobrist == zobrist && index.age == age {
            Some((index.best, index.eval, index.depth, index.ty))
        } else {
            None
        }
    }
}

// Note: Default only required so I can fill up the hash table safely
#[derive(Clone, Default)]
pub struct TTIndex {
    pub zobrist: u64,
    pub best: Move,
    pub eval: i32,
    pub depth: u8,
    pub age: u16,
    pub ty: TTIndexType,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum TTIndexType {
    #[default]
    Exact,
    Upper,
    Lower,
}

impl TTIndex {
    pub fn new(zobrist: u64, best: Move, eval: i32, depth: u8, age: u16, ty: TTIndexType) -> Self {
        Self {
            zobrist,
            best,
            eval,
            depth,
            age,
            ty,
        }
    }
}
