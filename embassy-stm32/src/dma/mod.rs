//! Direct Memory Access (DMA)
#![macro_use]

#[cfg(any(bdma, dma))]
mod dma_bdma;
#[cfg(any(bdma, dma))]
pub use dma_bdma::*;

#[cfg(gpdma)]
pub(crate) mod gpdma;
#[cfg(gpdma)]
pub use gpdma::*;

#[cfg(dmamux)]
mod dmamux;
#[cfg(dmamux)]
pub(crate) use dmamux::*;

mod util;
pub(crate) use util::*;

pub(crate) mod ringbuffer;
pub mod word;

use core::mem;

use embassy_hal_internal::{impl_peripheral, Peripheral};

use crate::interrupt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum Dir {
    MemoryToPeripheral,
    PeripheralToMemory,
}

/// DMA request type alias. (also known as DMA channel number in some chips)
#[cfg(any(dma_v2, bdma_v2, gpdma, dmamux))]
pub type Request = u8;
/// DMA request type alias. (also known as DMA channel number in some chips)
#[cfg(not(any(dma_v2, bdma_v2, gpdma, dmamux)))]
pub type Request = ();

pub(crate) trait SealedChannel {
    fn id(&self) -> u8;
}

pub(crate) trait ChannelInterrupt {
    #[cfg_attr(not(feature = "rt"), allow(unused))]
    unsafe fn on_irq();
}

/// DMA channel.
#[allow(private_bounds)]
pub trait Channel: SealedChannel + Peripheral<P = Self> + Into<AnyChannel> + 'static {
    /// Type-erase (degrade) this pin into an `AnyChannel`.
    ///
    /// This converts DMA channel singletons (`DMA1_CH3`, `DMA2_CH1`, ...), which
    /// are all different types, into the same type. It is useful for
    /// creating arrays of channels, or avoiding generics.
    #[inline]
    fn degrade(self) -> AnyChannel {
        AnyChannel { id: self.id() }
    }
}

macro_rules! dma_channel_impl {
    ($channel_peri:ident, $index:expr) => {
        impl crate::dma::SealedChannel for crate::peripherals::$channel_peri {
            fn id(&self) -> u8 {
                $index
            }
        }
        impl crate::dma::ChannelInterrupt for crate::peripherals::$channel_peri {
            unsafe fn on_irq() {
                crate::dma::AnyChannel { id: $index }.on_irq();
            }
        }

        impl crate::dma::Channel for crate::peripherals::$channel_peri {}

        impl From<crate::peripherals::$channel_peri> for crate::dma::AnyChannel {
            fn from(x: crate::peripherals::$channel_peri) -> Self {
                crate::dma::Channel::degrade(x)
            }
        }
    };
}

/// Type-erased DMA channel.
pub struct AnyChannel {
    pub(crate) id: u8,
}
impl_peripheral!(AnyChannel);

impl AnyChannel {
    fn info(&self) -> &ChannelInfo {
        &crate::_generated::DMA_CHANNELS[self.id as usize]
    }
}

impl SealedChannel for AnyChannel {
    fn id(&self) -> u8 {
        self.id
    }
}
impl Channel for AnyChannel {}

const CHANNEL_COUNT: usize = crate::_generated::DMA_CHANNELS.len();
static STATE: [ChannelState; CHANNEL_COUNT] = [ChannelState::NEW; CHANNEL_COUNT];

/// "No DMA" placeholder.
///
/// You may pass this in place of a real DMA channel when creating a driver
/// to indicate it should not use DMA.
///
/// This often causes async functionality to not be available on the instance,
/// leaving only blocking functionality.
pub struct NoDma;

impl_peripheral!(NoDma);

// TODO: replace transmutes with core::ptr::metadata once it's stable
#[allow(unused)]
pub(crate) fn slice_ptr_parts<T>(slice: *const [T]) -> (usize, usize) {
    unsafe { mem::transmute(slice) }
}

#[allow(unused)]
pub(crate) fn slice_ptr_parts_mut<T>(slice: *mut [T]) -> (usize, usize) {
    unsafe { mem::transmute(slice) }
}

// safety: must be called only once at startup
pub(crate) unsafe fn init(
    cs: critical_section::CriticalSection,
    #[cfg(bdma)] bdma_priority: interrupt::Priority,
    #[cfg(dma)] dma_priority: interrupt::Priority,
    #[cfg(gpdma)] gpdma_priority: interrupt::Priority,
) {
    #[cfg(any(dma, bdma))]
    dma_bdma::init(
        cs,
        #[cfg(dma)]
        dma_priority,
        #[cfg(bdma)]
        bdma_priority,
    );
    #[cfg(gpdma)]
    gpdma::init(cs, gpdma_priority);
    #[cfg(dmamux)]
    dmamux::init(cs);
}
