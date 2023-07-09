use std::mem;

use bernt_position::Move;

pub struct TranspositionTable(Vec<TTIndex>, usize);

impl TranspositionTable {
    pub fn new_default() -> Self {
        Self(vec![TTIndex::default(); tt_size(16)], 0)
    }

    pub fn set_size(&mut self, size: usize) {
        self.0 = vec![TTIndex::default(); tt_size(size)];
        self.1 = 0;
    }

    pub fn hashfull(&self) -> usize {
        (self.1 * 1000) / self.0.len()
    }

    pub fn insert(&mut self, index: TTIndex) {
        let i = index.hash as usize % self.0.len();
        let old_index = &mut self.0[i];
        if old_index.depth == 0 {
            self.1 += 1;
        }
        if old_index.age < index.age || old_index.depth < index.depth {
            *old_index = index;
        }
    }

    pub fn lookup(&mut self, hash: u64) -> Option<(Move, i32, u8, TTIndexType)> {
        let i = hash as usize % self.0.len();
        let index = &self.0[i];
        if index.hash == (hash >> 32) as u32 && index.depth > 0 {
            Some((index.best, index.eval, index.depth, index.ty))
        } else {
            None
        }
    }
}

fn tt_size(size: usize) -> usize {
    size * 1000000 / mem::size_of::<TTIndex>()
}

#[derive(Clone, Default)]
#[repr(C)]
pub struct TTIndex {
    pub hash: u32,
    pub eval: i32,
    pub best: Move,
    pub depth: u8,
    pub age: u8,
    pub ty: TTIndexType,
}

impl TTIndex {
    pub fn new(hash: u64, eval: i32, best: Move, depth: u8, age: u8, ty: TTIndexType) -> Self {
        Self {
            hash: (hash >> 32) as u32,
            eval,
            best,
            depth,
            age,
            ty,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum TTIndexType {
    #[default]
    Exact,
    Upper,
    Lower,
}
