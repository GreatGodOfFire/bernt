// Thanks to https://github.com/cosmobobak for providing me with the seed and code for this
const SEED: u128 = 0x246C_CB2D_3B40_2853_9918_0A6D_BC3A_F444;

pub struct XorShiftState(u128);

impl XorShiftState {
    pub const fn new() -> Self {
        Self(SEED)
    }

    pub const fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;

        self.0 as u64 ^ (self.0 >> 64) as u64
    }
}
