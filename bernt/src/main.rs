mod bitboard;
mod movegen;
mod perft;
mod position;
mod search;
mod zobrist;

use std::{io::stdin, time::Instant};

use position::Position;
use search::search;

use crate::{movegen::movegen, perft::split_perft};

fn main() {
    let mut line = String::new();

    let mut pos = Position::startpos();
    let mut repetitions = vec![];

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
                println!("uciok");
            }
            "ucinewgame" => {
                pos = Position::startpos();
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
                    for m in &args[2..] {
                        let moves = movegen(&pos);

                        for n in &moves {
                            if n.to_string().as_str() == *m {
                                pos.make_move(*n);
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
                            let moves = movegen(&pos);

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
                        _ => {}
                    }
                }

                println!("bestmove {}", search(&pos, options, repetitions.clone()));
            }
            _ => {}
        }
    }
}

pub struct SearchOptions {
    wtime: u64,
    btime: u64,
    winc: u64,
    binc: u64,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            wtime: u64::MAX,
            btime: u64::MAX,
            winc: 0,
            binc: 0,
        }
    }
}
