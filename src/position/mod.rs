use crate::{
    movegen::{is_attacking, Code, Move},
    zobrist::hash,
};

use self::{
    piece::{
        Piece, PieceColor,
        PieceType::{self, *},
    },
    tt::TranspositionTable,
};

mod fen;
pub mod piece;
pub mod tt;

pub struct Position {
    pub bitboards: [[u64; 7]; 2],
    pub mailbox: [Piece; 64],
    pub to_move: PieceColor,

    pub fullmove_clock: u16,
    pub tt: TranspositionTable,

    incremental_state: [State; 256],
    state_index: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct State {
    en_passant: i8,
    halfmove_clock: u8,
    state_offset: u8,
    castling: [[bool; 2]; 2],
    captured: Option<PieceType>,
    zobrist: u64,
}

impl Position {
    pub fn is_in_check(&self, color: PieceColor) -> bool {
        is_attacking(
            self.bitboards[color][PieceType::King].trailing_zeros() as u8,
            self,
            !color,
        )
    }

    pub fn check_draws(&self) -> bool {
        if self.halfmove_clock() >= 100 {
            return true;
        }

        let start = self.state_index - (self.halfmove_clock() - self.state_offset()) as usize;
        for x in &self.incremental_state[start..=self.state_index] {
            let mut n = 0;
            for y in &self.incremental_state[start..=self.state_index] {
                if x.zobrist == y.zobrist {
                    n += 1;
                }

                if n >= 3 {
                    return true;
                }
            }
        }

        false
    }

    pub fn clear_incremental(&mut self) {
        let start = self.state_index - (self.halfmove_clock() - self.state_offset()) as usize;
        if start == 0 {
            return;
        }
        for x in start..=self.state_index {
            self.incremental_state[x - start] = self.incremental_state[x];
        }
        self.state_index -= start;
    }

    pub fn calc_zobrist(&mut self) -> u64 {
        let hash = hash(self);
        self.set_zobrist(hash);

        hash
    }

    pub fn make_move(&mut self, m: Move) {
        let to_move = self.to_move;
        self.incremental_state[self.state_index + 1] = self.incremental_state[self.state_index];
        self.state_index += 1;

        let mut piece = self.mailbox[m.from() as usize];
        let from_bit = 1u64 << m.from();
        let to_bit = 1u64 << m.to();

        if m.is_capture() || piece.ty == PieceType::Pawn {
            *self.halfmove_clock_mut() = 0;
            self.set_state_offset(0);
        } else {
            *self.halfmove_clock_mut() += 1;
        }

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
            self.mailbox[sq as usize].ty = PieceType::Empty;
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
        let m = Move::from_uci(self, s);
        if let Some(m) = m {
            // TODO: check if legal
            self.make_move(m);
        }
    }
}

// Getters and setters
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

    pub fn state_offset(&self) -> u8 {
        self.incremental_state[self.state_index].state_offset
    }

    pub fn set_state_offset(&mut self, state_offset: u8) {
        self.incremental_state[self.state_index].state_offset = state_offset;
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

    pub fn zobrist(&self) -> u64 {
        self.incremental_state[self.state_index].zobrist
    }

    pub fn set_zobrist(&mut self, zobrist: u64) {
        self.incremental_state[self.state_index].zobrist = zobrist;
    }
}
