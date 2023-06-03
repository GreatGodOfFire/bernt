use crate::{
    movegen::{
        bitboard::BitIter,
        count_checking,
        king::lookup_king,
        knight::single_knight_moves,
        pawn::single_pawn_attacks,
        pseudo_legal_movegen_captures,
        sliding::{single_bishop_moves, single_rook_moves},
        util::{RANK_2, RANK_7},
        Moves,
    },
    position::{
        piece::{
            PieceColor,
            PieceType::{self, *},
        },
        Position,
    },
};

use super::{timecontrol::TimeControl, MAX_DEPTH};

const QUEEN_WEIGHT: i32 = 900;
const ROOK_WEIGHT: i32 = 500;
const BISHOP_WEIGHT: i32 = 300;
const KNIGHT_WEIGHT: i32 = 300;
const PAWN_WEIGHT: i32 = 100;
const MOBILITY_WEIGHT: i32 = 10;

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
    let w = position.bitboards[0];
    let b = position.bitboards[1];
    let material = QUEEN_WEIGHT * count(w, b, Queen)
        + ROOK_WEIGHT * count(w, b, Rook)
        + BISHOP_WEIGHT * count(w, b, Bishop)
        + KNIGHT_WEIGHT * count(w, b, Knight)
        + PAWN_WEIGHT * count(w, b, Pawn);
    let wmobility = calculate_mobility(
        position.bitboards[PieceColor::White],
        position.bitboards[PieceColor::Black],
        PieceColor::White,
    );
    let bmobility = calculate_mobility(
        position.bitboards[PieceColor::Black],
        position.bitboards[PieceColor::White],
        PieceColor::Black,
    );

    let eval = material + MOBILITY_WEIGHT * (wmobility - bmobility);

    if position.to_move == PieceColor::White {
        eval
    } else {
        -eval
    }
}

fn count(w: [u64; 7], b: [u64; 7], piece: PieceType) -> i32 {
    w[piece].count_ones() as i32 - b[piece].count_ones() as i32
}

fn calculate_mobility(pieces: [u64; 7], opponent: [u64; 7], color: PieceColor) -> i32 {
    let mut n = 0;
    let empty = pieces[Empty] & opponent[Empty];

    for bishop in BitIter(pieces[Bishop] | pieces[Queen]) {
        n += single_bishop_moves(bishop, !pieces[Empty], !opponent[Empty]).count_ones();
    }
    for rook in BitIter(pieces[Rook] | pieces[Queen]) {
        n += single_rook_moves(rook, !pieces[Empty], !opponent[Empty]).count_ones();
    }
    for knight in BitIter(pieces[Knight]) {
        n += single_knight_moves(knight).count_ones();
    }
    for pawn in BitIter(pieces[Pawn]) {
        n += (single_pawn_attacks(pawn, color) & !opponent[Empty]).count_ones();
    }

    let movable_pawns = match color {
        PieceColor::White => pieces[Pawn] & (empty >> 8),
        PieceColor::Black => pieces[Pawn] & (empty << 8),
    };

    let promoting_piece_mask = match color {
        PieceColor::White => RANK_7,
        PieceColor::Black => RANK_2,
    };

    n += (movable_pawns & !promoting_piece_mask).count_ones();
    n += (movable_pawns & promoting_piece_mask).count_ones() * 4;

    n += (lookup_king(pieces[King].trailing_zeros() as u8) & !pieces[Empty]).count_ones();

    n as i32
}
