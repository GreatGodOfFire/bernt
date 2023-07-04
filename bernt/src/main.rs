use std::io::stdin;

use bernt_position::{Position, bitboard::print_bitboard, piece::PieceType};
use bernt_search::Limits;
use thread::start_main;

mod thread;

macro_rules! try_parse {
    ($var:ident.$field:ident, $ty:ty, $iter:ident; opt) => {
        if let Some(Ok($field)) = $iter.next().map(|x| x.parse::<$ty>()) {
            $var.$field = Some($field);
        } else {
            break;
        }
    };
    ($var:ident.$field:ident, $ty:ty, $iter:ident) => {
        if let Some(Ok($field)) = $iter.next().map(|x| x.parse::<$ty>()) {
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
                "go" => {
                    let mut limits = Limits::default();

                    while let Some(arg) = args.next() {
                        match arg {
                            "wtime" => {
                                try_parse!(limits.wtime, u64, args; opt)
                            }
                            "btime" => {
                                try_parse!(limits.btime, u64, args; opt)
                            }
                            "winc" => {
                                try_parse!(limits.winc, u64, args; opt)
                            }
                            "binc" => {
                                try_parse!(limits.binc, u64, args; opt)
                            }
                            "movestogo" => {
                                try_parse!(limits.movestogo, u64, args; opt)
                            }
                            "depth" => {
                                try_parse!(limits.depth, u8, args)
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
