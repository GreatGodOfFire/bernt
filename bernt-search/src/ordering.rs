use bernt_movegen::MoveList;
use bernt_position::{piece::Piece, Move, MoveType};

pub struct OrderedMoves(pub MoveList);

impl OrderedMoves {
    #[inline]
    pub fn new(mut moves: MoveList, board: &[Piece; 64], pv: Move) -> Self {
        moves.array[..moves.len as usize].sort_by_key(|x| 255 - move_score(*x, board, pv));

        Self(moves)
    }
}

#[rustfmt::skip]
const MVVLVA_LOOKUP: [[u8; 5]; 6] = [
        /* N   B   R   Q   P */ 
/* N */  [ 9,  8, 11, 15,  7],
/* B */  [10,  9, 11, 15,  7],
/* R */  [ 7,  7,  9, 13,  5],
/* Q */  [ 3,  3,  5,  9,  1],
/* P */  [11, 11, 13, 17,  9],
/* K */  [ 2,  2,  4,  8,  0],
];

fn move_score(m: Move, board: &[Piece; 64], pv: Move) -> u8 {
    if m == pv {
        return 255;
    }

    if m.is_capture() && m.ty != MoveType::EnPassantCapture {
        return 10 + MVVLVA_LOOKUP[board[m.from as usize].ty][board[m.to as usize].ty];
    }

    return 1;
}
