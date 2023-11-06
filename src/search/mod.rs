pub mod consts;
mod eval;
mod ordering;
mod qsearch;
mod timeman;
pub mod tt;

use std::time::{Duration, Instant};

use crate::{
    movegen::movegen,
    position::{Move, MoveFlag, PieceColor, PieceType, Position},
    search::eval::{flip, EG_PSTS, MG_PSTS, PHASE},
    zobrist, SearchOptions,
};

use self::{
    consts::*,
    eval::eval,
    timeman::TimeManager,
    tt::{TTEntry, TTEntryType, TT},
};

struct SearchContext<'a> {
    timeman: TimeManager,
    nodes: u64,
    repetitions: Vec<u64>,
    tt: &'a mut TT,
    killers: [[Move; 2]; 256],
    history: [[[u32; 64]; 6]; 2],
}

struct SearchPosition {
    pos: Position,
    eval: i32,
    mg_eval: i32,
    eg_eval: i32,
    phase: i32,
}

pub struct SearchResult {
    pub best: Move,
    pub score: i32,
    pub nodes: u64,
    pub elapsed: Duration,
}

pub fn search(
    pos: &Position,
    options: SearchOptions,
    repetitions: Vec<u64>,
    tt: &mut TT,
) -> SearchResult {
    let instant = Instant::now();

    let mut context = SearchContext {
        timeman: TimeManager::new(&options, pos.side),
        nodes: 0,
        repetitions,
        tt,
        killers: [[Move::NULL; 2]; 256],
        history: [[[u32::MAX; 64]; 6]; 2],
    };

    let mut best = (Move::NULL, -INF);

    let (eval, mg_eval, eg_eval, phase) = eval(&pos);

    let pos = SearchPosition {
        pos: pos.clone(),
        eval,
        mg_eval,
        eg_eval,
        phase,
    };

    for depth in 1..=options.depth {
        if depth > 1 && context.timeman.soft_stop() {
            break;
        }

        let mut window_size = ASP_WINDOW;
        let mut alpha = -INF;
        let mut beta = INF;

        if depth >= ASP_DEPTH {
            alpha = best.1 - window_size;
            beta = best.1 + window_size;
        }

        loop {
            let b = if let Some((m, score)) = context.negamax(&pos, alpha, beta, 0, depth, false) {
                (m, score)
            } else {
                break;
            };

            if b.1 <= alpha {
                beta = (alpha + beta) / 2;
                alpha -= window_size;
            } else if b.1 >= beta {
                beta += window_size;
            } else {
                best = b;
                break;
            }

            window_size += (window_size as f32 * ASP_INC_FACTOR) as i32;
        }

        let elapsed = instant.elapsed();
        let nodes = context.nodes;
        let nps = (nodes as f32 / elapsed.as_secs_f32()) as u64;
        let elapsed = elapsed.as_millis();
        let (m, score) = best;

        if options.info {
            println!(
                "info depth {depth} score cp {score} nodes {nodes} nps {nps} time {elapsed} pv {m}"
            );
        }
    }

    SearchResult {
        best: best.0,
        score: best.1,
        nodes: context.nodes,
        elapsed: instant.elapsed(),
    }
}

const INF: i32 = 1000000;
pub const CHECKMATE: i32 = 100000;

pub fn is_draw(pos: &Position, reps: &[u64]) -> bool {
    if pos.halfmove >= 100 {
        return true;
    }

    if pos.halfmove < 4 || reps.len() < 4 {
        return false;
    }

    let mut d = reps.len() - 1;
    let mut n = 0;

    while d > 0 {
        d -= 1;
        if reps[reps.len() - 1] == reps[d] {
            n += 1;
            if n == 2 {
                return true;
            }
        }
    }

    false
}

impl SearchContext<'_> {
    fn is_draw(&self, pos: &Position) -> bool {
        is_draw(pos, &self.repetitions)
    }

    fn update(&mut self, pos: &SearchPosition, m: Move, update_hash: bool) -> SearchPosition {
        use PieceType::*;

        let mut mg = pos.mg_eval;
        let mut eg = pos.eg_eval;
        let mut phase = pos.phase;

        let mut hash = *self.repetitions.last().unwrap();

        if m == Move::NULL {
            if update_hash {
                hash ^= zobrist::BLACK;
                self.repetitions.push(hash);
            }

            return SearchPosition {
                pos: pos.pos.make_move(m),
                eval: -pos.eval,
                mg_eval: -mg,
                eg_eval: -eg,
                phase,
            };
        }

        let to_bit = 1 << m.to;

        let mut piece = m.piece;
        let side = pos.pos.side;

        mg -= MG_PSTS[piece][flip(m.from, side) as usize];
        eg -= EG_PSTS[piece][flip(m.from, side) as usize];

        if update_hash {
            hash ^= zobrist::PIECES[m.from as usize][side][piece];

            if pos.pos.en_passant != 64 {
                hash ^= zobrist::EN_PASSANT[pos.pos.en_passant as usize % 8];
            }

            if m.flags == MoveFlag::DOUBLE_PAWN {
                hash ^= zobrist::EN_PASSANT[m.to as usize % 8];
            }

            if piece == PieceType::King {
                if pos.pos.castling[side][0] != 64 {
                    hash ^= zobrist::CASTLING[side][0];
                }
                if pos.pos.castling[side][1] != 64 {
                    hash ^= zobrist::CASTLING[side][1];
                }
            } else if m.from == pos.pos.castling[side][0] {
                hash ^= zobrist::CASTLING[side][0];
            } else if m.from == pos.pos.castling[side][1] {
                hash ^= zobrist::CASTLING[side][1];
            }
        }

        match m.flags {
            MoveFlag::CASTLE_LEFT => {
                if update_hash {
                    hash ^= zobrist::PIECES[m.to as usize - 2][side][Rook];
                    hash ^= zobrist::PIECES[m.to as usize + 1][side][Rook];
                }
                mg -= MG_PSTS[Rook][flip(m.to - 2, side) as usize];
                eg -= EG_PSTS[Rook][flip(m.to - 2, side) as usize];
                mg += MG_PSTS[Rook][flip(m.to + 1, side) as usize];
                eg += EG_PSTS[Rook][flip(m.to + 1, side) as usize];
            }
            MoveFlag::CASTLE_RIGHT => {
                if update_hash {
                    hash ^= zobrist::PIECES[m.to as usize + 1][side][Rook];
                    hash ^= zobrist::PIECES[m.to as usize - 1][side][Rook];
                }
                mg -= MG_PSTS[Rook][flip(m.to + 1, side) as usize];
                eg -= EG_PSTS[Rook][flip(m.to + 1, side) as usize];
                mg += MG_PSTS[Rook][flip(m.to - 1, side) as usize];
                eg += EG_PSTS[Rook][flip(m.to - 1, side) as usize];
            }
            MoveFlag::EP => {
                let sq = (pos.pos.en_passant as i8
                    + match side {
                        PieceColor::White => -8,
                        PieceColor::Black => 8,
                    }) as u8;

                if update_hash {
                    hash ^= zobrist::PIECES[sq as usize][!side][Pawn];
                }
                mg += MG_PSTS[Pawn][flip(sq, !side) as usize];
                eg += EG_PSTS[Pawn][flip(sq, !side) as usize];
                phase -= PHASE[Pawn];
            }
            _ => {
                if m.flags & MoveFlag::CAP != 0 {
                    let mut target = Pawn;
                    for ty in [Knight, Bishop, Rook, Queen] {
                        if pos.pos.pieces[ty] & to_bit != 0 {
                            target = ty;
                            break;
                        }
                    }

                    mg += MG_PSTS[target][flip(m.to, !side) as usize];
                    eg += EG_PSTS[target][flip(m.to, !side) as usize];
                    phase -= PHASE[target];

                    if update_hash {
                        hash ^= zobrist::PIECES[m.to as usize][!side][target];
                        if target == Rook {
                            if m.to == pos.pos.castling[!side][0] {
                                hash ^= zobrist::CASTLING[!side][0];
                            } else if m.to == pos.pos.castling[!side][1] {
                                hash ^= zobrist::CASTLING[!side][1];
                            }
                        }
                    }
                }

                if m.flags & MoveFlag::PROMO != 0 {
                    piece = m.promotion();
                    phase += PHASE[m.promotion()] - PHASE[Pawn];
                }
            }
        }

        mg += MG_PSTS[piece][flip(m.to, side) as usize];
        eg += EG_PSTS[piece][flip(m.to, side) as usize];
        if update_hash {
            hash ^= zobrist::PIECES[m.to as usize][side][piece];
            hash ^= zobrist::BLACK;
            self.repetitions.push(hash);
        }

        return SearchPosition {
            pos: pos.pos.make_move(m),
            eval: (-mg * phase.min(24) + -eg * (24 - phase.min(24))) / 24,
            mg_eval: -mg,
            eg_eval: -eg,
            phase,
        };
    }

    fn hash(&self) -> u64 {
        *self.repetitions.last().unwrap()
    }

    fn negamax(
        &mut self,
        pos: &SearchPosition,
        alpha: i32,
        beta: i32,
        ply: u8,
        mut depth: u8,
        is_nm: bool,
    ) -> Option<(Move, i32)> {
        let in_check = pos.pos.in_check(pos.pos.side);
        if in_check && depth < 3 {
            depth += 1;
        }
        if depth == 0 {
            return Some((Move::NULL, self.qsearch(pos, ply, alpha, beta)));
        }

        let mut n_moves = 0;

        let mut best = (Move::NULL, alpha);
        let (tt_move, tt_eval, tt_depth, tt_ty) = self.tt.lookup(self.hash()).unwrap_or_default();

        if tt_depth >= depth
            && (tt_ty == TTEntryType::Exact
                || (tt_ty == TTEntryType::Lower && tt_eval >= beta)
                || (tt_ty == TTEntryType::Upper && alpha >= tt_eval))
            && alpha + 1 == beta
        {
            // TODO: check with is_pseudolegal
            return Some((tt_move, tt_eval));
        }

        if beta - alpha == 1
            && !is_nm
            && !in_check
            && depth >= NMP_REDUCTION
            && pos.eval >= beta
            && ply > 0
            && (pos.pos.pieces[PieceType::Pawn] & pos.pos.colors[pos.pos.side]).count_ones() > 0
        {
            let pos = self.update(pos, Move::NULL, true);
            let (_, score) =
                self.negamax(&pos, -beta, -beta + 1, ply + 1, depth - NMP_REDUCTION, true)?;
            self.repetitions.pop();
            if -score >= beta {
                return Some((Move::NULL, -score));
            }
        }

        let mut search_pv = true;

        for m in &self.order_moves(movegen::<true>(&pos.pos), &pos, tt_move, ply) {
            let pos = self.update(pos, *m, true);

            if !pos.pos.in_check(!pos.pos.side) {
                n_moves += 1;

                self.nodes += 1;
                if self.nodes % 2048 == 0 && self.timeman.hard_stop() && !(ply == 0 && depth == 1) {
                    return None;
                }

                let lmr_reduction =
                    (LMR_BASE + (depth as f32).ln() * (n_moves as f32).ln() / LMR_DIV) as u8;

                let res = if self.is_draw(&pos.pos) {
                    Some((Move::NULL, 0))
                } else {
                    if search_pv {
                        self.negamax(&pos, -beta, -alpha, ply + 1, depth - 1, is_nm)
                    } else {
                        let red = if !m.capture() && beta - alpha == 1 && n_moves >= 3 && depth > 1
                        {
                            lmr_reduction.clamp(1, depth - 1)
                        } else {
                            1
                        };

                        let mut res =
                            self.negamax(&pos, -best.1 - 1, -best.1, ply + 1, depth - red, is_nm);
                        if let Some(r) = res {
                            if -r.1 > best.1 {
                                res = self.negamax(&pos, -beta, -alpha, ply + 1, depth - 1, is_nm);
                            }
                        }

                        res
                    }
                };

                if let Some(res) = res {
                    if -res.1 > best.1 {
                        best = (*m, -res.1);
                        search_pv = false;
                        if -res.1 >= beta {
                            if m.flags == MoveFlag::QUIET && self.killers[ply as usize][0] != *m {
                                self.killers[ply as usize][1] = self.killers[ply as usize][0];
                                self.killers[ply as usize][0] = *m;
                                self.history[!pos.pos.side][m.piece][m.to as usize] -=
                                    depth as u32 * depth as u32;
                            }
                            self.repetitions.pop();
                            return Some((*m, -res.1));
                        }
                    }
                } else {
                    if best.0 != Move::NULL && ply == 0 && depth > 1 {
                        return Some(best);
                    }
                    return None;
                }
            }
            self.repetitions.pop();
        }

        if n_moves == 0 {
            if in_check {
                return Some((Move::NULL, -CHECKMATE - 255 + ply as i32));
            } else {
                return Some((Move::NULL, 0));
            }
        }

        let ty = if best.1 >= beta {
            TTEntryType::Lower
        } else if best.1 <= alpha {
            TTEntryType::Upper
        } else {
            TTEntryType::Exact
        };

        self.tt
            .insert(TTEntry::new(self.hash(), best.1, best.0, depth, 0, ty));

        Some(best)
    }
}
