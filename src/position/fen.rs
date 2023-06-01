use crate::zobrist::hash;

use super::{
    piece::{Piece, PieceColor, PieceType::*},
    tt::TranspositionTable,
    Position, State,
};

impl Position {
    pub fn startpos() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut bitboards = [[0u64; 7]; 2];
        let mut mailbox = [Piece::EMPTY; 64];

        let parts = fen.split(' ').collect::<Vec<_>>();
        if parts.len() < 4 {
            return None;
        }
        let ranks = parts[0].split('/');
        for (i, rank) in ranks.enumerate() {
            let i = 7 - i;
            if rank == "8" {
                continue;
            }

            let mut j = 0;
            let mut idx = 0;
            let chars = rank.chars().collect::<Vec<_>>();
            while j < 8 {
                let c = chars[idx];

                match c {
                    '1' => j += 1,
                    '2' => j += 2,
                    '3' => j += 3,
                    '4' => j += 4,
                    '5' => j += 5,
                    '6' => j += 6,
                    '7' => j += 7,
                    'P' | 'N' | 'B' | 'R' | 'Q' | 'K' | 'p' | 'n' | 'b' | 'r' | 'q' | 'k' => {
                        let ty = match c.to_ascii_lowercase() {
                            'p' => Pawn,
                            'n' => Knight,
                            'b' => Bishop,
                            'r' => Rook,
                            'q' => Queen,
                            'k' => King,
                            _ => unreachable!(),
                        };
                        let color = match c.is_lowercase() {
                            true => PieceColor::Black,
                            false => PieceColor::White,
                        };

                        let piece = Piece { ty, color };
                        let square = i * 8 + j as usize;
                        mailbox[square] = piece;
                        bitboards[piece] |= 1 << square;

                        j += 1;
                    }
                    _ => return None,
                }
                idx += 1;
            }
        }
        bitboards[PieceColor::White][Empty] = !bitboards[0].iter().fold(0, |x, y| x | y);
        bitboards[PieceColor::Black][Empty] = !bitboards[1].iter().fold(0, |x, y| x | y);

        let to_move = match parts[1] {
            "w" => PieceColor::White,
            "b" => PieceColor::Black,
            _ => return None,
        };

        let castling = match parts[2] {
            "-" => [[false; 2]; 2],
            s => {
                let mut castling = [[false; 2]; 2];

                for c in s.chars() {
                    match c {
                        'K' => castling[PieceColor::White][1] = true,
                        'Q' => castling[PieceColor::White][0] = true,
                        'k' => castling[PieceColor::Black][1] = true,
                        'q' => castling[PieceColor::Black][0] = true,
                        _ => return None,
                    }
                }

                castling
            }
        };

        let en_passant = match parts[3] {
            "-" => -1,
            s => {
                let chars = s.chars().collect::<Vec<char>>();
                if chars.len() != 2 {
                    return None;
                }
                let file = chars[0] as u8 - b'a';
                let rank = chars[1] as u8 - b'1';

                if file > 7 || (rank != 2 && rank != 5) {
                    return None;
                }

                (rank * 8 + file) as i8
            }
        };

        let halfmove_clock = parts.get(4).unwrap_or(&"0").parse().ok()?;
        let fullmove_clock = parts.get(5).unwrap_or(&"1").parse().ok()?;

        let mut x = Self {
            bitboards,
            mailbox,
            to_move,
            tt: TranspositionTable::new_default(),
            fullmove_clock,
            incremental_state: [State {
                en_passant,
                castling,
                halfmove_clock,
                state_offset: halfmove_clock,
                captured: None,
                zobrist: 0,
            }; 256],
            state_index: 0,
        };

        x.set_zobrist(hash(&x));

        Some(x)
    }
}
