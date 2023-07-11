use std::{
    env,
    fs::{self, read_to_string},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use bernt_position::{piece::PieceType, Position};
use bernt_search::eval::{flip, GAMEPHASE_INC};
use params::Params;

mod params;

struct TuningPosition {
    indices: [[(PieceType, u8); 16]; 2],
    counters: [u8; 2],
    phase: i32,
    result: f64,
}

impl TuningPosition {
    fn new(pos: Position, result: f64) -> Self {
        let mut indices = [[(PieceType::Empty, 0); 16]; 2];
        let mut counters = [0; 2];
        let mut phase = 0;

        for (sq, piece) in pos.mailbox().iter().enumerate() {
            if piece.ty != PieceType::Empty {
                phase += GAMEPHASE_INC[piece.ty];
                indices[piece.color][counters[piece.color] as usize].0 = piece.ty;
                indices[piece.color][counters[piece.color] as usize].1 = sq as u8;
                counters[piece.color] += 1;
            }
        }

        // phase = 256 - phase.min(256);
        // TODO: Change this when adding opening table back in
        phase = phase.min(256);

        Self {
            indices,
            counters,
            phase,
            result,
        }
    }
}

fn main() {
    if let Some(path) = env::args().nth(1) {
        match fs::metadata(&path) {
            Ok(m) if m.is_file() => {
                let positions = read_to_string(path).unwrap();
                let mut lines = positions.lines();
                lines.next().unwrap();

                let positions: Vec<_> = lines
                    .map(|x| {
                        let (fen, result) = x.split_once(':').unwrap();
                        let result = result.parse().unwrap();
                        TuningPosition::new(Position::from_fen(fen).unwrap(), result)
                    })
                    .collect();

                let n_epochs = env::args()
                    .nth(2)
                    .and_then(|x| x.parse().ok())
                    .unwrap_or(u64::MAX);

                println!("{}", tune(&positions, Params::default(), n_epochs));
            }
            Ok(m) if m.is_dir() => {
                eprintln!("Expected a file")
            }
            _ => eprintln!("Invalid path: {path}"),
        }
    } else {
        eprintln!("USAGE: {} <PATH>", env::args().next().unwrap());
    }
}

fn tune(positions: &[TuningPosition], initial: Params, n_epochs: u64) -> Params {
    let param_count = Params::param_count();
    let (k, mut best_e) = initial_error(positions, &initial);
    let mut best = initial;
    let mut improved = true;

    let mut epoch = 1;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("Received Ctrl-C, finishing last iteration...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    println!("\n");

    while improved && epoch <= n_epochs && running.load(Ordering::SeqCst) {
        print!("\x1b[F\x1b[F");
        println!("E:     {best_e:.7}");
        println!("Epoch: {epoch}");
        epoch += 1;

        improved = false;
        for i in 0..param_count {
            best[i] += 1;
            let add_e = error(positions, &best, k);
            if add_e < best_e {
                best_e = add_e;
                improved = true;
            } else {
                best[i] -= 2;
                let sub_e = error(positions, &best, k);
                if sub_e < best_e {
                    best_e = sub_e;
                    improved = true;
                } else {
                    best[i] += 1;
                }
            }
        }
    }

    best
}

fn initial_error(positions: &[TuningPosition], params: &Params) -> (f64, f64) {
    const GR: f64 = 1.61803399f64;

    let mut a = 0.0;
    let mut b = 2.0;

    let mut k1 = b - (b - a) / GR;
    let mut k2 = a + (b - a) / GR;

    while (b - a).abs() > 0.01 {
        let f1 = error(positions, params, k1);
        let f2 = error(positions, params, k2);
        if f1 < f2 {
            b = k2;
        } else {
            a = k1;
        }
        k1 = b - (b - a) / GR;
        k2 = a + (b - a) / GR;
    }

    let k = (b + a) / 2.0;
    println!("K:     {k:.7}");
    (k, error(positions, params, k))
}

fn error(positions: &[TuningPosition], params: &Params, k: f64) -> f64 {
    let mut sum = 0f64;
    let mut n = 0;

    for pos in positions.iter() {
        sum += single_error(pos, params, k);
        n += 1;
    }

    sum / n as f64
}

fn single_error(pos: &TuningPosition, params: &Params, k: f64) -> f64 {
    let sigmoid = sigmoid(eval(pos, params), k);

    (pos.result - sigmoid).powi(2)
}

fn eval(pos: &TuningPosition, params: &Params) -> f64 {
    // let mut opening = 0;
    let mut midgame = 0;
    let mut endgame = 0;

    for (ty, index) in &pos.indices[0][..pos.counters[0] as usize] {
        // opening += params.opening[*ty][*index as usize];
        midgame += params.midgame[*ty][*index as usize];
        endgame += params.endgame[*ty][*index as usize];
    }
    for (ty, index) in &pos.indices[1][..pos.counters[1] as usize] {
        // opening -= params.opening[*ty][flip(*index as usize)];
        midgame -= params.midgame[*ty][flip(*index as usize)];
        endgame -= params.endgame[*ty][flip(*index as usize)];
    }

    let eval = (midgame * pos.phase + (256 - pos.phase) * endgame) / 256;
    // let eval = ((opening * (128 - pos.phase).max(0) * 2)
    //     + (midgame * (-(pos.phase - 128).abs() * 2 + 256))
    //     + (endgame * (pos.phase - 128).max(0) * 2))
    //     / 256;

    eval as f64
}

fn sigmoid(x: f64, k: f64) -> f64 {
    1.0 / (1.0 + (-k * x / 100.0).exp())
}
