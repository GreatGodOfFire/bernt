mod eval;
mod ordering;
mod timeman;
pub mod tt;

use std::time::{Duration, Instant};

use crate::{
    movegen::movegen,
    position::{Move, MoveFlag, PieceColor, PieceType, Position},
    search::eval::{flip, PSTS},
    zobrist, SearchOptions,
};

use self::{
    eval::eval,
    timeman::TimeManager,
    tt::{TTEntry, TTEntryType, TT},
};

struct SearchContext<'a> {
    timeman: TimeManager,
    nodes: u64,
    repetitions: Vec<u64>,
    tt: &'a mut TT,
}

struct SearchPosition {
    pos: Position,
    eval: i32,
}

pub struct SearchResult {
    pub best: Move,
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
    };

    let mut best = Move::NULL;

    let pos = SearchPosition {
        pos: pos.clone(),
        eval: eval(&pos),
    };

    for depth in 1..=options.depth {
        if let Some((m, eval)) = context.negamax(&pos, -INF, INF, 0, depth) {
            let elapsed = instant.elapsed();

            let nodes = context.nodes;
            let nps = (nodes as f32 / elapsed.as_secs_f32()) as u64;
            let elapsed = elapsed.as_millis();

            if options.info {
                println!(
                    "info depth {depth} score cp {eval} nodes {nodes} nps {nps} time {elapsed} pv {m}"
                );
            }

            best = m;
        } else {
            break;
        }
    }

    SearchResult {
        best,
        nodes: context.nodes,
        elapsed: instant.elapsed(),
    }
}

const INF: i32 = 1000000;
const CHECKMATE: i32 = 100000;

impl SearchContext<'_> {
    fn is_draw(&self, pos: &Position) -> bool {
        if pos.halfmove >= 100 {
            return true;
        }

        if pos.halfmove < 4 || self.repetitions.len() < 4 {
            return false;
        }

        let mut d = self.repetitions.len() - 1;
        let mut n = 0;

        while d > 0 {
            d -= 1;
            if self.repetitions[self.repetitions.len() - 1] == self.repetitions[d] {
                n += 1;
                if n == 2 {
                    return true;
                }
            }
        }

        false
    }

    fn update(&mut self, pos: &SearchPosition, m: Move) -> SearchPosition {
        use PieceType::*;

        let mut eval = pos.eval;

        let to_bit = 1 << m.to;

        let mut piece = m.piece;
        let side = pos.pos.side;

        let mut hash = *self.repetitions.last().unwrap();

        hash ^= zobrist::PIECES[m.from as usize][side][piece];
        eval -= PSTS[piece][flip(m.from, side) as usize];

        if pos.pos.en_passant > 0 {
            hash ^= zobrist::EN_PASSANT[pos.pos.en_passant as usize % 8];
        }

        if m.flags == MoveFlag::DOUBLE_PAWN {
            hash ^= zobrist::EN_PASSANT[m.to as usize % 8];
        }

        if piece == PieceType::King {
            if pos.pos.castling[side][0] >= 0 {
                hash ^= zobrist::CASTLING[side][0];
            }
            if pos.pos.castling[side][1] >= 0 {
                hash ^= zobrist::CASTLING[side][1];
            }
        } else if m.from == pos.pos.castling[side][0] as u8 {
            hash ^= zobrist::CASTLING[side][0];
        } else if m.from == pos.pos.castling[side][1] as u8 {
            hash ^= zobrist::CASTLING[side][1];
        }

        match m.flags {
            MoveFlag::CASTLE_LEFT => {
                hash ^= zobrist::PIECES[m.to as usize - 2][side][Rook];
                hash ^= zobrist::PIECES[m.to as usize + 1][side][Rook];
                eval -= PSTS[Rook][flip(m.to - 2, side) as usize];
                eval += PSTS[Rook][flip(m.to + 1, side) as usize];
            }
            MoveFlag::CASTLE_RIGHT => {
                hash ^= zobrist::PIECES[m.to as usize + 1][side][Rook];
                hash ^= zobrist::PIECES[m.to as usize - 1][side][Rook];
                eval -= PSTS[Rook][flip(m.to + 1, side) as usize];
                eval += PSTS[Rook][flip(m.to - 1, side) as usize];
            }
            MoveFlag::EP => {
                let sq = (pos.pos.en_passant
                    + match side {
                        PieceColor::White => -8,
                        PieceColor::Black => 8,
                    }) as u8;

                hash ^= zobrist::PIECES[sq as usize][!side][Pawn];
                eval += PSTS[Pawn][flip(sq, side) as usize];
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

                    hash ^= zobrist::PIECES[m.to as usize][!side][target];
                    eval += PSTS[target][flip(m.to, !side) as usize];

                    if target == Rook {
                        if m.to == pos.pos.castling[!side][0] as u8 {
                            hash ^= zobrist::CASTLING[!side][0];
                        } else if m.to == pos.pos.castling[!side][1] as u8 {
                            hash ^= zobrist::CASTLING[!side][1];
                        }
                    }
                }

                if m.flags & MoveFlag::PROMO != 0 {
                    piece = m.promotion();
                }
            }
        }

        hash ^= zobrist::PIECES[m.to as usize][side][piece];
        eval += PSTS[piece][flip(m.to, side) as usize];
        hash ^= zobrist::BLACK;

        self.repetitions.push(hash);

        return SearchPosition {
            pos: pos.pos.make_move(m),
            eval: -eval,
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
        plies: u8,
        depth: u8,
    ) -> Option<(Move, i32)> {
        if depth == 0 {
            return Some((Move::NULL, pos.eval));
        }

        let in_check = pos.pos.in_check(pos.pos.side);
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

        let mut search_pv = true;

        for m in &self.order_moves(movegen(&pos.pos), tt_move) {
            let pos = self.update(pos, *m);

            if !pos.pos.in_check(!pos.pos.side) {
                n_moves += 1;

                self.nodes += 1;
                if self.nodes % 2048 == 0 && self.timeman.stop() && !(plies == 0 && depth == 1) {
                    return None;
                }

                let res = if self.is_draw(&pos.pos) {
                    (Move::NULL, 0)
                } else {
                    if search_pv {
                        self.negamax(&pos, -beta, -alpha, plies + 1, depth - 1)?
                    } else {
                        let mut res =
                            self.negamax(&pos, -best.1 - 1, -best.1, plies + 1, depth - 1)?;
                        if -res.1 > best.1 {
                            res = self.negamax(&pos, -beta, -alpha, plies + 1, depth - 1)?;
                        }

                        res
                    }
                };

                if -res.1 > best.1 {
                    best = (*m, -res.1);
                    search_pv = false;
                    if res.1 >= beta {
                        self.repetitions.pop();
                        return Some((*m, -res.1));
                    }
                }
            }
            self.repetitions.pop();
        }

        if n_moves == 0 {
            if in_check {
                return Some((Move::NULL, -CHECKMATE + plies as i32));
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
