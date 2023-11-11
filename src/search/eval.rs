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
      36,   75,   57,   59,   73,   99,  134,   64,
      51,   89,   69,   77,   85,   73,  129,   72,
      48,   79,   76,   96,   98,   85,   96,   49,
      53,   97,   98,  114,  121,   97,   99,   46,
      90,  137,  129,  149,  141,  140,  104,   89,
     194,  213,  225,  216,  191,  133,  121,  116,
       0,    0,    0,    0,    0,    0,    0,    0,
];
#[rustfmt::skip]
const MG_KNIGHT: [i32; 64] = [
     281,  294,  264,  305,  298,  303,  311,  256,
     261,  280,  305,  316,  321,  342,  324,  302,
     291,  320,  324,  343,  353,  339,  352,  318,
     317,  324,  342,  344,  366,  365,  370,  345,
     315,  329,  376,  409,  376,  423,  357,  372,
     297,  350,  395,  413,  410,  460,  379,  344,
     285,  276,  379,  368,  361,  373,  316,  313,
     249,  306,  297,  296,  313,  305,  311,  297,
];
#[rustfmt::skip]
const MG_BISHOP: [i32; 64] = [
     275,  313,  332,  316,  330,  336,  316,  303,
     334,  346,  339,  341,  364,  337,  377,  303,
     344,  377,  359,  361,  347,  372,  369,  350,
     350,  373,  383,  382,  383,  368,  365,  343,
     378,  364,  382,  431,  396,  392,  370,  384,
     320,  356,  384,  385,  397,  401,  384,  388,
     317,  366,  362,  354,  340,  375,  340,  337,
     288,  323,  331,  328,  325,  317,  328,  314,
];
#[rustfmt::skip]
const MG_ROOK: [i32; 64] = [
     454,  452,  468,  474,  477,  469,  397,  440,
     438,  513,  456,  426,  455,  475,  474,  374,
     435,  450,  459,  454,  464,  456,  473,  419,
     437,  448,  452,  456,  473,  480,  489,  451,
     458,  455,  476,  490,  535,  519,  504,  468,
     463,  504,  490,  531,  539,  551,  510,  500,
     490,  515,  572,  561,  565,  550,  527,  515,
     530,  577,  541,  555,  570,  526,  520,  532,
];
#[rustfmt::skip]
const MG_QUEEN: [i32; 64] = [
     878,  845,  868,  894,  859,  843,  877,  855,
     882,  877,  905,  896,  903,  893,  888,  871,
     896,  896,  895,  895,  881,  901,  895,  887,
     885,  883,  885,  893,  908,  896,  896,  882,
     884,  895,  910,  931,  939,  954,  902,  927,
     874,  919,  921,  955,  971, 1012,  968,  940,
     888,  898,  924,  933,  940,  969,  963,  966,
     891,  908,  934,  938,  947,  950,  943,  938,
];
#[rustfmt::skip]
const MG_KING: [i32; 64] = [
     -26,    0,   39,  -41,   37,  -22,   65,   71,
       7,   -0,   38,  -22,  -62,   -3,   28,   50,
      -5,   -8,  -31,  -56,  -67,  -35,    2,    2,
      -2,  -14,  -10,   -9,  -22,  -14,   -4,  -30,
       4,   29,   -2,  -15,    5,   -1,    5,  -10,
      12,    6,    9,    3,    9,    1,  -19,    8,
      10,   10,   -0,   -2,   80,  -24,    1,   -7,
       5,   20,    6,    8,    3,  -10,   -4,    4,
];
#[rustfmt::skip]
const EG_PAWN: [i32; 64] = [
       0,    0,    0,    0,    0,    0,    0,    0,
     152,  148,  140,  121,  145,  136,  129,  121,
     152,  135,  116,  123,  118,  122,  117,  130,
     164,  135,  124,  123,  101,  104,  133,  130,
     171,  169,  137,  124,  118,  130,  144,  158,
     222,  238,  218,  203,  177,  185,  210,  203,
     266,  274,  277,  225,  202,  242,  283,  254,
       0,    0,    0,    0,    0,    0,    0,    0,
];
#[rustfmt::skip]
const EG_KNIGHT: [i32; 64] = [
     243,  203,  259,  261,  245,  248,  198,  240,
     255,  274,  235,  259,  240,  215,  261,  286,
     231,  260,  241,  266,  256,  250,  240,  223,
     248,  228,  276,  271,  273,  255,  256,  218,
     266,  257,  272,  152,  278,  259,  295,  258,
     241,  240,  274,  272,  284,  251,  272,  275,
     259,  280,  212,  263,  259,  270,  287,  282,
     270,  249,  275,  274,  270,  284,  273,  265,
];
#[rustfmt::skip]
const EG_BISHOP: [i32; 64] = [
     251,  299,  239,  279,  258,  266,  282,  284,
     238,  255,  290,  278,  265,  279,  264,  271,
     279,  273,  288,  295,  292,  282,  282,  293,
     292,  270,  270,  283,  301,  291,  283,  293,
     224,  295,  292,  277,  302,  313,  306,  272,
     303,  263,  280,  302,  276,  307,  315,  294,
     294,  275,  274,  293,  323,  243,  283,  289,
     303,  312,  312,  291,  293,  300,  247,  303,
];
#[rustfmt::skip]
const EG_ROOK: [i32; 64] = [
     448,  457,  456,  465,  457,  445,  514,  429,
     385,  315,  431,  455,  445,  408,  419,  429,
     457,  439,  461,  457,  456,  473,  463,  462,
     474,  467,  495,  480,  479,  457,  468,  486,
     478,  492,  495,  478,  468,  478,  483,  468,
     498,  483,  495,  482,  496,  499,  490,  487,
     501,  479,  475,  476,  493,  489,  491,  486,
     465,  433,  462,  446,  457,  477,  484,  479,
];
#[rustfmt::skip]
const EG_QUEEN: [i32; 64] = [
     868,  880,  851,  821,  861,  854,  873,  873,
     868,  894,  807,  846,  829,  873,  857,  862,
     883,  864,  889,  873,  902,  894,  888,  868,
     874,  909,  927,  947,  907,  927,  944,  950,
     893,  898,  920,  928,  952,  923,  960,  937,
     897,  912,  925,  925,  944,  954,  942,  931,
     897,  927,  947,  952,  961,  944,  925,  923,
     890,  907,  916,  930,  940,  943,  925,  919,
];
#[rustfmt::skip]
const EG_KING: [i32; 64] = [
     -11,  -30,  -43,  -28,  -67,  -21,  -37,  -56,
     -20,  -19,  -13,   -8,   10,   -1,  -14,  -33,
     -35,   -3,   -6,   11,   21,   22,  -10,  -24,
       4,  -16,   -1,   20,   20,   10,    3,    4,
     -35,  -92,   12,   41,   35,   25,   20,    8,
     -55,   15,   -3,    3,   16,   24,    7,  -71,
       8,   22,   42,   19, -179,   11,  -33,  -10,
      37,  170,  155,   44,   41,   59,   36,   -6,
];
