use bernt_position::piece::PieceType::*;
use bernt_search::eval::pst;
use core::fmt;
use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct Params {
    // pub opening: [[i32; 64]; 6],
    pub midgame: [[i32; 64]; 6],
    pub endgame: [[i32; 64]; 6],
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (table, name) in [
            // (self.opening, "OPENING"),
            (self.midgame, "MIDGAME"),
            (self.endgame, "ENDGAME"),
        ] {
            for ty in [Pawn, Knight, Bishop, Rook, Queen, King] {
                let squares = table[ty];

                writeln!(f, "#[rustfmt::skip]").unwrap();
                writeln!(
                    f,
                    "const {name}_{}: [i32; 64] = [",
                    format!("{ty:?}").to_uppercase()
                )?;
                for rank in 0..8 {
                    write!(f, "    ")?;
                    for file in 0..8 {
                        write!(f, "{:>4}, ", squares[rank * 8 + file])?;
                    }
                    writeln!(f)?;
                }
                writeln!(f, "];")?;
            }
        }

        Ok(())
    }
}

impl Params {
    pub const fn default() -> Self {
        Self {
            // opening: pst::OPENING,
            midgame: pst::MIDGAME,
            endgame: pst::ENDGAME,
        }
    }

    pub const fn param_count() -> usize {
        // 64 * 6 * 3
        64 * 6 * 2
    }
}

impl Index<usize> for Params {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        // if index < 64 * 6 {
        //     &self.opening[index / 64][index % 64]
        // } else if index < 64 * 6 * 2 {
        //     let index = index - 64 * 6;
        //     &self.midgame[index / 64][index % 64]
        // } else if index < 64 * 6 * 3 {
        //     let index = index - 64 * 6 * 2;
        //     &self.endgame[index / 64][index % 64]
        // } else {
        //     panic!()
        // }
        if index < 64 * 6 {
            &self.midgame[index / 64][index % 64]
        } else if index < 64 * 6 * 2 {
            let index = index - 64 * 6;
            &self.endgame[index / 64][index % 64]
        } else {
            panic!()
        }
    }
}

impl IndexMut<usize> for Params {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // if index < 64 * 6 {
        //     &mut self.opening[index / 64][index % 64]
        // } else if index < 64 * 6 * 2 {
        //     let index = index - 64 * 6;
        //     &mut self.midgame[index / 64][index % 64]
        // } else if index < 64 * 6 * 3 {
        //     let index = index - 64 * 6 * 2;
        //     &mut self.endgame[index / 64][index % 64]
        // } else {
        //     panic!()
        // }
        if index < 64 * 6 {
            &mut self.midgame[index / 64][index % 64]
        } else if index < 64 * 6 * 2 {
            let index = index - 64 * 6;
            &mut self.endgame[index / 64][index % 64]
        } else {
            panic!()
        }
    }
}
