use crate::{
    bitloop,
    position::{Move, MoveFlag, PieceType, Position},
};

use super::{is_attacking, MoveList};

pub fn king_moves<const QUIETS: bool>(
    king: u64,
    empty: u64,
    them: u64,
    mut movelist: &mut MoveList,
) {
    let from = king.trailing_zeros() as u8;
    let moves = LOOKUP[from as usize];

    if QUIETS {
        bitloop!(moves & empty => to, {
            movelist += Move::new(from, to, MoveFlag::QUIET, PieceType::King);
        });
    }

    bitloop!(moves & them => to, {
        movelist += Move::new(from, to, MoveFlag::CAP, PieceType::King);
    });
}

pub fn lookup_king(king: u8) -> u64 {
    LOOKUP[king as usize]
}

pub fn castling_moves(king: u64, empty: u64, pos: &Position, mut movelist: &mut MoveList) {
    let side = pos.side;

    let rank = (!empty >> (side as u8 * 56)) & 0xff;
    let king_square = king.trailing_zeros() as u8;

    if QUEENSIDE_CASTLE & rank == 0
        && pos.castling[side][0] >= 0
        && !is_attacking(king_square - 1, pos, !side)
    {
        movelist += Move::new(
            king_square,
            king_square - 2,
            MoveFlag::CASTLE_LEFT,
            PieceType::King,
        );
    }

    if KINGSIDE_CASTLE & rank == 0
        && pos.castling[side][1] >= 0
        && !is_attacking(king_square + 1, pos, !side)
    {
        movelist += Move::new(
            king_square,
            king_square + 2,
            MoveFlag::CASTLE_RIGHT,
            PieceType::King,
        );
    }
}

// pub fn frc_castling_moves(
//     king: u64,
//     color: PieceColor,
//     empty: u64,
//     position: &Position,
//     movelist: &mut MoveList,
// ) {
//     let king_square = king.trailing_zeros() as u8;
//     let offset = 56 * color as u8;
//     let rank = ((!(empty | king)) >> offset) & 0xff;

//     let rook = position.castling[color][0];
//     if rook >= 0
//         && (rank ^ (1 << (rook % 8)))
//             & (HORIZONTAL_RAY_BETWEEN[king_square as usize % 8][1]
//                 | HORIZONTAL_RAY_BETWEEN[rook as usize % 8][4])
//             == 0
//     {
//         let rook = rook as u8;
//         let diff = king_square - 2 - offset;
//         if diff == 1 {
//             movelist.add(Move::new(
//                 king_square,
//                 rook,
//                 MoveType::LeftCastle,
//                 PieceType::King,
//             ));
//         } else if diff == 2 {
//             if !is_attacking(king_square - 1, position, !color) {
//                 movelist.add(Move::new(
//                     king_square,
//                     rook,
//                     MoveType::LeftCastle,
//                     PieceType::King,
//                 ));
//             }
//         } else {
//             let mut not_attacked = true;
//             for i in 2 + offset..=king_square - 1 {
//                 if is_attacking(i, position, !color) {
//                     not_attacked = false;
//                     break;
//                 }
//             }
//             if not_attacked {
//                 movelist.add(Move::new(
//                     king_square,
//                     rook,
//                     MoveType::LeftCastle,
//                     PieceType::King,
//                 ));
//             }
//         }
//     }

//     let rook = position.castling[color][1];
//     if rook >= 0
//         && (rank ^ (1 << (rook % 8)))
//             & (HORIZONTAL_RAY_BETWEEN[king_square as usize % 8][7]
//                 | HORIZONTAL_RAY_BETWEEN[rook as usize % 8][4])
//             == 0
//     {
//         let rook = rook as u8;
//         let diff = 6 + 56 * color as u8 - king_square;
//         if diff == 1 {
//             movelist.add(Move::new(
//                 king_square,
//                 rook,
//                 MoveType::RightCastle,
//                 PieceType::King,
//             ));
//         } else if diff == 2 {
//             if !is_attacking(king_square + 1, position, !color) {
//                 movelist.add(Move::new(
//                     king_square,
//                     rook,
//                     MoveType::RightCastle,
//                     PieceType::King,
//                 ));
//             }
//         } else {
//             let mut not_attacked = true;
//             for i in king_square + 1..=6 + offset {
//                 if is_attacking(i, position, !color) {
//                     not_attacked = false;
//                     break;
//                 }
//             }
//             if not_attacked {
//                 movelist.add(Move::new(
//                     king_square,
//                     rook,
//                     MoveType::RightCastle,
//                     PieceType::King,
//                 ));
//             }
//         }
//     }
// }

const QUEENSIDE_CASTLE: u64 = 0xe;
const KINGSIDE_CASTLE: u64 = 0x60;

// const HORIZONTAL_RAY_BETWEEN: [[u64; 8]; 8] = generate_horizontal_rays();

// const fn generate_horizontal_rays() -> [[u64; 8]; 8] {
//     let mut lookup = [[0; 8]; 8];

//     let mut start = 0;
//     while start < 8 {
//         let mut end = 0;
//         while end < 8 {
//             if start > end {
//                 let mut i = end + 1;
//                 while i < start {
//                     lookup[start][end] |= 1 << i;
//                     i += 1;
//                 }
//             } else {
//                 let mut i = start + 1;
//                 while i < end {
//                     lookup[start][end] |= 1 << i;
//                     i += 1;
//                 }
//             }

//             end += 1;
//         }

//         start += 1;
//     }

//     lookup
// }

const LOOKUP: [u64; 64] = generate_lookup();

const fn generate_lookup() -> [u64; 64] {
    let mut attacks = [0u64; 64];
    let mut i = 0;

    while i < 64 {
        let mut king = 1u64 << i;

        let mut moves = (king & !FILE_A) >> 1 | (king & !FILE_H) << 1;
        king |= moves;
        moves |= king << 8 | king >> 8;
        attacks[i] = moves;

        i += 1;
    }

    attacks
}

pub const FILE_A: u64 = 0x101010101010101;
pub const FILE_H: u64 = 0x8080808080808080;
