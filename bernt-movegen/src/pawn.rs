use bernt_position::{bitboard::BitIter, piece::PieceColor, Move, MoveFlags};

use crate::MoveList;

use super::util::{east, north, south, west, RANK_1, RANK_2, RANK_7, RANK_8};

pub fn single_pawn_moves(pawns: u64, color: PieceColor, empty: u64, movelist: &mut MoveList) {
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
        movelist.add(Move::new(from, to, MoveFlags::Quiet));
    }

    for (from, to) in
        BitIter(movable_pawns & promoting_piece_mask).zip(BitIter(moves & promotions_mask))
    {
        movelist.add(Move::new(from, to, MoveFlags::KnightPromotion));
        movelist.add(Move::new(from, to, MoveFlags::BishopPromotion));
        movelist.add(Move::new(from, to, MoveFlags::RookPromotion));
        movelist.add(Move::new(from, to, MoveFlags::QueenPromotion));
    }
}

pub fn double_pawn_moves(pawns: u64, color: PieceColor, empty: u64, movelist: &mut MoveList) {
    match color {
        PieceColor::White => double_pawn_moves_white(pawns, empty, movelist),
        PieceColor::Black => double_pawn_moves_black(pawns, empty, movelist),
    }
}

fn double_pawn_moves_white(pawns: u64, empty: u64, movelist: &mut MoveList) {
    const RANK: u64 = 0xff << 8;

    let free_pawns = pawns & RANK & empty >> 8 & empty >> 16;
    let moves = free_pawns << 16;

    for (from, to) in BitIter(free_pawns).zip(BitIter(moves)) {
        movelist.add(Move::new(from, to, MoveFlags::DoublePawnPush));
    }
}

fn double_pawn_moves_black(pawns: u64, empty: u64, movelist: &mut MoveList) {
    const RANK: u64 = 0xff << 48;

    let free_pawns = pawns & RANK & empty << 8 & empty << 16;
    let moves = free_pawns >> 16;

    for (from, to) in BitIter(free_pawns).zip(BitIter(moves)) {
        movelist.add(Move::new(from, to, MoveFlags::DoublePawnPush));
    }
}

pub fn pawn_attacks(pawns: u64, color: PieceColor, enemies: u64, movelist: &mut MoveList) {
    for from in BitIter(pawns) {
        for to in BitIter(ATTACKS_LOOKUP[color][from as usize] & enemies & !(RANK_1 | RANK_8)) {
            movelist.add(Move::new(from, to, MoveFlags::Capture));
        }

        // Promotions
        for to in BitIter(ATTACKS_LOOKUP[color][from as usize] & enemies & (RANK_1 | RANK_8)) {
            movelist.add(Move::new(from, to, MoveFlags::KnightPromotionCapture));
            movelist.add(Move::new(from, to, MoveFlags::BishopPromotionCapture));
            movelist.add(Move::new(from, to, MoveFlags::RookPromotionCapture));
            movelist.add(Move::new(from, to, MoveFlags::QueenPromotionCapture));
        }
    }
}

pub fn single_pawn_attacks(pawn: u8, color: PieceColor) -> u64 {
    ATTACKS_LOOKUP[color][pawn as usize]
}

pub fn en_passant(pawns: u64, color: PieceColor, en_passant: i8, movelist: &mut MoveList) {
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
        movelist.add(Move::new(
            from,
            en_passant as u8,
            MoveFlags::EnPassantCapture,
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
