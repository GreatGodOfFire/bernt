use bpaf::{construct, short, Parser};

mod movegen;

enum Command {
    Movegen(u8),
}

fn main() {
    let depth = short('d')
        .long("depth")
        .help("Set maximum depth to test")
        .argument("DEPTH")
        .fallback(5u8);
    let movegen = construct!(Command::Movegen(depth))
        .to_options()
        .descr("Test the move generator (requires stockfish)")
        .command("movegen");

    let commands = construct!([movegen]).to_options();

    match commands.fallback_to_usage().run() {
        Command::Movegen(depth) => movegen::test(depth),
    }
}
