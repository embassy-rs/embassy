#![macro_use]

use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;
use embassy::interrupt::InterruptExt;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

use crate::gpio::sealed::Pin as _;
use crate::gpio::{self, AnyPin};
use crate::gpio::{Pin as GpioPin, PselBits};
use crate::interrupt::Interrupt;
use crate::util::{slice_ptr_parts, slice_ptr_parts_mut};
use crate::{pac, util::slice_in_ram_or};

pub use embedded_hal_02::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
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
    phantom: PhantomData<&'d mut T>,
}

#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
    pub mode: Mode,
    pub orc: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::M1,
            mode: MODE_0,
            orc: 0x00,
        }
    }
}

impl<'d, T: Instance> Spim<'d, T> {
    pub fn new(
        spim: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        sck: impl Unborrow<Target = impl GpioPin> + 'd,
        miso: impl Unborrow<Target = impl GpioPin> + 'd,
        mosi: impl Unborrow<Target = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(sck, miso, mosi);
        Self::new_inner(
            spim,
            irq,
            sck.degrade(),
            Some(miso.degrade()),
            Some(mosi.degrade()),
            config,
        )
    }

    pub fn new_txonly(
        spim: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        sck: impl Unborrow<Target = impl GpioPin> + 'd,
        mosi: impl Unborrow<Target = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(sck, mosi);
        Self::new_inner(spim, irq, sck.degrade(), None, Some(mosi.degrade()), config)
    }

    pub fn new_rxonly(
        spim: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        sck: impl Unborrow<Target = impl GpioPin> + 'd,
        miso: impl Unborrow<Target = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(sck, miso);
        Self::new_inner(spim, irq, sck.degrade(), Some(miso.degrade()), None, config)
    }

    fn new_inner(
        _spim: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        sck: AnyPin,
        miso: Option<AnyPin>,
        mosi: Option<AnyPin>,
        config: Config,
    ) -> Self {
        unborrow!(irq);

        let r = T::regs();

        // Configure pins
        sck.conf().write(|w| w.dir().output().drive().h0h1());
        if let Some(mosi) = &mosi {
            mosi.conf().write(|w| w.dir().output().drive().h0h1());
        }
        if let Some(miso) = &miso {
            miso.conf().write(|w| w.input().connect().drive().h0h1());
        }

        match config.mode.polarity {
            Polarity::IdleHigh => {
                sck.set_high();
                if let Some(mosi) = &mosi {
                    mosi.set_high();
                }
            }
            Polarity::IdleLow => {
                sck.set_low();
                if let Some(mosi) = &mosi {
                    mosi.set_low();
                }
            }
        }

        // Select pins.
        r.psel.sck.write(|w| unsafe { w.bits(sck.psel_bits()) });
        r.psel.mosi.write(|w| unsafe { w.bits(mosi.psel_bits()) });
        r.psel.miso.write(|w| unsafe { w.bits(miso.psel_bits()) });

        // Enable SPIM instance.
        r.enable.write(|w| w.enable().enabled());

        // Configure mode.
        let mode = config.mode;
        r.config.write(|w| {
            match mode {
                MODE_0 => {
                    w.order().msb_first();
                    w.cpol().active_high();
                    w.cpha().leading();
                }
                MODE_1 => {
                    w.order().msb_first();
                    w.cpol().active_high();
                    w.cpha().trailing();
                }
                MODE_2 => {
                    w.order().msb_first();
                    w.cpol().active_low();
                    w.cpha().leading();
                }
                MODE_3 => {
                    w.order().msb_first();
                    w.cpol().active_low();
                    w.cpha().trailing();
                }
            }

            w
        });

        // Configure frequency.
        let frequency = config.frequency;
        r.frequency.write(|w| w.frequency().variant(frequency));

        // Set over-read character
        let orc = config.orc;
        r.orc.write(|w| unsafe { w.orc().bits(orc) });

        // Disable all events interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self {
            phantom: PhantomData,
        }
    }

    fn on_interrupt(_: *mut ()) {
        let r = T::regs();
        let s = T::state();

        if r.events_end.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.end().clear());
        }
    }

    fn prepare(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
        slice_in_ram_or(tx, Error::DMABufferNotInDataMemory)?;
        // NOTE: RAM slice check for rx is not necessary, as a mutable
        // slice can only be built from data located in RAM.

        compiler_fence(Ordering::SeqCst);

        let r = T::regs();

        // Set up the DMA write.
        let (ptr, len) = slice_ptr_parts(tx);
        r.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as _) });
        r.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        // Set up the DMA read.
        let (ptr, len) = slice_ptr_parts_mut(rx);
        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as _) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        // Reset and enable the event
        r.events_end.reset();
        r.intenset.write(|w| w.end().set());

        // Start SPI transaction.
        r.tasks_start.write(|w| unsafe { w.bits(1) });

        Ok(())
    }

    fn blocking_inner(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
        self.prepare(rx, tx)?;

        // Wait for 'end' event.
        while T::regs().events_end.read().bits() == 0 {}

        compiler_fence(Ordering::SeqCst);

        Ok(())
    }

    async fn async_inner(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
        self.prepare(rx, tx)?;

        // Wait for 'end' event.
        poll_fn(|cx| {
            T::state().end_waker.register(cx.waker());
            if T::regs().events_end.read().bits() != 0 {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);

        Ok(())
    }

    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.blocking_inner(data, &[])
    }

    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.blocking_inner(read, write)
    }

    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.blocking_inner(data, data)
    }

    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.blocking_inner(&mut [], data)
    }

    pub async fn read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.async_inner(data, &[]).await
    }

    pub async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.async_inner(read, write).await
    }

    pub async fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.async_inner(data, data).await
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.async_inner(&mut [], data).await
    }
}

impl<'d, T: Instance> Drop for Spim<'d, T> {
    fn drop(&mut self) {
        trace!("spim drop");

        // TODO check for abort, wait for xxxstopped

        // disable!
        let r = T::regs();
        r.enable.write(|w| w.enable().disabled());

        gpio::deconfigure_pin(r.psel.sck.read().bits());
        gpio::deconfigure_pin(r.psel.miso.read().bits());
        gpio::deconfigure_pin(r.psel.mosi.read().bits());

        trace!("spim drop: done");
    }
}

pub(crate) mod sealed {
    use embassy::waitqueue::AtomicWaker;

    use super::*;

    pub struct State {
        pub end_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                end_waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static pac::spim0::RegisterBlock;
        fn state() -> &'static State;
    }
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_spim {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::spim::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::spim0::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::spim::sealed::State {
                static STATE: crate::spim::sealed::State = crate::spim::sealed::State::new();
                &STATE
            }
        }
        impl crate::spim::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}

// ====================

mod eh02 {
    use super::*;

    impl<'d, T: Instance> embedded_hal_02::blocking::spi::Transfer<u8> for Spim<'d, T> {
        type Error = Error;
        fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
            self.blocking_transfer_in_place(words)?;
            Ok(words)
        }
    }

    impl<'d, T: Instance> embedded_hal_02::blocking::spi::Write<u8> for Spim<'d, T> {
        type Error = Error;

        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(words)
        }
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl embedded_hal_1::spi::Error for Error {
        fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
            match *self {
                Self::TxBufferTooLong => embedded_hal_1::spi::ErrorKind::Other,
                Self::RxBufferTooLong => embedded_hal_1::spi::ErrorKind::Other,
                Self::DMABufferNotInDataMemory => embedded_hal_1::spi::ErrorKind::Other,
            }
        }
    }

    impl<'d, T: Instance> embedded_hal_1::spi::ErrorType for Spim<'d, T> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_1::spi::blocking::Read<u8> for Spim<'d, T> {
        fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_transfer(words, &[])
        }

        fn read_transaction(&mut self, words: &mut [&mut [u8]]) -> Result<(), Self::Error> {
            for buf in words {
                self.blocking_read(buf)?
            }
            Ok(())
        }
    }

    impl<'d, T: Instance> embedded_hal_1::spi::blocking::Write<u8> for Spim<'d, T> {
        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(words)
        }

        fn write_transaction(&mut self, words: &[&[u8]]) -> Result<(), Self::Error> {
            for buf in words {
                self.blocking_write(buf)?
            }
            Ok(())
        }

        fn write_iter<WI>(&mut self, words: WI) -> Result<(), Self::Error>
        where
            WI: IntoIterator<Item = u8>,
        {
            for w in words {
                self.blocking_write(&[w])?;
            }
            Ok(())
        }
    }

    impl<'d, T: Instance> embedded_hal_1::spi::blocking::ReadWrite<u8> for Spim<'d, T> {
        fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
            self.blocking_transfer(read, write)
        }

        fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_transfer_in_place(words)
        }

        fn transaction<'a>(
            &mut self,
            operations: &mut [embedded_hal_1::spi::blocking::Operation<'a, u8>],
        ) -> Result<(), Self::Error> {
            use embedded_hal_1::spi::blocking::Operation;
            for o in operations {
                match o {
                    Operation::Read(b) => self.blocking_read(b)?,
                    Operation::Write(b) => self.blocking_write(b)?,
                    Operation::Transfer(r, w) => self.blocking_transfer(r, w)?,
                    Operation::TransferInPlace(b) => self.blocking_transfer_in_place(b)?,
                }
            }
            Ok(())
        }
    }
}

#[cfg(all(feature = "unstable-traits", feature = "nightly"))]
mod eh1a {
    use super::*;
    use core::future::Future;

    impl<'d, T: Instance> embedded_hal_async::spi::Read<u8> for Spim<'d, T> {
        type ReadFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn read<'a>(&'a mut self, words: &'a mut [u8]) -> Self::ReadFuture<'a> {
            self.read(words)
        }

        type ReadTransactionFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn read_transaction<'a>(
            &'a mut self,
            words: &'a mut [&'a mut [u8]],
        ) -> Self::ReadTransactionFuture<'a> {
            async move {
                for buf in words {
                    self.read(buf).await?
                }
                Ok(())
            }
        }
    }

    impl<'d, T: Instance> embedded_hal_async::spi::Write<u8> for Spim<'d, T> {
        type WriteFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn write<'a>(&'a mut self, data: &'a [u8]) -> Self::WriteFuture<'a> {
            self.write(data)
        }

        type WriteTransactionFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn write_transaction<'a>(
            &'a mut self,
            words: &'a [&'a [u8]],
        ) -> Self::WriteTransactionFuture<'a> {
            async move {
                for buf in words {
                    self.write(buf).await?
                }
                Ok(())
            }
        }
    }

    impl<'d, T: Instance> embedded_hal_async::spi::ReadWrite<u8> for Spim<'d, T> {
        type TransferFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn transfer<'a>(&'a mut self, rx: &'a mut [u8], tx: &'a [u8]) -> Self::TransferFuture<'a> {
            self.transfer(rx, tx)
        }

        type TransferInPlaceFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn transfer_in_place<'a>(
            &'a mut self,
            words: &'a mut [u8],
        ) -> Self::TransferInPlaceFuture<'a> {
            self.transfer_in_place(words)
        }

        type TransactionFuture<'a>
        where
            Self: 'a,
        = impl Future<Output = Result<(), Self::Error>> + 'a;

        fn transaction<'a>(
            &'a mut self,
            operations: &'a mut [embedded_hal_async::spi::Operation<'a, u8>],
        ) -> Self::TransactionFuture<'a> {
            use embedded_hal_1::spi::blocking::Operation;
            async move {
                for o in operations {
                    match o {
                        Operation::Read(b) => self.read(b).await?,
                        Operation::Write(b) => self.write(b).await?,
                        Operation::Transfer(r, w) => self.transfer(r, w).await?,
                        Operation::TransferInPlace(b) => self.transfer_in_place(b).await?,
                    }
                }
                Ok(())
            }
        }
    }
}
