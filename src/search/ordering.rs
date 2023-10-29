use crate::{
    movegen::MoveList,
    position::{Move, MoveFlag},
};

use super::{SearchContext, SearchPosition};

impl SearchContext<'_> {
    pub(super) fn order_moves(
        &self,
        mut moves: MoveList,
        pos: &SearchPosition,
        pv: Move,
        plies: u8,
    ) -> MoveList {
        moves.moves[..moves.len as usize].sort_unstable_by_key(|x| {
            move_score(
                *x,
                pos,
                pv,
                self.killers[plies as usize],
                &self.history[pos.pos.side],
            )
        });

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

fn move_score(
    m: Move,
    pos: &SearchPosition,
    pv: Move,
    killers: [Move; 2],
    history: &[[u32; 64]; 6],
) -> u8 {
    if m == pv {
        return 0;
    }
    if m.capture() && m.flags != MoveFlag::EP {
        return 20 - MVVLVA_LOOKUP[m.piece][pos.pos.piece_at(m.to).ty];
    }

    if killers.contains(&m) {
        return 12;
    }

    if m.flags == MoveFlag::QUIET {
        let history_score = history[m.piece][m.to as usize];

        if history_score > 1000 {
            return 30;
        }
    }

    255
}
