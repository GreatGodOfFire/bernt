use crate::movegen::{Code, Move};

use self::piece::{
    Piece, PieceColor,
    PieceType::{self, *},
};

pub mod piece;

#[derive(Debug, Clone)]
pub struct Position {
    pub bitboards: [[u64; 7]; 2],
    pub mailbox: [Piece; 64],
    pub to_move: PieceColor,
    pub castling: [[bool; 2]; 2],
    pub en_passant: i8,

    pub halfmove_clock: u8,
    pub fullmove_clock: u16,
}

impl Position {
    pub fn make_move(&self, m: Move) -> Self {
        let mut pos = self.clone();

        pos.en_passant = -1;

        let mut piece = pos.mailbox[m.from() as usize];
        let from_bit = 1u64 << m.from();
        let to_bit = 1u64 << m.to();

        // en passant
        if m.code() == Code::DoublePawnPush {
            pos.en_passant = match pos.to_move {
                PieceColor::White => m.to() as i8 - 8,
                PieceColor::Black => m.to() as i8 + 8,
            };
        }

        // captures
        if m.is_capture() {
            let target = pos.mailbox[m.to() as usize];
            pos.bitboards[target] ^= to_bit;
            pos.bitboards[!pos.to_move][PieceType::Empty] ^= to_bit;
        }

        // promotions
        if let Some(ty) = m.promotion() {
            piece.ty = ty;
            pos.bitboards[piece] ^= from_bit;
        } else {
            pos.bitboards[piece] ^= from_bit | to_bit;
        }

        // castling
        if m.code() == Code::QueenCastle {
            pos.mailbox[m.to() as usize + 1] = pos.mailbox[m.to() as usize - 2];
            pos.mailbox[m.to() as usize - 2].ty = PieceType::Empty;
            pos.bitboards[pos.to_move][PieceType::Rook] ^= to_bit >> 2 | to_bit << 1;
            pos.bitboards[pos.to_move][PieceType::Empty] ^= to_bit >> 2 | to_bit << 1;
        }
        if m.code() == Code::KingCastle {
            pos.mailbox[m.to() as usize - 1] = pos.mailbox[m.to() as usize + 1];
            pos.mailbox[m.to() as usize + 1].ty = PieceType::Empty;
            pos.bitboards[pos.to_move][PieceType::Rook] ^= to_bit << 1 | to_bit >> 1;
            pos.bitboards[pos.to_move][PieceType::Empty] ^= to_bit << 1 | to_bit >> 1;
        }

        // take castling rights
        if piece.ty == PieceType::King {
            pos.castling[pos.to_move] = [false, false];
        }
        if piece.ty == PieceType::Rook && (m.from() == 0 || m.from() == 56) {
            pos.castling[pos.to_move][0] = false;
        }
        if piece.ty == PieceType::Rook && (m.from() == 7 || m.from() == 63) {
            pos.castling[pos.to_move][1] = false;
        }

        pos.mailbox[m.from() as usize].ty = PieceType::Empty;
        pos.mailbox[m.to() as usize] = piece;
        pos.bitboards[pos.to_move][PieceType::Empty] ^= from_bit | to_bit;

        pos.fullmove_clock += pos.to_move as u16;
        pos.halfmove_clock += 1;

        pos.to_move = !pos.to_move;        

        pos
    }

    pub fn make_move_uci(&self, s: &str) -> Position {
        if s.len() > 3 && s.len() < 6 {
            let chars: Vec<_> = s.chars().collect();

            let from = chars[0] as u16 - 'a' as u16 + (chars[1] as u16 - '1' as u16) * 8;
            let to = chars[2] as u16 - 'a' as u16 + (chars[3] as u16 - '1' as u16) * 8;

            let promotion = if s.len() == 5 {
                let x = match chars[4] {
                    'n' => PieceType::Knight,
                    'b' => PieceType::Bishop,
                    'r' => PieceType::Rook,
                    'q' => PieceType::Queen,
                    _ => panic!(),
                };
                Some(x)
            } else {
                None
            };
            let piece = self.mailbox[from as usize];
            let destination = self.mailbox[to as usize];

            let mut code = 0;

            match (piece.ty, to as i8 - from as i8) {
                (PieceType::King, 2) => {
                    return self.make_move(Move::new(from, to, Code::KingCastle))
                }
                (PieceType::King, -2) => {
                    return self.make_move(Move::new(from, to, Code::QueenCastle))
                }
                (PieceType::Pawn, -8 | 8) => {
                    return self.make_move(Move::new(from, to, Code::DoublePawnPush))
                }
                _ => {}
            }

            if to as i8 == self.en_passant {
                return self.make_move(Move::new(from, to, Code::EnPassantCapture));
            }

            if destination.ty != PieceType::Empty {
                code |= 1 << 2;
            }

            if let Some(promotion) = promotion {
                code |= promotion as u16 | 1 << 3;
            }

            self.make_move(Move::new(from, to, unsafe { std::mem::transmute(code) }))
        } else {
            panic!()
        }
    }

    pub fn startpos() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut bitboards = [[0u64; 7]; 2];
        let mut mailbox = [Piece::EMPTY; 64];
        // let en_passant: i8,

        let parts = fen.split(" ").collect::<Vec<_>>();
        if parts.len() != 6 {
            return None;
        }
        let ranks = parts[0].split("/");
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
                        let square = i as usize * 8 + j as usize;
                        mailbox[square] = piece;
                        bitboards[piece] |= 1 << square;

                        j += 1;
                        idx += 1;
                    }
                    _ => return None,
                }
            }
        }
        bitboards[PieceColor::White][PieceType::Empty] = !bitboards[0].iter().fold(0, |x, y| x | y);
        bitboards[PieceColor::Black][PieceType::Empty] = !bitboards[1].iter().fold(0, |x, y| x | y);

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
                let file = chars[0] as u8 - 'a' as u8;
                let rank = chars[1] as u8 - '1' as u8;

                if file > 7 || (rank != 2 && rank != 5) {
                    return None;
                }

                (rank * 8 + file) as i8
            }
        };

        let halfmove_clock = parts[4].parse().ok()?;
        let fullmove_clock = parts[5].parse().ok()?;

        Some(Self {
            bitboards,
            mailbox,
            to_move,
            castling,
            en_passant,
            halfmove_clock,
            fullmove_clock,
        })
    }
}
