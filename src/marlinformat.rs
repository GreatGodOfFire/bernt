use crate::{
    bitloop,
    position::{PieceColor, PieceType, Position},
};

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

    pub fn unpack(packed: Self) -> (Position, u16, i16, u8, u8) {
        let mut pos = Position::empty();

        let mut occupied = packed.occupied();

        let packed_pieces = packed.pieces();

        let mut seen_king = [false; 2];
        let mut castling = [[64; 2]; 2];

        while occupied != 0 {
            let sq = occupied.trailing_zeros() as u8;
            occupied &= occupied - 1;

            let mut ty = packed_pieces as u8 & 0b111;
            let color = packed_pieces as u8 & 0b1000 != 0;

            if ty == PieceType::King as u8 {
                seen_king[color as usize] = true;
            }

            if ty == UNMOVED_ROOK {
                ty = PieceType::Rook as u8;
                castling[color as usize][seen_king[color as usize] as usize] = sq;
            }

            pos.pieces[ty as usize] |= 1 << sq;
            pos.colors[color as usize] |= 1 << sq;
        }

        pos.side = if packed.stm() {
            PieceColor::Black
        } else {
            PieceColor::White
        };

        pos.en_passant = packed.ep_square();
        pos.halfmove = packed.halfmove();
        let fullmove = packed.fullmove();
        let eval = packed.eval();
        let wdl = packed.wdl();
        let extra = packed.extra();

        (pos, fullmove, eval, wdl, extra)
    }
}

impl PackedBoard {
    pub fn occupied(&self) -> u64 {
        u64::from_le(self.occupied)
    }
    pub fn pieces(&self) -> u128 {
        u128::from_le(self.pieces)
    }
    pub fn stm(&self) -> bool {
        self.stm_ep & 0b10000000 != 0
    }
    pub fn ep_square(&self) -> u8 {
        self.stm_ep & 0b1111111
    }
    pub fn halfmove(&self) -> u8 {
        self.halfmove
    }
    pub fn fullmove(&self) -> u16 {
        u16::from_le(self.fullmove)
    }
    pub fn eval(&self) -> i16 {
        i16::from_le(self.eval)
    }
    pub fn wdl(&self) -> u8 {
        self.wdl
    }
    pub fn extra(&self) -> u8 {
        self.extra
    }

    pub fn set_occupied(&mut self, occupied: u64) {
        self.occupied = occupied.to_be();
    }
    pub fn set_pieces(&mut self, pieces: u128) {
        self.pieces = pieces.to_le();
    }
    pub fn set_stm(&mut self, stm: bool) {
        self.stm_ep = (stm as u8) << 7 | self.stm_ep & 0b1111111;
    }
    pub fn set_ep_square(&mut self, ep: u8) {
        self.stm_ep = self.stm_ep & 0b10000000 | ep & 0b1111111;
    }
    pub fn set_halfmove(&mut self, halfmove: u8) {
        self.halfmove = halfmove
    }
    pub fn set_fullmove(&mut self, fullmove: u16) {
        self.fullmove = fullmove.to_le()
    }
    pub fn set_eval(&mut self, eval: i16) {
        self.eval = eval.to_le()
    }
    pub fn set_wdl(&mut self, wdl: u8) {
        self.wdl = wdl
    }
    pub fn set_extra(&mut self, extra: u8) {
        self.extra = extra
    }

    pub fn to_bytes(&self) -> &[u8; 32] {
        bytemuck::cast_ref(self)
    }
    pub fn from_bytes(bytes: &[u8]) -> &Self {
        bytemuck::from_bytes(bytes)
    }
}
