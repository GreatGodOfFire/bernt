trait PrintableConst {
    fn spsa(&self, name: &str, min: Self, max: Self);
    fn ctt(&self, name: &str, min: Self, max: Self);
    fn uci(&self, name: &str, min: Self, max: Self);
}

macro_rules! impl_print {
    ($($int:ident),*; $($float:ident),*) => {
        $(
            impl PrintableConst for $int {
                fn spsa(&self, name: &str, min: Self, max: Self) {
                    println!(
                        "{name}, int, {self}, {min}, {max}, {}, 0.002",
                        (*self as f32 / 10.0).max(0.5)
                    );
                }

                fn ctt(&self, name: &str, min: Self, max: Self) {
                    println!(
                        r#"    "{name}": "Integer({min}, {max})","#,
                    );
                }

                fn uci(&self, name: &str, min: Self, max: Self) {
                    println!("option name {name} type spin default {self} min {min} max {max}");
                }
            }
        )*
        $(
            impl PrintableConst for $float {
                fn spsa(&self, name: &str, min: Self, max: Self) {
                    println!(
                        "{name}, float, {self}, {min}, {max}, {}, 0.002",
                        (*self as f32 / 10.0).max(0.5)
                    );
                }

                fn ctt(&self, name: &str, min: Self, max: Self) {
                    println!(
                        r#"    "{name}": "Real({min}, {max})","#,
                    );
                }

                fn uci(&self, name: &str, _: Self, _: Self) {
                    println!("option name {name} type string default {self}");
                }
            }
        )*
    }
}

impl_print! {
    u8, u16, u32, u64, i8, i16, i32, i64;
    f32, f64
}

macro_rules! consts {
    ($(const $name:ident($min:literal..=$max:literal): $ty:ty = $default:literal),*) => {
        #[derive(Clone)]
        pub struct SearchConsts {
            $(
                pub $name: $ty
            ),*
        }
        impl Default for SearchConsts {
            fn default() -> Self {
                Self {
                    $($name: $default),*
                }
            }
        }
        impl SearchConsts {
            pub fn set(&mut self, name: &str, value: &str) -> Result<(), ()> {
                match name {
                    $(stringify!($name) => self.$name = value.parse::<$ty>().unwrap().clamp($min, $max),)*
                    _ => return Err(()),
                }

                Ok(())
            }

            pub fn print_spsa(&self) {
                $(
                    self.$name.spsa(stringify!($name), $min, $max);
                )*
            }
            pub fn print_ctt(&self) {
                $(
                    self.$name.ctt(stringify!($name), $min, $max);
                )*
            }
            pub fn print_uci(&self) {
                $(
                    self.$name.uci(stringify!($name), $min, $max);
                )*
            }
        }
    };
}

consts! {
    const asp_depth(1..=6): u8 = 4,
    const asp_window(5..=200): i32 = 114,
    const asp_inc_factor(0.1..=2.5): f32 = 0.47,

    const nmp_reduction(2..=5): u8 = 3,

    const lmr_base(0.5..=1.5): f32 = 1.13,
    const lmr_div(1.0..=5.0): f32 = 3.25,
    const lmr_n_moves(1..=5): u16 = 5,

    const lmp_depth(1..=6): u8 = 2,
    const lmp_base(1..=10): u16 = 2,
    const lmp_mul(1..=10): u16 = 5,
    const lmp_pow(1..=4): u32 = 2,

    const fp_depth(1..=8): u8 = 5,
    const fp_base(1..=1000): i32 = 259,
    const fp_mul(1..=1000): i32 = 527,

    const rfp_depth(1..=6): u8 = 3,
    const rfp_margin(10..=300): i32 = 82,

    const hist_mul(1..=1000): i32 = 365,
    const hist_add(-1000..=1000): i32 = -427,
    const conthist_mul(1..=1000): i32 = 365,
    const conthist_add(-1000..=1000): i32 = -427,

    const time_harddiv(1.5..=8.0): f32 = 3.79,
    const time_softdiv(20.0..=80.0): f32 = 44.61
}

#[rustfmt::skip]
pub const MVVLVA_LOOKUP: [[i32; 5]; 6] = [
        /* P   N   B   R   Q */ 
/* P */  [ 9, 11, 11, 13, 17],
/* N */  [ 7,  9,  8, 11, 15],
/* B */  [ 7, 10,  9, 11, 15],
/* R */  [ 5,  7,  7,  9, 13],
/* Q */  [ 1,  3,  3,  5,  9],
/* K */  [ 0,  2,  2,  4,  8],
];
