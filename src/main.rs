use std::time::Instant;

use bernt::movegen::perft;
use bernt::position::Position;

fn main() {
    let position = Position::startpos();
    let instant = Instant::now();
    let nodes = perft(&position, 5);
    let elapsed = instant.elapsed();

    for (m, x) in nodes.divided {
        println!("{m:?}: {x}");
    }
    println!("Nodes: {}", nodes.all);
    println!("Elapsed: {elapsed:?}");
    println!(
        "Nodes per second: {}",
        (nodes.all as f32 / instant.elapsed().as_secs_f32()).round() as u64
    );
}
