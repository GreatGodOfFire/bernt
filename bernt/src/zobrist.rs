#![allow(clippy::needless_range_loop)]
use crate::{position::PieceType, Position};

pub const PIECES: [[[u64; 6]; 2]; 64] = _RANDOMS.0;
pub const BLACK: u64 = _RANDOMS.1;
pub const CASTLING: [[u64; 2]; 2] = _RANDOMS.2;
pub const EN_PASSANT: [u64; 8] = _RANDOMS.3;

impl Position {
    pub fn hash(&self) -> u64 {
        let mut hash = BLACK * self.side as u64;

        for sq in 0..64 {
            let piece = self.piece_at(sq as u8);
            if piece.ty != PieceType::None {
                hash ^= PIECES[sq][piece.color][piece.ty];
            }
        }

        for (x, y) in self.castling.iter().zip(CASTLING.iter()) {
            hash ^= (x[0] as u64).wrapping_mul(y[0]);
            hash ^= (x[1] as u64).wrapping_mul(y[1]);
        }

        if self.en_passant != -1 {
            hash ^= EN_PASSANT[self.en_passant as usize % 8];
        }

        hash
    }
}

#[allow(clippy::type_complexity)]
const _RANDOMS: ([[[u64; 6]; 2]; 64], u64, [[u64; 2]; 2], [u64; 8]) = gen_randoms();

#[allow(clippy::type_complexity)]
const fn gen_randoms() -> ([[[u64; 6]; 2]; 64], u64, [[u64; 2]; 2], [u64; 8]) {
    let mut state = XorShiftState::new();

    let mut pieces = [[[0u64; 6]; 2]; 64];

    let mut sq = 0;
    while sq < 64 {
        let mut color = 0;
        while color < 2 {
            let mut piece = 0;
            while piece < 6 {
                (pieces[sq][color][piece], state) = state.next();

                piece += 1;
            }

            color += 1;
        }

        sq += 1;
    }

    let (black, state) = state.next();

    let (a, state) = state.next();
    let (b, state) = state.next();
    let (c, state) = state.next();
    let (d, mut state) = state.next();
    let castling = [[a, b], [c, d]];

    let mut en_passant = [0; 8];
    let mut x = 0;
    while x < 8 {
        (en_passant[x], state) = state.next();

        x += 1;
    }

    (pieces, black, castling, en_passant)
}

const SEED: u128 = 0x246C_CB2D_3B40_2853_9918_0A6D_BC3A_F444;

struct XorShiftState(u128);

impl XorShiftState {
    const fn new() -> Self {
        Self(SEED)
    }

    const fn next(mut self) -> (u64, Self) {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;

        (self.0 as u64 ^ (self.0 >> 64) as u64, self)
    }
}
impl Position {}
