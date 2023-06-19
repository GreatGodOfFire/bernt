#![feature(const_trait_impl)]

use std::{fmt::Display, mem};

use bitboard::Bitboard;
use piece::{Piece, PieceColor, PieceType};
use stack::{Stack, State};

pub mod bitboard;
pub mod fen;
pub mod piece;
mod stack;

pub struct Position {
    bitboards: [[Bitboard; 7]; 2],
    mailbox: [Piece; 64],
    to_move: PieceColor,

    fullmove_clock: u16,

    stack: Stack<State>,
    repetition_table: Stack<u64>,
}

impl Position {
    pub fn new_empty(&self) -> Self {
        let white_bitboard = [Bitboard::empty(); 7];

        Self {
            bitboards: [white_bitboard, white_bitboard],
            mailbox: [Piece::default(); 64],
            to_move: PieceColor::White,
            fullmove_clock: 1,
            stack: Stack::new(),
            repetition_table: Stack::new(),
        }
    }

    pub fn check_draws(&self) -> bool {
        if self.halfmove_clock() >= 100 {
            return true;
        }

        if self.halfmove_clock() < 4 {
            return false;
        }

        let mut d = 4;
        let mut n = 0;

        while d <= self.halfmove_clock() as usize {
            if self.repetition_table[0] == self.repetition_table[d] {
                n += 1;
                if n == 2 {
                    return true;
                }
            }

            d += 2;
        }

        false
    }

    pub fn make_move(&mut self, m: Move) {
        self.stack.clone_push();

        let mut piece = self.mailbox[m.from as usize];
        let from_bit = Bitboard(1 << m.from);
        let to_bit = Bitboard(1 << m.to);

        let to_move = self.to_move;

        if m.is_capture() || piece.ty == PieceType::Pawn {
            self.set_halfmove_clock(0);
        } else {
            *self.halfmove_clock_mut() += 1;
        }

        if m.flags == MoveFlags::EnPassantCapture {
            let sq = self.en_passant()
                + match to_move {
                    PieceColor::White => -8,
                    PieceColor::Black => 8,
                };

            let target = self.mailbox[sq as usize];
            let sq_bit = Bitboard(1 << sq);

            self.bitboards[target] ^= sq_bit;
            self.bitboards[!to_move][PieceType::Occupied] ^= sq_bit;
            self.mailbox[sq as usize].ty = PieceType::Empty;

            self.set_captured(None);
        } else if m.is_capture() {
            let target = self.mailbox[m.to as usize];
            self.bitboards[target] ^= to_bit;
            self.bitboards[!to_move][PieceType::Occupied] ^= to_bit;

            // take castling rights
            if target.ty == PieceType::Rook {
                if m.to == self.castling()[!to_move][0] as u8 {
                    self.castling_mut()[!to_move][0] = -1;
                }
                if m.to == self.castling()[!to_move][1] as u8 {
                    self.castling_mut()[!to_move][1] = -1;
                }
            }

            self.set_captured(Some(target.ty));
        } else {
            self.set_captured(None);
        }

        self.set_en_passant(-1);

        // en passant
        if m.flags == MoveFlags::DoublePawnPush {
            self.set_en_passant(match to_move {
                PieceColor::White => m.to as i8 - 8,
                PieceColor::Black => m.to as i8 + 8,
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
        if m.flags == MoveFlags::QueenCastle {
            let rook = self.castling()[to_move][0] as usize;

            self.mailbox[m.to as usize + 1] = self.mailbox[rook];
            self.mailbox[rook].ty = PieceType::Empty;

            self.bitboards[to_move][PieceType::Rook] ^= Bitboard(1 << rook) | to_bit << 1;
            self.bitboards[to_move][PieceType::Occupied] ^= Bitboard(1 << rook) | to_bit << 1;
        }
        if m.flags == MoveFlags::KingCastle {
            let rook = self.castling()[to_move][1] as usize;

            self.mailbox[m.to as usize - 1] = self.mailbox[rook];
            self.mailbox[rook].ty = PieceType::Empty;

            self.bitboards[to_move][PieceType::Rook] ^= Bitboard(1 << rook) | to_bit >> 1;
            self.bitboards[to_move][PieceType::Occupied] ^= Bitboard(1 << rook) | to_bit >> 1;
        }

        // take castling rights
        if piece.ty == PieceType::King {
            self.castling_mut()[to_move] = [-1, -1];
        }
        if piece.ty == PieceType::Rook && m.from == self.castling()[to_move][0] as u8 {
            self.castling_mut()[to_move][0] = -1;
        }
        if piece.ty == PieceType::Rook && m.from == self.castling()[to_move][1] as u8 {
            self.castling_mut()[to_move][1] = -1;
        }

        self.mailbox[m.from as usize].ty = PieceType::Empty;
        self.mailbox[m.to as usize] = piece;
        self.bitboards[to_move][PieceType::Occupied] ^= from_bit | to_bit;

        self.fullmove_clock += to_move as u16;

        self.to_move = !to_move;
    }

    pub fn unmake_move(&mut self, m: Move) {
        let to_move = !self.to_move;
        self.fullmove_clock -= to_move as u16;

        let from_bit = Bitboard(1 << m.from);
        let to_bit = Bitboard(1 << m.to);

        self.bitboards[to_move][PieceType::Occupied] ^= from_bit | to_bit;

        match m.promotion() {
            Some(ty) => {
                self.mailbox[m.from as usize].ty = PieceType::Pawn;
                self.mailbox[m.from as usize].color = to_move;
                self.bitboards[to_move][PieceType::Pawn] ^= from_bit;
                self.bitboards[to_move][ty] ^= to_bit;
            }
            None => {
                let ty = self.mailbox[m.to as usize].ty;
                self.mailbox[m.from as usize].ty = ty;
                self.mailbox[m.from as usize].color = to_move;
                self.bitboards[to_move][ty] ^= from_bit | to_bit;
            }
        }

        if let Some(captured) = self.captured() {
            self.mailbox[m.to as usize].ty = captured;
            self.mailbox[m.to as usize].color = !to_move;
            self.bitboards[!to_move][captured] ^= to_bit;
            self.bitboards[!to_move][PieceType::Occupied] ^= to_bit;
        } else {
            self.mailbox[m.to as usize].ty = PieceType::Empty;
        }

        self.stack.discard_top();

        if m.flags == MoveFlags::QueenCastle {
            let rook_from = self.castling()[to_move][0] as usize;
            let rook_to = m.to as usize + 1;
            self.mailbox[rook_from] = self.mailbox[rook_to];
            self.mailbox[rook_to].ty = PieceType::Empty;
            self.bitboards[to_move][PieceType::Rook] ^=
                Bitboard(1 << rook_from) | Bitboard(1 << rook_to);
            self.bitboards[to_move][PieceType::Occupied] ^=
                Bitboard(1 << rook_from) | Bitboard(1 << rook_to);
        } else if m.flags == MoveFlags::KingCastle {
            let rook_from = self.castling()[to_move][1] as usize;
            let rook_to = m.to as usize - 1;
            self.mailbox[rook_from] = self.mailbox[rook_to];
            self.mailbox[rook_to].ty = PieceType::Empty;
            self.bitboards[to_move][PieceType::Rook] ^=
                Bitboard(1 << rook_from) | Bitboard(1 << rook_to);
            self.bitboards[to_move][PieceType::Occupied] ^=
                Bitboard(1 << rook_from) | Bitboard(1 << rook_to);
        }

        if m.flags == MoveFlags::EnPassantCapture {
            let sq = (self.en_passant()
                + match to_move {
                    PieceColor::White => -8,
                    PieceColor::Black => 8,
                }) as usize;
            self.mailbox[sq].ty = PieceType::Pawn;
            self.mailbox[sq].color = !to_move;
            self.bitboards[!to_move][PieceType::Pawn] ^= Bitboard(1 << sq);
            self.bitboards[!to_move][PieceType::Empty] ^= Bitboard(1 << sq);
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

    // Clears the stack
    pub fn finalize_moves(&mut self) {
        self.stack.clear();
        self.repetition_table.retain(self.halfmove_clock());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub flags: MoveFlags,
}

impl Move {
    #[inline]
    pub fn new(from: u8, to: u8, flags: MoveFlags) -> Self {
        Self { from, to, flags }
    }

    #[inline]
    pub fn is_capture(&self) -> bool {
        self.flags as u8 & 4 != 0
    }

    #[inline]
    pub fn promotion(&self) -> Option<PieceType> {
        if self.flags as u8 & 8 != 0 {
            // Safety: Safe because there are only 2 bits are remaining and PieceType has more than 2**2 = 4 variants (starting at 0)
            Some(unsafe { mem::transmute(self.flags as u8 & 3) })
        } else {
            None
        }
    }

    #[inline]
    pub fn null() -> Self {
        Self::new(0, 0, MoveFlags::Quiet)
    }

    pub fn from_uci(position: &Position, s: &str) -> Option<Self> {
        if s.len() > 3 && s.len() < 6 {
            let chars: Vec<_> = s.chars().collect();

            let from = chars[0] as u8 - b'a' + (chars[1] as u8 - b'1') * 8;
            let to = chars[2] as u8 - b'a' + (chars[3] as u8 - b'1') * 8;

            if from >= 64 || to >= 64 {
                return None;
            }

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
            let piece = position.mailbox[from as usize];
            let destination = position.mailbox[to as usize];

            let mut code = 0;

            match (piece.ty, to as i8 - from as i8) {
                (PieceType::King, 2) => {
                    return Some(Move::new(from, to, MoveFlags::KingCastle));
                }
                (PieceType::King, -2) => {
                    return Some(Move::new(from, to, MoveFlags::QueenCastle));
                }
                (PieceType::Pawn, -16 | 16) => {
                    return Some(Move::new(from, to, MoveFlags::DoublePawnPush));
                }
                _ => {}
            }

            if to as i8 == position.en_passant() && piece.ty == PieceType::Pawn {
                return Some(Move::new(from, to, MoveFlags::EnPassantCapture));
            }

            if destination.ty != PieceType::Empty {
                code |= 1 << 2;
            }

            if let Some(promotion) = promotion {
                code |= promotion as u16 | 1 << 3;
            }

            // Safety: Only the bits 0b0100 and/or 0b1011 will be set. The only values for which
            // `Code` has no enum values are 0b0110 and 0b0111 but both can't be reached as both
            // require the 0b0010 bit which can only be set with the 0b1000 bit.
            Some(Move::new(from, to, unsafe { mem::transmute(code) }))
        } else {
            None
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        s.push((b'a' + self.from % 8) as char);
        s.push((b'1' + self.from / 8) as char);
        s.push((b'a' + self.to % 8) as char);
        s.push((b'1' + self.to / 8) as char);

        if let Some(promotion) = self.promotion() {
            const PROMOTIONS: [char; 4] = ['n', 'b', 'r', 'q'];

            s.push(PROMOTIONS[promotion])
        }

        f.write_str(&s)
    }
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveFlags {
    Quiet = 0,
    DoublePawnPush,
    KingCastle,
    QueenCastle,
    Capture,
    EnPassantCapture,
    KnightPromotion = 8,
    BishopPromotion,
    RookPromotion,
    QueenPromotion,
    KnightPromotionCapture,
    BishopPromotionCapture,
    RookPromotionCapture,
    QueenPromotionCapture,
}

impl From<MoveFlags> for u8 {
    fn from(code: MoveFlags) -> Self {
        code as u8
    }
}

impl Position {
    #[inline]
    pub fn fullmove_clock(&self) -> u16 {
        self.fullmove_clock
    }

    #[inline]
    pub fn to_move(&self) -> PieceColor {
        self.to_move
    }

    #[inline]
    pub fn mailbox(&self) -> &[Piece; 64] {
        &self.mailbox
    }

    #[inline]
    pub fn bitboards(&self) -> &[[Bitboard; 7]; 2] {
        &self.bitboards
    }

    #[inline]
    pub fn bitboards_mut(&mut self) -> &mut [[Bitboard; 7]; 2] {
        &mut self.bitboards
    }

    // Incremental values
    #[inline]
    pub fn en_passant(&self) -> i8 {
        self.stack[0].en_passant
    }

    #[inline]
    pub fn set_en_passant(&mut self, en_passant: i8) {
        self.stack[0].en_passant = en_passant;
    }

    #[inline]
    pub fn halfmove_clock(&self) -> u8 {
        self.stack[0].halfmove_clock
    }

    #[inline]
    pub fn halfmove_clock_mut(&mut self) -> &mut u8 {
        &mut self.stack[0].halfmove_clock
    }

    #[inline]
    pub fn set_halfmove_clock(&mut self, halfmove_clock: u8) {
        self.stack[0].halfmove_clock = halfmove_clock;
    }

    #[inline]
    pub fn castling(&self) -> &[[i8; 2]; 2] {
        &self.stack[0].castling
    }

    #[inline]
    pub fn castling_mut(&mut self) -> &mut [[i8; 2]; 2] {
        &mut self.stack[0].castling
    }

    #[inline]
    pub fn captured(&self) -> Option<PieceType> {
        self.stack[0].captured
    }

    #[inline]
    pub fn set_captured(&mut self, captured: Option<PieceType>) {
        self.stack[0].captured = captured;
    }

    #[inline]
    pub fn zobrist(&self) -> u64 {
        self.repetition_table[0]
    }

    #[inline]
    pub fn set_zobrist(&mut self, zobrist: u64) {
        self.repetition_table[0] = zobrist;
    }
}
