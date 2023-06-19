use bernt_position::{bitboard::Bitboard, Move, MoveFlags};

use crate::{flags, MoveList};

pub fn knight_moves<const FLAGS: u8>(
    knights: Bitboard,
    free_squares: Bitboard,
    enemies: Bitboard,
    movelist: &mut MoveList,
) {
    for knight in knights {
        let moves = single_knight_moves(knight) & free_squares;

        if FLAGS & flags::QUIET != 0 {
            for to in moves & !enemies {
                movelist.add(Move::new(knight, to, MoveFlags::Quiet));
            }
        }

        if FLAGS & flags::CAPTURES != 0 {
            for to in moves & enemies {
                movelist.add(Move::new(knight, to, MoveFlags::Capture));
            }
        }
    }
}

pub fn single_knight_moves(knight: u8) -> Bitboard {
    KNIGHT_MOVES[knight as usize]
}

const KNIGHT_MOVES: [Bitboard; 64] = generate_moves();

const fn generate_moves() -> [Bitboard; 64] {
    let mut attacks = [Bitboard(0); 64];
    let mut i = 0;

    while i < 64 {
        let mut attack = 0u64;

        let rank = i / 8;
        let file = i % 8;

        if rank < 6 {
            if file < 7 {
                attack |= 1 << (file + 1 + (rank + 2) * 8);
            }
            if file > 0 {
                attack |= 1 << (file - 1 + (rank + 2) * 8);
            }
        }
        if rank > 1 {
            if file < 7 {
                attack |= 1 << (file + 1 + (rank - 2) * 8);
            }
            if file > 0 {
                attack |= 1 << (file - 1 + (rank - 2) * 8);
            }
        }
        if file < 6 {
            if rank < 7 {
                attack |= 1 << (file + 2 + (rank + 1) * 8);
            }
            if rank > 0 {
                attack |= 1 << (file + 2 + (rank - 1) * 8);
            }
        }
        if file > 1 {
            if rank < 7 {
                attack |= 1 << (file - 2 + (rank + 1) * 8);
            }
            if rank > 0 {
                attack |= 1 << (file - 2 + (rank - 1) * 8);
            }
        }

        attacks[i] = Bitboard(attack);
        i += 1;
    }

    attacks
}
