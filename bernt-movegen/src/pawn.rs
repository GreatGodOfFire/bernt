use bernt_position::{bitboard::Bitboard, Move, MoveFlags};

use crate::{
    flags,
    util::{east, north, south, west, RANK_1, RANK_2, RANK_7, RANK_8},
    MoveList,
};

pub fn pawn_moves<const IS_WHITE: bool, const FLAGS: u8>(
    pawns: Bitboard,
    empty: Bitboard,
    enemies: Bitboard,
    en_passant: i8,
    movelist: &mut MoveList,
) {
    if FLAGS & flags::QUIET != 0 {
        single_pawn_moves::<IS_WHITE, FLAGS>(pawns, empty, movelist);
        double_pawn_moves::<IS_WHITE>(pawns, empty, movelist);
    } else if FLAGS & flags::PROMOTIONS != 0 {
        single_pawn_moves::<IS_WHITE, FLAGS>(pawns, empty, movelist);
    }
    if FLAGS & flags::CAPTURES != 0 {
        pawn_attacks::<IS_WHITE, FLAGS>(pawns, enemies, movelist);
        en_passant_capture::<IS_WHITE>(pawns, en_passant, movelist);
    }
    // if FLAGS & flags::QUIET != 0 {
    //     single_pawn_moves::<IS_WHITE, FLAGS>(pawns, empty, movelist);
    // }
}

fn single_pawn_moves<const IS_WHITE: bool, const FLAGS: u8>(
    pawns: Bitboard,
    empty: Bitboard,
    movelist: &mut MoveList,
) {
    let pawns = pawns & if IS_WHITE { empty >> 8 } else { empty << 8 };
    let moves = if IS_WHITE { pawns << 8 } else { pawns >> 8 };
    let (promoting, promotion) = if IS_WHITE {
        (RANK_7, RANK_8)
    } else {
        (RANK_2, RANK_1)
    };

    if FLAGS & flags::QUIET != 0 {
        for (from, to) in (pawns & !promoting).zip(moves & !promotion) {
            movelist.add(Move::new(from, to, MoveFlags::Quiet));
        }
    }

    if FLAGS & flags::PROMOTIONS != 0 {
        for (from, to) in (pawns & promoting).zip(moves & promotion) {
            movelist.add(Move::new(from, to, MoveFlags::QueenPromotion));
            movelist.add(Move::new(from, to, MoveFlags::RookPromotion));
            movelist.add(Move::new(from, to, MoveFlags::BishopPromotion));
            movelist.add(Move::new(from, to, MoveFlags::KnightPromotion));
        }
    }
}

fn double_pawn_moves<const IS_WHITE: bool>(
    pawns: Bitboard,
    empty: Bitboard,
    movelist: &mut MoveList,
) {
    let rank = Bitboard(if IS_WHITE { 0xff << 8 } else { 0xff << 48 });

    let free_pawns = if IS_WHITE {
        pawns & rank & empty >> 8 & empty >> 16
    } else {
        pawns & rank & empty << 8 & empty << 16
    };
    let moves = if IS_WHITE {
        free_pawns << 16
    } else {
        free_pawns >> 16
    };

    for (from, to) in free_pawns.zip(moves) {
        movelist.add(Move::new(from, to, MoveFlags::DoublePawnPush));
    }
}

fn pawn_attacks<const IS_WHITE: bool, const FLAGS: u8>(
    pawns: Bitboard,
    enemies: Bitboard,
    movelist: &mut MoveList,
) {
    for from in pawns {
        for to in ATTACKS_LOOKUP[!IS_WHITE as usize][from as usize] & enemies & !(RANK_1 | RANK_8) {
            movelist.add(Move::new(from, to, MoveFlags::Capture));
        }

        if FLAGS & flags::PROMOTIONS != 0 {
            for to in
                ATTACKS_LOOKUP[!IS_WHITE as usize][from as usize] & enemies & (RANK_1 | RANK_8)
            {
                movelist.add(Move::new(from, to, MoveFlags::QueenPromotionCapture));
                movelist.add(Move::new(from, to, MoveFlags::RookPromotionCapture));
                movelist.add(Move::new(from, to, MoveFlags::BishopPromotionCapture));
                movelist.add(Move::new(from, to, MoveFlags::KnightPromotionCapture));
            }
        }
    }
}

pub fn single_pawn_attacks<const IS_WHITE: bool>(pawn: u8) -> Bitboard {
    ATTACKS_LOOKUP[!IS_WHITE as usize][pawn as usize]
}

fn en_passant_capture<const IS_WHITE: bool>(pawns: Bitboard, en_passant: i8, movelist: &mut MoveList) {
    if en_passant < 0 {
        return;
    }

    let shift = if IS_WHITE { south } else { north };

    let bit = Bitboard(1 << (en_passant as u8));
    let pawns = (shift(east(bit)) | shift(west(bit))) & pawns;

    for from in pawns {
        movelist.add(Move::new(
            from,
            en_passant as u8,
            MoveFlags::EnPassantCapture,
        ));
    }
}

const ATTACKS_LOOKUP: [[Bitboard; 64]; 2] = generate_attacks_lookup();

const fn generate_attacks_lookup() -> [[Bitboard; 64]; 2] {
    let mut attacks = [[Bitboard(0); 64]; 2];

    let mut i = 0;
    while i < 64 {
        let bit = Bitboard(1u64 << i);
        // Lets hope they will make const impl possible again -_-
        attacks[0][i] = Bitboard(north(east(bit)).0 | north(west(bit)).0);
        attacks[1][i] = Bitboard(south(east(bit)).0 | south(west(bit)).0);

        i += 1;
    }

    attacks
}
