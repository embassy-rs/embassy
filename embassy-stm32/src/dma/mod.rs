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

use core::marker::PhantomData;

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

pub(crate) trait SealedChannel: StoppablePeripheral {
    fn id(&self) -> u8;
}

/// A Channel that has not been type-erased
#[allow(private_bounds)]
pub trait TypedChannel: Channel {
    /// The interrupt type for this DMA channel.
    type Interrupt: interrupt::typelevel::Interrupt;

    #[doc(hidden)]
    #[cfg_attr(not(feature = "rt"), allow(unused))]
    unsafe fn on_irq();
}

/// DMA channel.
#[allow(private_bounds)]
pub trait Channel: SealedChannel + PeripheralType + 'static {}

/// Degrade a TypedChannel into an AnyChannel
#[inline]
pub fn dma_into<'a, T: TypedChannel, I: interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>>(
    channel: Peri<'a, T>,
    _irq: &I,
) -> Peri<'a, AnyChannel> {
    unsafe {
        Peri::new_unchecked(AnyChannel {
            id: channel.id(),
            #[cfg(feature = "low-power")]
            stop_mode: channel.stop_mode(),
        })
    }
}

/// DMA interrupt handler.
#[allow(private_bounds)]
pub struct InterruptHandler<T: TypedChannel> {
    _phantom: PhantomData<T>,
}

impl<T: TypedChannel> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::on_irq();
    }
}

macro_rules! dma_channel_impl {
    ($channel_peri:ident, $index:expr, $stop_mode:ident, $irq:ty) => {
        impl crate::rcc::StoppablePeripheral for crate::peripherals::$channel_peri {
            #[cfg(feature = "low-power")]
            fn stop_mode(&self) -> crate::rcc::StopMode {
                crate::rcc::StopMode::$stop_mode
            }
        }

        impl crate::dma::SealedChannel for crate::peripherals::$channel_peri {
            fn id(&self) -> u8 {
                $index
            }
        }

        impl crate::dma::TypedChannel for crate::peripherals::$channel_peri {
            type Interrupt = $irq;

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

impl StoppablePeripheral for AnyChannel {
    #[cfg(feature = "low-power")]
    fn stop_mode(&self) -> crate::rcc::StopMode {
        self.stop_mode
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
