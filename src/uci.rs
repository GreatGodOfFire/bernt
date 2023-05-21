use std::io::stdin;

pub fn recv() -> UciMessage {
    loop {
        if let Some(msg) = try_recv() {
            return msg;
        }
    }
}

fn try_recv() -> Option<UciMessage> {
    let mut msg = String::new();
    stdin().read_line(&mut msg).unwrap();
    let mut args = msg.split_whitespace();

    match args.next() {
        Some("uci") => Some(UciMessage::Uci),
        Some("isready") => Some(UciMessage::IsReady),
        Some("ucinewgame") => Some(UciMessage::UciNewGame),
        Some("position") => {
            let pos = match args.next()? {
                "fen" => {
                    let mut fen = String::new();

                    while let Some(s) = args.next() {
                        if s == "moves" {
                            let moves = args.map(|x| x.to_owned()).collect();
                            return Some(UciMessage::Position(
                                UciPosition::Fen(fen.trim().to_owned()),
                                moves,
                            ));
                        }

                        fen.push_str(s);
                        fen.push(' ');
                    }

                    return Some(UciMessage::Position(
                        UciPosition::Fen(fen.trim().to_owned()),
                        vec![],
                    ));
                }
                "startpos" => UciPosition::Startpos,
                _ => return None,
            };

            let moves = if let Some("moves") = args.next() {
                args.map(|x| x.to_owned()).collect()
            } else {
                vec![]
            };
            Some(UciMessage::Position(pos, moves))
        }
        Some("perft") => {
            let depth = args.next()?.parse().ok()?;

            Some(UciMessage::Perft(depth))
        }
        Some("go") => {
            let mut time = Limits {
                wtime: None,
                btime: None,
                winc: None,
                binc: None,
                movestogo: None,
                depth: None,
            };

            while let Some(arg) = args.next() {
                match arg {
                    "wtime" => time.wtime = Some(args.next()?.parse().ok()?),
                    "btime" => time.btime = Some(args.next()?.parse().ok()?),
                    "winc" => time.winc = Some(args.next()?.parse().ok()?),
                    "binc" => time.binc = Some(args.next()?.parse().ok()?),
                    "movestogo" => time.movestogo = Some(args.next()?.parse().ok()?),
                    "depth" => time.depth = Some(args.next()?.parse().ok()?),
                    _ => return None,
                }
            }

            Some(UciMessage::Go(time))
        }
        Some("stop") => Some(UciMessage::Stop),
        Some(x) => {
            println!("Unknown command: {x}");
            None
        }
        None => None,
    }
}

pub fn send(m: UciMessage) {
    match m {
        UciMessage::IdName(name) => println!("id name {name}"),
        UciMessage::IdAuthor(author) => println!("id author {author}"),
        UciMessage::UciOk => println!("uciok"),
        UciMessage::ReadyOk => println!("readyok"),
        _ => {}
    }
}

pub enum UciMessage {
    Uci,
    IdName(String),
    IdAuthor(String),
    UciOk,

    IsReady,
    ReadyOk,

    UciNewGame,
    Position(UciPosition, Vec<String>),
    Go(Limits),

    Perft(u8),

    Stop,
}

pub struct Limits {
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u64>,
    pub depth: Option<u8>,
}

pub enum UciPosition {
    Fen(String),
    Startpos,
}
