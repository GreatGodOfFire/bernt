mod marlinformat;

use std::{
    env::args,
    fs::{self, OpenOptions},
    io::Write,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use chrono::Local;

use crate::{
    movegen::movegen,
    position::{Move, PieceColor, PieceType, Position},
    search::{is_draw, search, tt::TT, CHECKMATE},
    SearchOptions,
};
use argh::{EarlyExit, FromArgs};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use marlinformat::PackedBoard;

#[derive(FromArgs, Debug)]
/// Datagen config
struct Config {
    /// depth to search
    #[argh(option, short = 'd')]
    pub depth: u8,

    /// how many games to play
    #[argh(option, short = 'g')]
    pub games: u64,

    /// how many threads to spawn
    #[argh(option, short = 't')]
    pub threads: u8,
}

pub fn datagen() {
    let config = match Config::from_args(
        &["datagen"],
        &args()
            .collect::<Vec<_>>()
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()[2..],
    ) {
        Ok(c) => c,
        Err(EarlyExit { output, .. }) => {
            eprintln!("{output}");
            return;
        }
    };

    println!(
        "Generating {} games with depth {} on {} thread(s)",
        config.games, config.depth, config.threads
    );

    let games_per_t = config.games / config.threads as u64;
    let remainder = config.games - games_per_t * config.threads as u64;

    let time = Local::now();
    let folder = format!(
        "data/{}-{}g/",
        time.format("%Y-%m-%d_%H:%M:%S"),
        config.games
    );
    fs::create_dir_all(&folder).unwrap();

    let mut handles = vec![];
    let n_games = Arc::new(AtomicU64::new(0));

    for id in 0..config.threads {
        let mut n = games_per_t;
        if id == 0 {
            n += remainder;
        }

        let folder = folder.clone();
        let n_games = n_games.clone();

        let builder = thread::Builder::new().stack_size(8_000_000);

        handles.push(builder.spawn(move || {
            generate_games(id, n, folder, config.depth, n_games)
        }).unwrap());
    }

    handles.push(thread::spawn(move || {
        let pb = ProgressBar::new(config.games);
        pb.set_style(
            ProgressStyle::with_template(
                &format!("[{{elapsed_precise}}] [{{wide_bar:.cyan}}] {{pos:>{0}}}/{{len:>{0}}} games [{{eta}} at {{per_sec}}]", config.games.ilog10()+1),
            )
            .unwrap()
            .with_key(
                "per_sec",
                move |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    write!(w, "{:.3} games/s", state.per_sec()).unwrap()
                },
            )
            .progress_chars("=>-"),
        );

        let mut n = 0;

        while n < config.games {
            n = n_games.load(Ordering::SeqCst);
            pb.set_position(n);
            thread::sleep(Duration::from_millis(12));
        }

        pb.finish();
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    return;
}

fn generate_games(id: u8, games: u64, folder: String, depth: u8, n_games: Arc<AtomicU64>) {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(format!("{folder}/thread{id}.bin"))
        .unwrap();

    for _ in 0..games {
        let positions = game(depth);
        file.write_all(bytemuck::cast_slice(&positions)).unwrap();
        n_games.fetch_add(1, Ordering::SeqCst);
    }
}

fn game(depth: u8) -> Vec<PackedBoard> {
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
        return game(depth);
    }

    // prevent positions with mate
    let moves = movegen::<true>(&pos);
    let mut mate = true;
    for m in &moves {
        let p = pos.make_move(*m);
        if !p.in_check(!p.side) {
            mate = false;
            break;
        }
    }

    if mate {
        return game(depth);
    }

    let mut options = SearchOptions::default();
    options.depth = depth;
    options.info = false;

    let mut tt = TT::new_default();

    let mut res = search(&pos, options.clone(), reps.clone(), &mut tt);

    if res.score > 1000 {
        return game(depth);
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

        if is_draw(&pos, &reps) || res.best == Move::NULL {
            break 1;
        }
    };

    for pos in positions.iter_mut() {
        pos.set_wdl(game_res);
    }

    return positions;
}
