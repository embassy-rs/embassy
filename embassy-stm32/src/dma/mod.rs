//! Direct Memory Access (DMA)
#![macro_use]

#[cfg(any(bdma, dma, mdma))]
mod dma_bdma;

#[cfg(any(bdma, dma, mdma))]
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

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};

use crate::interrupt;

/// The direction of a DMA transfer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Dir {
    /// Transfer from memory to a peripheral.
    MemoryToPeripheral,
    /// Transfer from a peripheral to memory.
    PeripheralToMemory,
    /// Transfer from memory to another memory address.
    MemoryToMemory,
}

/// Which pointer in the transfer to increment.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Increment {
    /// DMA will not increment either of the addresses.
    None,
    /// DMA will increment the peripheral address.
    Peripheral,
    /// DMA will increment the memory address.
    Memory,
    /// DMA will increment both peripheral and memory addresses simultaneously.
    Both,
}

/// DMA request type alias. (also known as DMA channel number in some chips)
#[cfg(any(dma_v2, bdma_v2, gpdma, dmamux))]
pub type Request = u8;
/// DMA request type alias. (also known as DMA channel number in some chips)
#[cfg(not(any(dma_v2, bdma_v2, gpdma, dmamux)))]
pub type Request = ();

/// DMA channel driver
pub struct Channel<'d> {
    pub(crate) id: u8,
    phantom: PhantomData<&'d ()>,
}

impl<'d> Channel<'d> {
    /// Create a new DMA channel driver.
    pub fn new<T: ChannelInstance>(
        _ch: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, crate::dma::InterruptHandler<T>> + 'd,
    ) -> Self {
        Self {
            id: T::ID,
            phantom: PhantomData,
        }
    }

    /// Reborrow the channel, allowing it to be used in multiple places.
    pub fn reborrow(&mut self) -> Channel<'_> {
        Channel {
            id: self.id,
            phantom: PhantomData,
        }
    }

    pub(crate) unsafe fn clone_unchecked(&self) -> Channel<'d> {
        Channel {
            id: self.id,
            phantom: PhantomData,
        }
    }
}

pub(crate) trait SealedChannelInstance {
    const ID: u8;
}

/// DMA channel.
#[allow(private_bounds)]
pub trait ChannelInstance: SealedChannelInstance + PeripheralType + 'static {
    /// The interrupt type for this DMA channel.
    type Interrupt: interrupt::typelevel::Interrupt;
}

/// DMA interrupt handler.
#[allow(private_bounds)]
pub struct InterruptHandler<T: ChannelInstance> {
    _phantom: PhantomData<T>,
}

impl<T: ChannelInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_irq(T::ID)
    }
}

macro_rules! dma_channel_impl {
    ($channel_peri:ident, $index:expr, $irq:ty) => {
        impl crate::dma::SealedChannelInstance for crate::peripherals::$channel_peri {
            const ID: u8 = $index;
        }

        impl crate::dma::ChannelInstance for crate::peripherals::$channel_peri {
            type Interrupt = $irq;
        }
    };
}

const CHANNEL_COUNT: usize = crate::_generated::DMA_CHANNELS.len();
static STATE: [ChannelState; CHANNEL_COUNT] = [ChannelState::NEW; CHANNEL_COUNT];

pub(crate) fn info(id: u8) -> &'static ChannelInfo {
    &crate::_generated::DMA_CHANNELS[id as usize]
}

// safety: must be called only once at startup
pub(crate) unsafe fn init(
    cs: critical_section::CriticalSection,
    #[cfg(bdma)] bdma_priority: interrupt::Priority,
    #[cfg(dma)] dma_priority: interrupt::Priority,
    #[cfg(gpdma)] gpdma_priority: interrupt::Priority,
    #[cfg(mdma)] mdma_priority: interrupt::Priority,
) {
    #[cfg(any(dma, bdma))]
    dma_bdma::init(
        cs,
        #[cfg(dma)]
        dma_priority,
        #[cfg(bdma)]
        bdma_priority,
        #[cfg(mdma)]
        mdma_priority,
    );
    #[cfg(gpdma)]
    gpdma::init(cs, gpdma_priority);
    #[cfg(dmamux)]
    dmamux::init(cs);
}
