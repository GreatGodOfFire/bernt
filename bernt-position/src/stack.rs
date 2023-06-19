use std::ops::{Index, IndexMut};

use crate::piece::PieceType;

pub(crate) struct Stack<T> {
    array: [T; 256],
    len: u8,
}

impl<T> Index<usize> for Stack<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len as usize {
            panic!()
        }

        &self.array[self.len as usize - 1 - index]
    }
}

impl<T> IndexMut<usize> for Stack<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len as usize {
            panic!()
        }

        &mut self.array[self.len as usize - 1 - index]
    }
}

impl<T: Default + Clone> Stack<T> {
    #[inline]
    pub fn new() -> Self
    where
        T: Copy,
    {
        Self {
            array: [T::default(); 256],
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, state: T) {
        self.array[self.len as usize] = state;
        self.len += 1;
    }

    #[inline]
    pub fn clone_push(&mut self) {
        self.array[self.len as usize] = self.array[(self.len - 1) as usize].clone();
        self.len += 1;
    }

    pub fn discard_top(&mut self) {
        self.len -= 1;
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn retain(&mut self, n: u8) {
        if n >= self.len {
            return;
        }

        for x in 0..n {
            self.array[x as usize] = self.array[(self.len - 1 - n + x) as usize].clone();
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct State {
    pub en_passant: i8,
    pub halfmove_clock: u8,
    pub castling: [[i8; 2]; 2],
    pub captured: Option<PieceType>,
}
