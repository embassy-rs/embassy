#![allow(unused)]

use core::{mem, ops, ptr, slice};

use aligned::{A4, Aligned};
use embassy_net_driver_channel::driver::PacketBuf;
use embassy_time::{Duration, Ticker};

use crate::WithContext;

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

pub(crate) fn is_aligned(a: u32, x: u32) -> bool {
    (a & (x - 1)) == 0
}

pub(crate) fn round_down(x: u32, a: u32) -> u32 {
    debug_assert!(a.is_power_of_two());

    x & !(a - 1)
}

pub(crate) fn round_up(x: u32, a: u32) -> u32 {
    debug_assert!(a.is_power_of_two());

    (x + (a - 1)) & !(a - 1)
}

pub(crate) async fn try_until(mut func: impl AsyncFnMut() -> bool, duration: Duration) -> crate::Result<()> {
    let tick = Duration::from_millis(1);
    let mut ticker = Ticker::every(tick);
    let ticks = duration.as_ticks() / tick.as_ticks();

    for _ in 0..ticks {
        if func().await {
            return Ok(());
        }

        ticker.next().await;
    }

    Err(crate::Error)
}

/// Buffer with space for a cmd
pub struct WriteBuffer {
    buf: Aligned<A4, [u8]>,
}

impl WriteBuffer {
    pub fn new(buf: &mut Aligned<A4, [u8]>) -> &mut Self {
        unsafe { &mut *(buf as *mut Aligned<A4, [u8]> as *mut Self) }
    }

    pub fn cmd(&mut self) -> &mut [u8] {
        &mut self.buf[..4]
    }

    pub fn buf(&mut self) -> &mut [u8] {
        &mut self.buf[4..]
    }

    pub fn cmd_buf(&self) -> &Aligned<A4, [u8]> {
        &self.buf
    }
}

impl ops::Index<ops::RangeTo<usize>> for WriteBuffer {
    type Output = Self;

    fn index(&self, mut range: ops::RangeTo<usize>) -> &Self::Output {
        range.end += 4;

        unsafe { &*(&self.buf[range] as *const Aligned<A4, [u8]> as *const [u8] as *const WriteBuffer) }
    }
}

impl ops::IndexMut<ops::RangeTo<usize>> for WriteBuffer {
    fn index_mut(&mut self, mut range: ops::RangeTo<usize>) -> &mut Self::Output {
        range.end += 4;

        unsafe { &mut *(&mut self.buf[range] as *mut Aligned<A4, [u8]> as *mut [u8] as *mut WriteBuffer) }
    }
}

/// The driver DMAs SDPCM frames directly into/out of `PacketBuf` storage, so
/// the pool must be at least 4-byte aligned, and big enough for a full frame
/// plus the headroom the driver reserves in front of it. The biggest frame is
/// the SDPCM header, BDC header and a full MTU ethernet frame: 1530 bytes; the
/// RX headroom (`RX_HEADER_SPACE` in `runner.rs`) is 259 bytes: 1789 total.
/// Enabled via the `packet-buf-align-4` and `packet-buf-size-2048` features on
/// `embassy-net-driver-channel`.
const _: () = {
    core::assert!(
        embassy_net_driver_channel::driver::config::PACKET_BUF_ALIGN >= 4,
        "cyw43 requires the `packet-buf-align-4` (or higher) feature on xarxa-driver"
    );
    core::assert!(
        embassy_net_driver_channel::driver::config::PACKET_BUF_SIZE >= 1789,
        "cyw43 requires a packet buffer size of at least 1789 bytes (e.g. the `packet-buf-size-2048` feature) on xarxa-driver"
    );
};

/// View the whole backing store of a `PacketBuf` as an `Aligned<A4, [u8]>`.
///
/// This is only sound because the packet pool is configured with at least
/// 4-byte alignment (asserted above).
pub(crate) fn packetbuf_storage(buf: &mut PacketBuf) -> &mut Aligned<A4, [u8]> {
    unsafe { &mut *(buf.storage_mut() as *mut [u8] as *mut Aligned<A4, [u8]>) }
}
