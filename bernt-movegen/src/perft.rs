use bernt_position::Position;

use crate::{is_in_check, movegen, Moves};

pub fn perft(position: &mut Position, depth: u8) -> u64 {
    let moves = movegen(position);
    let mut n = 0;

    if let Moves::PseudoLegalMoves(moves) = moves {
        for m in moves {
            position.make_move(m);

            if !is_in_check(position, !position.to_move()) {
                let x = internal_perft(position, depth - 1);
                n += x;

                if cfg!(feature = "perftree") {
                    println!("{m} {x}");
                } else {
                    println!("{m}: {x}");
                }
            }

            position.unmake_move(m);
        }
    }

    n
}

fn internal_perft(position: &mut Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut n = 0;

    let moves = movegen(position);

    if let Moves::PseudoLegalMoves(moves) = moves {
        for m in moves {
            position.make_move(m);

            if !is_in_check(position, !position.to_move()) {
                let x = internal_perft(position, depth - 1);
                n += x;
            }

            position.unmake_move(m);
        }
    }

    n
}
