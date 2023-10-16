use std::{
    fmt,
    fs::{self, OpenOptions},
    io::{stdin, stdout, Write},
    thread,
};

use chrono::Local;

use crate::{
    marlinformat::PackedBoard,
    movegen::movegen,
    position::{PieceColor, PieceType, Position},
    search::{is_draw, search, tt::TT, CHECKMATE},
    SearchOptions,
};

macro_rules! config {
    ($($name:ident: $ty:ty = $value:literal),*) => {
        #[derive(Clone, Debug)]
        pub struct Config {
            $( pub $name: $ty ),*
        }

        impl Default for Config {
            fn default() -> Self {
                Self {
                    $( $name: $value ),*
                }
            }
        }

        impl Config {
            pub fn set_option(&mut self, name: &str, value: &str) {
                match name {
                    $(
                        stringify!($name) => {
                            let Ok(value) = value.parse().map_err(|_| ()) else {
                                eprintln!("Failed to parse {} as an {}", value, stringify!($ty));
                                return;
                            };
                            self.$name = value;
                        }
                    ),*
                    _ => eprintln!("Unknown option \"{name}\"")
                }
            }
        }

        impl fmt::Display for Config {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                $(
                    writeln!(f, "- {}: {}", stringify!($name), self.$name)?;
                )*

                Ok(())
            }
        }
    };
}

config! {
    num_games: u64 = 100,
    num_threads: u8 = 1,
    depth: u8 = 5,
    dfrc: bool = false
}

pub fn datagen() {
    let mut config = Config::default();

    println!("Current datagen configuration:");
    println!("{config}");

    loop {
        let prompt = prompt();
        let line: Vec<&str> = prompt.split_whitespace().collect();
        if let Some(cmd) = line.get(0) {
            match *cmd {
                "start" | "go" => {
                    let games_per_t = config.num_games / config.num_threads as u64;
                    let remainder = config.num_games - games_per_t * config.num_threads as u64;

                    let time = Local::now();
                    let folder = format!(
                        "data/{}-{}g/",
                        time.format("%Y-%m-%d_%H:%M:%S"),
                        config.num_games
                    );
                    fs::create_dir_all(&folder).unwrap();

                    let mut handles = vec![];

                    for id in 0..config.num_threads {
                        let mut n = games_per_t;
                        if id == 0 {
                            n += remainder;
                        }

                        let folder = folder.clone();

                        handles.push(thread::spawn(move || {
                            generate_games(id, n, folder, config.depth)
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    return;
                }
                "set" => {
                    if line.len() < 3 {
                        eprintln!("Usage: set <name> <value>");
                    }

                    config.set_option(line[1], line[2]);
                    println!("Updated config:");
                    println!("{config}");
                }
                "quit" | "stop" => return,
                _ => eprintln!("Unknown command \"{cmd}\""),
            }
        }
    }
}

fn prompt() -> String {
    print!(">>> ");
    stdout().flush().unwrap();

    let mut line = String::new();
    stdin().read_line(&mut line).unwrap();

    line
}

pub fn generate_games(id: u8, num_games: u64, folder: String, depth: u8) {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(format!("{folder}/thread{id}.bin"))
        .unwrap();

    for _ in 0..num_games {
        let positions = game(depth);
        file.write_all(bytemuck::cast_slice(&positions)).unwrap();
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

    let mut options = SearchOptions::default();
    options.depth = depth;

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

        if is_draw(&pos, &reps) {
            break 1;
        }
    };

    for pos in positions.iter_mut() {
        pos.set_wdl(game_res);
    }

    return positions;
}
