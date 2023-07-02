use bernt_position::{
    piece::PieceType::{self, *},
    Position,
};

#[inline]
pub fn evaluate(position: &Position) -> i32 {
    evaluate_with(position, Tables::default())
}

pub fn evaluate_with(position: &Position, tables: Tables) -> i32 {
    let mut mg = [0; 2];
    let mut eg = [0; 2];
    let mut game_phase = 0;

    for sq in 0..64 {
        let piece = position.mailbox()[sq];
        if piece.ty != PieceType::Empty {
            mg[piece.color] += tables.mg_table[piece.ty];
            eg[piece.color] += tables.eg_table[piece.ty];
            game_phase += tables.gamephase_inc[piece.ty];
        }
    }

    let mg_score = mg[position.to_move()] - mg[!position.to_move()];
    let eg_score = eg[position.to_move()] - eg[!position.to_move()];
    let mg_phase = game_phase.min(24);
    let eg_phase = 24 - mg_phase;

    (mg_score * mg_phase + eg_score * eg_phase) / 24
}

pub struct Tables {
    gamephase_inc: [i32; 6],
    mg_table: [i32; 6],
    eg_table: [i32; 6],
}

impl Tables {
    pub const fn default() -> Self {
        Self {
            gamephase_inc: GAMEPHASE_INC,
            mg_table: MG_TABLE,
            eg_table: EG_TABLE,
        }
    }
}

const GAMEPHASE_INC: [i32; 6] = [1, 1, 2, 4, 0, 0];
const MG_TABLE: [i32; 6] = {
    let mut table = [0; 6];

    table[Pawn as usize] = 90;
    table[Knight as usize] = 300;
    table[Bishop as usize] = 300;
    table[Rook as usize] = 500;
    table[Queen as usize] = 900;
    table[King as usize] = 0;

    table
};
const EG_TABLE: [i32; 6] = {
    let mut table = [0; 6];

    table[Pawn as usize] = 110;
    table[Knight as usize] = 270;
    table[Bishop as usize] = 330;
    table[Rook as usize] = 550;
    table[Queen as usize] = 1000;
    table[King as usize] = 0;

    table
};
