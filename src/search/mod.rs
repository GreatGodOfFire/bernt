use crate::{
    movegen::{self, movegen, Move, Moves},
    position::{piece::PieceColor, Position},
    uci::Limits,
};

use self::{eval::evaluate, timecontrol::TimeControl};

mod eval;
mod timecontrol;

const MAX_DEPTH: u8 = 10;

pub fn start_search(position: &mut Position, time: Limits) -> Move {
    if let movegen::Moves::PseudoLegalMoves(moves) = movegen(position) {
        let tc = TimeControl::new(&time, position.to_move);

        let mut legal_moves = vec![];

        let mut best = (MIN_EVAL, Move::null(), vec![]);
        let max_depth = time.depth.unwrap_or(MAX_DEPTH);

        // first depth 1
        for &m in &moves {
            position.make_move(m);

            if !position.is_in_check(!position.to_move) {
                legal_moves.push(m);
                position.calc_zobrist();
                if position.check_draws() {
                    if best.0 < 0 {
                        best.0 = 0;
                        best.1 = m;
                    }

                    position.unmake_move(m);
                    continue;
                }

                let score = -evaluate(position);
                if score > best.0 {
                    best.0 = score;
                    best.1 = m;
                }
            }

            position.unmake_move(m);
        }
        println!(
            "info depth 1 score cp {} time {} nodes {} nps {} pv {:?}",
            best.0,
            tc.start.elapsed().as_millis(),
            legal_moves.len(),
            (legal_moves.len() as f64 / tc.start.elapsed().as_secs_f64()) as u64,
            best.1
        );

        let idx = legal_moves.iter().position(|&m| m == best.1).unwrap();
        legal_moves.swap(idx, 0);

        let mut depth = 2;

        let mut previous_best;

        if legal_moves.len() == 1 {
            return legal_moves[0];
        }

        while !tc.stop() && depth <= max_depth {
            previous_best = best.clone();
            best.0 = MIN_EVAL;

            let mut nodes = 0;

            for &m in &legal_moves {
                position.make_move(m);

                if !position.is_in_check(!position.to_move) {
                    position.calc_zobrist();
                    if position.check_draws() {
                        if best.0 < 0 {
                            best.0 = 0;
                            best.1 = m;
                            best.2 = vec![];
                        }

                        position.unmake_move(m);
                        continue;
                    }

                    if let Some((score, pv)) =
                        alpha_beta(MIN_EVAL, MAX_EVAL, depth - 1, position, &tc, &mut nodes)
                    {
                        let score = -score;
                        if score > best.0 {
                            best.0 = score;
                            best.1 = m;
                            best.2 = pv;
                        }
                    } else {
                        return previous_best.1;
                    }
                }

                position.unmake_move(m);
            }

            if best.0.abs() == CHECKMATE {
                print!(
                    "info depth {depth} score mate {} time {} nodes {nodes} nps {} pv {:?}",
                    depth.div_ceil(2) as i32 * best.0.signum(),
                    tc.start.elapsed().as_millis(),
                    (nodes as f64 / tc.start.elapsed().as_secs_f64()) as u64,
                    best.1
                );
                for m in best.2.iter().rev() {
                    print!(" {m:?}");
                }
                println!();
                return best.1;
            }

            print!(
                "info depth {depth} score cp {} time {} nodes {nodes} nps {} pv {:?}",
                best.0,
                tc.start.elapsed().as_millis(),
                (nodes as f64 / tc.start.elapsed().as_secs_f64()) as u64,
                best.1
            );
            for m in best.2.iter().rev() {
                print!(" {m:?}");
            }
            println!();
            depth += 1;
            let idx = legal_moves.iter().position(|&m| m == best.1).unwrap();
            legal_moves.swap(idx, 0);
        }

        best.1
    } else {
        unreachable!()
    }
}

const MAX_EVAL: i32 = 2000000000;
const MIN_EVAL: i32 = -2000000000;
const CHECKMATE: i32 = 100000;

fn alpha_beta(
    mut alpha: i32,
    beta: i32,
    depth: u8,
    position: &mut Position,
    tc: &TimeControl,
    nodes: &mut u64,
) -> Option<(i32, Vec<Move>)> {
    if depth == 0 {
        return Some((evaluate(position), vec![]));
    }
    match movegen(position) {
        Moves::PseudoLegalMoves(moves) => {
            let mut pv = vec![];
            let in_check = position.is_in_check(position.to_move);
            for m in moves {
                position.make_move(m);

                if !position.is_in_check(!position.to_move) {
                    *nodes += 1;
                    if *nodes % 4096 == 0 {
                        if tc.stop() {
                            return None;
                        }
                    }
                    position.calc_zobrist();
                    if position.check_draws() {
                        if 0 >= beta {
                            position.unmake_move(m);
                            return Some((0, vec![m]));
                        }
                        if 0 > alpha {
                            alpha = 0;
                            pv = vec![m];
                        }
                    }
                    let (score, mut _pv) =
                        alpha_beta(-beta, -alpha, depth - 1, position, tc, nodes)?;
                    _pv.push(m);
                    let score = -score;
                    if score >= beta {
                        position.unmake_move(m);
                        return Some((beta, _pv));
                    }
                    if score > alpha {
                        alpha = score;
                        pv = _pv;
                    }
                }

                position.unmake_move(m);
            }

            if pv.is_empty() && in_check {
                Some((-CHECKMATE, vec![]))
            } else if alpha == MIN_EVAL {
                Some((0, vec![]))
            } else {
                Some((alpha, pv))
            }
        }
        Moves::Stalemate => Some((0, vec![])),
        Moves::Checkmate => Some((-CHECKMATE, vec![])), 
    }
}
