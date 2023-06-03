use self::magic::{BISHOP_ATTACKS, BISHOP_MAGICS, ROOK_ATTACKS, ROOK_MAGICS};

use super::{bitboard::BitIter, Code, Move};

mod magic;

pub fn rook_moves(rooks: u64, player_pieces: u64, opponent_pieces: u64, out: &mut Vec<Move>) {
    for rook in BitIter(rooks) {
        let magic = ROOK_MAGICS[rook as usize];
        let moves = ROOK_ATTACKS[((magic
            .magic
            .wrapping_mul((player_pieces | opponent_pieces) & magic.mask))
            >> (64 - magic.bits)) as usize
            + magic.offset]
            & !player_pieces;

        for to in BitIter(moves & !opponent_pieces) {
            out.push(Move::new(rook as u16, to as u16, Code::Quiet));
        }

        for to in BitIter(moves & opponent_pieces) {
            out.push(Move::new(rook as u16, to as u16, Code::Capture));
        }
    }
}

pub fn rook_captures(rooks: u64, player_pieces: u64, opponent_pieces: u64, out: &mut Vec<Move>) {
    for rook in BitIter(rooks) {
        let magic = ROOK_MAGICS[rook as usize];
        let moves = ROOK_ATTACKS[((magic
            .magic
            .wrapping_mul((player_pieces | opponent_pieces) & magic.mask))
            >> (64 - magic.bits)) as usize
            + magic.offset]
            & !player_pieces;

        for to in BitIter(moves & opponent_pieces) {
            out.push(Move::new(rook as u16, to as u16, Code::Capture));
        }
    }
}

pub fn single_rook_moves(rook: u8, player_pieces: u64, opponent_pieces: u64) -> u64 {
    let magic = ROOK_MAGICS[rook as usize];

    ROOK_ATTACKS[((magic
        .magic
        .wrapping_mul((player_pieces | opponent_pieces) & magic.mask))
        >> (64 - magic.bits)) as usize
        + magic.offset]
        & !player_pieces
}

pub fn bishop_moves(bishops: u64, player_pieces: u64, opponent_pieces: u64, out: &mut Vec<Move>) {
    for bishop in BitIter(bishops) {
        let magic = BISHOP_MAGICS[bishop as usize];
        let moves = BISHOP_ATTACKS[((magic
            .magic
            .wrapping_mul((player_pieces | opponent_pieces) & magic.mask))
            >> (64 - magic.bits)) as usize
            + magic.offset]
            & !player_pieces;

        for to in BitIter(moves & !opponent_pieces) {
            out.push(Move::new(bishop as u16, to as u16, Code::Quiet));
        }

        for to in BitIter(moves & opponent_pieces) {
            out.push(Move::new(bishop as u16, to as u16, Code::Capture));
        }
    }
}

pub fn bishop_captures(
    bishops: u64,
    player_pieces: u64,
    opponent_pieces: u64,
    out: &mut Vec<Move>,
) {
    for bishop in BitIter(bishops) {
        let magic = BISHOP_MAGICS[bishop as usize];
        let moves = BISHOP_ATTACKS[((magic
            .magic
            .wrapping_mul((player_pieces | opponent_pieces) & magic.mask))
            >> (64 - magic.bits)) as usize
            + magic.offset]
            & !player_pieces;

        for to in BitIter(moves & opponent_pieces) {
            out.push(Move::new(bishop as u16, to as u16, Code::Capture));
        }
    }
}

pub fn single_bishop_moves(bishop: u8, player_pieces: u64, opponent_pieces: u64) -> u64 {
    let magic = BISHOP_MAGICS[bishop as usize];

    BISHOP_ATTACKS[((magic
        .magic
        .wrapping_mul((player_pieces | opponent_pieces) & magic.mask))
        >> (64 - magic.bits)) as usize
        + magic.offset]
        & !player_pieces
}
