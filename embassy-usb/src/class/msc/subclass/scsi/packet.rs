pub trait BitField {
    type Output;

    fn get(data: &[u8], offset: usize, size: usize) -> Self::Output;
    fn set(data: &mut [u8], offset: usize, size: usize, val: Self);
}

impl BitField for u8 {
    type Output = u8;

    #[inline]
    fn get(data: &[u8], offset: usize, size: usize) -> Self::Output {
        let byte = offset / 8;
        let bit = offset % 8;
        let mask = (0xFF >> size) << bit;
        (data[byte] & mask) >> bit
    }

    #[inline]
    fn set(data: &mut [u8], offset: usize, size: usize, val: Self) {
        let byte = offset / 8;
        let bit = offset % 8;
        data[byte] = 0;
    }
}

#[macro_export]
macro_rules! gen_packet {
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
            data: T
        }

        impl $name<[u8; $size]> {
            const SIZE: usize = $size;

            pub fn new() -> Self {
                Self {
                    data: [0u8; Self::SIZE]
                }
            }
        }

        impl<T: AsRef<[u8]>> $name<T> {
            pub const unsafe fn new_unchecked(data: T) -> Self {
                Self { data }
            }

            pub fn from_bytes(buf: T) -> Option<Self> {
                if buf.as_ref().len() < $name::SIZE {
                    None
                } else {
                    Some(unsafe { Self::new_unchecked(buf) })
                }
            }

            $(
                #[inline]
                pub fn $field(&self) -> <$ty as crate::class::msc::subclass::scsi::packet::BitField>::Output {
                    const _: () = core::assert!($offset + $bit_size <= $size * 8, "Field offset is out of range");
                    <$ty as crate::class::msc::subclass::scsi::packet::BitField>::get(self.data.as_ref(), $offset, $bit_size)
                }
            )*
        }

        impl<T: AsRef<[u8]> + AsMut<[u8]>> $name<T> {
            $(
                paste::paste! {
                    #[inline]
                    pub fn [<set_$field>](&mut self, val: $ty) {
                        <$ty as crate::class::msc::subclass::scsi::packet::BitField>::set(self.data.as_mut(), $offset, $bit_size, val)
                    }
                }
            )*
        }
    }
}

// gen_packet!(pub struct Test<8> {
//     #[offset = 8 * 7, size = 3]
//     test: u8,
// });

#[macro_export]
macro_rules! gen_enum {
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

        impl crate::class::msc::subclass::scsi::packet::BitField for $name {
            type Output = Result<Self, $ty>;

            #[inline]
            fn get(data: &[u8], offset: usize, size: usize) -> Self::Output {
                let val = <$ty as crate::class::msc::subclass::scsi::packet::BitField>::get(data, offset, size);
                Self::try_from(val)
            }

            #[inline]
            fn set(data: &mut [u8], offset: usize, size: usize, val: Self) {
                <$ty as crate::class::msc::subclass::scsi::packet::BitField>::set(data, offset, size, val.into());
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
