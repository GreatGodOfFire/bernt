use crate::position::{Position, piece::PieceType};

use super::{Move, Code};

pub struct OrderedMoves(Vec<(i32, Move)>);

impl OrderedMoves {
    pub fn new(moves: &[Move], position: &Position, pv_move: Move) -> Self {
        Self(moves.iter().map(|&m| (score(m, position, pv_move), m)).collect())
    }
}

// MVVLVA_LOOKUP[attacker][target]
// Adjust the values later
#[rustfmt::skip]
const MVVLVA_LOOKUP: [[i32; 5]; 6] = [
        /* N   B   R   Q   P */ 
/* N */  [ 0, -1,  2,  6, -2],
/* B */  [ 1,  0,  2,  6, -2],
/* R */  [-2, -2,  0,  4, -4],
/* Q */  [-6, -6, -4,  0, -8],
/* P */  [ 2,  2,  4,  8,  0],
/* K */  [-7, -7, -5, -1, -9],
];

fn score(m: Move, position: &Position, pv_move: Move) -> i32 {
    if m == pv_move {
        return 1000;
    }

    let mut score = 0;

    if let Some(promotion) = m.promotion() {
        score += 98;
        score += promotion as i32;
    }
    if m.code() == Code::EnPassantCapture {
        score += 100;
    } else if m.is_capture() {
        score += 100;
        score += MVVLVA_LOOKUP[position.mailbox[m.from() as usize].ty][position.mailbox[m.to() as usize].ty];
    }

    score
}

impl Iterator for OrderedMoves {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let mut idx = 0;
        let mut next = (-1, Move::null());

        for (i, m) in self.0.iter().enumerate() {
            if m.0 > next.0 {
                next = *m;
                idx = i;
            }
        }

        if next.0 == -1 {
            None
        } else {
            self.0[idx].0 = -1;
            Some(next.1)
        }
    }
}
