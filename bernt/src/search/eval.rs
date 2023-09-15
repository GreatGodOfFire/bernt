use crate::position::Position;

pub fn eval(pos: &Position) -> i32 {
    let mut eval = 0;

    for ty in 0..6 {
        for (side, sign) in [(pos.side, 1), (!pos.side, -1)] {
            eval +=
                (pos.pieces[ty] & pos.colors[side]).count_ones() as i32 * PIECE_VALUE[ty] * sign;
        }
    }

    eval
}

const PIECE_VALUE: [i32; 6] = [100, 320, 330, 500, 900, 0];
