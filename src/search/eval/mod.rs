use crate::{
    movegen::{pseudo_legal_movegen_captures, Moves},
    position::{piece::PieceType, Position},
};

mod pst;

use self::pst::{EG_TABLE, MG_TABLE};

use super::{timecontrol::TimeControl, MAX_DEPTH};

pub fn quiesce(
    position: &mut Position,
    plies: u8,
    mut alpha: i32,
    beta: i32,
    tc: &TimeControl,
    nodes: &mut u64,
) -> Option<i32> {
    let eval = evaluate(position);
    if eval >= beta {
        return Some(beta);
    }
    if alpha < eval {
        alpha = eval;
    }

    if plies >= MAX_DEPTH {
        return Some(alpha);
    }

    let captures = pseudo_legal_movegen_captures(position);
    match captures {
        Moves::PseudoLegalMoves(captures) => {
            for m in captures {
                position.make_move(m);
                if !position.is_in_check(!position.to_move) {
                    *nodes += 1;
                    if *nodes % 4096 == 0 {
                        if tc.stop() {
                            return None;
                        }
                    }
                    let eval = -quiesce(position, plies + 1, -beta, -alpha, tc, nodes)?;

                    if eval >= beta {
                        position.unmake_move(m);
                        return Some(beta);
                    }
                    if eval > alpha {
                        alpha = eval;
                    }
                }
                position.unmake_move(m);
            }
        }
        _ => {}
    }

    Some(alpha)
}

pub fn evaluate(position: &Position) -> i32 {
    let mut mg = [0; 2];
    let mut eg = [0; 2];
    let mut game_phase = 0;

    for sq in 0..64 {
        let piece = position.mailbox[sq];
        if piece.ty != PieceType::Empty {
            mg[piece.color] += MG_TABLE[piece][sq];
            eg[piece.color] += EG_TABLE[piece][sq];
            game_phase += GAMEPHASE_INC[piece.ty];
        }
    }

    let mg_score = mg[position.to_move] - mg[!position.to_move];
    let eg_score = eg[position.to_move] - eg[!position.to_move];
    let mg_phase = game_phase.min(24);
    let eg_phase = 24 - mg_phase;
    let eval = (mg_score * mg_phase + eg_score * eg_phase) / 24;

    eval
}

const GAMEPHASE_INC: [i32; 6] = [1, 1, 2, 4, 0, 0];
