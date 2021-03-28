use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;
use embassy::traits;
use embassy::util::{wake_on_interrupt, PeripheralBorrow};
use embassy_extras::unborrow;
use futures::future::poll_fn;
use traits::spi::FullDuplex;

use crate::gpio::sealed::Pin as _;
use crate::gpio::{OptionalPin, Pin as GpioPin};
use crate::interrupt::{self, Interrupt};
use crate::{pac, peripherals, slice_in_ram_or};

pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
pub use pac::spim0::frequency::FREQUENCY_A as Frequency;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    /// EasyDMA can only read from data memory, read only buffers in flash will fail.
    DMABufferNotInDataMemory,
}

pub struct Spim<'d, T: Instance> {
    peri: T,
    irq: T::Interrupt,
    phantom: PhantomData<&'d mut T>,
}

pub struct Config {
    pub frequency: Frequency,
    pub mode: Mode,
    pub orc: u8,
}

impl<'d, T: Instance> Spim<'d, T> {
    pub fn new(
        spim: impl PeripheralBorrow<Target = T> + 'd,
        irq: impl PeripheralBorrow<Target = T::Interrupt> + 'd,
        sck: impl PeripheralBorrow<Target = impl GpioPin> + 'd,
        miso: impl PeripheralBorrow<Target = impl OptionalPin> + 'd,
        mosi: impl PeripheralBorrow<Target = impl OptionalPin> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(spim, irq, sck, miso, mosi);

        let r = spim.regs();

        // Configure pins
        sck.conf().write(|w| w.dir().output().drive().h0h1());
        if let Some(mosi) = mosi.pin_mut() {
            mosi.conf().write(|w| w.dir().output().drive().h0h1());
        }
        if let Some(miso) = miso.pin_mut() {
            miso.conf().write(|w| w.input().connect().drive().h0h1());
        }

        match config.mode.polarity {
            Polarity::IdleHigh => {
                sck.set_high();
                if let Some(mosi) = mosi.pin_mut() {
                    mosi.set_high();
                }
            }
            Polarity::IdleLow => {
                sck.set_low();
                if let Some(mosi) = mosi.pin_mut() {
                    mosi.set_low();
                }
            }
        }

        // Select pins.
        // Note: OptionalPin reports 'disabled' for psel_bits when no pin was selected.
        r.psel.sck.write(|w| unsafe { w.bits(sck.psel_bits()) });
        r.psel.mosi.write(|w| unsafe { w.bits(mosi.psel_bits()) });
        r.psel.miso.write(|w| unsafe { w.bits(miso.psel_bits()) });

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
            peri: spim,
            irq,
            phantom: PhantomData,
        }
    }
}

impl<'d, T: Instance> FullDuplex<u8> for Spim<'d, T> {
    type Error = Error;

    #[rustfmt::skip]
    type WriteFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;
    #[rustfmt::skip]
    type ReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;
    #[rustfmt::skip]
    type WriteReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn read<'a>(self: Pin<&'a mut Self>, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
        self.read_write(data, &[])
    }
    fn write<'a>(self: Pin<&'a mut Self>, data: &'a [u8]) -> Self::WriteFuture<'a> {
        self.read_write(&mut [], data)
    }

    fn read_write<'a>(
        self: Pin<&'a mut Self>,
        rx: &'a mut [u8],
        tx: &'a [u8],
    ) -> Self::WriteReadFuture<'a> {
        async move {
            let this = unsafe { self.get_unchecked_mut() };
            slice_in_ram_or(rx, Error::DMABufferNotInDataMemory)?;
            slice_in_ram_or(tx, Error::DMABufferNotInDataMemory)?;

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started.
            compiler_fence(Ordering::SeqCst);

            let r = this.peri.regs();

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

            // Wait for 'end' event.
            poll_fn(|cx| {
                let r = this.peri.regs();

                if r.events_end.read().bits() != 0 {
                    r.events_end.reset();
                    return Poll::Ready(());
                }

                wake_on_interrupt(&mut this.irq, cx.waker());

                Poll::Pending
            })
            .await;

            Ok(())
        }
    }
}

mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&self) -> &pac::spim0::RegisterBlock;
    }
}

pub trait Instance: sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_instance {
    ($type:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$type {
            fn regs(&self) -> &pac::spim0::RegisterBlock {
                unsafe { &*pac::$type::ptr() }
            }
        }
        impl Instance for peripherals::$type {
            type Interrupt = interrupt::$irq;
        }
    };
}

#[cfg(feature = "52810")]
impl_instance!(SPIM0, SPIM0_SPIS0_SPI0);
#[cfg(not(feature = "52810"))]
impl_instance!(SPIM0, SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl_instance!(SPIM1, SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl_instance!(SPIM2, SPIM2_SPIS2_SPI2);

#[cfg(any(feature = "52833", feature = "52840"))]
impl_instance!(SPIM3, SPIM3);
