#![feature(const_eval_limit)]
#![const_eval_limit = "5000000"]

use std::time::Instant;

use movegen::perft;
use position::Position;

mod movegen;
mod position;

const PERFT_DEPTH: u8 = 5;
fn main() {
    let position = Position::startpos();
    let instant = Instant::now();
    let nodes = perft(&position, PERFT_DEPTH);

    println!("Nodes: {nodes}");
    println!("Elapsed: {:?}", instant.elapsed());
    println!(
        "Nodes per second: {}",
        (nodes as f32 / instant.elapsed().as_secs_f32()).round() as u64
    );
}
