use bernt_movegen::{is_in_check, pseudo_legal_movegen_captures, Moves};
use bernt_position::{
    piece::{
        PieceColor,
        PieceType::{self, *},
    },
    Position,
};

use crate::MAX_DEPTH;

pub mod pst;

pub fn quiesce(position: &mut Position, plies: u8, alpha: i32, beta: i32) -> i32 {
    let mut alpha = alpha;
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
        for m in &captures {
            position.make_move(m);
            if !is_in_check(position, !position.to_move()) {
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

#[inline]
pub fn evaluate(position: &Position) -> i32 {
    // let mut opening = 0;
    let mut midgame = 0;
    let mut endgame = 0;

    let mut phase = 0;

    for sq in 0..64 {
        let piece = position.mailbox()[sq];
        if piece.ty != PieceType::Empty {
            phase += GAMEPHASE_INC[piece.ty];
            if piece.color == PieceColor::White {
                // opening += tables.opening[piece.ty][sq];
                midgame += pst::MIDGAME[piece.ty][sq];
                endgame += pst::ENDGAME[piece.ty][sq];
            } else {
                // opening -= tables.opening[piece.ty][flip(sq)];
                midgame -= pst::MIDGAME[piece.ty][flip(sq)];
                endgame -= pst::ENDGAME[piece.ty][flip(sq)];
            }
        }
    }

    let phase = phase.min(max_phase());
    let eval = (midgame * phase + (max_phase() - phase) * endgame) / max_phase();

    // let phase = 256 - phase.min(256);
    // let eval = ((opening * (128 - phase).max(0) * 2)
    //     + (midgame * (-(phase - 128).abs() * 2 + 256))
    //     + (endgame * (phase - 128).max(0) * 2))
    //     / 256;

    if position.to_move() == PieceColor::White {
        eval
    } else {
        -eval
    }
}

pub const GAMEPHASE_INC: [i32; 6] = [4, 4, 8, 16, 0, 0];

const fn max_phase() -> i32 {
    GAMEPHASE_INC[Pawn as usize] * 16
        + GAMEPHASE_INC[Knight as usize] * 4
        + GAMEPHASE_INC[Bishop as usize] * 4
        + GAMEPHASE_INC[Rook as usize] * 4
        + GAMEPHASE_INC[Queen as usize] * 2
}

#[inline]
pub fn flip(sq: usize) -> usize {
    sq ^ 56
}
