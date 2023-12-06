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
      36,   57,   55,   52,   52,  100,  140,   56,
      41,   71,   63,   62,   75,   72,  129,   82,
      44,   68,   63,   85,   82,   86,   94,   72,
      59,   74,   79,   94,   99,   96,   93,   53,
      71,  104,  157,  154,  137,  164,  137,   81,
     198,  207,  218,  214,  171,  134,  121,  126,
       0,    0,    0,    0,    0,    0,    0,    0,
];
#[rustfmt::skip]
const MG_KNIGHT: [i32; 64] = [
     266,  255,  245,  284,  281,  269,  266,  208,
     219,  260,  282,  284,  286,  308,  290,  275,
     251,  270,  292,  312,  318,  301,  306,  288,
     282,  310,  323,  297,  322,  317,  338,  309,
     294,  308,  340,  362,  331,  366,  338,  355,
     298,  305,  340,  375,  388,  401,  345,  320,
     286,  286,  346,  342,  325,  355,  313,  318,
     268,  303,  301,  305,  312,  317,  319,  299,
];
#[rustfmt::skip]
const MG_BISHOP: [i32; 64] = [
     273,  293,  291,  296,  298,  314,  305,  310,
     342,  308,  351,  312,  328,  327,  355,  322,
     315,  338,  325,  339,  338,  335,  337,  348,
     311,  335,  348,  353,  374,  337,  330,  334,
     315,  333,  332,  396,  365,  362,  330,  332,
     341,  348,  339,  368,  358,  390,  368,  355,
     292,  313,  343,  336,  347,  343,  317,  334,
     320,  324,  323,  316,  320,  325,  328,  339,
];
#[rustfmt::skip]
const MG_ROOK: [i32; 64] = [
     448,  453,  463,  466,  471,  461,  452,  452,
     438,  452,  462,  456,  476,  480,  489,  505,
     450,  467,  460,  470,  460,  467,  494,  481,
     442,  451,  451,  476,  474,  474,  482,  474,
     444,  477,  500,  518,  507,  513,  487,  497,
     476,  516,  528,  535,  551,  529,  527,  499,
     482,  508,  561,  582,  553,  556,  530,  532,
     516,  505,  532,  556,  552,  524,  514,  517,
];
#[rustfmt::skip]
const MG_QUEEN: [i32; 64] = [
     867,  849,  873,  893,  878,  840,  892,  865,
     871,  877,  901,  902,  909,  907,  875,  891,
     892,  881,  901,  889,  896,  910,  920,  892,
     874,  894,  897,  897,  909,  918,  932,  917,
     885,  887,  911,  913,  939,  942,  934,  925,
     899,  898,  893,  922,  959,  990, 1007,  949,
     865,  871,  915,  928,  951,  961,  924,  965,
     905,  909,  918,  934,  945,  954,  939,  940,
];
#[rustfmt::skip]
const MG_KING: [i32; 64] = [
     -19,   51,   32,  -37,   29,  -42,   66,   49,
     -10,    8,   12,  -20,  -22,  -28,   23,   36,
       2,   11,   -3,  -12,  -43,   -8,   -1,  -29,
       1,   -4,  -10,  -13,  -21,  -14,   -8,   -5,
      -2,    1,   -6,  -12,  -12,   -2,   11,    1,
      -1,   -4,   -5,   -7,    2,   12,    4,    6,
      -4,   14,    2,  -13,    0,    3,    3,   15,
      -1,   30,   -6,   -9,   -3,   -3,    4,    9,
];
#[rustfmt::skip]
const EG_PAWN: [i32; 64] = [
       0,    0,    0,    0,    0,    0,    0,    0,
     155,  161,  144,  151,  142,  131,  124,  119,
     153,  145,  126,  136,  129,  127,  123,  117,
     156,  156,  130,  124,  114,  118,  126,  119,
     195,  172,  147,  142,  126,  124,  143,  144,
     261,  240,  218,  200,  186,  140,  164,  176,
     300,  320,  308,  261,  260,  219,  260,  201,
       0,    0,    0,    0,    0,    0,    0,    0,
];
#[rustfmt::skip]
const EG_KNIGHT: [i32; 64] = [
     241,  250,  264,  266,  257,  254,  266,  209,
     251,  289,  261,  288,  258,  266,  286,  267,
     273,  277,  281,  277,  286,  282,  251,  278,
     265,  270,  308,  309,  307,  286,  295,  259,
     273,  291,  303,  280,  308,  300,  323,  276,
     283,  304,  285,  298,  323,  290,  303,  301,
     258,  290,  306,  303,  306,  301,  302,  301,
     291,  268,  287,  329,  291,  310,  310,  295,
];
#[rustfmt::skip]
const EG_BISHOP: [i32; 64] = [
     281,  285,  274,  301,  294,  287,  305,  300,
     300,  301,  301,  314,  305,  302,  300,  314,
     317,  309,  326,  317,  327,  303,  305,  282,
     314,  326,  329,  316,  306,  314,  299,  302,
     307,  340,  334,  330,  332,  317,  335,  309,
     318,  311,  330,  322,  329,  356,  325,  296,
     305,  330,  317,  338,  338,  331,  316,  287,
     339,  313,  321,  335,  328,  326,  318,  342,
];
#[rustfmt::skip]
const EG_ROOK: [i32; 64] = [
     470,  469,  481,  485,  468,  466,  463,  415,
     457,  459,  481,  463,  450,  450,  428,  339,
     466,  472,  476,  474,  463,  466,  440,  392,
     491,  504,  494,  486,  490,  468,  474,  451,
     509,  510,  492,  498,  479,  479,  488,  491,
     509,  482,  484,  482,  482,  491,  501,  491,
     515,  509,  484,  487,  493,  491,  509,  497,
     481,  499,  480,  475,  469,  492,  499,  490,
];
#[rustfmt::skip]
const EG_QUEEN: [i32; 64] = [
     882,  878,  858,  840,  863,  854,  877,  875,
     876,  890,  877,  872,  843,  873,  869,  889,
     893,  906,  906,  897,  899,  888,  893,  891,
     876,  897,  906,  934,  918,  917,  873,  914,
     883,  901,  921,  952,  951,  939,  926,  931,
     888,  911,  931,  947,  951,  958,  941,  923,
     905,  933,  947,  956,  956,  956,  917,  931,
     896,  916,  930,  946,  942,  955,  923,  914,
];
#[rustfmt::skip]
const EG_KING: [i32; 64] = [
     -10,   -3,    1,   -0,  -50,    0,  -31,  -39,
     -24,    8,   -2,    7,   15,   20,    9,    0,
      -5,    1,    6,   11,   30,   21,   16,   12,
     -10,   -0,   13,   30,   21,   24,   25,   14,
     -14,    6,    9,   26,   22,   31,   44,   29,
       7,    6,   13,    9,   23,   72,   91,   65,
    -154, -187, -119,  -12,   88,  125,   60,   82,
    -129, -283, -192,   -5,   48,   65,   20,   47,
];
