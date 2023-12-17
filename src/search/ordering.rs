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

impl SearchContext<'_> {
    pub(super) fn movepicker(
        &self,
        moves: MoveList,
        pos: &SearchPosition,
        pv: Move,
        ply: u8,
    ) -> MovePicker {
        let mut scores = vec![];

        for m in &moves {
            scores.push(self.move_score(*m, pos, pv, ply))
        }

        MovePicker { scores, moves }
    }

    fn move_score(&self, m: Move, pos: &SearchPosition, pv: Move, ply: u8) -> i32 {
        if m == pv {
            return i32::MAX;
        }
        if m.capture() && m.flags != MoveFlag::EP {
            return i32::MAX - 120 + MVVLVA_LOOKUP[m.piece][pos.pos.piece_at(m.to).ty];
        }

        if self.killers[ply as usize].contains(&m) {
            return i32::MAX - 112;
        }

        if m.promotion() != PieceType::None {
            return i32::MAX - 130;
        }

        if m.flags == MoveFlag::QUIET {
            let mut score = self.history[pos.pos.side][m.piece][m.to as usize];

            if ply > 0 {
                let prev = self.move_stack[ply as usize - 1];
                score += self.continuations[pos.pos.side][m.piece][m.to as usize][prev.piece]
                    [prev.to as usize];
            }

            return score;
        }

        0
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
