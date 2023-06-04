use std::time::Duration;

use crate::{
    movegen::{self, movegen, Move, Moves},
    position::{
        tt::{TTIndex, TTIndexType},
        Position,
    },
    uci::Limits,
};

use self::{
    eval::{evaluate, quiesce},
    timecontrol::TimeControl,
};

mod eval;
mod timecontrol;

const MAX_DEPTH: u8 = 10;

pub fn start_search(position: &mut Position, time: Limits) -> Move {
    if let movegen::Moves::PseudoLegalMoves(moves) = movegen(position) {
        let tc = TimeControl::new(&time, position.to_move);

        let mut legal_moves = vec![];

        let mut best = (MIN_EVAL, Move::null(), vec![]);
        let max_depth = time.depth.unwrap_or(MAX_DEPTH);
        let mut nodes = 0;

        // first depth 1
        for &m in &moves {
            position.make_move(m);

            if !position.is_in_check(!position.to_move) {
                nodes += 1;
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

        if best.0.abs() == CHECKMATE {
            println!(
                "info depth 1 score mate 1 time {} nodes {nodes} nps {} pv {:?}",
                tc.start.elapsed().as_millis(),
                (nodes as f64 / tc.start.elapsed().as_secs_f64()) as u64,
                best.1
            );
            return best.1;
        }
        print_info(
            1,
            best.0,
            best.1,
            &best.2,
            tc.start.elapsed(),
            nodes,
            position.tt.hashfull(),
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

            for &m in &legal_moves {
                position.make_move(m);

                nodes += 1;
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
                    alpha_beta(position, MIN_EVAL, MAX_EVAL, 1, depth - 1, &tc, &mut nodes)
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
            print_info(
                depth,
                best.0,
                best.1,
                &best.2,
                tc.start.elapsed(),
                nodes,
                position.tt.hashfull(),
            );

            depth += 1;
            let idx = legal_moves.iter().position(|&m| m == best.1).unwrap();
            legal_moves.swap(idx, 0);
        }

        best.1
    } else {
        unreachable!()
    }
}

fn print_info(
    depth: u8,
    score: i32,
    m: Move,
    pv: &[Move],
    elapsed: Duration,
    nodes: u64,
    hashfull: usize,
) {
    print!(
        "info depth {depth} score cp {} time {} hashfull {hashfull} nodes {nodes} nps {} pv {:?}",
        score,
        elapsed.as_millis(),
        (nodes as f64 / elapsed.as_secs_f64()) as u64,
        m,
    );
    for m in pv.iter().rev() {
        print!(" {m:?}");
    }
    println!();
}

const MAX_EVAL: i32 = 2000000000;
const MIN_EVAL: i32 = -2000000000;
const CHECKMATE: i32 = 100000;

fn alpha_beta(
    position: &mut Position,
    alpha: i32,
    beta: i32,
    plies: u8,
    depth: u8,
    tc: &TimeControl,
    nodes: &mut u64,
) -> Option<(i32, Vec<Move>)> {
    if depth == 0 {
        return Some((quiesce(position, plies, alpha, beta, tc, nodes)?, vec![]));
    }

    match movegen(position) {
        Moves::PseudoLegalMoves(moves) => {
            let mut score = alpha;

            if let Some((_m, eval, d, ty)) = position
                .tt
                .lookup(position.zobrist(), position.fullmove_clock)
            {
                if d >= depth
                    && (ty == TTIndexType::Exact
                        || ty == TTIndexType::Lower && eval >= beta
                        || ty == TTIndexType::Upper && alpha >= eval)
                {
                    return Some((eval, vec![]));
                }
            }

            let mut pv = vec![];
            let mut legal_moves_count = 0;
            let in_check = position.is_in_check(position.to_move);
            for m in moves {
                position.make_move(m);

                if !position.is_in_check(!position.to_move) {
                    legal_moves_count += 1;

                    *nodes += 1;
                    if *nodes % 4096 == 0 && tc.stop() {
                        return None;
                    }

                    position.calc_zobrist();
                    if position.check_draws() {
                        if 0 >= beta {
                            position.unmake_move(m);
                            return Some((beta, vec![m]));
                        }
                        if 0 > score {
                            score = 0;
                            pv = vec![m];
                        }
                        position.unmake_move(m);
                        continue;
                    }

                    let (s, mut _pv) =
                        alpha_beta(position, -beta, -score, plies + 1, depth - 1, tc, nodes)?;
                    _pv.push(m);
                    let s = -s;
                    if s >= beta {
                        position.unmake_move(m);
                        position.tt.insert(TTIndex::new(
                            position.zobrist(),
                            m,
                            s,
                            depth,
                            position.fullmove_clock,
                            TTIndexType::Lower,
                        ));
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
                Some((-CHECKMATE, pv))
            } else if legal_moves_count == 0 {
                Some((0, pv))
            } else {
                if !pv.is_empty() {
                    let ty = if score >= beta {
                        TTIndexType::Lower
                    } else if score <= alpha {
                        TTIndexType::Upper
                    } else {
                        TTIndexType::Exact
                    };

                    position.tt.insert(TTIndex::new(
                        position.zobrist(),
                        pv[0],
                        score,
                        depth,
                        position.fullmove_clock,
                        ty,
                    ));
                }
                Some((score, pv))
            }
        }
        Moves::Stalemate => Some((0, vec![])),
        Moves::Checkmate => Some((-CHECKMATE, vec![])),
    }
}
