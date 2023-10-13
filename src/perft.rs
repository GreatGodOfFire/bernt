use crate::{movegen, position::Position};

pub fn split_perft(pos: &Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut i = 0;

    for m in &movegen::<true>(pos) {
        let pos = pos.make_move(*m);

        if !pos.in_check(!pos.side) {
            let res = perft(&pos, depth - 1);
            println!("{m}: {res}");
            i += res;
        }
    }

    i
}

fn perft(pos: &Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut i = 0;

    for m in &movegen::<true>(pos) {
        let pos = pos.make_move(*m);

        if !pos.in_check(!pos.side) {
            let res = perft(&pos, depth - 1);
            i += res;
        }
    }

    i
}
