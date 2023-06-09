use crate::{
    movegen::{pseudo_legal_movegen_captures, Moves},
    position::{piece::PieceType, Position},
};

mod pst;

use self::pst::{EG_TABLE, MG_TABLE};

use super::MAX_DEPTH;

pub fn quiesce(
    position: &mut Position,
    plies: u8,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    let eval = evaluate(position);
    if eval >= beta {
        return beta;
    }
    if alpha < eval {
        alpha = eval;
    }

    if plies >= MAX_DEPTH {
        return alpha;
    }

    let captures = pseudo_legal_movegen_captures(position);
    if let Moves::PseudoLegalMoves(captures) = captures {
        for m in captures {
            position.make_move(m);
            if !position.is_in_check(!position.to_move) {
                let eval = -quiesce(position, plies + 1, -beta, -alpha);

                if eval >= beta {
                    position.unmake_move(m);
                    return beta;
                }
                if eval > alpha {
                    alpha = eval;
                }
            }
            position.unmake_move(m);
        }
    }

    alpha
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

    (mg_score * mg_phase + eg_score * eg_phase) / 24
}

const GAMEPHASE_INC: [i32; 6] = [1, 1, 2, 4, 0, 0];
