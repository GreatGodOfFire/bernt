use std::{sync::atomic::AtomicBool, time::Duration};

use bernt_movegen::{is_in_check, movegen, MoveList, Moves};
use bernt_position::{piece::PieceType, Move, Position};
use eval::{evaluate, quiesce};
use timecontrol::TimeControl;

pub mod eval;
mod timecontrol;

pub struct SearchState {
    pub position: Position,
    pub limits: Limits,
}

#[derive(Debug, Clone)]
pub struct Limits {
    pub depth: u8,
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u64>,
}

pub const MAX_DEPTH: u8 = 255;
pub const MAX_EVAL: i32 = 1000000;
pub const CHECKMATE: i32 = 100000;

impl Default for Limits {
    fn default() -> Self {
        Self {
            depth: MAX_DEPTH,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            movestogo: None,
        }
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            position: Position::new_empty(),
            limits: Limits::default(),
        }
    }

    pub fn search(&mut self, stop: &AtomicBool, print: bool) -> Option<Move> {
        if let Moves::PseudoLegalMoves(movelist) = movegen(&self.position) {
            let tc = TimeControl::new(&self.limits, self.position.to_move(), stop);
            let mut legal_moves = MoveList::new();

            let mut best = (-MAX_EVAL, Move::null(), MoveList::new());
            let mut nodes = 0;

            for m in &movelist {
                self.position.make_move(m);

                if !is_in_check(&self.position, !self.position.to_move()) {
                    nodes += 1;
                    legal_moves.add(m);
                    self.position.calc_zobrist();
                    if self.position.check_draws() {
                        if best.0 < 0 {
                            best.0 = 0;
                            best.1 = m;
                        }

                        self.position.unmake_move(m);
                        continue;
                    }

                    let score = -evaluate(&self.position);
                    if score > best.0 {
                        best.0 = score;
                        best.1 = m;
                    }
                }

                self.position.unmake_move(m);
            }

            if legal_moves.is_empty() {
                return None;
            } else if legal_moves.len() == 1 {
                return Some(legal_moves[0]);
            }

            if best.0.abs() == CHECKMATE {
                if print {
                    println!("info depth 1 score mate 1 nodes {nodes} pv {}", best.1);
                }
                return Some(best.1);
            }
            if print {
                print_info(1, best.0, best.1, &best.2, tc.start.elapsed(), nodes);
            }

            let mut depth = 2;

            let mut previous_best;

            while !tc.stop() && depth <= self.limits.depth {
                nodes = 0;

                previous_best = best.1;
                best.0 = -MAX_EVAL;

                for m in &legal_moves {
                    self.position.make_move(m);

                    nodes += 1;
                    self.position.calc_zobrist();

                    if self.position.check_draws() {
                        if best.0 < 0 {
                            best.0 = 0;
                            best.1 = m;
                            best.2.clear();
                        }

                        self.position.unmake_move(m);
                        continue;
                    }

                    if let Some((score, pv)) = alpha_beta(
                        &mut self.position,
                        -MAX_EVAL,
                        MAX_EVAL,
                        1,
                        depth - 1,
                        &tc,
                        &mut nodes,
                        self.limits.depth,
                    ) {
                        let score = -score;
                        if score > best.0 {
                            best.0 = score;
                            best.1 = m;
                            best.2 = pv;
                        }
                    } else {
                        self.position.unmake_move(m);
                        return Some(previous_best);
                    }

                    self.position.unmake_move(m);
                }

                if best.0.abs() >= CHECKMATE {
                    let plies = best.0.abs() - CHECKMATE;

                    if print {
                        print!(
                            "info depth {depth} score mate {} time {} nodes {nodes} nps {} pv {}",
                            1 + ((plies - 1) / 2) * best.0.signum(),
                            tc.start.elapsed().as_millis(),
                            (nodes as f64 / tc.start.elapsed().as_secs_f64()) as u64,
                            best.1
                        );
                        for m in (&best.2).into_iter().rev() {
                            print!(" {m}");
                        }
                        println!();
                    }
                    return Some(best.1);
                }
                if print {
                    print_info(depth, best.0, best.1, &best.2, tc.start.elapsed(), nodes);
                }

                depth += 1;
            }

            Some(best.1)
        } else {
            None
        }
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::too_many_arguments)]
fn alpha_beta(
    position: &mut Position,
    alpha: i32,
    beta: i32,
    plies: u8,
    depth: u8,
    tc: &TimeControl,
    nodes: &mut u64,
    max_depth: u8,
) -> Option<(i32, MoveList)> {
    if depth == 0 {
        return Some((
            quiesce(position, plies + 1, alpha, beta, max_depth),
            MoveList::new(),
        ));
    }

    match movegen(position) {
        Moves::PseudoLegalMoves(moves) => {
            let mut score = alpha;

            let mut pv = MoveList::new();
            let mut legal_moves_count = 0;
            let in_check = is_in_check(position, position.to_move());
            for m in &moves {
                position.make_move(m);

                if position.bitboards()[position.to_move()][PieceType::King] != 0
                    && position.bitboards()[!position.to_move()][PieceType::King] != 0
                    && !is_in_check(position, !position.to_move())
                {
                    legal_moves_count += 1;

                    *nodes += 1;
                    if *nodes % 2048 == 0 && tc.stop() {
                        return None;
                    }

                    position.calc_zobrist();
                    if position.check_draws() {
                        if 0 >= beta {
                            position.unmake_move(m);
                            let mut movelist = MoveList::new();
                            movelist.add(m);
                            return Some((beta, movelist));
                        }
                        if 0 > score {
                            score = 0;
                            pv.clear();
                            pv.add(m);
                        }
                        position.unmake_move(m);
                        continue;
                    }

                    let (s, mut _pv) = alpha_beta(
                        position,
                        -beta,
                        -score,
                        plies + 1,
                        depth - 1,
                        tc,
                        nodes,
                        max_depth,
                    )?;
                    _pv.add(m);
                    let s = -s;
                    if s >= beta {
                        position.unmake_move(m);
                        return Some((beta, _pv));
                    }
                    if s > score {
                        score = s;
                        pv = _pv;
                    }
                }

                position.unmake_move(m);
            }

            if legal_moves_count == 0 && in_check {
                Some((-CHECKMATE - plies as i32, pv))
            } else if legal_moves_count == 0 {
                Some((0, pv))
            } else {
                Some((score, pv))
            }
        }
        Moves::Stalemate => Some((0, MoveList::new())),
        Moves::Checkmate => Some((-CHECKMATE - plies as i32, MoveList::new())),
    }
}

fn print_info(depth: u8, score: i32, m: Move, pv: &MoveList, elapsed: Duration, nodes: u64) {
    print!(
        "info depth {depth} score cp {} time {} nodes {nodes} nps {} pv {}",
        score,
        elapsed.as_millis(),
        (nodes as f64 / elapsed.as_secs_f64()) as u64,
        m,
    );
    for m in pv.into_iter().rev() {
        print!(" {m}");
    }
    println!();
}
