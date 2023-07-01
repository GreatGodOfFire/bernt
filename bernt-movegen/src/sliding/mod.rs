use bernt_position::{bitboard::BitIter, Move, MoveFlags};

use crate::{flags, MoveList};

use self::magic::{BISHOP_ATTACKS, BISHOP_MAGICS, ROOK_ATTACKS, ROOK_MAGICS};

mod magic;

pub fn rook_moves<const FLAGS: u8>(
    rooks: u64,
    player_pieces: u64,
    opponent_pieces: u64,
    movelist: &mut MoveList,
) {
    for rook in BitIter(rooks) {
        let moves = single_rook_moves(rook, player_pieces, opponent_pieces);

        if FLAGS & flags::QUIET != 0 {
            for to in BitIter(moves & !opponent_pieces) {
                movelist.add(Move::new(rook, to, MoveFlags::Quiet));
            }
        }

        if FLAGS & flags::CAPTURES != 0 {
            for to in BitIter(moves & opponent_pieces) {
                movelist.add(Move::new(rook, to, MoveFlags::Capture));
            }
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

pub fn bishop_moves<const FLAGS: u8>(
    bishops: u64,
    player_pieces: u64,
    opponent_pieces: u64,
    movelist: &mut MoveList,
) {
    for bishop in BitIter(bishops) {
        let moves = single_bishop_moves(bishop, player_pieces, opponent_pieces);

        if FLAGS & flags::QUIET != 0 {
            for to in BitIter(moves & !opponent_pieces) {
                movelist.add(Move::new(bishop, to, MoveFlags::Quiet));
            }
        }

        if FLAGS & flags::CAPTURES != 0 {
            for to in BitIter(moves & opponent_pieces) {
                movelist.add(Move::new(bishop, to, MoveFlags::Capture));
            }
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
