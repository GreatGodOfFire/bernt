use std::ops::{Index, IndexMut, Not};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PieceType {
    Knight,
    Bishop,
    Rook,
    Queen,
    Pawn,
    King,
    #[default]
    Empty,
}

impl PieceType {
    #[allow(non_upper_case_globals)]
    pub const Occupied: Self = Self::Empty;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PieceColor {
    #[default]
    White = 0,
    Black,
}

impl Not for PieceColor {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl<T, const N: usize, const M: usize> Index<Piece> for [[T; N]; M] {
    type Output = T;

    fn index(&self, index: Piece) -> &Self::Output {
        &self[index.color as usize][index.ty]
    }
}

impl<T, const N: usize, const M: usize> IndexMut<Piece> for [[T; N]; M] {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self[index.color as usize][index.ty]
    }
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
