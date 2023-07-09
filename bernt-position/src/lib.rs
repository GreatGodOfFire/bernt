#![feature(const_mut_refs)]

use std::{fmt::Display, mem};

use piece::{
    Piece, PieceColor,
    PieceType::{self, *},
};
use stack::{Stack, State};

pub mod bitboard;
pub mod fen;
pub mod piece;
mod stack;
mod zobrist;

#[derive(Clone)]
pub struct Position {
    bitboards: [[u64; 7]; 2],
    mailbox: [Piece; 64],
    to_move: PieceColor,

    fullmove_clock: u16,

    stack: Stack<State>,
    repetition_table: Stack<u64>,

    variant: Variant,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
    #[default]
    Standard,
    FRC,
}

impl Position {
    pub fn new_empty() -> Self {
        let bitboards = [[0; 7]; 2];

        Self {
            bitboards,
            mailbox: [Piece::default(); 64],
            to_move: PieceColor::White,
            fullmove_clock: 1,
            stack: Stack::new(),
            repetition_table: Stack::new(),
            variant: Variant::Standard,
        }
    }

    pub fn calc_zobrist(&mut self) -> u64 {
        let hash = zobrist::hash(self);
        self.set_zobrist(hash);

        hash
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

        while d < self.halfmove_clock() as usize {
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
        let to_move = self.to_move;
        self.stack.clone_push();
        self.repetition_table.clone_push();

        let mut piece = self.mailbox[m.from as usize];
        let from_bit = 1u64 << m.from;
        let to_bit = 1u64 << m.to;

        if m.is_capture() || piece.ty == PieceType::Pawn {
            *self.halfmove_clock_mut() = 0;
        } else {
            *self.halfmove_clock_mut() += 1;
        }

        // captures
        if m.ty == MoveType::EnPassantCapture {
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
            let target = self.mailbox[m.to as usize];
            self.bitboards[target] ^= to_bit;
            self.bitboards[!self.to_move][PieceType::Empty] ^= to_bit;

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
        if m.ty == MoveType::DoublePawnPush {
            self.set_en_passant(match self.to_move {
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
        if m.ty == MoveType::LeftCastle {
            // self.mailbox[m.to as usize + 1] = self.mailbox[m.to as usize - 2];
            self.mailbox[m.to as usize + 1] = self.mailbox[self.castling()[to_move][0] as usize];
            self.mailbox[m.to as usize - 2].ty = PieceType::Empty;
            self.bitboards[self.to_move][PieceType::Rook] ^= to_bit >> 2 | to_bit << 1;
            self.bitboards[self.to_move][PieceType::Empty] ^= to_bit >> 2 | to_bit << 1;
        }
        if m.ty == MoveType::RightCastle {
            self.mailbox[m.to as usize - 1] = self.mailbox[self.castling()[to_move][1] as usize];
            self.mailbox[m.to as usize + 1].ty = PieceType::Empty;
            self.bitboards[self.to_move][PieceType::Rook] ^= to_bit << 1 | to_bit >> 1;
            self.bitboards[self.to_move][PieceType::Empty] ^= to_bit << 1 | to_bit >> 1;
        }

        // take castling rights
        if piece.ty == PieceType::King {
            self.castling_mut()[to_move] = [-1, -1];
        }
        if m.from == self.castling()[to_move][0] as u8 {
            self.castling_mut()[to_move][0] = -1;
        }
        if m.from == self.castling()[to_move][1] as u8 {
            self.castling_mut()[to_move][1] = -1;
        }

        self.mailbox[m.from as usize].ty = PieceType::Empty;
        self.mailbox[m.to as usize] = piece;
        self.bitboards[self.to_move][PieceType::Empty] ^= from_bit | to_bit;

        self.fullmove_clock += self.to_move as u16;

        self.to_move = !self.to_move;
    }

    pub fn unmake_move(&mut self, m: Move) {
        let to_move = !self.to_move;
        self.fullmove_clock -= to_move as u16;

        self.bitboards[to_move][Empty] ^= 1 << m.from | 1 << m.to;

        match m.promotion() {
            Some(ty) => {
                self.mailbox[m.from as usize].ty = PieceType::Pawn;
                self.mailbox[m.from as usize].color = to_move;
                self.bitboards[to_move][PieceType::Pawn] ^= 1 << m.from;
                self.bitboards[to_move][ty] ^= 1 << m.to;
            }
            None => {
                let ty = self.mailbox[m.to as usize].ty;
                self.mailbox[m.from as usize].ty = ty;
                self.mailbox[m.from as usize].color = to_move;
                self.bitboards[to_move][ty] ^= 1 << m.from | 1 << m.to;
            }
        }

        if m.ty == MoveType::LeftCastle {
            let rook_from = to_move as usize * 56;
            let rook_to = rook_from + 3;
            self.mailbox[rook_from] = self.mailbox[rook_to];
            self.mailbox[rook_to].ty = PieceType::Empty;
            self.bitboards[to_move][PieceType::Rook] ^= 1 << rook_from | 1 << rook_to;
            self.bitboards[to_move][Empty] ^= 1 << rook_from | 1 << rook_to;
        } else if m.ty == MoveType::RightCastle {
            let rook_from = 7 + to_move as usize * 56;
            let rook_to = rook_from - 2;
            self.mailbox[rook_from] = self.mailbox[rook_to];
            self.mailbox[rook_to].ty = PieceType::Empty;
            self.bitboards[to_move][PieceType::Rook] ^= 1 << rook_from | 1 << rook_to;
            self.bitboards[to_move][Empty] ^= 1 << rook_from | 1 << rook_to;
        }

        if let Some(captured) = self.captured() {
            self.mailbox[m.to as usize].ty = captured;
            self.mailbox[m.to as usize].color = !to_move;
            self.bitboards[!to_move][captured] ^= 1 << m.to;
            self.bitboards[!to_move][Empty] ^= 1 << m.to;
        } else {
            self.mailbox[m.to as usize].ty = PieceType::Empty;
        }

        self.stack.discard_top();
        self.repetition_table.discard_top();

        if m.ty == MoveType::EnPassantCapture {
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

    // Clears the stack
    pub fn finalize_moves(&mut self) {
        self.stack.retain(1);
        self.repetition_table.retain(self.halfmove_clock().max(1));
    }

    pub fn variant(&self) -> Variant {
        self.variant
    }

    pub fn set_variant(&mut self, variant: Variant) {
        self.variant = variant;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub ty: MoveType,
}

impl Move {
    #[inline]
    pub fn new(from: u8, to: u8, ty: MoveType) -> Self {
        Self { from, to, ty }
    }

    #[inline]
    pub fn is_capture(&self) -> bool {
        self.ty as u8 & 4 != 0
    }

    #[inline]
    pub fn promotion(&self) -> Option<PieceType> {
        if self.ty as u8 & 8 != 0 {
            // Safety: Safe because there are only 2 bits are remaining and PieceType has more than 2**2 = 4 variants (starting at 0)
            Some(unsafe { mem::transmute(self.ty as u8 & 3) })
        } else {
            None
        }
    }

    #[inline]
    pub fn null() -> Self {
        Self::new(0, 0, MoveType::Quiet)
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
                    return Some(Move::new(from, to, MoveType::RightCastle));
                }
                (PieceType::King, -2) => {
                    return Some(Move::new(from, to, MoveType::LeftCastle));
                }
                (PieceType::Pawn, -16 | 16) => {
                    return Some(Move::new(from, to, MoveType::DoublePawnPush));
                }
                _ => {}
            }

            if to as i8 == position.en_passant() && piece.ty == PieceType::Pawn {
                return Some(Move::new(from, to, MoveType::EnPassantCapture));
            }

            if destination.ty != PieceType::Empty {
                code |= 1 << 2;
            }

            if let Some(promotion) = promotion {
                code |= promotion as u16 | 1 << 3;
            }

            // Safety: Only the bits 0b0100 and/or 0b1011 will be set. The only values for which
            // `MoveFlags` has no enum values are 0b0110 and 0b0111 but both can't be reached as both
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u16)]
pub enum MoveType {
    #[default]
    Quiet = 0,
    DoublePawnPush,
    RightCastle,
    LeftCastle,
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

impl MoveType {
    fn new_promotion(ty: PieceType) -> Self {
        match ty {
            Knight => Self::KnightPromotion,
            Bishop => Self::BishopPromotion,
            Rook => Self::RookPromotion,
            Queen => Self::QueenPromotion,
            _ => panic!(),
        }
    }
    fn new_promotion_capture(ty: PieceType) -> Self {
        match ty {
            Knight => Self::KnightPromotionCapture,
            Bishop => Self::BishopPromotionCapture,
            Rook => Self::RookPromotionCapture,
            Queen => Self::QueenPromotionCapture,
            _ => panic!(),
        }
    }
}

impl From<MoveType> for u8 {
    fn from(code: MoveType) -> Self {
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
    pub fn bitboards(&self) -> &[[u64; 7]; 2] {
        &self.bitboards
    }

    #[inline]
    pub fn bitboards_mut(&mut self) -> &mut [[u64; 7]; 2] {
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
