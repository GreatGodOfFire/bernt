pub mod bench;
pub mod bitboard;
#[cfg(feature = "datagen")]
pub mod datagen;
pub mod movegen;
pub mod perft;
pub mod position;
pub mod search;
pub mod zobrist;

use std::{env, io::stdin, time::Instant};

use crate::{
    bench::bench,
    movegen::movegen,
    perft::split_perft,
    position::Position,
    search::{search, tt::TT},
};

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() >= 2 {
        match args[1].as_str() {
            "bench" => bench(),
            #[cfg(feature = "datagen")]
            "datagen" => datagen::datagen(),
            _ => {}
        }
        return;
    }

    let mut line = String::new();

    let mut pos = Position::startpos();
    let mut repetitions = vec![pos.hash()];
    let mut tt = TT::new_default();

    loop {
        line.clear();
        stdin().read_line(&mut line).unwrap();

        let args: Vec<_> = line.split_whitespace().collect();

        if args.len() == 0 {
            continue;
        }

        match args[0] {
            "quit" => return,
            "isready" => println!("readyok"),
            "uci" => {
                println!("id name bernt");
                println!("id author GreatGodOfFire");
                println!("option name Hash type spin default 16 min 1 max 262144");
                // For OpenBench
                println!("option name Threads type spin default 1 min 1 max 1");
                println!("uciok");
            }
            "setoption" => {
                if let Some(option) = parse_setoption(&args[1..]) {
                    match option {
                        UciOption::Hash(mb) => tt.set_size(mb),
                        UciOption::Threads(_) => {}
                    }
                }
            }
            "ucinewgame" => {
                pos = Position::startpos();
                tt = TT::new(tt.size());
            }
            "perft" => {
                let depth = args[1].parse().unwrap();
                let instant = Instant::now();
                let res = split_perft(&pos, depth);
                let elapsed = instant.elapsed();
                println!("Elapsed: {elapsed:?}");
                println!("Leaf Nodes: {res}");
                println!(
                    "Leaf Nodes per second: {}",
                    (res as f32 / instant.elapsed().as_secs_f32()) as u64
                );
            }
            "position" => {
                let fen = args[1];

                repetitions = vec![];

                if fen == "startpos" {
                    pos = Position::startpos();

                    repetitions.push(pos.hash());

                    for m in &args[2..] {
                        let moves = movegen::<true>(&pos);

                        for n in &moves {
                            if n.to_string().as_str() == *m {
                                pos = pos.make_move(*n);
                                repetitions.push(pos.hash());
                                break;
                            }
                        }
                    }
                } else {
                    let moves_start = args
                        .iter()
                        .position(|x| *x == "moves")
                        .unwrap_or(args.len());

                    let fen = args[2..moves_start].join(" ");
                    pos = Position::from_fen(&fen);

                    repetitions.push(pos.hash());

                    if moves_start < args.len() {
                        for m in &args[moves_start + 1..] {
                            let moves = movegen::<true>(&pos);

                            for n in &moves {
                                if n.to_string().as_str() == *m {
                                    pos = pos.make_move(*n);
                                    repetitions.push(pos.hash());
                                    break;
                                }
                            }

                            repetitions = repetitions
                                [repetitions.len().saturating_sub(pos.halfmove as usize + 1)..]
                                .to_vec();
                        }
                    }
                }
            }
            "go" => {
                let mut iter = args[1..].iter();
                let mut options = SearchOptions::default();

                while let Some(arg) = iter.next() {
                    match *arg {
                        "wtime" => options.wtime = iter.next().unwrap().parse().unwrap(),
                        "btime" => options.btime = iter.next().unwrap().parse().unwrap(),
                        "winc" => options.winc = iter.next().unwrap().parse().unwrap(),
                        "binc" => options.binc = iter.next().unwrap().parse().unwrap(),
                        "depth" => options.depth = iter.next().unwrap().parse().unwrap(),
                        _ => {}
                    }
                }

                println!(
                    "bestmove {}",
                    search(&pos, options, repetitions.clone(), &mut tt).best
                );
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct SearchOptions {
    pub wtime: u64,
    pub btime: u64,
    pub winc: u64,
    pub binc: u64,
    pub depth: u8,
    pub info: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            wtime: u64::MAX,
            btime: u64::MAX,
            winc: 0,
            binc: 0,
            depth: 255,
            info: true,
        }
    }
}

enum UciOption {
    Hash(usize),
    Threads(u8),
}

fn parse_setoption(args: &[&str]) -> Option<UciOption> {
    use UciOption::*;

    if args.len() == 0 || args[0] != "name" {
        eprintln!("no \"name\" after \"setoption\"");
        return None;
    }

    if args.len() == 1 {
        eprintln!("no option name given");
        return None;
    }

    let mut option;

    match args[1] {
        "Hash" => {
            option = Hash(0);
        }
        "Threads" => {
            option = Threads(0);
        }
        o => {
            eprintln!("unknown uci option {o}");
            return None;
        }
    }

    if args.len() == 2 || args[2] != "value" {
        eprintln!("no \"value\" after name");
        return None;
    }

    if args.len() == 3 {
        eprintln!("no value given");
        return None;
    }

    if matches!(option, Hash(_) | Threads(_)) {
        if let Ok(value) = args[3].parse() {
            option = match option {
                Hash(_) => Hash(value),
                Threads(_) => Threads(value as u8),
            };
        } else {
            eprintln!("unable to parse {} as a number", args[3]);
        }
    }

    Some(option)
}
