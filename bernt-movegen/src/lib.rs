use bernt_position::{
    bitboard::Bitboard,
    piece::{
        PieceColor::{self, *},
        PieceType::{self, *},
    },
    Move, Position,
};
use king::{castling_moves, king_moves, lookup_king};
use knight::{knight_moves, single_knight_moves};
use pawn::{pawn_moves, single_pawn_attacks};
use sliding::{bishop_moves, rook_moves, single_bishop_moves, single_rook_moves};

mod king;
mod knight;
mod pawn;
pub mod perft;
mod sliding;
mod util;

pub(crate) mod flags {
    pub const QUIET: u8 = 0x1;
    pub const CAPTURES: u8 = 0x2;
    pub const PROMOTIONS: u8 = 0x4;
    pub const ALL: u8 = 0xff;
}

pub enum Moves {
    PseudoLegalMoves(MoveList),
    Stalemate,
    Checkmate,
}

pub struct MoveList {
    array: Box<[Move; 256]>,
    len: u8,
    idx: u8,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            array: Box::new([Move::null(); 256]),
            len: 0,
            idx: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn add(&mut self, m: Move) {
        self.array[self.len as usize] = m;
        self.len += 1;
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for MoveList {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }

        let x = self.array[self.idx as usize];
        self.idx += 1;
        Some(x)
    }
}

pub fn is_in_check(position: &Position, color: PieceColor) -> bool {
    is_attacking(
        position.bitboards()[color][PieceType::King].trailing_zeros() as u8,
        position,
        !color,
    )
}

pub fn is_attacking(square: u8, position: &Position, to_move: PieceColor) -> bool {
    let opponent_pieces = position.bitboards()[!to_move][PieceType::Occupied];
    let player_pieces = position.bitboards()[to_move][PieceType::Occupied];

    let opponent_rooks = position.bitboards()[to_move][Rook] | position.bitboards()[to_move][Queen];
    let rooks = single_rook_moves(square, Bitboard(0), player_pieces | opponent_pieces);
    if !(rooks & opponent_rooks).is_empty() {
        return true;
    }
    let opponent_bishops =
        position.bitboards()[to_move][Bishop] | position.bitboards()[to_move][Queen];
    let bishops = single_bishop_moves(square, Bitboard(0), player_pieces | opponent_pieces);
    if !(bishops & opponent_bishops).is_empty() {
        return true;
    }
    let knights = single_knight_moves(square);
    if !(knights & position.bitboards()[to_move][Knight]).is_empty() {
        return true;
    }
    let pawns = if to_move == PieceColor::White {
        single_pawn_attacks::<false>(square)
    } else {
        single_pawn_attacks::<true>(square)
    };
    if !(pawns & position.bitboards()[to_move][Pawn]).is_empty() {
        return true;
    }
    if !(Bitboard(1 << square)
        & lookup_king(position.bitboards()[to_move][King].trailing_zeros() as u8))
    .is_empty()
    {
        return true;
    }

    false
}

pub fn count_checking(position: &Position) -> u32 {
    let mut n = 0;

    let to_move = position.to_move();
    let king = position.bitboards()[to_move][King];
    let king_sq = king.trailing_zeros() as u8;
    let player_pieces = !position.bitboards()[to_move][Empty];
    let opponent_pieces = !position.bitboards()[!to_move][Empty];

    let opponent_rooks =
        position.bitboards()[!to_move][Rook] | position.bitboards()[!to_move][Queen];
    let rooks = single_rook_moves(king_sq, Bitboard(0), player_pieces | opponent_pieces);

    n += (rooks & opponent_rooks).count_ones();
    if n >= 2 {
        return n;
    }

    let opponent_bishops =
        position.bitboards()[!to_move][Bishop] | position.bitboards()[!to_move][Queen];
    let bishops = single_bishop_moves(king_sq, Bitboard(0), player_pieces | opponent_pieces);

    n += (bishops & opponent_bishops).count_ones();
    if n >= 2 {
        return n;
    }

    let knights = single_knight_moves(king_sq);
    n += (knights & position.bitboards()[!to_move][Knight]).count_ones();
    if n >= 2 {
        return n;
    }

    let pawns = if to_move == PieceColor::White {
        single_pawn_attacks::<false>(king_sq)
    } else {
        single_pawn_attacks::<true>(king_sq)
    };
    n += (pawns & position.bitboards()[!to_move][Pawn]).count_ones();
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

pub fn king_movegen(position: &Position) -> Moves {
    let mut moves = MoveList::new();
    let to_move = position.to_move();
    let empty = position.bitboards()[White][Empty] & position.bitboards()[Black][Empty];

    let player = position.bitboards()[to_move];
    let opponent = position.bitboards()[!to_move];

    king_moves::<{ flags::ALL }>(player[King], empty, !opponent[Empty], &mut moves);

    if moves.is_empty() {
        Moves::Checkmate
    } else {
        Moves::PseudoLegalMoves(moves)
    }
}

pub fn pseudo_legal_movegen(position: &Position, in_check: bool) -> Moves {
    let to_move = position.to_move();
    let empty = !position.bitboards()[White][PieceType::Occupied]
        & !position.bitboards()[Black][PieceType::Occupied];

    let player = position.bitboards()[to_move];
    let opponent = position.bitboards()[!to_move];

    let mut moves = MoveList::new();

    // Sliding
    rook_moves::<{ flags::ALL }>(
        player[Rook] | player[Queen],
        player[PieceType::Occupied],
        opponent[PieceType::Occupied],
        &mut moves,
    );
    bishop_moves::<{ flags::ALL }>(
        player[Bishop] | player[Queen],
        player[PieceType::Occupied],
        opponent[PieceType::Occupied],
        &mut moves,
    );

    // Knights
    knight_moves::<{ flags::ALL }>(
        player[Knight],
        !player[PieceType::Occupied],
        opponent[PieceType::Occupied],
        &mut moves,
    );

    // King
    king_moves::<{ flags::ALL }>(
        player[King],
        empty,
        opponent[PieceType::Occupied],
        &mut moves,
    );
    if !in_check {
        castling_moves(player[King], to_move, empty, position, &mut moves);
    }

    // Pawns
    if to_move == White {
        pawn_moves::<true, { flags::ALL }>(player[Pawn], empty, opponent[PieceType::Occupied], position.en_passant(), &mut moves);
    } else {
        pawn_moves::<false, { flags::ALL }>(player[Pawn], empty, opponent[PieceType::Occupied], position.en_passant(), &mut moves);
    }

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
