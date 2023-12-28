pub const ASP_DEPTH: u8 = 4;
pub const ASP_WINDOW: i32 = 114;
pub const ASP_INC_FACTOR: f32 = 0.47;

pub const NMP_REDUCTION: u8 = 3;

pub const LMR_BASE: f32 = 1.13;
pub const LMR_DIV: f32 = 3.25;
pub const LMR_NMOVES: u16 = 5;

pub const LMP_DEPTH: u8 = 2;
pub const LMP_BASE: u16 = 2;
pub const LMP_MUL: u16 = 5;
pub const LMP_POW: u32 = 2;

pub const RFP_DEPTH: u8 = 3;
pub const RFP_MARGIN: i32 = 82;

pub const FP_DEPTH: u8 = 5;
pub const FP_BASE: i32 = 259;
pub const FP_MUL: i32 = 527;

pub const TIMEMAN_HARDDIV: f32 = 3.79;
pub const TIMEMAN_SOFTDIV: f32 = 44.61;

pub const HIST_MUL: i32 = 365;
pub const HIST_ADD: i32 = -427;
pub const CONTHIST_MUL: i32 = 365;
pub const CONTHIST_ADD: i32 = -427;

#[rustfmt::skip]
pub const MVVLVA_LOOKUP: [[i32; 5]; 6] = [
        /* P   N   B   R   Q */ 
/* P */  [ 9, 11, 11, 13, 17],
/* N */  [ 7,  9,  8, 11, 15],
/* B */  [ 7, 10,  9, 11, 15],
/* R */  [ 5,  7,  7,  9, 13],
/* Q */  [ 1,  3,  3,  5,  9],
/* K */  [ 0,  2,  2,  4,  8],
];
