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
    let mut args = msg.trim().split_whitespace();

    match args.next() {
        Some("uci") => Some(UciMessage::Uci),
        Some("isready") => Some(UciMessage::IsReady),
        Some("ucinewgame") => Some(UciMessage::UciNewGame),
        Some("position") => {
            let pos = match args.next()? {
                "fen" => UciPosition::Fen(args.remainder()?.to_owned()),
                "startpos" => UciPosition::Startpos,
                _ => return None,
            };

            let moves = args.map(|x| x.to_owned()).collect();
            Some(UciMessage::Position(pos, moves))
        }
        Some("perft") => {
            let depth = args.next()?.parse().ok()?;

            Some(UciMessage::Perft(depth))
        }
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

    Perft(u8),
}

pub enum UciPosition {
    Fen(String),
    Startpos,
}
