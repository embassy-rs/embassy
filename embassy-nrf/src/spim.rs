use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;
use embassy::traits;
use embassy::util::WakerRegistration;
use embassy_extras::peripheral::{PeripheralMutex, PeripheralState};
use futures::future::poll_fn;
use traits::spi::FullDuplex;

use crate::interrupt::{self, Interrupt};
use crate::{pac, slice_in_ram_or};

pub use crate::hal::spim::{
    Frequency, Mode, Phase, Pins, Polarity, MODE_0, MODE_1, MODE_2, MODE_3,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    /// EasyDMA can only read from data memory, read only buffers in flash will fail.
    DMABufferNotInDataMemory,
}

struct State<T: Instance> {
    spim: T,
    waker: WakerRegistration,
}

pub struct Spim<T: Instance> {
    inner: PeripheralMutex<State<T>>,
}

pub struct Config {
    pub pins: Pins,
    pub frequency: Frequency,
    pub mode: Mode,
    pub orc: u8,
}

impl<T: Instance> Spim<T> {
    pub fn new(mut spim: T, irq: T::Interrupt, config: Config) -> Self {
        let r = spim.regs();

        // Select pins.
        r.psel.sck.write(|w| {
            unsafe { w.bits(config.pins.sck.psel_bits()) };
            w.connect().connected()
        });

        match config.pins.mosi {
            Some(mosi) => r.psel.mosi.write(|w| {
                unsafe { w.bits(mosi.psel_bits()) };
                w.connect().connected()
            }),
            None => r.psel.mosi.write(|w| w.connect().disconnected()),
        }
        match config.pins.miso {
            Some(miso) => r.psel.miso.write(|w| {
                unsafe { w.bits(miso.psel_bits()) };
                w.connect().connected()
            }),
            None => r.psel.miso.write(|w| w.connect().disconnected()),
        }

        // Enable SPIM instance.
        r.enable.write(|w| w.enable().enabled());

        // Configure mode.
        let mode = config.mode;
        r.config.write(|w| {
            // Can't match on `mode` due to embedded-hal, see https://github.com/rust-embedded/embedded-hal/pull/126
            if mode == MODE_0 {
                w.order().msb_first();
                w.cpol().active_high();
                w.cpha().leading();
            } else if mode == MODE_1 {
                w.order().msb_first();
                w.cpol().active_high();
                w.cpha().trailing();
            } else if mode == MODE_2 {
                w.order().msb_first();
                w.cpol().active_low();
                w.cpha().leading();
            } else {
                w.order().msb_first();
                w.cpol().active_low();
                w.cpha().trailing();
            }
            w
        });

        // Configure frequency.
        let frequency = config.frequency;
        r.frequency.write(|w| w.frequency().variant(frequency));

        // Set over-read character
        let orc = config.orc;
        r.orc.write(|w|
            // The ORC field is 8 bits long, so any u8 is a valid value to write.
            unsafe { w.orc().bits(orc) });

        // Disable all events interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        Self {
            inner: PeripheralMutex::new(
                State {
                    spim,
                    waker: WakerRegistration::new(),
                },
                irq,
            ),
        }
    }

    fn inner(self: Pin<&mut Self>) -> Pin<&mut PeripheralMutex<State<T>>> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner) }
    }
}

impl<T: Instance> FullDuplex<u8> for Spim<T> {
    type Error = Error;

    #[rustfmt::skip]
    type WriteFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;
    #[rustfmt::skip]
    type ReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;
    #[rustfmt::skip]
    type WriteReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn read<'a>(self: Pin<&'a mut Self>, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move { todo!() }
    }
    fn write<'a>(self: Pin<&'a mut Self>, data: &'a [u8]) -> Self::WriteFuture<'a> {
        async move { todo!() }
    }

    fn read_write<'a>(
        mut self: Pin<&'a mut Self>,
        rx: &'a mut [u8],
        tx: &'a [u8],
    ) -> Self::WriteReadFuture<'a> {
        async move {
            slice_in_ram_or(rx, Error::DMABufferNotInDataMemory)?;
            slice_in_ram_or(tx, Error::DMABufferNotInDataMemory)?;

            self.as_mut().inner().register_interrupt();
            self.as_mut().inner().with(|s, _irq| {
                // Conservative compiler fence to prevent optimizations that do not
                // take in to account actions by DMA. The fence has been placed here,
                // before any DMA action has started.
                compiler_fence(Ordering::SeqCst);

                let r = s.spim.regs();

                // Set up the DMA write.
                r.txd
                    .ptr
                    .write(|w| unsafe { w.ptr().bits(tx.as_ptr() as u32) });
                r.txd
                    .maxcnt
                    .write(|w| unsafe { w.maxcnt().bits(tx.len() as _) });

                // Set up the DMA read.
                r.rxd
                    .ptr
                    .write(|w| unsafe { w.ptr().bits(rx.as_mut_ptr() as u32) });
                r.rxd
                    .maxcnt
                    .write(|w| unsafe { w.maxcnt().bits(rx.len() as _) });

                // Reset and enable the event
                r.events_end.reset();
                r.intenset.write(|w| w.end().set());

                // Start SPI transaction.
                r.tasks_start.write(|w| unsafe { w.bits(1) });

                // Conservative compiler fence to prevent optimizations that do not
                // take in to account actions by DMA. The fence has been placed here,
                // after all possible DMA actions have completed.
                compiler_fence(Ordering::SeqCst);
            });

            // Wait for 'end' event.
            poll_fn(|cx| {
                self.as_mut().inner().with(|s, _irq| {
                    let r = s.spim.regs();
                    if r.events_end.read().bits() != 0 {
                        return Poll::Ready(());
                    }
                    s.waker.register(cx.waker());
                    Poll::Pending
                })
            })
            .await;

            Ok(())
        }
    }
}

impl<U: Instance> PeripheralState for State<U> {
    type Interrupt = U::Interrupt;
    fn on_interrupt(&mut self) {
        if self.spim.regs().events_end.read().bits() != 0 {
            self.spim.regs().intenclr.write(|w| w.end().clear());
            self.waker.wake()
        }
    }
}

mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&mut self) -> &pac::spim0::RegisterBlock;
    }
}

pub trait Instance: sealed::Instance {
    type Interrupt: Interrupt;
}

macro_rules! make_impl {
    ($SPIMx:ident, $IRQ:ident) => {
        impl sealed::Instance for pac::$SPIMx {
            fn regs(&mut self) -> &pac::spim0::RegisterBlock {
                self
            }
        }
        impl Instance for pac::$SPIMx {
            type Interrupt = interrupt::$IRQ;
        }
    };
}

#[cfg(feature = "52810")]
make_impl!(SPIM0, SPIM0_SPIS0_SPI0);
#[cfg(not(feature = "52810"))]
make_impl!(SPIM0, SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
make_impl!(SPIM1, SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
make_impl!(SPIM2, SPIM2_SPIS2_SPI2);

#[cfg(any(feature = "52833", feature = "52840"))]
make_impl!(SPIM3, SPIM3);
