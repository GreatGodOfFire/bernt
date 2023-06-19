#![allow(unused)]

use bernt_position::bitboard::Bitboard;

pub const FILE_A: Bitboard = Bitboard(0x101010101010101);
pub const FILE_B: Bitboard = Bitboard(0x202020202020202);
pub const FILE_C: Bitboard = Bitboard(0x404040404040404);
pub const FILE_D: Bitboard = Bitboard(0x808080808080808);
pub const FILE_E: Bitboard = Bitboard(0x1010101010101010);
pub const FILE_F: Bitboard = Bitboard(0x2020202020202020);
pub const FILE_G: Bitboard = Bitboard(0x4040404040404040);
pub const FILE_H: Bitboard = Bitboard(0x8080808080808080);

pub const RANK_1: Bitboard = Bitboard(0xff);
pub const RANK_2: Bitboard = Bitboard(0xff << 8);
pub const RANK_3: Bitboard = Bitboard(0xff << 16);
pub const RANK_4: Bitboard = Bitboard(0xff << 24);
pub const RANK_5: Bitboard = Bitboard(0xff << 32);
pub const RANK_6: Bitboard = Bitboard(0xff << 40);
pub const RANK_7: Bitboard = Bitboard(0xff << 48);
pub const RANK_8: Bitboard = Bitboard(0xff << 56);

#[inline]
pub const fn north(x: Bitboard) -> Bitboard {
    Bitboard((x.0 & !RANK_8.0) << 8)
}

#[inline]
pub const fn south(x: Bitboard) -> Bitboard {
    Bitboard((x.0 & !RANK_1.0) >> 8)
}

#[inline]
pub const fn east(x: Bitboard) -> Bitboard {
    Bitboard((x.0 & !FILE_H.0) << 1)
}

#[inline]
pub const fn west(x: Bitboard) -> Bitboard {
    Bitboard((x.0 & !FILE_A.0) >> 1)
}
