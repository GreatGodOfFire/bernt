use crate::{
    movegen::MoveList,
    position::{Move, MoveFlag, PieceType},
};

use super::{consts::MVVLVA_LOOKUP, eval::MG_PSTS, SearchContext, SearchPosition};

impl SearchContext<'_> {
    pub(super) fn order_mvvlva(&self, mut moves: MoveList, pos: &SearchPosition) -> MoveList {
        moves.moves[..moves.len as usize].sort_unstable_by_key(|x| 255 - mvvlva(*x, pos));

        moves
    }
}

pub struct MovePicker {
    moves: MoveList,
    scores: Vec<i32>,
}

impl MovePicker {
    pub(super) fn new(
        moves: MoveList,
        pos: &SearchPosition,
        tt_move: Move,
        killers: &[Move; 2],
        history: &[[i32; 64]; 6],
        counter_move: Move,
    ) -> Self {
        let mut scores = vec![];

        for m in &moves {
            scores.push(move_score(*m, pos, tt_move, killers, history, counter_move))
        }

        Self { scores, moves }
    }
}

impl Iterator for MovePicker {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.moves.len == 0 {
            return None;
        }

        let mut best = 0;
        let mut idx = 0;

        for (i, &score) in self.scores.iter().enumerate() {
            if score > best {
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

fn mvvlva(m: Move, pos: &SearchPosition) -> i32 {
    if m.capture() && m.flags != MoveFlag::EP {
        MVVLVA_LOOKUP[m.piece][pos.pos.piece_at(m.to).ty]
    } else {
        0
    }
}

fn move_score(
    m: Move,
    pos: &SearchPosition,
    pv: Move,
    killers: &[Move; 2],
    history: &[[i32; 64]; 6],
    counter_move: Move,
) -> i32 {
    if m == pv {
        return i32::MAX;
    }
    if m.capture() && m.flags != MoveFlag::EP {
        return i32::MAX - 120 + MVVLVA_LOOKUP[m.piece][pos.pos.piece_at(m.to).ty];
    }

    if killers.contains(&m) {
        return i32::MAX - 112;
    }

    if m.promotion() != PieceType::None {
        return i32::MAX - 130;
    }

    if m.flags == MoveFlag::QUIET {
        return history[m.piece][m.to as usize]
            + (m == counter_move) as i32 * 500
            + MG_PSTS[m.piece][m.to as usize];
    }

    0
}
