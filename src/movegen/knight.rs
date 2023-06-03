use super::{bitboard::BitIter, Code, Move};

pub fn knight_moves(knights: u64, free_squares: u64, enemies: u64, out: &mut Vec<Move>) {
    for knight in BitIter(knights) {
        let moves = KNIGHT_MOVES[knight as usize] & free_squares;
        for m in BitIter(moves & !enemies) {
            out.push(Move::new(knight as u16, m as u16, Code::Quiet));
        }

        for m in BitIter(moves & enemies) {
            out.push(Move::new(knight as u16, m as u16, Code::Capture));
        }
    }
}

pub fn knight_captures(knights: u64, free_squares: u64, enemies: u64, out: &mut Vec<Move>) {
    for knight in BitIter(knights) {
        let moves = KNIGHT_MOVES[knight as usize] & free_squares;

        for m in BitIter(moves & enemies) {
            out.push(Move::new(knight as u16, m as u16, Code::Capture));
        }
    }
}

pub fn single_knight_moves(knight: u8) -> u64 {
    KNIGHT_MOVES[knight as usize]
}

const KNIGHT_MOVES: [u64; 64] = generate_moves();

const fn generate_moves() -> [u64; 64] {
    let mut attacks = [0u64; 64];
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

        attacks[i] = attack;
        i += 1;
    }

    attacks
}
