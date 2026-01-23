#![allow(unused)]

use core::slice;

use aligned::{A4, Aligned};
use embassy_time::{Duration, Ticker};

/// Defines a `repr(u8)` enum and implements a `from()` associated function to instantiate it from
/// a `u8`, defaulting to the variant decorated with `#[default]`.
macro_rules! enum_from_u8 {
    (
        $( #[$enum_attr:meta] )*
        enum $enum:ident {
            // NOTE: The default variant must be the first variant.
            // Additionally, the `#[default]` attribute must be placed before any other attributes
            // on the variant, to avoid a parsing ambiguity.
            #[default]
            $( #[$default_variant_attr:meta] )*
            $default_variant:ident = $default_value:literal,
            $(
                $( #[$variant_attr:meta] )*
                $variant:ident = $value:literal
            ),+
            $(,)?
        }
    ) => {
        $( #[$enum_attr] )*
        #[repr(u8)]
        pub enum $enum {
            $( #[$default_variant_attr] )*
            $default_variant = $default_value,
            $(
                $( #[$variant_attr] )*
                $variant = $value
            ),+
        }

        impl $enum {
            pub fn from(value: u8) -> Self {
                match value {
                    $default_value => Self::$default_variant,
                    $( $value => Self::$variant ),+,
                    _ => Self::$default_variant,
                }
            }
        }
    };
}
pub(crate) use enum_from_u8;

pub(crate) const fn slice8_mut(x: &mut [u32]) -> &mut [u8] {
    let len = size_of_val(x);
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
}

pub(crate) const fn slice16_mut(x: &mut [u32]) -> &mut [u16] {
    let len = size_of_val(x) / 2;
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
}

pub(crate) const fn aligned_mut(x: &mut [u32]) -> &mut Aligned<A4, [u8]> {
    let len = size_of_val(x);
    unsafe { core::mem::transmute(slice::from_raw_parts_mut(x.as_mut_ptr() as *mut u8, len)) }
}

pub(crate) const fn aligned_ref(x: &[u32]) -> &Aligned<A4, [u8]> {
    let len = size_of_val(x);
    unsafe { core::mem::transmute(slice::from_raw_parts(x.as_ptr() as *const u8, len)) }
}

pub(crate) const fn slice32_mut(x: &mut Aligned<A4, [u8]>) -> &mut [u32] {
    let len = (size_of_val(x) + 3) / 4;
    unsafe { slice::from_raw_parts_mut(x as *mut Aligned<A4, [u8]> as *mut u32, len) }
}

pub(crate) const fn slice32_ref(x: &Aligned<A4, [u8]>) -> &[u32] {
    let len = (size_of_val(x) + 3) / 4;
    unsafe { slice::from_raw_parts(x as *const Aligned<A4, [u8]> as *const u32, len) }
}

pub(crate) fn is_aligned(a: u32, x: u32) -> bool {
    (a & (x - 1)) == 0
}

pub(crate) fn round_down(x: u32, a: u32) -> u32 {
    x & !(a - 1)
}

pub(crate) fn round_up(x: u32, a: u32) -> u32 {
    ((x + a - 1) / a) * a
}

pub(crate) async fn try_until(mut func: impl AsyncFnMut() -> bool, duration: Duration) -> bool {
    let tick = Duration::from_millis(1);
    let mut ticker = Ticker::every(tick);
    let ticks = duration.as_ticks() / tick.as_ticks();

    for _ in 0..ticks {
        if func().await {
            return true;
        }

        ticker.next().await;
    }

    false
}
