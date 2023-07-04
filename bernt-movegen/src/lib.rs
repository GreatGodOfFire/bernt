use std::{ops::Index, slice::SliceIndex};

use bernt_position::{
    piece::{
        PieceColor::{self, *},
        PieceType::{self, *},
    },
    Move, Position,
};

use self::{
    king::{castling_moves, king_moves, lookup_king},
    knight::{knight_moves, single_knight_moves},
    pawn::{double_pawn_moves, en_passant, pawn_attacks, single_pawn_attacks, single_pawn_moves},
    sliding::{bishop_moves, rook_moves, single_bishop_moves, single_rook_moves},
};

pub mod king;
pub mod knight;
pub mod pawn;
pub mod perft;
pub mod sliding;
pub mod util;

pub(crate) mod flags {
    pub const QUIET: u8 = 0x1;
    pub const CAPTURES: u8 = 0x2;
    pub const PROMOTIONS: u8 = 0x4;
    pub const FRC: u8 = 0x8;
    pub const DEFAULT: u8 = QUIET | CAPTURES | PROMOTIONS;
    pub const DEFAULT_FRC: u8 = DEFAULT | FRC;
}

#[derive(Debug, Clone)]
pub enum Moves {
    PseudoLegalMoves(MoveList),
    Stalemate,
    Checkmate,
}

#[derive(Debug, Clone)]
pub struct MoveList {
    array: Box<[Move; 256]>,
    len: u8,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            array: Box::new([Move::null(); 256]),
            len: 0,
        }
    }

    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn add(&mut self, m: Move) {
        self.array[self.len as usize] = m;
        self.len += 1;
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl From<MoveList> for Box<[Move]> {
    fn from(movelist: MoveList) -> Self {
        movelist.array
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = Move;

    type IntoIter = MoveListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MoveListIter(self, 0, self.len)
    }
}

pub struct MoveListIter<'a>(&'a MoveList, u8, u8);

impl<'a> Iterator for MoveListIter<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 >= self.2 {
            return None;
        }
        let x = self.0[self.1 as usize];

        self.1 += 1;

        Some(x)
    }
}

impl<'a> DoubleEndedIterator for MoveListIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.2 <= self.1 {
            return None;
        }

        self.2 -= 1;

        Some(self.0[self.2 as usize])
    }
}

impl<'a> ExactSizeIterator for MoveListIter<'a> {
    fn len(&self) -> usize {
        self.0.len as usize
    }
}

impl<Idx> Index<Idx> for MoveList
where
    Idx: SliceIndex<[Move], Output = Move>,
{
    type Output = Move;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.array[index]
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
    let opponent_pieces = !position.bitboards()[!to_move][Empty];
    let player_pieces = !position.bitboards()[to_move][Empty];

    let opponent_rooks = position.bitboards()[to_move][Rook] | position.bitboards()[to_move][Queen];
    let rooks = single_rook_moves(square, 0, player_pieces | opponent_pieces);
    if rooks & opponent_rooks != 0 {
        return true;
    }
    let opponent_bishops =
        position.bitboards()[to_move][Bishop] | position.bitboards()[to_move][Queen];
    let bishops = single_bishop_moves(square, 0, player_pieces | opponent_pieces);
    if bishops & opponent_bishops != 0 {
        return true;
    }
    let knights = single_knight_moves(square);
    if knights & position.bitboards()[to_move][Knight] != 0 {
        return true;
    }
    let pawns = single_pawn_attacks(square, !to_move);
    if pawns & position.bitboards()[to_move][Pawn] != 0 {
        return true;
    }
    if (1 << square) & lookup_king(position.bitboards()[to_move][King].trailing_zeros() as u8) != 0
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
    let rooks = single_rook_moves(king_sq, 0, player_pieces | opponent_pieces);

    n += (rooks & opponent_rooks).count_ones();
    if n >= 2 {
        return n;
    }

    let opponent_bishops =
        position.bitboards()[!to_move][Bishop] | position.bitboards()[!to_move][Queen];
    let bishops = single_bishop_moves(king_sq, 0, player_pieces | opponent_pieces);

    n += (bishops & opponent_bishops).count_ones();
    if n >= 2 {
        return n;
    }

    let knights = single_knight_moves(king_sq);
    n += (knights & position.bitboards()[!to_move][Knight]).count_ones();
    if n >= 2 {
        return n;
    }

    let pawns = single_pawn_attacks(king_sq, to_move);
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
    let mut movelist = MoveList::new();
    let to_move = position.to_move();
    let empty = position.bitboards()[White][Empty] & position.bitboards()[Black][Empty];

    let player = position.bitboards()[to_move];
    let opponent = position.bitboards()[!to_move];

    king_moves::<{ flags::DEFAULT }>(player[King], empty, !opponent[Empty], &mut movelist);

    if movelist.is_empty() {
        Moves::Checkmate
    } else {
        Moves::PseudoLegalMoves(movelist)
    }
}

pub fn pseudo_legal_movegen(position: &Position, in_check: bool) -> Moves {
    let to_move = position.to_move();
    let empty = position.bitboards()[White][Empty] & position.bitboards()[Black][Empty];

    let player = position.bitboards()[to_move];
    let opponent = position.bitboards()[!to_move];

    let mut movelist = MoveList::new();

    // Sliding
    rook_moves::<{ flags::DEFAULT }>(
        player[Rook] | player[Queen],
        !player[Empty],
        !opponent[Empty],
        &mut movelist,
    );
    bishop_moves::<{ flags::DEFAULT }>(
        player[Bishop] | player[Queen],
        !player[Empty],
        !opponent[Empty],
        &mut movelist,
    );

    // Knights
    knight_moves::<{ flags::DEFAULT }>(
        player[Knight],
        player[Empty],
        !opponent[Empty],
        &mut movelist,
    );

    // King
    king_moves::<{ flags::DEFAULT }>(player[King], empty, !opponent[Empty], &mut movelist);
    if !in_check {
        castling_moves(player[King], to_move, empty, position, &mut movelist);
    }

    // Pawns
    single_pawn_moves(player[Pawn], to_move, empty, &mut movelist);
    double_pawn_moves(player[Pawn], to_move, empty, &mut movelist);
    pawn_attacks(player[Pawn], to_move, !opponent[Empty], &mut movelist);
    en_passant(player[Pawn], to_move, position.en_passant(), &mut movelist);

    if movelist.is_empty() {
        if in_check {
            Moves::Checkmate
        } else {
            Moves::Stalemate
        }
    } else {
        Moves::PseudoLegalMoves(movelist)
    }
}

pub fn pseudo_legal_movegen_captures(position: &Position) -> Moves {
    let to_move = position.to_move();

    let player = position.bitboards()[to_move];
    let opponent = position.bitboards()[!to_move];

    let mut moves = MoveList::new();

    // Sliding
    rook_moves::<{ flags::CAPTURES }>(
        player[Rook] | player[Queen],
        !player[Empty],
        !opponent[Empty],
        &mut moves,
    );
    bishop_moves::<{ flags::CAPTURES }>(
        player[Bishop] | player[Queen],
        !player[Empty],
        !opponent[Empty],
        &mut moves,
    );

    // Knights
    knight_moves::<{ flags::CAPTURES }>(
        player[Knight],
        player[Empty],
        !opponent[Empty],
        &mut moves,
    );

    // King
    king_moves::<{ flags::CAPTURES }>(player[King], 0, !opponent[Empty], &mut moves);

    // Pawns
    pawn_attacks(player[Pawn], to_move, !opponent[Empty], &mut moves);
    en_passant(player[Pawn], to_move, position.en_passant(), &mut moves);

    if moves.is_empty() {
        Moves::Stalemate
    } else {
        Moves::PseudoLegalMoves(moves)
    }
}
