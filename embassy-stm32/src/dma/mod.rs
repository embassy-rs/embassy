//! Direct Memory Access (DMA)
#![macro_use]

#[cfg(any(bdma, dma))]
mod dma_bdma;

#[cfg(any(bdma, dma))]
pub use dma_bdma::*;

#[cfg(gpdma)]
pub(crate) mod gpdma;
#[cfg(gpdma)]
pub use gpdma::ringbuffered::*;
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

use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};

use crate::interrupt;
use crate::rcc::StoppablePeripheral;

/// The direction of a DMA transfer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Dir {
    /// Transfer from memory to a peripheral.
    MemoryToPeripheral,
    /// Transfer from a peripheral to memory.
    PeripheralToMemory,
}

/// DMA request type alias. (also known as DMA channel number in some chips)
#[cfg(any(dma_v2, bdma_v2, gpdma, dmamux))]
pub type Request = u8;
/// DMA request type alias. (also known as DMA channel number in some chips)
#[cfg(not(any(dma_v2, bdma_v2, gpdma, dmamux)))]
pub type Request = ();

impl<'a> StoppablePeripheral for Peri<'a, AnyChannel> {
    #[cfg(feature = "low-power")]
    fn stop_mode(&self) -> crate::rcc::StopMode {
        self.stop_mode
    }
}

pub(crate) trait SealedChannel {
    #[cfg(not(stm32n6))]
    fn id(&self) -> u8;
    #[cfg(feature = "low-power")]
    fn stop_mode(&self) -> crate::rcc::StopMode;
}

#[cfg(not(stm32n6))]
pub(crate) trait ChannelInterrupt {
    #[cfg_attr(not(feature = "rt"), allow(unused))]
    unsafe fn on_irq();
}

/// DMA channel.
#[allow(private_bounds)]
pub trait Channel: SealedChannel + PeripheralType + Into<AnyChannel> + 'static {}

#[cfg(not(stm32n6))]
macro_rules! dma_channel_impl {
    ($channel_peri:ident, $index:expr, $stop_mode:ident) => {
        impl crate::dma::SealedChannel for crate::peripherals::$channel_peri {
            fn id(&self) -> u8 {
                $index
            }

            #[cfg(feature = "low-power")]
            fn stop_mode(&self) -> crate::rcc::StopMode {
                crate::rcc::StopMode::$stop_mode
            }
        }
        impl crate::dma::ChannelInterrupt for crate::peripherals::$channel_peri {
            unsafe fn on_irq() {
                crate::dma::AnyChannel {
                    id: $index,
                    #[cfg(feature = "low-power")]
                    stop_mode: crate::rcc::StopMode::$stop_mode,
                }
                .on_irq();
            }
        }

        impl crate::dma::Channel for crate::peripherals::$channel_peri {}

        impl From<crate::peripherals::$channel_peri> for crate::dma::AnyChannel {
            fn from(val: crate::peripherals::$channel_peri) -> Self {
                Self {
                    id: crate::dma::SealedChannel::id(&val),
                    #[cfg(feature = "low-power")]
                    stop_mode: crate::dma::SealedChannel::stop_mode(&val),
                }
            }
        }
    };
}

/// Type-erased DMA channel.
pub struct AnyChannel {
    pub(crate) id: u8,
    #[cfg(feature = "low-power")]
    pub(crate) stop_mode: crate::rcc::StopMode,
}
impl_peripheral!(AnyChannel);

impl AnyChannel {
    fn info(&self) -> &ChannelInfo {
        &crate::_generated::DMA_CHANNELS[self.id as usize]
    }
}

impl SealedChannel for AnyChannel {
    #[cfg(not(stm32n6))]
    fn id(&self) -> u8 {
        self.id
    }

    #[cfg(feature = "low-power")]
    fn stop_mode(&self) -> crate::rcc::StopMode {
        self.stop_mode
    }
}
impl Channel for AnyChannel {}

const CHANNEL_COUNT: usize = crate::_generated::DMA_CHANNELS.len();
static STATE: [ChannelState; CHANNEL_COUNT] = [ChannelState::NEW; CHANNEL_COUNT];

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
