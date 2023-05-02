pub const FILE_A: u64 = 0x101010101010101;
pub const FILE_B: u64 = 0x202020202020202;
pub const FILE_C: u64 = 0x404040404040404;
pub const FILE_D: u64 = 0x808080808080808;
pub const FILE_E: u64 = 0x1010101010101010;
pub const FILE_F: u64 = 0x2020202020202020;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;

pub const RANK_1: u64 = 0xff;
pub const RANK_2: u64 = 0xff << 8;
pub const RANK_3: u64 = 0xff << 16;
pub const RANK_4: u64 = 0xff << 24;
pub const RANK_5: u64 = 0xff << 32;
pub const RANK_6: u64 = 0xff << 40;
pub const RANK_7: u64 = 0xff << 48;
pub const RANK_8: u64 = 0xff << 56;

#[inline]
pub const fn north(x: u64) -> u64 {
    (x & !RANK_8) << 8
}

#[inline]
pub const fn south(x: u64) -> u64 {
    (x & !RANK_1) >> 8
}

#[inline]
pub const fn east(x: u64) -> u64 {
    (x & !FILE_H) << 1
}

#[inline]
pub const fn west(x: u64) -> u64 {
    (x & !FILE_A) >> 1
}
