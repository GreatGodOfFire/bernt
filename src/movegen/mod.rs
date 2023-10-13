mod king;
mod knight;
mod sliding;

use std::ops;

use crate::{
    bitloop,
    movegen::{
        king::{castling_moves, king_moves, lookup_king},
        knight::{knight_moves, single_knight_moves},
        sliding::{bishop_moves, queen_moves, rook_moves, single_bishop_moves},
    },
    position::{Move, MoveFlag, PieceColor, PieceType, Position},
};

use self::sliding::single_rook_moves;

pub struct MoveList {
    pub moves: [Move; 256],
    pub len: u8,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move::NULL; 256],
            len: 0,
        }
    }

    pub fn len(&self) -> u8 {
        self.len
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;

    type IntoIter = std::slice::Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.moves[..self.len as usize].iter()
    }
}

impl ops::AddAssign<Move> for &mut MoveList {
    fn add_assign(&mut self, rhs: Move) {
        self.moves[self.len as usize] = rhs;
        self.len += 1;
    }
}

const RANK_2: u64 = 0xff00;
const RANK_7: u64 = 0xff000000000000;

impl Position {
    pub fn in_check(&self, side: PieceColor) -> bool {
        is_attacking(
            (self.pieces[PieceType::King] & self.colors[side]).trailing_zeros() as u8,
            self,
            !side,
        )
    }
}

fn is_attacking(square: u8, pos: &Position, side: PieceColor) -> bool {
    use PieceType::*;

    let us = pos.colors[side as usize];
    let them = pos.colors[!side as usize];
    let occupied = us | them;

    ((pos.pieces[Rook] | pos.pieces[Queen]) & us & single_rook_moves(square, 0, occupied)
        | (pos.pieces[Bishop] | pos.pieces[Queen]) & us & single_bishop_moves(square, 0, occupied)
        | pos.pieces[Knight] & us & single_knight_moves(square)
        | pos.pieces[Pawn] & us & single_pawn_attacks(square, !side)
        | (1 << square) & lookup_king((pos.pieces[King] & us).trailing_zeros() as u8))
        != 0
}

fn single_pawn_attacks(pawn: u8, color: PieceColor) -> u64 {
    PAWN_ATTACKS_LOOKUP[color][pawn as usize]
}

const PAWN_ATTACKS_LOOKUP: [[u64; 64]; 2] = generate_attacks_lookup();

const fn generate_attacks_lookup() -> [[u64; 64]; 2] {
    let mut attacks = [[0; 64]; 2];

    let mut i = 0;
    while i < 64 {
        let bit = 1u64 << i;
        attacks[0][i] = north(east(bit)) | north(west(bit));
        attacks[1][i] = south(east(bit)) | south(west(bit));

        i += 1;
    }

    attacks
}

#[inline]
pub const fn north(x: u64) -> u64 {
    (x & !(0xff << 56)) << 8
}

#[inline]
pub const fn south(x: u64) -> u64 {
    (x & !0xff) >> 8
}

#[inline]
pub const fn east(x: u64) -> u64 {
    (x & !FILE_H) << 1
}

#[inline]
pub const fn west(x: u64) -> u64 {
    (x & !FILE_A) >> 1
}

pub const FILE_A: u64 = 0x101010101010101;
pub const FILE_H: u64 = 0x8080808080808080;

pub fn movegen<const QUIETS: bool>(pos: &Position) -> MoveList {
    use PieceType::*;

    let mut moves = MoveList::new();

    let occupied = pos.colors[0] | pos.colors[1];
    let empty = !occupied;
    let us = pos.colors[pos.side as usize];
    let them = pos.colors[!pos.side as usize];

    queen_moves::<QUIETS>(pos.pieces[Queen] & us, us, them, &mut moves);
    rook_moves::<QUIETS>(pos.pieces[Rook] & us, us, them, &mut moves);
    bishop_moves::<QUIETS>(pos.pieces[Bishop] & us, us, them, &mut moves);

    knight_moves::<QUIETS>(pos.pieces[Knight] & us, !us, them, &mut moves);

    if !pos.in_check(pos.side) && QUIETS {
        castling_moves(pos.pieces[King] & us, empty, pos, &mut moves);
    }

    pawn_moves::<QUIETS>(
        pos.pieces[Pawn] & us,
        empty,
        them,
        pos.en_passant,
        pos.side,
        &mut moves,
    );
    king_moves::<QUIETS>(pos.pieces[King] & us, empty, them, &mut moves);

    moves
}

fn shift(side: PieceColor, bitboard: u64, by: u8) -> u64 {
    if side == PieceColor::White {
        bitboard << by
    } else {
        bitboard >> by
    }
}

fn pawn_moves<const QUIETS: bool>(
    pawns: u64,
    empty: u64,
    them: u64,
    en_passant: i8,
    side: PieceColor,
    mut moves: &mut MoveList,
) {
    use PieceType::*;

    if QUIETS {
        let double_push_rank = match side {
            PieceColor::White => RANK_2,
            PieceColor::Black => RANK_7,
        };

        let (fw, bw): (fn(_) -> _, fn(_) -> _) = match side {
            PieceColor::White => (north, south),
            PieceColor::Black => (south, north),
        };

        let push = pawns & bw(empty);
        bitloop!(push => from, fw(push) => to, {
            if to >= 56 || to <= 7 {
                moves += Move::new(from, to, MoveFlag::PROMO, Pawn);
                moves += Move::new(from, to, MoveFlag::PROMO | 1, Pawn);
                moves += Move::new(from, to, MoveFlag::PROMO | 2, Pawn);
                moves += Move::new(from, to, MoveFlag::PROMO | 3, Pawn);
            } else {
                moves += Move::new(from, to, MoveFlag::QUIET, Pawn);
            }
        });

        let double_push = push & double_push_rank & bw(bw(empty));
        bitloop!(double_push => from, fw(fw(double_push)) => to, {
            moves += Move::new(from, to, MoveFlag::DOUBLE_PAWN, Pawn);
        });
    }

    let (left_mask, right_mask) = match side {
        PieceColor::White => (!FILE_A, !FILE_H),
        PieceColor::Black => (!FILE_H, !FILE_A),
    };

    let left_attacks = pawns & left_mask & shift(!side, them, 7);
    let right_attacks = pawns & right_mask & shift(!side, them, 9);

    bitloop!(left_attacks => from, shift(side, left_attacks, 7) => to, {
        if to >= 56 || to <= 7 {
            moves += Move::new(from, to, MoveFlag::CAP | MoveFlag::PROMO, Pawn);
            moves += Move::new(from, to, MoveFlag::CAP | MoveFlag::PROMO | 1, Pawn);
            moves += Move::new(from, to, MoveFlag::CAP | MoveFlag::PROMO | 2, Pawn);
            moves += Move::new(from, to, MoveFlag::CAP | MoveFlag::PROMO | 3, Pawn);
        } else {
            moves += Move::new(from, to, MoveFlag::CAP, Pawn);
        }
    });

    bitloop!(right_attacks => from, shift(side, right_attacks, 9) => to, {
        if to >= 56 || to <= 7 {
            moves += Move::new(from, to, MoveFlag::CAP | MoveFlag::PROMO, Pawn);
            moves += Move::new(from, to, MoveFlag::CAP | MoveFlag::PROMO | 1, Pawn);
            moves += Move::new(from, to, MoveFlag::CAP | MoveFlag::PROMO | 2, Pawn);
            moves += Move::new(from, to, MoveFlag::CAP | MoveFlag::PROMO | 3, Pawn);
        } else {
            moves += Move::new(from, to, MoveFlag::CAP, Pawn);
        }
    });

    if en_passant >= 0 {
        let ep_bit = 1 << en_passant;
        let ep_squares = shift(!side, ep_bit, 7) | shift(!side, ep_bit, 9);
        let ep_rank = match side {
            PieceColor::White => RANK_5,
            PieceColor::Black => RANK_4,
        };
        let ep_pawns = pawns & ep_squares & ep_rank;
        bitloop!(ep_pawns => from, {
            moves += Move::new(from, en_passant as u8, MoveFlag::EP, Pawn);
        });
    }
}

pub const RANK_4: u64 = 0xff << 24;
pub const RANK_5: u64 = 0xff << 32;

#[cfg(test)]
mod tests {
    use crate::{perft::split_perft, position::Position};

    macro_rules! test_perft {
        ($name:ident($fen:literal, $depth:literal) = $res:literal) => {
            #[test]
            fn $name() {
                let pos = Position::from_fen($fen);

                assert_eq!(split_perft(&pos, $depth), $res)
            }
        };
        ($name:ident(startpos, $depth:literal) = $res:literal) => {
            test_perft!(
                $name(
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -",
                    $depth
                ) = $res
            );
        };
    }

    test_perft!(startpos_d6(startpos, 6) = 119060324);
    test_perft!(
        kiwipete_d5(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
            5
        ) = 193690690
    );
    test_perft!(pos3_d6("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -", 6) = 11030083);
    test_perft!(
        pos4_d5(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq -",
            5
        ) = 15833292
    );
    test_perft!(pos5_d5("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ -", 5) = 89941194);
    test_perft!(
        pos6_d5(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - -",
            5
        ) = 164075551
    );
}
