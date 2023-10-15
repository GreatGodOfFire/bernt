use std::{fs::OpenOptions, io::Write};

use bernt::{
    marlinformat::PackedBoard,
    movegen::movegen,
    position::{PieceColor, PieceType, Position},
    search::{is_draw, search, tt::TT, CHECKMATE},
    SearchOptions,
};

pub fn datagen(id: u8, num_games: u64, folder: &str) {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(format!("{folder}/thread{id}.bin"))
        .unwrap();

    for _ in 0..num_games {
        let positions = game();
        file.write_all(bytemuck::cast_slice(&positions)).unwrap();
    }
}

fn game() -> Vec<PackedBoard> {
    let mut pos = Position::startpos();
    let mut reps = vec![pos.hash()];

    let mut positions = Vec::with_capacity(256);

    // play random moves
    'rand: for _ in 0..8 {
        let mut moves = movegen::<true>(&pos);
        fastrand::shuffle(&mut moves.moves[0..moves.len as usize]);
        for m in &moves {
            let p = pos.make_move(*m);
            if !p.in_check(!p.side) {
                pos = p;
                reps.push(pos.hash());
                continue 'rand;
            }
        }
        return game();
    }

    let mut options = SearchOptions::default();
    options.depth = 6;

    let mut tt = TT::new_default();

    let mut res = search(&pos, options.clone(), reps.clone(), &mut tt);

    if res.score > 1000 {
        return game();
    }

    let game_res = loop {
        if !res.best.capture() && res.best.promotion() == PieceType::None {
            positions.push(PackedBoard::pack(&pos, 0, res.score, 0, 0));
        }
        pos = pos.make_move(res.best);
        reps.push(pos.hash());

        res = search(&pos, options.clone(), reps.clone(), &mut tt);

        if res.score.abs() >= CHECKMATE {
            if (res.score.signum() == 1 && pos.side == PieceColor::White)
                || (res.score.signum() == -1 && pos.side == PieceColor::Black)
            {
                break 2;
            } else {
                break 0;
            }
        }

        if is_draw(&pos, &reps) {
            break 1;
        }
    };

    for pos in positions.iter_mut() {
        pos.set_wdl(game_res);
    }

    return positions;
}
