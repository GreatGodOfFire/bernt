use std::{
    io::{BufRead, BufReader, Write},
    process::{exit, Child, Command, Stdio},
};

use bernt_movegen::perft::perft;
use bernt_position::Position;
use owo_colors::OwoColorize;

struct Stockfish(Child);

impl Stockfish {
    fn new() -> Self {
        let child = Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match child {
            Ok(child) => Self(child),
            Err(e) => {
                eprintln!("Failed to start stockfish, is it installed?");
                eprintln!("Error: {e}");
                exit(1);
            }
        }
    }

    fn perft(&mut self, fen: &str, depth: u8) -> u64 {
        self.0
            .stdin
            .as_ref()
            .unwrap()
            .write_fmt(format_args!("position fen {fen}\ngo perft {depth}\n"))
            .unwrap();

        for line in BufReader::new(self.0.stdout.as_mut().unwrap()).lines() {
            if let Some((_, n)) = line.unwrap().split_once("Nodes searched: ") {
                return n.parse().unwrap();
            }
        }

        panic!();
    }
}

pub fn test(depth: u8) {
    let groups: &[(&str, &[&str])] = &[("Standard", STANDARD)];

    let max_len = groups.iter().map(|(x, _)| x.len()).max().unwrap();

    let mut stockfish = Stockfish::new();

    let running = "----".bright_black();
    let fail = "FAIL".red();
    let pass = "PASS".green();

    for (name, a) in groups {
        print!("[{running}] {name:0$} ", max_len);
        progress_bar(0, a.len());

        let mut line_offset = 0;
        let mut passed = true;

        for (i, fen) in a.iter().enumerate() {
            let n = perft(&mut Position::from_fen(fen).unwrap(), depth);
            let s = stockfish.perft(fen, depth);
            if n != s {
                print!("\x1b[{line_offset}E");
                println!("  [{fail}] '{fen}': {} {s}", n.to_string().red().bold());
                line_offset += 1;
                print!("\x1b[{line_offset}F");
                passed = false;
            }

            print!("\x1b[F[{}] {name:1$} ", running, max_len);
            progress_bar(i + 1, a.len());
        }

        if passed {
            println!("\x1b[F[{pass}]");
            print!("\x1b[{line_offset}E");
        } else {
            println!("\x1b[F[{fail}]");
            print!("\x1b[{line_offset}E");
        }
    }
}

fn progress_bar(now: usize, goal: usize) {
    let n_elements = now * 30 / goal;
    let mut s = "=".repeat(n_elements);
    if n_elements < 30 {
        s.push('>');
    }
    print!("[{:<30}", s);
    println!("] {now}/{goal}");
}

const STANDARD: &[&str] = &[
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ -",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - -",
];
