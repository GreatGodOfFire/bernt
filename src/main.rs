use bernt::movegen::perft_print;
use bernt::position::Position;

#[cfg(not(feature = "perftree"))]
fn main() {
    use std::time::Instant;

    let position = Position::startpos();
    let instant = Instant::now();
    let nodes = perft_print(&position, 1);
    let elapsed = instant.elapsed();

    println!();
    println!("Nodes: {}", nodes.all);
    println!("Elapsed: {elapsed:?}");
    println!(
        "Nodes per second: {}",
        (nodes.all as f32 / instant.elapsed().as_secs_f32()).round() as u64
    );
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
            position = position.make_move_uci(m);
        }
    }

    let x = perft_print(&position, depth);
    println!();
    println!("{}", x.all);
}
