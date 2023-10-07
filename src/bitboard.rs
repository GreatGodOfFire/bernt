#[macro_export]
macro_rules! bitloop {
    ($a:expr => $abit:ident, $b:expr => $bbit:ident, $code:block) => {
        let mut a = $a;
        let mut b = $b;

        while a != 0 {
            let $abit = a.trailing_zeros() as u8;
            let $bbit = b.trailing_zeros() as u8;
            a &= a - 1;
            b &= b - 1;

            $code
        }
    };
    ($a:expr => $abit:ident, $code:block) => {
        let mut a = $a;

        while a != 0 {
            let $abit = a.trailing_zeros() as u8;
            a &= a - 1;

            $code
        }
    };
}
