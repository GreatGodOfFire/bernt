mod magic;

use crate::{
    bitloop,
    position::{Move, MoveFlag, PieceType},
};

use self::magic::{BISHOP_ATTACKS, BISHOP_MAGICS, ROOK_ATTACKS, ROOK_MAGICS};

use super::MoveList;

pub fn queen_moves<const QUIETS: bool>(
    queens: u64,
    us: u64,
    them: u64,
    mut movelist: &mut MoveList,
) {
    bitloop!(queens => queen, {
        let moves = single_rook_moves(queen, us, them)
            | single_bishop_moves(queen, us, them);

        if QUIETS {
            bitloop!(moves & !them => to, {
                movelist += Move::new(queen, to, MoveFlag::QUIET, PieceType::Queen);
            });
        }
        bitloop!(moves & them => to, {
            movelist += Move::new(queen, to, MoveFlag::CAP, PieceType::Queen);
        });
    });
}

pub fn rook_moves<const QUIETS: bool>(rooks: u64, us: u64, them: u64, mut movelist: &mut MoveList) {
    bitloop!(rooks => rook, {
        let moves = single_rook_moves(rook, us, them);

        if QUIETS {
            bitloop!(moves & !them => to, {
                movelist += Move::new(rook, to, MoveFlag::QUIET, PieceType::Rook);
            });
        }
        bitloop!(moves & them => to, {
            movelist += Move::new(rook, to, MoveFlag::CAP, PieceType::Rook);
        });
    });
}

#[inline]
pub fn single_rook_moves(rook: u8, us: u64, them: u64) -> u64 {
    let magic = ROOK_MAGICS[rook as usize];

    ROOK_ATTACKS[((magic.magic.wrapping_mul((us | them) & magic.mask)) >> (64 - magic.bits))
        as usize
        + magic.offset]
        & !us
}

pub fn bishop_moves<const QUIETS: bool>(
    bishops: u64,
    us: u64,
    them: u64,
    mut movelist: &mut MoveList,
) {
    bitloop!(bishops => bishop, {
        let moves = single_bishop_moves(bishop, us, them);

        if QUIETS {
            bitloop!(moves & !them => to, {
                movelist += Move::new(bishop, to, MoveFlag::QUIET, PieceType::Bishop);
            });
        }
        bitloop!(moves & them => to, {
            movelist += Move::new(bishop, to, MoveFlag::CAP, PieceType::Bishop);
        });
    });
}

#[inline]
pub fn single_bishop_moves(bishop: u8, us: u64, them: u64) -> u64 {
    let magic = BISHOP_MAGICS[bishop as usize];

    BISHOP_ATTACKS[((magic.magic.wrapping_mul((us | them) & magic.mask)) >> (64 - magic.bits))
        as usize
        + magic.offset]
        & !us
}
