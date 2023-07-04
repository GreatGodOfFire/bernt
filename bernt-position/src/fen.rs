use crate::{piece::PieceType, stack::Stack, Variant};

use super::{
    piece::{Piece, PieceColor, PieceType::*},
    Position, State,
};

impl Position {
    pub fn startpos() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn as_fen(&self, include_clocks: bool) -> String {
        let mut empty_squares = 0;

        let mut buf = String::with_capacity(80);

        for rank in self.mailbox.chunks(8).rev() {
            for piece in rank {
                match piece.ty {
                    Empty => {
                        empty_squares += 1;
                    }
                    x => {
                        if empty_squares != 0 {
                            buf.push((b'0' + empty_squares as u8) as char);
                            empty_squares = 0;
                        }
                        let is_upper = piece.color == PieceColor::White;

                        let mut c = match x {
                            Knight => 'n',
                            Bishop => 'b',
                            Rook => 'r',
                            Queen => 'q',
                            Pawn => 'p',
                            King => 'k',
                            Empty => unreachable!(),
                        };
                        if is_upper {
                            c = c.to_ascii_uppercase();
                        }
                        buf.push(c);
                    }
                }
            }
            if empty_squares != 0 {
                buf.push((b'0' + empty_squares as u8) as char);
                empty_squares = 0;
            }
            buf.push('/');
        }
        buf.pop();
        buf.push(' ');
        if self.to_move() == PieceColor::White {
            buf.push('w');
        } else {
            buf.push('b');
        }

        buf.push(' ');

        let mut has_castling = false;
        if self.castling()[0][1] != -1 {
            has_castling = true;
            buf.push('K');
        }
        if self.castling()[0][0] != -1 {
            has_castling = true;
            buf.push('Q');
        }
        if self.castling()[1][1] != -1 {
            has_castling = true;
            buf.push('k');
        }
        if self.castling()[1][0] != -1 {
            has_castling = true;
            buf.push('q');
        }
        if !has_castling {
            buf.push('-');
        }

        buf.push(' ');

        if self.en_passant() != -1 {
            let ep = self.en_passant() as u8;
            buf.push((b'a' + ep % 8) as char);
            buf.push((b'1' + ep / 8) as char);
        } else {
            buf.push('-');
        }

        if include_clocks {
            buf.push(' ');
            buf.push_str(&format!("{}", self.halfmove_clock()));
            buf.push(' ');
            buf.push_str(&format!("{}", self.fullmove_clock()));
        }

        buf
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut bitboards = [[0; 7]; 2];
        bitboards[0][PieceType::Empty] = u64::MAX;
        bitboards[1][PieceType::Empty] = u64::MAX;
        let mut mailbox = [Piece::EMPTY; 64];

        let parts = fen.split(' ').collect::<Vec<_>>();
        if parts.len() < 4 {
            return None;
        }
        let ranks = parts[0].split('/');

        let mut wking_file = None;
        let mut bking_file = None;

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
                        let color = match c.is_ascii_lowercase() {
                            true => PieceColor::Black,
                            false => PieceColor::White,
                        };

                        let piece = Piece { ty, color };

                        match piece {
                            Piece {
                                ty: King,
                                color: PieceColor::White,
                            } => wking_file = Some(j),
                            Piece {
                                ty: King,
                                color: PieceColor::Black,
                            } => bking_file = Some(j),
                            _ => {}
                        }
                        let square = i * 8 + j as usize;
                        mailbox[square] = piece;
                        bitboards[piece] |= 1 << square;
                        bitboards[color][Empty] ^= 1 << square;

                        j += 1;
                    }
                    _ => return None,
                }
                idx += 1;
            }
        }

        let to_move = match parts[1] {
            "w" => PieceColor::White,
            "b" => PieceColor::Black,
            _ => return None,
        };

        let wking_file = wking_file?;
        let bking_file = bking_file?;

        let castling = match parts[2] {
            "-" => [[-1; 2]; 2],
            s => {
                let mut castling = [[-1; 2]; 2];

                for c in s.chars() {
                    match c {
                        'K' => castling[PieceColor::White][1] = 7,
                        'Q' => castling[PieceColor::White][0] = 0,
                        'k' => castling[PieceColor::Black][1] = 63,
                        'q' => castling[PieceColor::Black][0] = 56,
                        // Support for Shredder-FEN for FRC
                        c if c.is_ascii_uppercase() => {
                            let rook_file = c as i8 - b'A' as i8;

                            if rook_file < wking_file {
                                castling[PieceColor::White][0] = rook_file;
                            } else {
                                castling[PieceColor::White][1] = rook_file;
                            }
                        }
                        c if c.is_ascii_lowercase() => {
                            let rook_file = c as i8 - b'A' as i8;

                            if rook_file < bking_file {
                                castling[PieceColor::Black][0] = rook_file;
                            } else {
                                castling[PieceColor::Black][1] = rook_file;
                            }
                        }
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

        let mut stack = Stack::new();

        stack.push(State {
            en_passant,
            halfmove_clock,
            castling,
            captured: None,
        });

        let mut repetition_table = Stack::new();
        repetition_table.push(0);

        Some(Self {
            bitboards,
            mailbox,
            to_move,
            fullmove_clock,
            stack,
            repetition_table,
            variant: Variant::Standard,
        })
    }
}
