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
    type Get<'a>;
    type Set<'a>;

    fn assert<const OFFSET: usize, const SIZE: usize>() {}
    fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Get<'a>;
    fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self::Set<'_>);
}

impl<const N: usize> PackedField for [u8; N] {
    type Get<'a> = &'a [u8];
    type Set<'a> = &'a [u8];

    fn assert<const OFFSET: usize, const SIZE: usize>() {
        const_assert!(N: usize, SIZE: usize => SIZE == N*8, "Incorrect array size");
        const_assert!(OFFSET: usize => OFFSET % 8 == 0, "bit packing for arrays is not supported");
    }

    fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Get<'a> {
        let byte = OFFSET / 8;
        let size_bytes = SIZE / 8;
        &data[byte..byte + size_bytes]
    }

    fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self::Set<'_>) {
        let byte = OFFSET / 8;
        let size_bytes = SIZE / 8;
        data[byte..byte + size_bytes].copy_from_slice(&val)
    }
}

impl PackedField for bool {
    type Get<'a> = bool;
    type Set<'a> = bool;

    fn assert<const OFFSET: usize, const SIZE: usize>() {
        const_assert!(SIZE: usize => SIZE == 1, "bool size must equal 1");
    }

    #[inline]
    fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Get<'a> {
        let byte = OFFSET / 8;
        let bit = OFFSET % 8;
        (data[byte] & (1 << bit)) != 0
    }

    #[inline]
    fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self::Set<'_>) {
        let byte = OFFSET / 8;
        let bit = OFFSET % 8;
        let mask = (val as u8) << bit;
        data[byte] = mask | (data[byte] & !mask);
    }
}

impl PackedField for u8 {
    type Get<'a> = u8;
    type Set<'a> = u8;

    fn assert<const OFFSET: usize, const SIZE: usize>() {
        const_assert!(SIZE: usize => SIZE <= 8, "u8 is not large enough");
        const_assert!(OFFSET: usize, SIZE: usize => {
            let bit = OFFSET % 8;
            SIZE <= (8 - bit)
        }, "bit packing across byte boundary is not supported");
    }

    #[inline]
    fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Get<'a> {
        let byte = OFFSET / 8;
        let bit = OFFSET % 8;
        let mask = (0xFF >> (8 - SIZE)) << bit;
        (data[byte] & mask) >> bit
    }

    #[inline]
    fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self::Set<'_>) {
        let byte = OFFSET / 8;
        let bit = OFFSET % 8;
        let mask = (0xFF >> (8 - SIZE)) << bit;
        data[byte] = (val << bit) | (data[byte] & !mask);
    }
}

/// Big Endian
pub struct BE<T>(T);

/// Little Endian
pub struct LE<T>(T);

macro_rules! impl_packed_field_int {
    ($ty:ty, $size:literal) => {
        impl_packed_field_int!(BE<$ty>, $ty, $size, from_be_bytes, to_be_bytes);
        impl_packed_field_int!(LE<$ty>, $ty, $size, from_le_bytes, to_le_bytes);
    };
    ($wrapper:ty, $ty:ty, $size:literal, $from_bytes_fn:ident, $to_bytes_fn:ident) => {
        impl PackedField for $wrapper {
            type Get<'a> = $ty;
            type Set<'a> = $ty;

            fn assert<const OFFSET: usize, const SIZE: usize>() {
                const_assert!(SIZE: usize => SIZE == $size, "type size mismatch");
                // most protocols only use bit packing at byte (u8) boundaries, so this is okay for now
                const_assert!(OFFSET: usize => OFFSET % 8 == 0, "bit packing for this type is not supported");
            }

            #[inline]
            fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Get<'a> {
                let byte = OFFSET / 8;
                <$ty>::$from_bytes_fn(data[byte..byte+(SIZE/8)].try_into().unwrap())
            }

            #[inline]
            fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self::Set<'_>) {
                let byte = OFFSET / 8;
                data[byte..byte+(SIZE/8)].copy_from_slice(&val.$to_bytes_fn())
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
                $(#[doc = $field_doc:literal])*
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
            /// Packed struct size in bytes
            pub const SIZE: usize = $size;

            pub fn new() -> Self {
                Self {
                    data: [0u8; Self::SIZE]
                }
            }
        }

        impl<T: AsRef<[u8]>> $name<T> {
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

            // Generate getter methods for fields
            $(
                $(#[doc = $field_doc])*
                #[inline]
                pub fn $field(&self) -> <$ty as crate::packed::PackedField>::Get<'_> {
                    const _: () = core::assert!($offset + $bit_size <= $size * 8, "Field offset is out of range");
                    <$ty as crate::packed::PackedField>::assert::<{$offset}, {$bit_size}>();
                    <$ty as crate::packed::PackedField>::get::<{$offset}, {$bit_size}>(self.data.as_ref())
                }
            )*
        }

        impl<T: AsRef<[u8]> + AsMut<[u8]>> $name<T> {
            // Generate setter methods for fields
            $(
                paste::paste! {
                    $(#[doc = $field_doc])*
                    #[inline]
                    pub fn [<set_$field>](&mut self, val: <$ty as crate::packed::PackedField>::Set<'_>) {
                        const _: () = core::assert!($offset + $bit_size <= $size * 8, "Field offset is out of range");
                        <$ty as crate::packed::PackedField>::assert::<{$offset}, {$bit_size}>();
                        <$ty as crate::packed::PackedField>::set::<{$offset}, {$bit_size}>(self.data.as_mut(), val)
                    }
                }
            )*
        }

        impl crate::packed::PackedField for $name<[u8; $size]> {
            type Get<'a> = $name<&'a [u8]>;
            type Set<'a> = $name<&'a [u8]>;

            fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Get<'a> {
                <[u8; $size] as crate::packed::PackedField>::assert::<OFFSET, SIZE>();
                let val = <[u8; $size] as crate::packed::PackedField>::get::<OFFSET, SIZE>(data);
                unsafe { Self::Get::from_bytes_unchecked(val) }
            }

            fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self::Set<'_>) {
                <[u8; $size] as crate::packed::PackedField>::assert::<OFFSET, SIZE>();
                <[u8; $size] as crate::packed::PackedField>::set::<OFFSET, SIZE>(data, &val.data);
            }
        }

        impl<T: AsRef<[u8]>> core::fmt::Debug for $name<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name))
                    $(
                        .field(stringify!($field), &self.$field())
                    )*
                    .finish()
            }
        }

        #[cfg(feature = "defmt")]
        impl<T: AsRef<[u8]>> defmt::Format for $name<T> {
            fn format(&self, f: defmt::Formatter) {
                defmt::write!(f, "{} {{ ", stringify!($name));
                $(
                    defmt::write!(f, "{}: {} ", stringify!($field), self.$field());
                )*
                defmt::write!(f, "}}");
            }
        }
    }
}

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
            type Get<'a> = Result<Self, $ty>;
            type Set<'a> = Self;

            #[inline]
            fn get<'a, const OFFSET: usize, const SIZE: usize>(data: &'a [u8]) -> Self::Get<'a> {
                <$ty as crate::packed::PackedField>::assert::<OFFSET, SIZE>();
                let val = <$ty as crate::packed::PackedField>::get::<OFFSET, SIZE>(data);
                Self::try_from(val)
            }

            #[inline]
            fn set<const OFFSET: usize, const SIZE: usize>(data: &mut [u8], val: Self::Set<'_>) {
                <$ty as crate::packed::PackedField>::assert::<OFFSET, SIZE>();
                <$ty as crate::packed::PackedField>::set::<OFFSET, SIZE>(data, val.into());
            }
        }
    };
}
