use bernt::movegen::perft_print;
use bernt::position::Position;

#[cfg(not(feature = "perftree"))]
fn main() {
    use std::time::Instant;

    use bernt::uci::{recv, send, UciMessage, UciPosition};

    let mut position = Position::startpos();

    loop {
        match recv() {
            UciMessage::Uci => {
                send(UciMessage::IdName("Bernt".to_owned()));
                send(UciMessage::IdName("GreatGodOfFire".to_owned()));
                send(UciMessage::UciOk);
            }
            UciMessage::UciNewGame => {
                position = Position::startpos();
            }
            UciMessage::Position(pos, moves) => {
                position = match pos {
                    UciPosition::Fen(fen) => Position::from_fen(&fen).unwrap(),
                    UciPosition::Startpos => Position::startpos(),
                };
                for m in moves {
                    position.make_move_uci(&m);
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
            UciMessage::IsReady => send(UciMessage::IsReady),
            _ => todo!(),
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
