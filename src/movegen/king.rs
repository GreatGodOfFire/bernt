use crate::position::{piece::PieceColor, Position};

use super::{
    bitboard::BitIter,
    is_attacking,
    util::{FILE_A, FILE_H, RANK_1, RANK_8},
    Code, Move,
};

pub fn king_moves(king: u64, free_squares: u64, enemies: u64, out: &mut Vec<Move>) {
    let from = king.trailing_zeros();
    let moves = LOOKUP[from as usize];

    for to in BitIter(moves & free_squares) {
        out.push(Move::new(from as u16, to as u16, Code::Quiet));
    }

    for to in BitIter(moves & enemies) {
        out.push(Move::new(from as u16, to as u16, Code::Capture));
    }
}

pub fn lookup_king(king: u8) -> u64 {
    LOOKUP[king as usize]
}

pub fn castling_moves(
    king: u64,
    color: PieceColor,
    empty: u64,
    position: &Position,
    out: &mut Vec<Move>,
) {
    let row = (!empty >> (color as u8 * 56)) & 0xff;
    let king_square = king.trailing_zeros();

    if QUEENSIDE_CASTLE & row == 0
        && position.castling()[color][0]
        && !is_attacking(king_square as u8 - 1, position, !color)
    {
        out.push(Move::new(
            king_square as u16,
            king_square as u16 - 2,
            Code::QueenCastle,
        ));
    }

    if KINGSIDE_CASTLE & row == 0
        && position.castling()[color][1]
        && !is_attacking(king_square as u8 + 1, position, !color)
    {
        out.push(Move::new(
            king_square as u16,
            king_square as u16 + 2,
            Code::KingCastle,
        ));
    }
}

const QUEENSIDE_CASTLE: u64 = 0xe;
const KINGSIDE_CASTLE: u64 = 0x60;

const LOOKUP: [u64; 64] = generate_lookup();

const fn generate_lookup() -> [u64; 64] {
    let mut attacks = [0u64; 64];
    let mut i = 0;

    while i < 64 {
        let mut king = 1u64 << i;

        let mut moves = (king & !FILE_A) >> 1 | (king & !FILE_H) << 1;
        king |= moves;
        moves |= (king & !RANK_8) << 8 | (king & !RANK_1) >> 8;
        attacks[i] = moves;

        i += 1;
    }

    attacks
}
