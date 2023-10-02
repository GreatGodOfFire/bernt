use crate::position::Move;

pub struct TT(Vec<TTEntry>, usize);

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct TTEntry {
    pub hash: u64,
    pub eval: i32,
    pub best: Move,
    pub depth: u8,
    pub age: u8,
    pub ty: TTEntryType,
}

impl TT {
    pub fn new_default() -> Self {
        Self(vec![TTEntry::default(); tt_size(16)], 0)
    }

    pub fn set_size(&mut self, size: usize) {
        self.0 = vec![TTEntry::default(); tt_size(size)];
        self.1 = 0;
    }

    pub fn clear(&mut self) {
        self.0.fill(TTEntry::default());
        self.1 = 0;
    }

    pub fn hashfull(&self) -> usize {
        (self.1 * 1000) / self.0.len()
    }

    pub fn insert(&mut self, index: TTEntry) {
        let i = index.hash as usize % self.0.len();
        let old_index = &mut self.0[i];
        if score(old_index.age, old_index.depth) < score(index.age, index.depth) {
            if old_index.depth == 0 {
                self.1 += 1;
            }
            *old_index = index;
        }
    }

    pub fn lookup(&mut self, hash: u64) -> Option<(Move, i32, u8, TTEntryType)> {
        let i = hash as usize % self.0.len();
        let index = &self.0[i];
        if index.hash == hash && index.depth > 0 {
            Some((index.best, index.eval, index.depth, index.ty))
        } else {
            None
        }
    }
}

fn score(age: u8, depth: u8) -> u16 {
    age as u16 * 2 + depth as u16
}

fn tt_size(size: usize) -> usize {
    size * 1000000 / std::mem::size_of::<TTEntry>()
}

impl TTEntry {
    pub fn new(hash: u64, eval: i32, best: Move, depth: u8, age: u8, ty: TTEntryType) -> Self {
        Self {
            hash,
            eval,
            best,
            depth,
            age,
            ty,
        }
    }
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum TTEntryType {
    #[default]
    Exact,
    Upper,
    Lower,
}
