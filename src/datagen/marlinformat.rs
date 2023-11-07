use crate::{bitloop, position::Position};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedBoard {
    occupied: u64,
    pieces: u128,
    stm_ep: u8,
    halfmove: u8,
    fullmove: u16,
    eval: i16,
    wdl: u8,
    extra: u8,
}

unsafe impl bytemuck::Pod for PackedBoard {}
unsafe impl bytemuck::Zeroable for PackedBoard {}

const UNMOVED_ROOK: u8 = 6;

impl PackedBoard {
    pub fn pack(pos: &Position, fullmove: u16, eval: i32, wdl: u8, extra: u8) -> PackedBoard {
        let occupied = (pos.colors[0] | pos.colors[1]).to_le();
        let mut pieces = 0u128;

        let mut offset = 0;

        bitloop!(occupied => sq, {
            let p = pos.piece_at(sq);
            let color = p.color;
            let mut ty = p.ty as u8;
            if pos.castling[0].contains(&sq) || pos.castling[1].contains(&sq) {
                ty = UNMOVED_ROOK;
            }
            let p = (color as u8) << 3 | ty;

            pieces |= (p as u128) << offset;
            offset += 4;
        });

        let stm_ep = (pos.side as u8) << 7 | pos.en_passant as u8;
        let halfmove = pos.halfmove;
        let fullmove = fullmove.to_le();
        let eval = (eval.clamp(i16::MIN as i32, i16::MAX as i32) as i16).to_le();

        Self {
            occupied,
            pieces: pieces.to_le(),
            stm_ep,
            halfmove,
            fullmove,
            eval,
            wdl,
            extra,
        }
    }
}

impl PackedBoard {
    pub fn set_wdl(&mut self, wdl: u8) {
        self.wdl = wdl
    }
}
