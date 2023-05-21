use std::mem;

use crate::position::{
    piece::{
        PieceColor::{self, *},
        PieceType::{self, *},
    },
    Position,
};

use self::{
    king::{castling_moves, king_moves, lookup_king},
    knight::{knight_moves, single_knight_moves},
    pawn::{double_pawn_moves, en_passant, pawn_attacks, single_pawn_attacks, single_pawn_moves},
    sliding::{bishop_moves, rook_moves, single_bishop_moves, single_rook_moves},
};

pub mod bitboard;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod sliding;
pub mod util;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move(u16);

impl Move {
    #[inline]
    pub fn new(from: u16, to: u16, code: Code) -> Self {
        Self(from | (to << 6) | ((code as u16) << 12))
    }

    #[inline]
    pub fn from(&self) -> u16 {
        self.0 & 0x3f
    }

    #[inline]
    pub fn to(&self) -> u16 {
        (self.0 >> 6) & 0x3f
    }

    #[inline]
    pub fn code(&self) -> Code {
        unsafe { mem::transmute(self.0 >> 12) }
    }

    #[inline]
    pub fn is_capture(&self) -> bool {
        self.0 & 0x4000 != 0
    }

    #[inline]
    pub fn promotion(&self) -> Option<PieceType> {
        if self.0 & 0x8000 != 0 {
            // Safety: Safe because there are only 2 bits are remaining and PieceType has more than 2**2 = 4 variants (starting at 0)
            Some(unsafe { mem::transmute(((self.0 & 0x3000) >> 12) as u8) })
        } else {
            None
        }
    }

    pub fn null() -> Self {
        Self::new(0, 0, Code::Quiet)
    }

    pub fn from_uci(position: &Position, s: &str) -> Option<Self> {
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
            let piece = position.mailbox[from as usize];
            let destination = position.mailbox[to as usize];

            let mut code = 0;

            match (piece.ty, to as i8 - from as i8) {
                (PieceType::King, 2) => {
                    return Some(Move::new(from, to, Code::KingCastle));
                }
                (PieceType::King, -2) => {
                    return Some(Move::new(from, to, Code::QueenCastle));
                }
                (PieceType::Pawn, -16 | 16) => {
                    return Some(Move::new(from, to, Code::DoublePawnPush));
                }
                _ => {}
            }

            if to as i8 == position.en_passant() {
                return Some(Move::new(from, to, Code::EnPassantCapture));
            }

            if destination.ty != PieceType::Empty {
                code |= 1 << 2;
            }

            if let Some(promotion) = promotion {
                code |= promotion as u16 | 1 << 3;
            }

            Some(Move::new(from, to, unsafe { std::mem::transmute(code) }))
        } else {
            None
        }
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let promotion = match self.promotion() {
            Some(Knight) => "n",
            Some(Bishop) => "b",
            Some(Rook) => "r",
            Some(Queen) => "q",
            _ => "",
        };

        f.write_fmt(format_args!(
            "{}{}{promotion}",
            format_square(self.from() as u8),
            format_square(self.to() as u8)
        ))
    }
}

fn format_square(sq: u8) -> String {
    let file = sq % 8;
    let rank = sq / 8;
    format!("{}{}", (file + b'a') as char, (rank + b'1') as char)
}

// inspired by: https://www.chessprogramming.org/Encoding_Moves#From-To_Based
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
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

#[derive(Debug, PartialEq, Eq)]
pub enum Moves {
    PseudoLegalMoves(Vec<Move>),
    Stalemate,
    Checkmate,
}

pub struct PerftResult {
    pub all: u64,
    pub divided: Vec<(Move, u64)>,
}

pub fn perft_print(position: &mut Position, depth: u8) -> u64 {
    let mut divided = vec![];

    if depth == 0 {
        return 1;
    }

    let moves = movegen(position);

    match moves {
        Moves::PseudoLegalMoves(moves) => {
            let mut n = 0;

            for m in moves {
                position.make_move(m);
                if !position.is_in_check(!position.to_move) {
                    let x = perft(position, depth - 1);
                    n += x;

                    divided.push((m, x));
                    if cfg!(feature = "perftree") {
                        println!("{m:?} {x}");
                    } else {
                        println!("{m:?}: {x}");
                    }
                } else {
                    println!("{m:?}");
                }
                position.unmake_move(m);
            }

            n
        }
        Moves::Stalemate | Moves::Checkmate => 1,
    }
}

pub fn perft(position: &mut Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = movegen(position);

    match moves {
        Moves::PseudoLegalMoves(moves) => {
            let mut n = 0;

            for m in moves {
                position.make_move(m);
                if !position.is_in_check(!position.to_move) {
                    let x = perft(position, depth - 1);
                    n += x;
                } else {
                }
                position.unmake_move(m);
            }

            n
        }
        Moves::Stalemate | Moves::Checkmate => 1,
    }
}

pub fn is_attacking(square: u8, position: &Position, to_move: PieceColor) -> bool {
    let opponent_pieces = !position.bitboards[!to_move][Empty];
    let player_pieces = !position.bitboards[to_move][Empty];

    let opponent_rooks = position.bitboards[to_move][Rook] | position.bitboards[to_move][Queen];
    let rooks = single_rook_moves(square, 0, player_pieces | opponent_pieces);
    if rooks & opponent_rooks != 0 {
        return true;
    }
    let opponent_bishops = position.bitboards[to_move][Bishop] | position.bitboards[to_move][Queen];
    let bishops = single_bishop_moves(square, 0, player_pieces | opponent_pieces);
    if bishops & opponent_bishops != 0 {
        return true;
    }
    let knights = single_knight_moves(square);
    if knights & position.bitboards[to_move][Knight] != 0 {
        return true;
    }
    let pawns = single_pawn_attacks(square, !to_move);
    if pawns & position.bitboards[to_move][Pawn] != 0 {
        return true;
    }
    if (1 << square) & lookup_king(position.bitboards[to_move][King].trailing_zeros() as u8) != 0 {
        return true;
    }

    false
}

pub fn count_checking(position: &Position) -> u32 {
    let mut n = 0;

    let to_move = position.to_move;
    let king = position.bitboards[to_move][King];
    let king_sq = king.trailing_zeros() as u8;
    let player_pieces = !position.bitboards[to_move][Empty];
    let opponent_pieces = !position.bitboards[!to_move][Empty];

    let opponent_rooks = position.bitboards[!to_move][Rook] | position.bitboards[!to_move][Queen];
    let rooks = single_rook_moves(king_sq, 0, player_pieces | opponent_pieces);

    n += (rooks & opponent_rooks).count_ones();
    if n >= 2 {
        return n;
    }

    let opponent_bishops =
        position.bitboards[!to_move][Bishop] | position.bitboards[!to_move][Queen];
    let bishops = single_bishop_moves(king_sq, 0, player_pieces | opponent_pieces);

    n += (bishops & opponent_bishops).count_ones();
    if n >= 2 {
        return n;
    }

    let knights = single_knight_moves(king_sq);
    n += (knights & position.bitboards[!to_move][Knight]).count_ones();
    if n >= 2 {
        return n;
    }

    let pawns = single_pawn_attacks(king_sq, to_move);
    n += (pawns & position.bitboards[!to_move][Pawn]).count_ones();
    if n >= 2 {
        return n;
    }

    n
}

pub fn movegen(position: &Position) -> Moves {
    let checking = count_checking(position);

    match checking {
        0 | 1 => pseudo_legal_movegen(position, checking == 1),
        _ => king_movegen(position),
    }
}

fn king_movegen(position: &Position) -> Moves {
    let mut moves = Vec::with_capacity(8);
    let to_move = position.to_move;
    let empty = position.bitboards[White][Empty] & position.bitboards[Black][Empty];

    let player = position.bitboards[to_move];
    let opponent = position.bitboards[!to_move];

    king_moves(player[King], empty, !opponent[Empty], &mut moves);

    if moves.is_empty() {
        Moves::Checkmate
    } else {
        Moves::PseudoLegalMoves(moves)
    }
}

fn pseudo_legal_movegen(position: &Position, in_check: bool) -> Moves {
    let to_move = position.to_move;
    let empty = position.bitboards[White][Empty] & position.bitboards[Black][Empty];

    let player = position.bitboards[to_move];
    let opponent = position.bitboards[!to_move];

    let mut moves = Vec::with_capacity(256);

    // Sliding
    rook_moves(
        player[Rook] | player[Queen],
        !player[Empty],
        !opponent[Empty],
        &mut moves,
    );
    bishop_moves(
        player[Bishop] | player[Queen],
        !player[Empty],
        !opponent[Empty],
        &mut moves,
    );

    // Knights
    knight_moves(player[Knight], player[Empty], !opponent[Empty], &mut moves);

    // King
    king_moves(player[King], empty, !opponent[Empty], &mut moves);
    if !in_check {
        castling_moves(player[King], to_move, empty, position, &mut moves);
    }

    // Pawns
    single_pawn_moves(player[Pawn], to_move, empty, &mut moves);
    double_pawn_moves(player[Pawn], to_move, empty, &mut moves);
    pawn_attacks(player[Pawn], to_move, !opponent[Empty], &mut moves);
    en_passant(player[Pawn], to_move, position.en_passant(), &mut moves);

    if moves.is_empty() {
        if in_check {
            Moves::Checkmate
        } else {
            Moves::Stalemate
        }
    } else {
        Moves::PseudoLegalMoves(moves)
    }
}
