use crate::{
    movegen::{
        bitboard::BitIter,
        king::lookup_king,
        knight::single_knight_moves,
        pawn::single_pawn_attacks,
        sliding::{single_bishop_moves, single_rook_moves},
        util::{RANK_2, RANK_7},
    },
    position::{
        piece::{
            PieceColor,
            PieceType::{self, *},
        },
        Position,
    },
};

const QUEEN_WEIGHT: i32 = 900;
const ROOK_WEIGHT: i32 = 500;
const BISHOP_WEIGHT: i32 = 300;
const KNIGHT_WEIGHT: i32 = 300;
const PAWN_WEIGHT: i32 = 100;
const MOBILITY_WEIGHT: i32 = 10;

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
