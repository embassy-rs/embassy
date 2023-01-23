/// A hack to allow compile-time assertions on const parameters.
/// Gets around `can't use generic parameters from outer function` error.
/// For some reason this assert is not shown in rust-analyzer, but cargo build catches it.
macro_rules! const_assert {
    ($($list:ident : $ty:ty),* => $expr:expr $(,$msg:literal)?) => {{
        struct Assert<$(const $list: usize,)*>;
        impl<$(const $list: $ty,)*> Assert<$($list,)*> {
            const OK: () = core::assert!($expr, $($msg)?);
        }
        Assert::<$($list,)*>::OK
    }};
}

pub trait PackedField {
    type Output<'a>;

    fn assert<const OFFSET: usize, const SIZE: usize>() {}
    fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Output<'a>;
    fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self);
}

impl PackedField for &[u8] {
    type Output<'a> = &'a [u8];

    fn assert<const OFFSET: usize, const SIZE: usize>() {
        const_assert!(OFFSET: usize => OFFSET % 8 == 0, "bit packing for u8 slices is not supported");
    }

    fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Output<'a> {
        data
    }

    fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self) {
        data.copy_from_slice(val)
    }
}

impl PackedField for &mut [u8] {
    type Output<'a> = &'a [u8];

    fn assert<const OFFSET: usize, const SIZE: usize>() {
        const_assert!(OFFSET: usize => OFFSET % 8 == 0, "bit packing for u8 slices is not supported");
    }

    fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Output<'a> {
        data
    }

    fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self) {
        data.copy_from_slice(val)
    }
}

impl<const N: usize> PackedField for [u8; N] {
    type Output<'a> = [u8; N];

    fn assert<const OFFSET: usize, const SIZE: usize>() {
        const_assert!(N: usize, SIZE: usize => SIZE == N, "Incorrect array size");
        const_assert!(OFFSET: usize => OFFSET % 8 == 0, "bit packing for arrays is not supported");
    }

    fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Output<'a> {
        data.try_into().unwrap()
    }

    fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self) {
        data.copy_from_slice(&val)
    }
}

impl PackedField for bool {
    type Output<'a> = bool;

    fn assert<const OFFSET: usize, const SIZE: usize>() {
        const_assert!(SIZE: usize => SIZE == 1, "bool size must equal 1");
    }

    #[inline]
    fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Output<'a> {
        let byte = OFFSET / 8;
        let bit = OFFSET % 8;
        (data[byte] & (1 << bit)) != 0
    }

    #[inline]
    fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self) {
        let byte = OFFSET / 8;
        let bit = OFFSET % 8;
        let mask = (val as u8) << bit;
        data[byte] = mask | (data[byte] & !mask);
    }
}

impl PackedField for u8 {
    type Output<'a> = u8;

    fn assert<const OFFSET: usize, const SIZE: usize>() {
        const_assert!(SIZE: usize => SIZE <= 8, "u8 is not large enough");
        const_assert!(OFFSET: usize, SIZE: usize => {
            let bit = OFFSET % 8;
            SIZE <= (8 - bit)
        }, "bit packing across byte boundary is not supported");
    }

    #[inline]
    fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Output<'a> {
        let byte = OFFSET / 8;
        let bit = OFFSET % 8;
        let mask = (0xFF >> SIZE) << bit;
        (data[byte] & mask) >> bit
    }

    #[inline]
    fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self) {
        let byte = OFFSET / 8;
        let bit = OFFSET % 8;
        let mask = (0xFF >> SIZE) << bit;
        data[byte] = (val << bit) | (data[byte] & !mask);
    }
}

macro_rules! impl_packed_field_int {
    ($ty:ty, $size:literal) => {
        impl PackedField for $ty {
            type Output<'a> = $ty;

            fn assert<const OFFSET: usize, const SIZE: usize>() {
                const_assert!(SIZE: usize => SIZE <= $size, "type is not large enough");
                // most protocols only use bit packing at byte (u8) boundaries, so this is okay for now
                const_assert!(OFFSET: usize => OFFSET % 8 == 0, "bit packing for this type is not supported");
            }

            #[inline]
            fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Output<'a> {
                let byte = OFFSET / 8;
                unsafe { *(data[byte..].as_ptr() as *const $ty)}
            }

            #[inline]
            fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self) {
                let byte = OFFSET / 8;
                unsafe { *(data[byte..].as_ptr() as *mut $ty) = val; }
            }
        }
    };
}

impl_packed_field_int!(u16, 16);
impl_packed_field_int!(u32, 32);
impl_packed_field_int!(u64, 64);

impl_packed_field_int!(i8, 8);
impl_packed_field_int!(i16, 16);
impl_packed_field_int!(i32, 32);
impl_packed_field_int!(i64, 64);

#[macro_export]
macro_rules! packed_struct {
    (
        $(#[$meta:meta])*
        $sv:vis struct $name:ident<$size:literal> {
            $(
                #[offset = $offset:expr, size = $bit_size:expr]
                $field:ident: $ty:ty,
            )*
        }
    ) => {
        $(#[$meta])*
        $sv struct $name<T: AsRef<[u8]>> {
            pub data: T
        }

        impl $name<[u8; $size]> {
            const SIZE: usize = $size;

            pub fn new() -> Self {
                Self {
                    data: [0u8; Self::SIZE]
                }
            }
        }

        impl<'d, T: AsRef<[u8]> + crate::packed::PackedField<Output<'d> = T> + 'd> $name<T> {
            pub const unsafe fn from_bytes_unchecked(data: T) -> Self {
                Self { data }
            }

            pub fn from_bytes(buf: T) -> Option<Self> {
                if buf.as_ref().len() < $name::SIZE {
                    None
                } else {
                    Some(unsafe { Self::from_bytes_unchecked(buf) })
                }
            }

            $(
                #[inline]
                pub fn $field(&self) -> <$ty as crate::packed::PackedField>::Output<'d> {
                    const _: () = core::assert!($offset + $bit_size <= $size * 8, "Field offset is out of range");
                    <$ty as crate::packed::PackedField>::assert::<{$offset}, {$bit_size}>();
                    <$ty as crate::packed::PackedField>::get::<{$offset}, {$bit_size}>(self.data.as_ref())
                }
            )*
        }

        impl<'d, T: AsRef<[u8]> + AsMut<[u8]> + crate::packed::PackedField<Output<'d> = T> + 'd> $name<T> {
            $(
                paste::paste! {
                    #[inline]
                    pub fn [<set_$field>](&mut self, val: $ty) {
                        const _: () = core::assert!($offset + $bit_size <= $size * 8, "Field offset is out of range");
                        <$ty as crate::packed::PackedField>::assert::<{$offset}, {$bit_size}>();
                        <$ty as crate::packed::PackedField>::set::<{$offset}, {$bit_size}>(self.data.as_mut(), val)
                    }
                }
            )*
        }

        impl<T: AsRef<[u8]> + for<'a> crate::packed::PackedField<Output<'a> = T>> crate::packed::PackedField for $name<T> {
            type Output<'a> = Self;

            fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Output<'a> {
                <T as crate::packed::PackedField>::assert::<OFFSET, SIZE>();
                let val = <T as crate::packed::PackedField>::get::<OFFSET, SIZE>(data);
                unsafe { Self::from_bytes_unchecked(val) }
            }

            fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self) {
                <T as crate::packed::PackedField>::assert::<OFFSET, SIZE>();
                <T as crate::packed::PackedField>::set::<OFFSET, SIZE>(data, val.data);
            }
        }

        impl<T: AsRef<[u8]> + for<'a> crate::packed::PackedField<Output<'a> = T>> core::fmt::Debug for $name<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name))
                    $(
                        .field(stringify!($field), &self.$field())
                    )*
                    .finish()
            }
        }
    }
}

// packed_struct!(pub struct Test<8> {
//     #[offset = 8 * 6, size = 8]
//     test: u8,
// });

// pub fn test() -> u8 {
//     let t = Test::new();
//     t.test()
// }

#[macro_export]
macro_rules! packed_enum {
    (
        $(#[$meta:meta])*
        $sv:vis enum $name:ident<$ty:ty> {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $variant_val:literal,
            )*
        }
    ) => {
        $(#[$meta])*
        $sv enum $name {
            $(
                $(#[$variant_meta])*
                $variant = $variant_val
            ),*
        }

        impl TryFrom<$ty> for $name {
            type Error = $ty;

            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                match value {
                    $($variant_val => Ok($name::$variant),)*
                    _ => Err(value)
                }
            }
        }

        impl From<$name> for $ty {
            fn from(value: $name) -> $ty {
                value as $ty
            }
        }

        impl crate::packed::PackedField for $name {
            type Output<'a> = Result<Self, $ty>;

            #[inline]
            fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Output<'a> {
                <$ty as crate::packed::PackedField>::assert::<OFFSET, SIZE>();
                let val = <$ty as crate::packed::PackedField>::get::<OFFSET, SIZE>(data);
                Self::try_from(val)
            }

            #[inline]
            fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self) {
                <$ty as crate::packed::PackedField>::assert::<OFFSET, SIZE>();
                <$ty as crate::packed::PackedField>::set::<OFFSET, SIZE>(data, val.into());
            }
        }
    };
}

// gen_enum! {
//     pub enum Testas<u8> {
//         Hello = 0b111,
//         Test = 0b1111,
//     }
// }
