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

    fullmove_clock: u16,

    incremental_state: [State; 256],
    state_index: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct State {
    en_passant: i8,
    halfmove_clock: u8,
    castling: [[bool; 2]; 2],
    captured: Option<PieceType>,
}

impl Position {
    pub fn en_passant(&self) -> i8 {
        self.incremental_state[self.state_index].en_passant
    }

    pub fn set_en_passant(&mut self, en_passant: i8) {
        self.incremental_state[self.state_index].en_passant = en_passant;
    }

    pub fn halfmove_clock(&self) -> u8 {
        self.incremental_state[self.state_index].halfmove_clock
    }

    pub fn halfmove_clock_mut(&mut self) -> &mut u8 {
        &mut self.incremental_state[self.state_index].halfmove_clock
    }

    pub fn castling(&self) -> [[bool; 2]; 2] {
        self.incremental_state[self.state_index].castling
    }

    pub fn castling_mut(&mut self) -> &mut [[bool; 2]; 2] {
        &mut self.incremental_state[self.state_index].castling
    }

    pub fn captured(&self) -> Option<PieceType> {
        self.incremental_state[self.state_index].captured
    }

    pub fn set_captured(&mut self, captured: Option<PieceType>) {
        self.incremental_state[self.state_index].captured = captured;
    }

    pub fn make_move(&mut self, m: Move) {
        let to_move = self.to_move;
        self.incremental_state[self.state_index + 1] = self.incremental_state[self.state_index];
        self.state_index += 1;

        let mut piece = self.mailbox[m.from() as usize];
        let from_bit = 1u64 << m.from();
        let to_bit = 1u64 << m.to();

        // captures
        if m.code() == Code::EnPassantCapture {
            let sq = (self.en_passant()
                + match self.to_move {
                    PieceColor::White => -8,
                    PieceColor::Black => 8,
                }) as u64;

            let target = self.mailbox[sq as usize];
            let sq_bit = 1 << sq;
            self.bitboards[target] ^= sq_bit;
            self.bitboards[!self.to_move][PieceType::Empty] ^= sq_bit;
            self.set_captured(None);
        } else if m.is_capture() {
            let target = self.mailbox[m.to() as usize];
            self.bitboards[target] ^= to_bit;
            self.bitboards[!self.to_move][PieceType::Empty] ^= to_bit;

            // take castling rights
            if target.ty == PieceType::Rook {
                if m.to().wrapping_sub(56 * !self.to_move as u16) == 0 {
                    self.incremental_state[self.state_index].castling[!self.to_move][0] = false;
                }
                if m.to().wrapping_sub(56 * !self.to_move as u16) == 7 {
                    self.incremental_state[self.state_index].castling[!self.to_move][1] = false;
                }
            }

            self.set_captured(Some(target.ty));
        } else {
            self.set_captured(None);
        }

        self.set_en_passant(-1);

        // en passant
        if m.code() == Code::DoublePawnPush {
            self.set_en_passant(match self.to_move {
                PieceColor::White => m.to() as i8 - 8,
                PieceColor::Black => m.to() as i8 + 8,
            });
        }

        // promotions
        if let Some(ty) = m.promotion() {
            self.bitboards[piece] ^= from_bit;
            piece.ty = ty;
            self.bitboards[piece] ^= to_bit;
        } else {
            self.bitboards[piece] ^= from_bit | to_bit;
        }

        // castling
        if m.code() == Code::QueenCastle {
            self.mailbox[m.to() as usize + 1] = self.mailbox[m.to() as usize - 2];
            self.mailbox[m.to() as usize - 2].ty = PieceType::Empty;
            self.bitboards[self.to_move][PieceType::Rook] ^= to_bit >> 2 | to_bit << 1;
            self.bitboards[self.to_move][PieceType::Empty] ^= to_bit >> 2 | to_bit << 1;
        }
        if m.code() == Code::KingCastle {
            self.mailbox[m.to() as usize - 1] = self.mailbox[m.to() as usize + 1];
            self.mailbox[m.to() as usize + 1].ty = PieceType::Empty;
            self.bitboards[self.to_move][PieceType::Rook] ^= to_bit << 1 | to_bit >> 1;
            self.bitboards[self.to_move][PieceType::Empty] ^= to_bit << 1 | to_bit >> 1;
        }

        // take castling rights
        if piece.ty == PieceType::King {
            self.castling_mut()[to_move] = [false, false];
        }
        if piece.ty == PieceType::Rook && (m.from() == 56 * self.to_move as u16) {
            self.castling_mut()[to_move][0] = false;
        }
        if piece.ty == PieceType::Rook && (m.from() == 56 * self.to_move as u16 + 7) {
            self.castling_mut()[to_move][1] = false;
        }

        self.mailbox[m.from() as usize].ty = PieceType::Empty;
        self.mailbox[m.to() as usize] = piece;
        self.bitboards[self.to_move][PieceType::Empty] ^= from_bit | to_bit;

        self.fullmove_clock += self.to_move as u16;
        *self.halfmove_clock_mut() += 1;

        self.to_move = !self.to_move;
    }

    pub fn unmake_move(&mut self, m: Move) {
        let to_move = !self.to_move;
        self.fullmove_clock -= to_move as u16;

        self.bitboards[to_move][Empty] ^= 1 << m.from() | 1 << m.to();

        match m.promotion() {
            Some(ty) => {
                self.mailbox[m.from() as usize].ty = PieceType::Pawn;
                self.mailbox[m.from() as usize].color = to_move;
                self.bitboards[to_move][PieceType::Pawn] ^= 1 << m.from();
                self.bitboards[to_move][ty] ^= 1 << m.to();
            }
            None => {
                let ty = self.mailbox[m.to() as usize].ty;
                self.mailbox[m.from() as usize].ty = ty;
                self.mailbox[m.from() as usize].color = to_move;
                self.bitboards[to_move][ty] ^= 1 << m.from() | 1 << m.to();
            }
        }

        if m.code() == Code::QueenCastle {
            let rook_from = to_move as usize * 56;
            let rook_to = rook_from + 3;
            self.mailbox[rook_from] = self.mailbox[rook_to];
            self.mailbox[rook_to].ty = PieceType::Empty;
            self.bitboards[to_move][PieceType::Rook] ^= 1 << rook_from | 1 << rook_to;
            self.bitboards[to_move][Empty] ^= 1 << rook_from | 1 << rook_to;
        } else if m.code() == Code::KingCastle {
            let rook_from = 7 + to_move as usize * 56;
            let rook_to = rook_from - 2;
            self.mailbox[rook_from] = self.mailbox[rook_to];
            self.mailbox[rook_to].ty = PieceType::Empty;
            self.bitboards[to_move][PieceType::Rook] ^= 1 << rook_from | 1 << rook_to;
            self.bitboards[to_move][Empty] ^= 1 << rook_from | 1 << rook_to;
        }

        if let Some(captured) = self.captured() {
            self.mailbox[m.to() as usize].ty = captured;
            self.mailbox[m.to() as usize].color = !to_move;
            self.bitboards[!to_move][captured] ^= 1 << m.to();
            self.bitboards[!to_move][Empty] ^= 1 << m.to();
        } else {
            self.mailbox[m.to() as usize].ty = PieceType::Empty;
        }

        self.state_index -= 1;

        if m.code() == Code::EnPassantCapture {
            let sq = (self.en_passant()
                + match to_move {
                    PieceColor::White => -8,
                    PieceColor::Black => 8,
                }) as usize;
            self.mailbox[sq].ty = PieceType::Pawn;
            self.mailbox[sq].color = !to_move;
            self.bitboards[!to_move][Pawn] ^= 1 << sq;
            self.bitboards[!to_move][Empty] ^= 1 << sq;
        }

        self.to_move = !self.to_move;
    }

    pub fn make_move_uci(&mut self, s: &str) {
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
                    self.make_move(Move::new(from, to, Code::KingCastle));
                    return;
                }
                (PieceType::King, -2) => {
                    self.make_move(Move::new(from, to, Code::QueenCastle));
                    return;
                }
                (PieceType::Pawn, -16 | 16) => {
                    self.make_move(Move::new(from, to, Code::DoublePawnPush));
                    return;
                }
                _ => {}
            }

            if to as i8 == self.en_passant() {
                self.make_move(Move::new(from, to, Code::EnPassantCapture));
                return;
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

        Some(Self {
            bitboards,
            mailbox,
            to_move,
            fullmove_clock,
            incremental_state: [State {
                en_passant,
                castling,
                halfmove_clock,
                captured: None,
            }; 256],
            state_index: 0,
        })
    }
}
