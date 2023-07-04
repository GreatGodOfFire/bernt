use std::{
    fs::File,
    io::{stdin, stdout, Write},
    sync::atomic::AtomicBool,
};

use bernt_movegen::{is_in_check, movegen, MoveList, Moves};
use bernt_position::{
    bitboard::print_bitboard,
    piece::{PieceColor, PieceType},
    Move, MoveFlags, Position,
};
use bernt_search::{eval::quiesce, SearchState, CHECKMATE, MAX_DEPTH, MAX_EVAL};

const DEPTH: u8 = 5;

fn main() {
    // print!("Number of positions: ");
    // stdout().flush().unwrap();
    // let mut n_pos = String::new();
    // stdin().read_line(&mut n_pos).unwrap();
    // let n_pos: u32 = n_pos.trim().parse().unwrap();
    let n_pos: u32 = 1000;

    // print!("Output file: ");
    // stdout().flush().unwrap();
    // let mut output = String::new();
    // stdin().read_line(&mut output).unwrap();
    let output = "out.txt";

    let mut i = 0;

    let mut output = File::create(output.trim()).unwrap();

    'games: while i < n_pos {
        let n_moves = fastrand::usize(6..=10);

        let mut state = SearchState::new();
        state.position = Position::startpos();

        for _ in 0..n_moves {
            let moves = movegen(&state.position);

            match moves {
                bernt_movegen::Moves::PseudoLegalMoves(moves) => {
                    let m = fastrand::choice(moves.into_iter()).unwrap();
                    state.position.calc_zobrist();
                    state.position.make_move(m);
                    let to_move = state.position.to_move();
                    if is_in_check(&state.position, !to_move) {
                        continue 'games;
                    }
                }
                _ => continue 'games,
            }
        }

        let s = evaluate(&mut state.position, -MAX_EVAL, MAX_EVAL, 0, DEPTH);

        if s.abs() > 1000 || s.abs() < 200 {
            continue;
        }

        let mut positions = vec![];

        state.position.calc_zobrist();
        state.limits.depth = DEPTH;

        println!("{}", state.position.as_fen(false));
        let res = loop {
            print_bitboard(state.position.bitboards()[state.position.to_move()][PieceType::King]);
            if let Some(m) = state.search(&AtomicBool::new(false), false) {
                println!("{m}:{:?}", m.flags);
                if m.flags == MoveFlags::Quiet && i < n_pos {
                    positions.push(state.position.as_fen(false));
                    i += 1;
                }
                state.position.make_move(m);
                state.position.calc_zobrist();
                if state.position.check_draws() || state.position.fullmove_clock() >= 300 {
                    break 0.5;
                }
                state.position.finalize_moves();
            } else {
                if is_in_check(&state.position, state.position.to_move()) {
                    if state.position.to_move() == PieceColor::White {
                        break 0.0;
                    } else {
                        break 1.0;
                    }
                } else {
                    break 0.5;
                }
            }
        };

        for pos in positions {
            output.write_fmt(format_args!("{pos}:{res:.1}\n")).unwrap();
        }

        i += 1;
    }
}

fn evaluate(position: &mut Position, alpha: i32, beta: i32, plies: u8, depth: u8) -> i32 {
    if depth == 0 {
        return quiesce(position, plies, alpha, beta, MAX_DEPTH);
    }

    match movegen(position) {
        Moves::PseudoLegalMoves(moves) => {
            let mut score = alpha;

            let mut pv = MoveList::new();
            let mut legal_moves_count = 0;
            let in_check = is_in_check(position, position.to_move());
            for m in &moves {
                position.make_move(m);

                if !is_in_check(position, !position.to_move()) {
                    legal_moves_count += 1;

                    position.calc_zobrist();
                    if position.check_draws() {
                        if 0 >= beta {
                            position.unmake_move(m);
                            let mut movelist = MoveList::new();
                            movelist.add(m);
                            return beta;
                        }
                        if 0 > score {
                            score = 0;
                            pv.clear();
                            pv.add(m);
                        }
                        position.unmake_move(m);
                        continue;
                    }

                    let s = -evaluate(position, -beta, -score, plies + 1, depth - 1);
                    if s >= beta {
                        position.unmake_move(m);
                        return beta;
                    }
                    if s > score {
                        score = s;
                    }
                }

                position.unmake_move(m);
            }

            if legal_moves_count == 0 && in_check {
                -CHECKMATE
            } else if legal_moves_count == 0 {
                0
            } else {
                score
            }
        }
        Moves::Stalemate => 0,
        Moves::Checkmate => -CHECKMATE,
    }
}
