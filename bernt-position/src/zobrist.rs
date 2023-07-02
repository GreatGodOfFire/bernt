use crate::{piece::PieceType, Position};

pub const PIECES: [[[u64; 6]; 2]; 64] = _RANDOMS.0;
pub const BLACK: u64 = _RANDOMS.1;
pub const CASTLING: [[u64; 2]; 2] = _RANDOMS.2;
pub const EN_PASSANT: [u64; 8] = _RANDOMS.3;

pub fn hash(position: &Position) -> u64 {
    let mut hash = BLACK * position.to_move as u64;

    for (sq, piece) in position.mailbox.iter().enumerate() {
        if piece.ty != PieceType::Empty {
            hash ^= PIECES[sq][*piece];
        }
    }

    for (x, y) in position.castling().iter().zip(CASTLING.iter()) {
        hash ^= (x[0] as u64).wrapping_mul(y[0]);
        hash ^= (x[1] as u64).wrapping_mul(y[1]);
    }

    if position.en_passant() != -1 {
        hash ^= EN_PASSANT[position.en_passant() as usize % 8];
    }

    hash
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
                pieces[sq][color][piece] = state.next();

                piece += 1;
            }

            color += 1;
        }

        sq += 1;
    }

    let black = state.next();

    let castling = [[state.next(), state.next()], [state.next(), state.next()]];

    let mut en_passant = [0; 8];
    let mut x = 0;
    while x < 8 {
        en_passant[x] = state.next();

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

    const fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;

        self.0 as u64 ^ (self.0 >> 64) as u64
    }
}
