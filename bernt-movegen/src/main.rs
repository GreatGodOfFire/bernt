#[cfg(not(feature = "perftree"))]
fn main() {
    use std::time::Instant;

    use bernt_movegen::perft::perft;
    use bernt_position::{Position, Variant};

    let env: Vec<_> = std::env::args().collect();
    let depth: u8 = env[1].parse().unwrap();
    let fen = env
        .get(2)
        .map(|x| x.as_str())
        .unwrap_or("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let variant = env.get(3).map(|x| x.as_str()).unwrap_or("standard");

    let variant = match variant {
        "standard" => Variant::Standard,
        "frc" => Variant::FRC,
        _ => panic!(),
    };

    let mut position = Position::from_fen(fen).unwrap();
    position.set_variant(variant);

    let instant = Instant::now();
    let n = perft(&mut position, depth);
    let elapsed = instant.elapsed();

    println!("Nodes: {n}");
    println!("Elapsed: {elapsed:?}");
    println!(
        "Nodes per second: {}",
        (n as f32 / elapsed.as_secs_f32()) as u64
    );
}

#[cfg(feature = "perftree")]
fn main() {
    use bernt_movegen::perft::perft;
    use bernt_position::Position;

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

    println!("\n{}", perft(&mut position, depth));
}
