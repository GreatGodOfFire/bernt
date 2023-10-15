pub mod bench;
pub mod bitboard;
pub mod marlinformat;
pub mod movegen;
pub mod perft;
pub mod position;
pub mod search;
pub mod zobrist;

#[derive(Clone)]
pub struct SearchOptions {
    pub wtime: u64,
    pub btime: u64,
    pub winc: u64,
    pub binc: u64,
    pub depth: u8,
    pub info: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            wtime: u64::MAX,
            btime: u64::MAX,
            winc: 0,
            binc: 0,
            depth: 255,
            info: true,
        }
    }
}
