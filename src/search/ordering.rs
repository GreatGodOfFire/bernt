use crate::{
    movegen::MoveList,
    position::{Move, MoveFlag},
};

use super::{SearchContext, SearchPosition};

impl SearchContext<'_> {
    pub(super) fn order_moves(&self, mut moves: MoveList, pv: Move) -> MoveList {
        moves.moves[..moves.len as usize].sort_unstable_by_key(|x| move_score(*x, pv));

        moves
    }

    pub(super) fn order_mvvlva(&self, mut moves: MoveList, pos: &SearchPosition) -> MoveList {
        moves.moves[..moves.len as usize].sort_unstable_by_key(|x| 255 - mvvlva(*x, pos));

        moves
    }
}

fn mvvlva(m: Move, pos: &SearchPosition) -> u8 {
    if m.capture() && m.flags != MoveFlag::EP {
        MVVLVA_LOOKUP[m.piece][pos.pos.piece_at(m.to).ty]
    } else {
        0
    }
}

#[rustfmt::skip]
pub const MVVLVA_LOOKUP: [[u8; 5]; 6] = [
        /* P   N   B   R   Q */ 
/* P */  [ 9, 12, 13, 17, 20],
/* N */  [ 8,  9, 11, 15, 19],
/* B */  [ 7,  7,  9, 14, 18],
/* R */  [ 3,  7,  8,  9, 16],
/* Q */  [ 1,  3,  7,  6,  9],
/* K */  [ 0,  0,  1,  4, 10],
];

fn move_score(m: Move, pv: Move) -> u8 {
    if m == pv {
        return 0;
    }

    255
}
