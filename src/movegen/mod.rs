use std::mem;

use crate::{
    position::{
        piece::{
            PieceColor::{self, *},
            PieceType::{self, *},
        },
        Position,
    },
    PERFT_DEPTH,
};

use self::{
    king::{castling_moves, king_moves},
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
        f.write_fmt(format_args!(
            "{}{}",
            format_square(self.from() as u8),
            format_square(self.to() as u8)
        ))
    }
}

fn format_square(sq: u8) -> String {
    let file = sq % 8;
    let rank = sq / 8;
    format!(
        "{}{}",
        (file + 'a' as u8) as char,
        (rank + '1' as u8) as char
    )
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
    KnightPromotion,
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
    LegalMoves(Vec<Move>),
    Stalemate,
    Checkmate,
}
pub fn perft(position: &Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = movegen(position);

    match moves {
        Moves::PseudoLegalMoves(moves) => {
            let mut n = 0;

            let positions: Vec<_> = moves
                .iter()
                .map(|m| (m, position.make_move(*m)))
                .filter(|(_, p)| !is_in_check(&p, !p.to_move))
                .collect();

            if positions.len() == 0 {
                return 1;
            }
            
            for (m, pos) in positions {
                let x = perft(&pos, depth - 1);
                if depth == PERFT_DEPTH {
                    println!("{m:?}: {x}");
                }
                n += x;
            }
            
            n
        }
        Moves::LegalMoves(moves) => {
            let mut n = 0;

            let positions: Vec<_> = moves
                .iter()
                .map(|m| (m, position.make_move(*m)))
                .collect();
            
            for (m, pos) in positions {
                let x = perft(&pos, depth - 1);
                if depth == PERFT_DEPTH {
                    println!("{m:?}: {x}");
                }
                n += x;
            }
            
            n

        }
        Moves::Stalemate | Moves::Checkmate => {
            1
        }
        _ => todo!(),
    }
}

pub fn is_in_check(position: &Position, to_move: PieceColor) -> bool {
    let king = position.bitboards[to_move][King];
    let king_sq = king.trailing_zeros() as u8;
    let player_pieces = !position.bitboards[to_move][Empty];
    let opponent_pieces = !position.bitboards[!to_move][Empty];

    let opponent_rooks = position.bitboards[!to_move][Rook] | position.bitboards[!to_move][Queen];
    let rooks = single_rook_moves(king_sq, player_pieces | opponent_pieces, opponent_rooks);
    if rooks & opponent_rooks != 0 {
        return true;
    }
    let opponent_bishops =
        position.bitboards[!to_move][Bishop] | position.bitboards[!to_move][Queen];
    let bishops = single_bishop_moves(king_sq, player_pieces, opponent_pieces);
    if bishops & opponent_bishops != 0 {
        return true;
    }
    let knights = single_knight_moves(king_sq);
    if knights & position.bitboards[!to_move][Knight] != 0 {
        return true;
    }
    let pawns = single_pawn_attacks(king_sq, to_move);
    if pawns & position.bitboards[!to_move][Pawn] != 0 {
        return true;
    }
    return false;
}

pub fn movegen(position: &Position) -> Moves {
    // Check if in check
    // If checkmated: return Moves::Checkmate
    // let (checking_pieces, attacked_squares) = checking_pieces(position);
    // let in_check = !checking_pieces.is_empty();

    // if in_check {
    //     legal_movegen(position, checking_pieces, attacked_squares)
    // } else {
    //     pseudo_legal_movegen(position, attacked_squares)
    // }
    //
    pseudo_legal_movegen(position, 0)
}

fn checking_pieces(position: &Position) -> (Vec<u8>, u64) {
    todo!()
}

fn legal_movegen(position: &Position, checking_pieces: Vec<u8>, attacked_squares: u64) -> Moves {
    let mut moves = vec![];
    let empty = position.bitboards[White][Empty] & position.bitboards[Black][Empty];

    if checking_pieces.len() == 1 {
        king_moves(
            position.bitboards[position.to_move][King],
            !attacked_squares & empty,
            !position.bitboards[!position.to_move][Empty],
            &mut moves,
        );
        todo!();
    } else {
        king_moves(
            position.bitboards[position.to_move][King],
            !attacked_squares & empty,
            !position.bitboards[!position.to_move][Empty],
            &mut moves,
        );
    }

    todo!();
}

fn pseudo_legal_movegen(position: &Position, attacked_squares: u64) -> Moves {
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
    castling_moves(
        player[King],
        to_move,
        attacked_squares,
        empty,
        position.castling[to_move],
        &mut moves,
    );

    // Pawns
    single_pawn_moves(player[Pawn], to_move, empty, &mut moves);
    double_pawn_moves(player[Pawn], to_move, empty, &mut moves);
    pawn_attacks(player[Pawn], to_move, !opponent[Empty], &mut moves);
    en_passant(player[Pawn], to_move, position.en_passant, &mut moves);

    Moves::PseudoLegalMoves(moves)
}
