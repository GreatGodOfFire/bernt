use bernt::movegen::perft_print;
use bernt::position::Position;

#[cfg(not(feature = "perftree"))]
fn main() {
    use std::time::Instant;

    use bernt::{
        position::tt::DEFAULT_HASH_SIZE,
        search::start_search,
        uci::{recv, UciMessage, UciOption, UciPosition},
    };

    let mut position = Position::startpos();
    let mut hash_size = DEFAULT_HASH_SIZE;

    loop {
        match recv() {
            UciMessage::Uci => {
                println!("id name Bernt {}", env!("CARGO_PKG_VERSION"));
                println!("id author GreatGodOfFire");

                println!("option name Hash type spin default 16 min 1 max 1048576");

                println!("uciok");
            }
            UciMessage::UciNewGame => {
                position = Position::startpos();
                position.tt.set_size(hash_size);
            }
            UciMessage::Position(pos, moves) => {
                position = match pos {
                    UciPosition::Fen(fen) => Position::from_fen(&fen).unwrap(),
                    UciPosition::Startpos => Position::startpos(),
                };
                for moves in moves.chunks(100) {
                    for m in moves {
                        position.make_move_uci(m);
                        position.calc_zobrist();
                    }
                    position.clear_incremental();
                }
            }
            UciMessage::Perft(depth) => {
                let instant = Instant::now();
                let nodes = perft_print(&mut position, depth);
                let elapsed = instant.elapsed();
                println!("Nodes: {}", nodes);
                println!("Elapsed: {elapsed:?}");
                println!(
                    "Nodes per second: {}",
                    (nodes as f32 / instant.elapsed().as_secs_f32()).round() as u64
                );
            }
            UciMessage::IsReady => println!("readyok"),
            UciMessage::Setoption(option) => match option {
                UciOption::Hash(hash) => {
                    hash_size = hash;
                    position.tt.set_size(hash_size);
                }
            },
            UciMessage::Go(time) => {
                let m = start_search(&mut position, time);
                println!("bestmove {m:?}");
            }
            UciMessage::Quit => break,
        }
    }
}

#[cfg(feature = "perftree")]
fn main() {
    let env: Vec<_> = std::env::args().collect();
    let depth: u8 = env[1].parse().unwrap();
    let fen = &env[2];
    let moves = env.get(3);

    let mut position = Position::from_fen(fen).unwrap();

    if let Some(moves) = moves {
        for m in moves.split(' ') {
            position.make_move_uci(m);
        }
    }

    let x = perft_print(&mut position, depth);
    println!();
    println!("{}", x);
}
