pub const ASP_DEPTH: u8 = 3;
pub const ASP_WINDOW: i32 = 95;
pub const ASP_INC_FACTOR: f32 = 1.0;

pub const NMP_REDUCTION: u8 = 3;

pub const LMR_BASE: f32 = 1.66;
pub const LMR_DIV: f32 = 3.41;
pub const LMR_NMOVES: u16 = 5;

pub const LMP_DEPTH: u8 = 2;
pub const LMP_BASE: u16 = 3;
pub const LMP_MUL: u16 = 5;
pub const LMP_POW: u32 = 3;

pub const RFP_DEPTH: u8 = 2;
pub const RFP_MARGIN: i32 = 101;

pub const FP_DEPTH: u8 = 5;
pub const FP_BASE: i32 = 252;
pub const FP_MUL: i32 = 493;

pub const TIMEMAN_HARDDIV: f32 = 4.0;
pub const TIMEMAN_SOFTDIV: f32 = 30.0;

#[rustfmt::skip]
pub const MVVLVA_LOOKUP: [[u32; 5]; 6] = [
        /* P   N   B   R   Q */ 
/* P */  [ 9, 11, 11, 13, 17],
/* N */  [ 7,  9,  8, 11, 15],
/* B */  [ 7, 10,  9, 11, 15],
/* R */  [ 5,  7,  7,  9, 13],
/* Q */  [ 1,  3,  3,  5,  9],
/* K */  [ 0,  2,  2,  4,  8],
];
