use crate::{position::Move, movegen::MoveList};

use super::{SearchContext, SearchPosition};

impl SearchContext<'_> {
    pub(super) fn order_moves(&self, mut moves: MoveList, pv: Move) -> MoveList {
        moves.moves[..moves.len as usize].sort_unstable_by_key(|x| move_score(*x, pv));

        moves
    }
}

fn move_score(m: Move, pv: Move) -> u8 {
    if m == pv {
        return 0;
    }

    255
}
