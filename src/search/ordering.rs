use crate::{
    movegen::MoveList,
    position::{Move, MoveFlag},
};

use super::{SearchContext, SearchPosition};

impl SearchContext<'_> {
    pub(super) fn order_moves(&self, mut moves: MoveList, pos: &SearchPosition, pv: Move) -> MoveList {
        moves.moves[..moves.len as usize].sort_unstable_by_key(|x| move_score(*x, pos, pv));

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
/* P */  [ 9, 11, 11, 13, 17],
/* N */  [ 7,  9,  8, 11, 15],
/* B */  [ 7, 10,  9, 11, 15],
/* R */  [ 5,  7,  7,  9, 13],
/* Q */  [ 1,  3,  3,  5,  9],
/* K */  [ 0,  2,  2,  4,  8],
];

fn move_score(m: Move, pos: &SearchPosition, pv: Move) -> u8 {
    if m == pv {
        return 0;
    }

    255 - mvvlva(m, pos)
}
