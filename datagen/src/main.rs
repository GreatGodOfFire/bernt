mod datagen;

use std::{
    fmt, fs,
    io::{stdin, stdout, Write},
};

use chrono::Local;

use crate::datagen::datagen;

struct Config {
    num_games: u64,
    num_threads: u8,
    depth: u8,
    dfrc: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_games: 100,
            num_threads: 1,
            depth: 4,
            dfrc: false,
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "- num_games: {}", self.num_games)?;
        writeln!(f, "- num_threads: {}", self.num_threads)?;
        writeln!(f, "- depth: {}", self.depth)?;
        writeln!(f, "- dfrc: {}", self.dfrc)?;

        Ok(())
    }
}

fn main() {
    let mut config = Config::default();

    println!("Current datagen configuration:");
    println!("{config}");

    loop {
        let prompt = prompt();
        let line: Vec<&str> = prompt.split_whitespace().collect();
        let cmd = line.get(0);
        if let Some(cmd) = cmd {
            match *cmd {
                "start" | "go" => {
                    let games_per_t = config.num_games / config.num_threads as u64;
                    let remainder = config.num_games - games_per_t * config.num_threads as u64;

                    let time = Local::now();
                    let folder = format!(
                        "data/{}-{}g/",
                        time.format("%Y-%m-%d_%H:%M:%S"),
                        config.num_games
                    );
                    fs::create_dir_all(&folder).unwrap();

                    for id in 0..config.num_threads {
                        if id == 0 {
                            datagen(id, games_per_t + remainder, &folder);
                        } else {
                            datagen(id, games_per_t, &folder);
                        }
                    }
                }
                "set" => {
                    todo!()
                }
                "quit" | "stop" => return,
                _ => eprintln!("Unknown command \"{cmd}\""),
            }
        }
    }
}

fn prompt() -> String {
    print!(">>> ");
    stdout().flush().unwrap();

    let mut line = String::new();
    stdin().read_line(&mut line).unwrap();

    line
}
