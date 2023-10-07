use std::{fmt, ops};

#[derive(Clone, PartialEq, Eq)]
pub struct Position {
    pub pieces: [u64; 6],
    pub colors: [u64; 2],
    pub castling: [[i8; 2]; 2],
    pub en_passant: i8,
    pub halfmove: u8,
    pub side: PieceColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub color: PieceColor,
    pub ty: PieceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Pawn = 0,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub flags: u8,
    pub piece: PieceType,
}

impl Position {
    pub fn startpos() -> Self {
        Position {
            pieces: [
                0x00ff00000000ff00,
                0x4200000000000042,
                0x2400000000000024,
                0x8100000000000081,
                0x0800000000000008,
                0x1000000000000010,
            ],
            colors: [0xffff, 0xffff000000000000],
            castling: [[0, 7], [56, 63]],
            en_passant: -1,
            halfmove: 0,
            side: PieceColor::White,
        }
    }

    pub fn make_move(&self, m: Move) -> Self {
        use PieceType::*;

        let mut pos = self.clone();

        let mut piece = m.piece;
        let side = pos.side;

        let from_bit = 1 << m.from;
        let to_bit = 1 << m.to;

        pos.pieces[piece] ^= from_bit;

        if m.capture() || piece == Pawn {
            pos.halfmove = 0;
        } else {
            pos.halfmove += 1;
        }

        let en_passant = pos.en_passant;

        if m.flags == MoveFlag::DOUBLE_PAWN {
            pos.en_passant = match pos.side {
                PieceColor::White => m.to as i8 - 8,
                PieceColor::Black => m.to as i8 + 8,
            }
        } else {
            pos.en_passant = -1;
        }

        if piece == PieceType::King {
            pos.castling[side] = [-1, -1];
        } else if m.from == pos.castling[side][0] as u8 {
            pos.castling[side][0] = -1;
        } else if m.from == pos.castling[side][1] as u8 {
            pos.castling[side][1] = -1;
        }

        match m.flags {
            MoveFlag::CASTLE_LEFT => {
                pos.pieces[Rook] ^= to_bit << 1 | to_bit >> 2;
                pos.colors[side] ^= to_bit << 1 | to_bit >> 2;
            }
            MoveFlag::CASTLE_RIGHT => {
                pos.pieces[Rook] ^= to_bit << 1 | to_bit >> 1;
                pos.colors[side] ^= to_bit << 1 | to_bit >> 1;
            }
            MoveFlag::EP => {
                let sq = (en_passant
                    + match side {
                        PieceColor::White => -8,
                        PieceColor::Black => 8,
                    }) as u8;

                let sq_bit = 1 << sq;
                pos.pieces[Pawn] ^= sq_bit;
                pos.colors[!side] ^= sq_bit;
            }
            _ => {
                if m.flags & MoveFlag::CAP != 0 {
                    let mut target = Pawn;
                    for ty in [Knight, Bishop, Rook, Queen] {
                        if pos.pieces[ty] & to_bit != 0 {
                            target = ty;
                            break;
                        }
                    }

                    pos.pieces[target] ^= to_bit;
                    pos.colors[!side] ^= to_bit;

                    if target == Rook {
                        if m.to == pos.castling[!side][0] as u8 {
                            pos.castling[!side][0] = -1;
                        } else if m.to == pos.castling[!side][1] as u8 {
                            pos.castling[!side][1] = -1;
                        }
                    }
                }

                if m.flags & MoveFlag::PROMO != 0 {
                    piece = m.promotion();
                }
            }
        }

        pos.pieces[piece] ^= to_bit;
        pos.colors[side] ^= from_bit ^ to_bit;

        pos.side = !pos.side;

        pos
    }

    pub fn piece_at(&self, sq: u8) -> Piece {
        use PieceType::*;

        let bit = 1 << sq;

        let mut piece = Piece {
            color: PieceColor::White,
            ty: None,
        };
        if self.colors[PieceColor::White] & bit != 0 {
            for ty in [Pawn, Knight, Bishop, Rook, Queen, King] {
                if self.pieces[ty] & bit != 0 {
                    piece.ty = ty;
                    break;
                }
            }
        } else if self.colors[PieceColor::Black] & bit != 0 {
            piece.color = PieceColor::Black;
            for ty in [Pawn, Knight, Bishop, Rook, Queen, King] {
                if self.pieces[ty] & bit != 0 {
                    piece.ty = ty;
                    break;
                }
            }
        }

        piece
    }

    pub fn from_fen(s: &str) -> Self {
        let mut pos = Position {
            pieces: [0; 6],
            colors: [0; 2],
            castling: [[-1; 2]; 2],
            en_passant: -1,
            halfmove: 0,
            side: PieceColor::White,
        };

        let mut i = 56;
        let parts: Vec<_> = s.split(' ').collect();

        for c in parts[0].chars() {
            if c >= '1' && c <= '8' {
                i += c as u8 - b'0';
            } else if c == '/' {
                i -= 16;
            } else {
                let color = c.is_lowercase();
                let ty = match c.to_ascii_lowercase() {
                    'p' => PieceType::Pawn,
                    'n' => PieceType::Knight,
                    'b' => PieceType::Bishop,
                    'r' => PieceType::Rook,
                    'q' => PieceType::Queen,
                    'k' => PieceType::King,
                    x => panic!("{x}"),
                };

                pos.pieces[ty as usize] |= 1 << i;
                pos.colors[color as usize] |= 1 << i;
                i += 1;
            }
        }

        if parts[1] == "b" {
            pos.side = PieceColor::Black;
        }

        for c in parts[2].chars() {
            match c {
                'Q' => pos.castling[0][0] = 0,
                'K' => pos.castling[0][1] = 7,
                'q' => pos.castling[1][0] = 56,
                'k' => pos.castling[1][1] = 63,
                // TODO: Shredder FEN
                _ => {}
            }
        }

        if parts[3] != "-" {
            pos.en_passant = uci_sq(parts[3]) as i8;
        }

        pos.halfmove = parts.get(4).and_then(|x| x.parse().ok()).unwrap_or(0);

        return pos;
    }
}

fn uci_sq(sq: &str) -> u8 {
    sq.as_bytes()[0] - b'a' + 8 * (sq.as_bytes()[1] as u8 - b'1')
}

pub struct MoveFlag;

impl MoveFlag {
    pub const QUIET: u8 = 0;
    pub const DOUBLE_PAWN: u8 = 1;
    pub const CASTLE_LEFT: u8 = 0b10;
    pub const CASTLE_RIGHT: u8 = 0b11;
    pub const EP: u8 = 0b101;

    pub const CAP: u8 = 0b100;
    pub const PROMO: u8 = 0b1000;
}

impl Move {
    pub const NULL: Move = Move {
        from: 0,
        to: 0,
        flags: MoveFlag::QUIET,
        piece: PieceType::Pawn,
    };

    pub fn new(from: u8, to: u8, flags: u8, piece: PieceType) -> Self {
        Self {
            from,
            to,
            flags,
            piece,
        }
    }

    pub fn capture(&self) -> bool {
        self.flags & MoveFlag::CAP != 0
    }

    pub fn promotion(&self) -> PieceType {
        use PieceType::*;

        if self.flags & MoveFlag::PROMO != 0 {
            match self.flags & 0b11 {
                0 => Knight,
                1 => Bishop,
                2 => Rook,
                3 => Queen,
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Default for Move {
    fn default() -> Self {
        Move::NULL
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const PROMOS: [&str; 7] = ["", "n", "b", "r", "q", "", ""];

        write!(
            f,
            "{}{}{}",
            format_sq(self.from),
            format_sq(self.to),
            PROMOS[self.promotion()]
        )
    }
}

fn format_sq(sq: u8) -> String {
    const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    const RANKS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];

    let mut s = String::new();

    let file = sq % 8;
    let rank = sq / 8;

    s.push(FILES[file as usize]);
    s.push(RANKS[rank as usize]);

    s
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PieceType::*;

        for (x, ty) in self
            .pieces
            .iter()
            .zip(&[Pawn, Knight, Bishop, Rook, Queen, King])
        {
            writeln!(f, "{ty:?}: {x:#018x}")?;
        }

        for (x, ty) in self.colors.iter().zip(&["White", "Black"]) {
            writeln!(f, "{ty}: {x:#018x}")?;
        }

        // TODO: Print more
        Ok(())
    }
}

impl<const N: usize, T> ops::Index<PieceColor> for [T; N] {
    type Output = T;

    fn index(&self, color: PieceColor) -> &T {
        &self[color as usize]
    }
}

impl<const N: usize, T> ops::IndexMut<PieceColor> for [T; N] {
    fn index_mut(&mut self, color: PieceColor) -> &mut Self::Output {
        &mut self[color as usize]
    }
}

impl<const N: usize, T> ops::Index<PieceType> for [T; N] {
    type Output = T;

    fn index(&self, ty: PieceType) -> &T {
        &self[ty as usize]
    }
}

impl<const N: usize, T> ops::IndexMut<PieceType> for [T; N] {
    fn index_mut(&mut self, ty: PieceType) -> &mut Self::Output {
        &mut self[ty as usize]
    }
}

impl ops::Not for PieceColor {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[test]
fn startpos_fen() {
    assert_eq!(
        Position::startpos(),
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    );
}
