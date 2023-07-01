#[repr(transparent)]
pub struct BitIter(pub u64);

impl Iterator for BitIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 != 0 {
            let sq = self.0.trailing_zeros() as u8;

            self.0 &= self.0 - 1;

            Some(sq)
        } else {
            None
        }
    }
}

pub fn print_bitboard(bitboard: u64) {
    println!("{:#018x}", bitboard);

    for bytes in bitboard.to_be_bytes() {
        for bit in 0..8 {
            if (bytes & (1 << bit)) != 0 {
                print!("1 ");
            } else {
                print!(". ");
            }
        }
        println!();
    }
}
