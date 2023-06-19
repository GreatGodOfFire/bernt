use bernt_position::{bitboard::Bitboard, piece::PieceColor, Move, MoveFlags, Position};

use crate::{flags, is_attacking, MoveList};

pub fn king_moves<const FLAGS: u8>(
    king: Bitboard,
    free_squares: Bitboard,
    enemies: Bitboard,
    movelist: &mut MoveList,
) {
    let from = king.trailing_zeros();
    let moves = KING_MOVES_LOOKUP[from as usize];

    if FLAGS & flags::QUIET != 0 {
        for to in moves & free_squares {
            movelist.add(Move::new(from as u8, to, MoveFlags::Quiet));
        }
    }

    if FLAGS & flags::CAPTURES != 0 {
        for to in moves & enemies {
            movelist.add(Move::new(from as u8, to, MoveFlags::Capture));
        }
    }
}

pub fn lookup_king(king: u8) -> Bitboard {
    KING_MOVES_LOOKUP[king as usize]
}

pub fn castling_moves(
    king: Bitboard,
    color: PieceColor,
    empty: Bitboard,
    position: &Position,
    movelist: &mut MoveList,
) {
    let row = (!empty >> (color as u8 * 56)) & Bitboard(0xff);
    let king_square = king.trailing_zeros() as u8;

    if (QUEENSIDE_CASTLE & row).is_empty()
        && position.castling()[color][0] != -1
        && !is_attacking(king_square - 1, position, !color)
    {
        movelist.add(Move::new(
            king_square,
            king_square - 2,
            MoveFlags::QueenCastle,
        ));
    }

    if (KINGSIDE_CASTLE & row).is_empty()
        && position.castling()[color][1] != -1
        && !is_attacking(king_square + 1, position, !color)
    {
        movelist.add(Move::new(
            king_square,
            king_square + 2,
            MoveFlags::KingCastle,
        ));
    }
}

const QUEENSIDE_CASTLE: Bitboard = Bitboard(0xe);
const KINGSIDE_CASTLE: Bitboard = Bitboard(0x60);

const KING_MOVES_LOOKUP: [Bitboard; 64] = generate_lookup();

const fn generate_lookup() -> [Bitboard; 64] {
    let mut attacks = [Bitboard(0); 64];
    let mut i = 0;

    while i < 64 {
        let mut king = 1u64 << i;

        let mut moves = (king & !0x101010101010101) >> 1 | (king & !0x8080808080808080) << 1;
        king |= moves;
        moves |= (king & !(0xff << 56)) << 8 | (king & !0xff) >> 8;
        attacks[i] = Bitboard(moves);

        i += 1;
    }

    attacks
}
