use std::{io::stdin, sync::atomic::AtomicBool, time::Instant};

use bernt_movegen::perft::perft;
use bernt_position::Position;
use bernt_search::{Limits, SearchState};
use thread::start_main;

mod thread;

macro_rules! try_parse {
    ($var:ident.$field:ident, $iter:ident; opt) => {
        if let Some(Ok($field)) = $iter.next().map(|x| x.parse()) {
            $var.$field = Some($field);
        } else {
            break;
        }
    };
    ($var:ident.$field:ident, $iter:ident) => {
        if let Some(Ok($field)) = $iter.next().map(|x| x.parse()) {
            $var.$field = $field;
        } else {
            break;
        }
    };
}

fn main() {
    let mut buf = String::new();
    let mut position = Position::startpos();

    let main_thread = start_main();

    loop {
        stdin().read_line(&mut buf).unwrap();

        let mut args = buf.trim().split_ascii_whitespace();

        if let Some(cmd) = args.next() {
            match cmd {
                "uci" => {
                    println!("id name bernt v{}", env!("CARGO_PKG_VERSION"));
                    println!("id author GreatGodOfFire");
                    println!("uciok");
                }
                "isready" => println!("readyok"),
                "ucinewgame" => {
                    position = Position::startpos();
                }
                "position" => {
                    position = match args.next() {
                        Some("startpos") => {
                            let mut position = Position::startpos();
                            position.calc_zobrist();

                            if args.next() == Some("moves") {
                                while let Some(m) = args.next() {
                                    position.make_move_uci(m);
                                    position.calc_zobrist();
                                    position.finalize_moves();
                                }
                            }

                            position
                        }
                        Some("fen") => {
                            let mut fen = String::new();
                            let mut moves = vec![];

                            while let Some(arg) = args.next() {
                                if arg == "moves" {
                                    while let Some(m) = args.next() {
                                        moves.push(m);
                                    }
                                } else {
                                    fen.push(' ');
                                    fen += arg;
                                }
                            }

                            position = Position::from_fen(fen.trim()).unwrap();

                            for m in moves {
                                position.make_move_uci(m);
                                position.calc_zobrist();
                                position.finalize_moves();
                            }

                            position
                        }
                        Some(_) => continue,
                        None => continue,
                    }
                }
                "perft" => {
                    if let Some(depth) = args.next().map(|x| x.parse().ok()).flatten() {
                        let now = Instant::now();
                        let x = perft(&mut position, depth);
                        let elapsed = now.elapsed();
                        println!("Nodes: {x}");
                        println!("Elapsed: {elapsed:?}");
                        println!("Nodes per second: {:.0}", x as f64 / elapsed.as_secs_f64());
                    }
                }
                "go" => {
                    let mut limits = Limits::default();

                    while let Some(arg) = args.next() {
                        match arg {
                            "wtime" => {
                                try_parse!(limits.wtime, args; opt)
                            }
                            "btime" => {
                                try_parse!(limits.btime, args; opt)
                            }
                            "winc" => {
                                try_parse!(limits.winc, args; opt)
                            }
                            "binc" => {
                                try_parse!(limits.binc, args; opt)
                            }
                            "movestogo" => {
                                try_parse!(limits.movestogo, args; opt)
                            }
                            "depth" => {
                                try_parse!(limits.depth, args)
                            }
                            _ => break,
                        }
                    }

                    main_thread.start_search(position.clone(), limits);
                }
                "stop" => main_thread.stop(),
                "quit" => return,
                _ => {}
            }
        }

        buf.clear();
    }
}
