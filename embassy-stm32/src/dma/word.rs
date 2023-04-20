#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WordSize {
    OneByte,
    TwoBytes,
    FourBytes,
}

impl WordSize {
    pub fn bytes(&self) -> usize {
        match self {
            Self::OneByte => 1,
            Self::TwoBytes => 2,
            Self::FourBytes => 4,
        }
    }
}

mod sealed {
    pub trait Word {}
}

pub trait Word: sealed::Word + Default + Copy + 'static {
    fn size() -> WordSize;
    fn bits() -> usize;
}

macro_rules! impl_word {
    (_, $T:ident, $bits:literal, $size:ident) => {
        impl sealed::Word for $T {}
        impl Word for $T {
            fn bits() -> usize {
                $bits
            }
            fn size() -> WordSize {
                WordSize::$size
            }
        }
    };
    ($T:ident, $uX:ident, $bits:literal, $size:ident) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, Default)]
        pub struct $T(pub $uX);
        impl_word!(_, $T, $bits, $size);
    };
}

impl_word!(U1, u8, 1, OneByte);
impl_word!(U2, u8, 2, OneByte);
impl_word!(U3, u8, 3, OneByte);
impl_word!(U4, u8, 4, OneByte);
impl_word!(U5, u8, 5, OneByte);
impl_word!(U6, u8, 6, OneByte);
impl_word!(U7, u8, 7, OneByte);
impl_word!(_, u8, 8, OneByte);
impl_word!(U9, u16, 9, TwoBytes);
impl_word!(U10, u16, 10, TwoBytes);
impl_word!(U11, u16, 11, TwoBytes);
impl_word!(U12, u16, 12, TwoBytes);
impl_word!(U13, u16, 13, TwoBytes);
impl_word!(U14, u16, 14, TwoBytes);
impl_word!(U15, u16, 15, TwoBytes);
impl_word!(_, u16, 16, TwoBytes);
impl_word!(U17, u32, 17, FourBytes);
impl_word!(U18, u32, 18, FourBytes);
impl_word!(U19, u32, 19, FourBytes);
impl_word!(U20, u32, 20, FourBytes);
impl_word!(U21, u32, 21, FourBytes);
impl_word!(U22, u32, 22, FourBytes);
impl_word!(U23, u32, 23, FourBytes);
impl_word!(U24, u32, 24, FourBytes);
impl_word!(U25, u32, 25, FourBytes);
impl_word!(U26, u32, 26, FourBytes);
impl_word!(U27, u32, 27, FourBytes);
impl_word!(U28, u32, 28, FourBytes);
impl_word!(U29, u32, 29, FourBytes);
impl_word!(U30, u32, 30, FourBytes);
impl_word!(U31, u32, 31, FourBytes);
impl_word!(_, u32, 32, FourBytes);
