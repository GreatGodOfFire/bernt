use crate::movegen;

use super::{SearchContext, SearchPosition};

impl SearchContext<'_> {
    pub fn qsearch(&mut self, pos: &SearchPosition, plies: u8, mut alpha: i32, beta: i32) -> i32 {
        let eval = pos.eval;
        if eval >= beta {
            return beta;
        }

        alpha = alpha.max(eval);

        if plies == 255 {
            return alpha;
        }

        let captures = movegen::<false>(&pos.pos);
        for m in &self.order_mvvlva(captures, pos) {
            let pos = self.update(&pos, *m, false);
            if !pos.pos.in_check(!pos.pos.side) {
                self.nodes += 1;
                let eval = -self.qsearch(&pos, plies + 1, -beta, -alpha);

                if eval >= beta {
                    return beta;
                }
                if eval > alpha {
                    alpha = eval;
                }
            }
        }

        alpha
    }
}
