use crate::{
    movegen::MoveList,
    position::{Move, MoveFlag, PieceType},
};

use super::{SearchContext, SearchPosition};

impl SearchContext<'_> {
    pub(super) fn order_mvvlva(&self, mut moves: MoveList, pos: &SearchPosition) -> MoveList {
        moves.moves[..moves.len as usize].sort_unstable_by_key(|x| 255 - mvvlva(*x, pos));

        moves
    }
}

pub struct MovePicker {
    moves: MoveList,
    scores: Vec<u32>,
}

impl MovePicker {
    pub(super) fn new(
        moves: MoveList,
        pos: &SearchPosition,
        tt_move: Move,
        killers: &[Move; 2],
        history: &[[u32; 64]; 6],
    ) -> Self {
        let mut scores = vec![];

        for m in &moves {
            scores.push(move_score(*m, pos, tt_move, killers, history))
        }

        Self {
            scores,
            moves,
        }
    }
}

impl Iterator for MovePicker {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.moves.len == 0 {
            return None;
        }

        let mut best = u32::MAX;
        let mut idx = 0;

        for (i, &score) in self.scores
            .iter()
            .enumerate()
        {
            if score < best {
                idx = i;
                best = score;
            }
        }

        self.moves.len -= 1;
        let len = self.moves.len as usize;
        self.scores.swap(idx, len);
        self.moves.moves.swap(idx, len);
        self.scores.pop();

        Some(self.moves.moves[len])
    }
}

fn mvvlva(m: Move, pos: &SearchPosition) -> u32 {
    if m.capture() && m.flags != MoveFlag::EP {
        MVVLVA_LOOKUP[m.piece][pos.pos.piece_at(m.to).ty]
    } else {
        0
    }
}

#[rustfmt::skip]
pub const MVVLVA_LOOKUP: [[u32; 5]; 6] = [
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
    killers: &[Move; 2],
    history: &[[u32; 64]; 6],
) -> u32 {
    if m == pv {
        return 0;
    }
    if m.capture() && m.flags != MoveFlag::EP {
        return 20 - MVVLVA_LOOKUP[m.piece][pos.pos.piece_at(m.to).ty];
    }

    if killers.contains(&m) {
        return 12;
    }

    if m.promotion() != PieceType::None {
        return 30;
    }

    if m.flags == MoveFlag::QUIET {
        return history[m.piece][m.to as usize];
    }

    u32::MAX
}
