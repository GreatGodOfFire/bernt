#![allow(long_running_const_eval)]

#[derive(Clone, Copy)]
pub struct Magic {
    pub magic: u64,
    pub mask: u64,
    pub offset: usize,
    pub bits: u32,
}

impl Magic {
    const fn default() -> Self {
        Self {
            magic: 0,
            mask: 0,
            offset: 0,
            bits: 0,
        }
    }
}

pub const ROOK_MAGICS: [Magic; 64] = magics(_ROOK_MAGICS, _ROOK_MASKS);
pub const BISHOP_MAGICS: [Magic; 64] = magics(_BISHOP_MAGICS, _BISHOP_MASKS);
pub const ROOK_ATTACKS: [u64; attack_count(_ROOK_MASKS_BITS)] = rook_attacks();
pub const BISHOP_ATTACKS: [u64; attack_count(_BISHOP_MASKS_BITS)] = bishop_attacks();

const fn attack_count(bits: [u8; 64]) -> usize {
    let mut i = 0;
    let mut sum = 0;

    while i < 64 {
        sum += 2usize.pow(bits[i] as usize as u32);
        i += 1;
    }

    sum
}

const fn rook_attacks() -> [u64; attack_count(_ROOK_MASKS_BITS)] {
    let mut attacks = [0; attack_count(_ROOK_MASKS_BITS)];
    let mut square = 0;

    while square < 64 {
        let mut j = 0;
        let magic = ROOK_MAGICS[square];
        let relevant_bits = magic.mask;
        while j < 1 << relevant_bits.count_ones() {
            let permutation = permutation(j, relevant_bits);

            attacks[((magic.magic.wrapping_mul(permutation)) >> (64 - magic.bits)) as usize
                + magic.offset] = rook_attack(square as u8, permutation);

            j += 1;
        }

        square += 1;
    }

    attacks
}

const fn bishop_attacks() -> [u64; attack_count(_BISHOP_MASKS_BITS)] {
    let mut attacks = [0; attack_count(_BISHOP_MASKS_BITS)];
    let mut square = 0;

    while square < 64 {
        let mut j = 0;
        let magic = BISHOP_MAGICS[square];
        let relevant_bits = magic.mask;
        while j < 1 << relevant_bits.count_ones() {
            let permutation = permutation(j, relevant_bits);

            attacks[((magic.magic.wrapping_mul(permutation & magic.mask)) >> (64 - magic.bits))
                as usize
                + magic.offset] = bishop_attack(square as u8, permutation);

            j += 1;
        }

        square += 1;
    }

    attacks
}

const fn permutation(n: u16, mut mask: u64) -> u64 {
    let mut result = 0;
    let mut i = 0;

    while mask != 0 {
        let j = mask.trailing_zeros();
        let bit = 1 << j;
        mask ^= bit;
        if (n & (1 << i)) != 0 {
            result |= bit;
        }

        i += 1;
    }

    result
}

const fn rook_attack(square: u8, blockings: u64) -> u64 {
    let mut result = 0;
    let rank = square / 8;
    let file = square % 8;
    let mut r;
    let mut f;

    r = rank + 1;
    while r < 8 {
        let bit = 1 << (file + r * 8);
        result |= bit;
        if blockings & bit != 0 {
            break;
        }
        r += 1;
    }
    if rank > 0 {
        r = rank - 1;
        while r < rank {
            let bit = 1 << (file + r * 8);
            result |= bit;
            if blockings & bit != 0 {
                break;
            }
            r = r.wrapping_sub(1);
        }
    }
    f = file + 1;
    while f < 8 {
        let bit = 1 << (f + rank * 8);
        result |= bit;
        if blockings & bit != 0 {
            break;
        }
        f += 1;
    }
    if file > 0 {
        f = file - 1;
        while f < file {
            let bit = 1 << (f + rank * 8);
            result |= bit;
            if blockings & bit != 0 {
                break;
            }
            f = f.wrapping_sub(1);
        }
    }
    result
}

const fn bishop_attack(square: u8, blockings: u64) -> u64 {
    let mut result = 0;
    let rank = square / 8;
    let file = square % 8;
    let (mut r, mut f);

    (r, f) = (rank + 1, file + 1);
    while r < 8 && f < 8 {
        let bit = 1 << (f + r * 8);
        result |= bit;
        if blockings & (1 << (f + r * 8)) != 0 {
            break;
        }
        r += 1;
        f += 1;
    }
    if file > 0 {
        (r, f) = (rank + 1, file - 1);
        while r < 8 && f < 8 {
            let bit = 1 << (f + r * 8);
            result |= bit;
            if blockings & (1 << (f + r * 8)) != 0 {
                break;
            }
            r += 1;
            f = f.wrapping_sub(1);
        }
    }
    if rank > 0 {
        (r, f) = (rank - 1, file + 1);
        while r < 8 && f < 8 {
            let bit = 1 << (f + r * 8);
            result |= bit;
            if blockings & (1 << (f + r * 8)) != 0 {
                break;
            }
            r = r.wrapping_sub(1);
            f += 1;
        }
    }
    if rank > 0 && file > 0 {
        (r, f) = (rank - 1, file - 1);
        while r < 8 && f < 8 {
            let bit = 1 << (f + r * 8);
            result |= bit;
            if blockings & (1 << (f + r * 8)) != 0 {
                break;
            }
            r = r.wrapping_sub(1);
            f = f.wrapping_sub(1);
        }
    }
    result
}

const fn magics(magics: [u64; 64], masks: [u64; 64]) -> [Magic; 64] {
    let mut m = [Magic::default(); 64];
    let mut i = 0;
    let mut offset = 0;

    while i < 64 {
        let bits = masks[i].count_ones();

        m[i] = Magic {
            magic: magics[i],
            mask: masks[i],
            offset,
            bits,
        };
        offset += 1 << bits as usize;
        i += 1;
    }

    m
}

const _ROOK_MAGICS: [u64; 64] = [
    0x40800020c0008074,
    0x0040001000406002,
    0x0100290040200011,
    0x0080100080080004,
    0x8880028004004800,
    0x2200108102001804,
    0x02000200110800c4,
    0x0b00035100028062,
    0x04088008804004a0,
    0x0001400050002008,
    0x0801002000410098,
    0x0011000810002101,
    0x0042001200040821,
    0x0002000408420010,
    0x0204000810010a1c,
    0x0119000210408100,
    0x8000808002400020,
    0x1030044009201040,
    0x0208808020001008,
    0x1000610029001001,
    0x1028010009001035,
    0x0201010008620400,
    0x48a0440008012210,
    0x80200200004100a4,
    0x8000400080009020,
    0x0810200040100142,
    0x8001004100200070,
    0x4400210100100208,
    0x000900c500102800,
    0x0882000280040080,
    0xe180022400880110,
    0x4004006200140881,
    0x0401204002801180,
    0x0004a00580804006,
    0x0206802001801000,
    0x0012001022004088,
    0x1208480080800c00,
    0x4000808e00804400,
    0x0200aa0884003001,
    0x400800e102000084,
    0x0a0020400c808000,
    0x010050002000c009,
    0x0401064020010010,
    0x0910011100090020,
    0x0002040008008080,
    0x00820004080a0010,
    0x0402000411060098,
    0x0b08010080c2000c,
    0x0405800020400180,
    0x2102c00900886500,
    0x3200401020010100,
    0x0000884010220200,
    0x8408980080340080,
    0x600c010040020040,
    0x0200620110880400,
    0x1640028400c90200,
    0x0802004411008022,
    0x0000211281004005,
    0x60400a0083204012,
    0x400200203004400a,
    0x0002000448306002,
    0x0002008804011002,
    0x0040080090210244,
    0x0000002048840102,
];
const _BISHOP_MAGICS: [u64; 64] = [
    0x1004013002048100,
    0xe090010204004200,
    0x001004048026201a,
    0x4124040088402400,
    0x0001104000021a0c,
    0x20010412400a0800,
    0x00004c0208404000,
    0x0002004100882000,
    0x9110062042040100,
    0x0800100204811210,
    0x40080806040c2080,
    0x0844110402800510,
    0x1000020610040191,
    0x28c0008804410220,
    0x004000440c204800,
    0x000000441c140a00,
    0x2810404010025884,
    0x0004006011040900,
    0x2050000804404028,
    0x0008040082084002,
    0x000102a820180104,
    0x0222003098040222,
    0x0240431108081480,
    0x4220488212440401,
    0x2020900254100200,
    0x00700c0090040088,
    0x00840242100110c0,
    0x4400404004010200,
    0x1802840001802000,
    0x0010108011018880,
    0x060200400228060c,
    0x0141091000440088,
    0x0c10101840300314,
    0x2481012000080802,
    0x020b080201010c02,
    0x0200220082480180,
    0x0178010040140224,
    0x8020104480010084,
    0x1030025082304400,
    0x20044408a4008085,
    0x0908281806080810,
    0x0a20880410002200,
    0x8002005144204800,
    0x00a0404200842800,
    0x100d8c050c000204,
    0x0001500102000240,
    0x8022020214224200,
    0x0088710404203089,
    0x2401042220640582,
    0x4004220804144000,
    0x2000010409044040,
    0x0100480020a81010,
    0x0900000861090020,
    0x8000042004011080,
    0x01a0a20a02042104,
    0x5802040800851020,
    0x4008210808046200,
    0x1021102101101002,
    0x0510000304030440,
    0x8400094102460802,
    0x80000008100a0200,
    0x0000005c20342442,
    0x00c0084801040414,
    0x0040100200410062,
];

const _ROOK_MASKS: [u64; 64] = rook_masks();
const _BISHOP_MASKS: [u64; 64] = bishop_masks();

const _ROOK_MASKS_BITS: [u8; 64] = masks_bits(_ROOK_MASKS);
const _BISHOP_MASKS_BITS: [u8; 64] = masks_bits(_BISHOP_MASKS);

const fn rook_masks() -> [u64; 64] {
    let mut masks = [0; 64];
    let mut i = 0;

    while i < 64 {
        masks[i] = rook_mask(i);
        i += 1;
    }

    masks
}

const fn rook_mask(square: usize) -> u64 {
    let mut result = 0;
    let rank = square / 8;
    let file = square % 8;
    let mut r;
    let mut f;

    r = rank + 1;
    while r < 7 {
        result |= 1 << (file + r * 8);
        r += 1;
    }
    if rank > 0 {
        r = rank - 1;
        while r > 0 {
            result |= 1 << (file + r * 8);
            r -= 1;
        }
    }
    f = file + 1;
    while f < 7 {
        result |= 1 << (f + rank * 8);
        f += 1;
    }
    if file > 0 {
        f = file - 1;
        while f > 0 {
            result |= 1 << (f + rank * 8);
            f -= 1;
        }
    }
    result
}

const fn masks_bits(masks: [u64; 64]) -> [u8; 64] {
    let mut bits = [0; 64];
    let mut i = 0;

    while i < 64 {
        bits[i] = masks[i].count_ones() as u8;
        i += 1;
    }
    bits
}

const fn bishop_masks() -> [u64; 64] {
    let mut masks = [0; 64];
    let mut i = 0;

    while i < 64 {
        masks[i] = bishop_mask(i);
        i += 1;
    }

    masks
}

const fn bishop_mask(square: usize) -> u64 {
    let mut result = 0;
    let rank = square / 8;
    let file = square % 8;

    if rank < 7 && file < 7 {
        let (mut r, mut f) = (rank + 1, file + 1);
        while r < 7 && f < 7 {
            result |= 1 << (f + r * 8);
            r += 1;
            f += 1;
        }
    }
    if rank < 7 && file > 0 {
        let (mut r, mut f) = (rank + 1, file - 1);
        while r < 7 && f > 0 {
            result |= 1 << (f + r * 8);
            r += 1;
            f -= 1;
        }
    }
    if rank > 0 && file < 7 {
        let (mut r, mut f) = (rank - 1, file + 1);
        while r > 0 && f < 7 {
            result |= 1 << (f + r * 8);
            r -= 1;
            f += 1;
        }
    }
    if rank > 0 && file > 0 {
        let (mut r, mut f) = (rank - 1, file - 1);
        while r > 0 && f > 0 {
            result |= 1 << (f + r * 8);
            r -= 1;
            f -= 1;
        }
    }
    result
}
