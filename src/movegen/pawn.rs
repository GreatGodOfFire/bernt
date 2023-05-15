use crate::position::piece::PieceColor;

use super::{
    bitboard::BitIter,
    util::{east, north, south, west, RANK_1, RANK_2, RANK_7, RANK_8},
    Code, Move,
};

pub fn single_pawn_moves(pawns: u64, color: PieceColor, empty: u64, out: &mut Vec<Move>) {
    let movable_pawns = match color {
        PieceColor::White => pawns & (empty >> 8),
        PieceColor::Black => pawns & (empty << 8),
    };
    let moves = match color {
        PieceColor::White => movable_pawns << 8,
        PieceColor::Black => movable_pawns >> 8,
    } & empty;

    let (promoting_piece_mask, promotions_mask) = match color {
        PieceColor::White => (RANK_7, RANK_8),
        PieceColor::Black => (RANK_2, RANK_1),
    };

    for (from, to) in
        BitIter(movable_pawns & !promoting_piece_mask).zip(BitIter(moves & !promotions_mask))
    {
        out.push(Move::new(from as u16, to as u16, Code::Quiet));
    }

    for (from, to) in
        BitIter(movable_pawns & promoting_piece_mask).zip(BitIter(moves & promotions_mask))
    {
        out.push(Move::new(from as u16, to as u16, Code::KnightPromotion));
        out.push(Move::new(from as u16, to as u16, Code::BishopPromotion));
        out.push(Move::new(from as u16, to as u16, Code::RookPromotion));
        out.push(Move::new(from as u16, to as u16, Code::QueenPromotion));
    }
}

pub fn double_pawn_moves(pawns: u64, color: PieceColor, empty: u64, out: &mut Vec<Move>) {
    match color {
        PieceColor::White => double_pawn_moves_white(pawns, empty, out),
        PieceColor::Black => double_pawn_moves_black(pawns, empty, out),
    }
}

fn double_pawn_moves_white(pawns: u64, empty: u64, out: &mut Vec<Move>) {
    const RANK: u64 = 0xff << 8;

    let free_pawns = pawns & RANK & empty >> 8 & empty >> 16;
    let moves = free_pawns << 16;

    for (from, to) in BitIter(free_pawns).zip(BitIter(moves)) {
        out.push(Move::new(from as u16, to as u16, Code::DoublePawnPush));
    }
}

fn double_pawn_moves_black(pawns: u64, empty: u64, out: &mut Vec<Move>) {
    const RANK: u64 = 0xff << 48;

    let free_pawns = pawns & RANK & empty << 8 & empty << 16;
    let moves = free_pawns >> 16;

    for (from, to) in BitIter(free_pawns).zip(BitIter(moves)) {
        out.push(Move::new(from as u16, to as u16, Code::DoublePawnPush));
    }
}

pub fn pawn_attacks(pawns: u64, color: PieceColor, enemies: u64, out: &mut Vec<Move>) {
    for from in BitIter(pawns) {
        for to in BitIter(ATTACKS_LOOKUP[color][from as usize] & enemies & !(RANK_1 | RANK_8)) {
            out.push(Move::new(from as u16, to as u16, Code::Capture));
        }

        // Promotions
        for to in BitIter(ATTACKS_LOOKUP[color][from as usize] & enemies & (RANK_1 | RANK_8)) {
            out.push(Move::new(
                from as u16,
                to as u16,
                Code::KnightPromotionCapture,
            ));
            out.push(Move::new(
                from as u16,
                to as u16,
                Code::BishopPromotionCapture,
            ));
            out.push(Move::new(
                from as u16,
                to as u16,
                Code::RookPromotionCapture,
            ));
            out.push(Move::new(
                from as u16,
                to as u16,
                Code::QueenPromotionCapture,
            ));
        }
    }
}

pub fn single_pawn_attacks(pawn: u8, color: PieceColor) -> u64 {
    ATTACKS_LOOKUP[color][pawn as usize]
}

pub fn en_passant(pawns: u64, color: PieceColor, en_passant: i8, out: &mut Vec<Move>) {
    if en_passant < 0 {
        return;
    }

    let shift = match color {
        PieceColor::White => south,
        PieceColor::Black => north,
    };

    let bit = 1 << (en_passant as u8);
    let pawns = (shift(east(bit)) | shift(west(bit))) & pawns;

    for from in BitIter(pawns) {
        out.push(Move::new(
            from as u16,
            en_passant as u16,
            Code::EnPassantCapture,
        ));
    }
}

const ATTACKS_LOOKUP: [[u64; 64]; 2] = generate_attacks_lookup();

const fn generate_attacks_lookup() -> [[u64; 64]; 2] {
    let mut attacks = [[0; 64]; 2];

    let mut i = 0;
    while i < 64 {
        let bit = 1u64 << i;
        attacks[0][i] = north(east(bit)) | north(west(bit));
        attacks[1][i] = south(east(bit)) | south(west(bit));

        i += 1;
    }

    attacks
}
