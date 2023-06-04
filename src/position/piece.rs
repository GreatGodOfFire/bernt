use std::ops::{Index, IndexMut, Not};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub ty: PieceType,
    pub color: PieceColor,
}

impl Piece {
    pub const EMPTY: Self = Self {
        ty: PieceType::Empty,
        color: PieceColor::White,
    };
}

impl<T, const N: usize, const M: usize> Index<Piece> for [[T; N]; M] {
    type Output = T;

    fn index(&self, index: Piece) -> &Self::Output {
        &self[index.color][index.ty]
    }
}

impl<T, const N: usize, const M: usize> IndexMut<Piece> for [[T; N]; M] {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self[index.color][index.ty]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Knight = 0,
    Bishop,
    Rook,
    Queen,
    Pawn,
    King,
    Empty,
}

impl<T, const N: usize> Index<PieceType> for [T; N] {
    type Output = T;

    fn index(&self, index: PieceType) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T, const N: usize> IndexMut<PieceType> for [T; N] {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceColor {
    White = 0,
    Black,
}

impl<T, const N: usize> Index<PieceColor> for [T; N] {
    type Output = T;

    fn index(&self, index: PieceColor) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T, const N: usize> IndexMut<PieceColor> for [T; N] {
    fn index_mut(&mut self, index: PieceColor) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl Not for PieceColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}
