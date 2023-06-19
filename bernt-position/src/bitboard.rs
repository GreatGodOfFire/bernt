#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline]
    pub fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub fn full() -> Self {
        Self(u64::MAX)
    }

    #[inline]
    pub fn count_ones(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub fn trailing_zeros(self) -> u32 {
        self.0.trailing_zeros()
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl Iterator for Bitboard {
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

impl std::ops::BitAnd for Bitboard {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitXor for Bitboard {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitXorAssign for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl std::ops::Not for Bitboard {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl std::ops::Mul for Bitboard {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

macro_rules! shift_impl {
    ($($t:ty),*) => {
        $(
            impl std::ops::Shl<$t> for Bitboard {
                type Output = Self;

                #[inline]
                fn shl(self, rhs: $t) -> Self {
                    Self(self.0 << rhs)
                }
            }
            impl std::ops::Shr<$t> for Bitboard {
                type Output = Self;

                #[inline]
                fn shr(self, rhs: $t) -> Self {
                    Self(self.0 >> rhs)
                }
            }
        )*
    };
}

shift_impl! {
    u8, u16, u32, u64,
    i8, i16, i32, i64
}
