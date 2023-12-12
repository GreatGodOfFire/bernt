use crate::{
    bitloop,
    position::{PieceColor, PieceType, Position},
};

pub fn eval(pos: &Position) -> (i32, i32, i32, i32) {
    let mut mg = 0;
    let mut eg = 0;
    let mut phase = 0;

    bitloop!(pos.colors[0] | pos.colors[1] => sq, {
        let piece = pos.piece_at(sq);

        if piece.ty != PieceType::None {
            let sign = if piece.color == pos.side { 1 } else { -1 };
            mg += MG_PSTS[piece.ty][flip(sq, piece.color) as usize] * sign;
            eg += EG_PSTS[piece.ty][flip(sq, piece.color) as usize] * sign;
            phase += PHASE[piece.ty];
        }
    });
    phase = phase.min(24);

    ((mg * phase + eg * (24 - phase)) / 24, mg, eg, phase)
}

#[inline]
pub(super) fn flip(sq: u8, side: PieceColor) -> u8 {
    if side == PieceColor::White {
        sq
    } else {
        sq ^ 56
    }
}

pub(super) const PHASE: [i32; 6] = [0, 1, 1, 2, 4, 0];

pub(super) const MG_PSTS: [[i32; 64]; 6] =
    [MG_PAWN, MG_KNIGHT, MG_BISHOP, MG_ROOK, MG_QUEEN, MG_KING];
pub(super) const EG_PSTS: [[i32; 64]; 6] =
    [EG_PAWN, EG_KNIGHT, EG_BISHOP, EG_ROOK, EG_QUEEN, EG_KING];

#[rustfmt::skip]
const MG_PAWN: [i32; 64] = [
       0,    0,    0,    0,    0,    0,    0,    0,
      38,   63,   53,   51,   50,   89,  122,   58,
      47,   73,   58,   61,   71,   67,  112,   68,
      42,   69,   64,   82,   81,   82,   91,   60,
      53,   72,   78,   91,   97,   84,   93,   53,
      66,   96,  127,  140,  141,  136,  147,   82,
     198,  193,  210,  210,  166,  133,  123,  121,
       0,    0,    0,    0,    0,    0,    0,    0,
];
#[rustfmt::skip]
const MG_KNIGHT: [i32; 64] = [
     268,  263,  256,  279,  272,  276,  272,  233,
     239,  260,  284,  292,  288,  308,  284,  281,
     269,  282,  294,  310,  325,  303,  309,  290,
     286,  307,  323,  304,  323,  319,  331,  300,
     301,  309,  344,  360,  337,  362,  333,  337,
     292,  311,  348,  377,  396,  401,  351,  312,
     279,  299,  328,  340,  332,  353,  301,  307,
     248,  302,  306,  302,  317,  310,  315,  299,
];
#[rustfmt::skip]
const MG_BISHOP: [i32; 64] = [
     290,  303,  300,  299,  283,  296,  298,  295,
     320,  317,  358,  314,  327,  327,  348,  327,
     314,  346,  326,  339,  336,  334,  336,  345,
     325,  316,  341,  358,  371,  335,  340,  323,
     321,  337,  343,  386,  363,  362,  333,  331,
     336,  357,  353,  379,  355,  394,  364,  359,
     304,  317,  328,  328,  338,  333,  318,  311,
     315,  325,  322,  316,  321,  319,  328,  328,
];
#[rustfmt::skip]
const MG_ROOK: [i32; 64] = [
     452,  460,  461,  467,  471,  460,  459,  449,
     440,  459,  462,  453,  461,  471,  469,  376,
     425,  444,  450,  455,  467,  457,  494,  438,
     435,  457,  449,  468,  482,  468,  476,  452,
     464,  471,  496,  517,  510,  518,  493,  489,
     485,  513,  534,  528,  550,  570,  531,  500,
     500,  509,  552,  575,  552,  565,  532,  530,
     531,  518,  537,  541,  536,  517,  514,  525,
];
#[rustfmt::skip]
const MG_QUEEN: [i32; 64] = [
     881,  860,  880,  897,  888,  845,  891,  877,
     877,  879,  906,  908,  903,  913,  871,  883,
     890,  893,  897,  897,  900,  902,  921,  897,
     888,  897,  901,  906,  911,  911,  920,  904,
     893,  892,  912,  913,  931,  931,  931,  917,
     896,  898,  911,  935,  955,  984,  984,  942,
     886,  886,  913,  930,  937,  961,  927,  958,
     910,  911,  922,  937,  939,  944,  945,  941,
];
#[rustfmt::skip]
const MG_KING: [i32; 64] = [
     -32,   53,   40,  -57,   37,  -22,   81,   65,
       8,    8,  -10,  -38,  -43,  -22,   36,   60,
       5,    8,   -7,  -21,  -42,  -15,   -6,  -15,
      -0,    6,   -5,  -10,  -11,  -14,   -1,   -3,
       5,    6,   -4,  -11,   -9,   -6,   11,    5,
       6,    3,   -1,   -5,   -1,    4,   -8,    6,
      -1,    0,   -3,   -6,   -9,   -4,    3,    3,
      -1,    5,  -17,   -3,   -1,    1,   -3,    2,
];
#[rustfmt::skip]
const EG_PAWN: [i32; 64] = [
       0,    0,    0,    0,    0,    0,    0,    0,
     150,  147,  132,  129,  134,  127,  123,  116,
     146,  134,  122,  123,  121,  122,  119,  117,
     151,  143,  124,  117,  113,  115,  123,  120,
     180,  159,  133,  126,  116,  116,  138,  139,
     235,  226,  207,  188,  172,  152,  172,  184,
     269,  289,  272,  246,  244,  231,  266,  238,
       0,    0,    0,    0,    0,    0,    0,    0,
];
#[rustfmt::skip]
const EG_KNIGHT: [i32; 64] = [
     246,  227,  263,  249,  264,  246,  251,  233,
     258,  292,  266,  270,  267,  252,  277,  267,
     263,  270,  282,  289,  291,  280,  259,  264,
     277,  283,  305,  314,  300,  295,  280,  258,
     269,  295,  310,  294,  307,  294,  299,  274,
     278,  294,  289,  292,  284,  295,  293,  290,
     273,  286,  303,  298,  296,  288,  292,  287,
     272,  276,  295,  297,  291,  297,  297,  285,
];
#[rustfmt::skip]
const EG_BISHOP: [i32; 64] = [
     290,  280,  271,  292,  302,  291,  296,  276,
     302,  303,  294,  308,  310,  307,  295,  278,
     311,  310,  329,  324,  330,  310,  299,  292,
     312,  330,  328,  323,  318,  319,  312,  307,
     310,  333,  327,  323,  337,  320,  333,  310,
     318,  320,  320,  317,  334,  340,  341,  301,
     308,  324,  319,  328,  330,  334,  320,  306,
     315,  318,  318,  334,  322,  322,  324,  333,
];
#[rustfmt::skip]
const EG_ROOK: [i32; 64] = [
     465,  473,  480,  482,  466,  466,  462,  427,
     457,  452,  474,  470,  459,  454,  424,  402,
     481,  479,  476,  478,  468,  471,  444,  436,
     494,  503,  503,  490,  480,  479,  475,  465,
     501,  511,  495,  490,  476,  464,  481,  480,
     500,  491,  487,  487,  475,  451,  475,  485,
     505,  505,  494,  486,  491,  484,  494,  497,
     476,  497,  484,  484,  485,  490,  491,  488,
];
#[rustfmt::skip]
const EG_QUEEN: [i32; 64] = [
     885,  888,  870,  842,  857,  858,  875,  881,
     883,  894,  871,  861,  879,  865,  864,  881,
     898,  899,  913,  893,  897,  918,  884,  893,
     882,  913,  907,  940,  926,  923,  917,  917,
     888,  911,  931,  952,  954,  940,  940,  933,
     894,  919,  935,  949,  947,  956,  930,  921,
     913,  941,  946,  954,  960,  947,  920,  929,
     893,  919,  930,  943,  943,  944,  927,  914,
];
#[rustfmt::skip]
const EG_KING: [i32; 64] = [
      -4,   -8,    4,    2,  -50,   -2,  -38,  -45,
       4,   16,   18,   16,   25,   18,    4,  -21,
      13,   18,   23,   22,   33,   22,   14,    6,
       2,   21,   28,   32,   26,   25,   19,    6,
      16,   40,   30,   30,   30,   36,   41,   21,
      21,   28,   32,   21,   12,   54,   66,   50,
     -72, -118,  -80,    4,   -3,  -81,   58,   32,
     -67, -259, -164,    5,   12,  -10,  -28,   -5,
];
