use bernt_position::{bitboard::BitIter, piece::PieceColor, Move, MoveType, Position};

use crate::{flags, MoveList};

use super::{
    is_attacking,
    util::{FILE_A, FILE_H, RANK_1, RANK_8},
};

pub fn king_moves<const FLAGS: u8>(
    king: u64,
    free_squares: u64,
    enemies: u64,
    movelist: &mut MoveList,
) {
    let from = king.trailing_zeros() as u8;
    let moves = LOOKUP[from as usize];

    if FLAGS & flags::QUIET != 0 {
        for to in BitIter(moves & free_squares) {
            movelist.add(Move::new(from, to, MoveType::Quiet));
        }
    }

    if FLAGS & flags::CAPTURES != 0 {
        for to in BitIter(moves & enemies) {
            movelist.add(Move::new(from, to, MoveType::Capture));
        }
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
    movelist: &mut MoveList,
) {
    let row = (!empty >> (color as u8 * 56)) & 0xff;
    let king_square = king.trailing_zeros() as u8;

    if QUEENSIDE_CASTLE & row == 0
        && position.castling()[color][0] >= 0
        && !is_attacking(king_square - 1, position, !color)
    {
        movelist.add(Move::new(
            king_square,
            king_square - 2,
            MoveType::LeftCastle,
        ));
    }

    if KINGSIDE_CASTLE & row == 0
        && position.castling()[color][1] >= 0
        && !is_attacking(king_square + 1, position, !color)
    {
        movelist.add(Move::new(
            king_square,
            king_square + 2,
            MoveType::RightCastle,
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
