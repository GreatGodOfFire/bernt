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
mod king;
mod knight;
mod pawn;
mod sliding;
mod util;

#[derive(Clone, Copy)]
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
                if !is_attacking(
                    position.bitboards[!position.to_move][King].trailing_zeros() as u8,
                    position,
                    position.to_move,
                ) {
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
                if !is_attacking(
                    position.bitboards[!position.to_move][King].trailing_zeros() as u8,
                    position,
                    position.to_move,
                ) {
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
        Moves::Stalemate
    } else {
        Moves::PseudoLegalMoves(moves)
    }
}
