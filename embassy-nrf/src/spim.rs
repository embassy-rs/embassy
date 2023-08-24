//! Serial Peripheral Instance in master mode (SPIM) driver.

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::{into_ref, PeripheralRef};
pub use embedded_hal_02::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
pub use pac::spim0::frequency::FREQUENCY_A as Frequency;

use crate::chip::FORCE_COPY_BUFFER_SIZE;
use crate::gpio::sealed::Pin as _;
use crate::gpio::{self, AnyPin, Pin as GpioPin, PselBits};
use crate::interrupt::typelevel::Interrupt;
use crate::util::{slice_in_ram_or, slice_ptr_parts, slice_ptr_parts_mut};
use crate::{interrupt, pac, Peripheral};

/// SPIM error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// TX buffer was too long.
    TxBufferTooLong,
    /// RX buffer was too long.
    RxBufferTooLong,
    /// EasyDMA can only read from data memory, read only buffers in flash will fail.
    BufferNotInRAM,
}

/// SPIM configuration.
#[non_exhaustive]
pub struct Config {
    /// Frequency
    pub frequency: Frequency,

    /// SPI mode
    pub mode: Mode,

    /// Overread character.
    ///
    /// When doing bidirectional transfers, if the TX buffer is shorter than the RX buffer,
    /// this byte will be transmitted in the MOSI line for the left-over bytes.
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

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        let s = T::state();

        if r.events_end.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.end().clear());
        }
    }
}

/// SPIM driver.
pub struct Spim<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Spim<'d, T> {
    /// Create a new SPIM driver.
    pub fn new(
        spim: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        miso: impl Peripheral<P = impl GpioPin> + 'd,
        mosi: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(sck, miso, mosi);
        Self::new_inner(
            spim,
            sck.map_into(),
            Some(miso.map_into()),
            Some(mosi.map_into()),
            config,
        )
    }

    /// Create a new SPIM driver, capable of TX only (MOSI only).
    pub fn new_txonly(
        spim: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        mosi: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(sck, mosi);
        Self::new_inner(spim, sck.map_into(), None, Some(mosi.map_into()), config)
    }

    /// Create a new SPIM driver, capable of RX only (MISO only).
    pub fn new_rxonly(
        spim: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        miso: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(sck, miso);
        Self::new_inner(spim, sck.map_into(), Some(miso.map_into()), None, config)
    }

    fn new_inner(
        spim: impl Peripheral<P = T> + 'd,
        sck: PeripheralRef<'d, AnyPin>,
        miso: Option<PeripheralRef<'d, AnyPin>>,
        mosi: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        into_ref!(spim);

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

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self { _p: spim }
    }

    fn prepare(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
        slice_in_ram_or(tx, Error::BufferNotInRAM)?;
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

    fn blocking_inner_from_ram(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
        self.prepare(rx, tx)?;

        // Wait for 'end' event.
        while T::regs().events_end.read().bits() == 0 {}

        compiler_fence(Ordering::SeqCst);

        Ok(())
    }

    fn blocking_inner(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Error> {
        match self.blocking_inner_from_ram(rx, tx) {
            Ok(_) => Ok(()),
            Err(Error::BufferNotInRAM) => {
                trace!("Copying SPIM tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..tx.len()];
                tx_ram_buf.copy_from_slice(tx);
                self.blocking_inner_from_ram(rx, tx_ram_buf)
            }
            Err(error) => Err(error),
        }
    }

    async fn async_inner_from_ram(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
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

    async fn async_inner(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Error> {
        match self.async_inner_from_ram(rx, tx).await {
            Ok(_) => Ok(()),
            Err(Error::BufferNotInRAM) => {
                trace!("Copying SPIM tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..tx.len()];
                tx_ram_buf.copy_from_slice(tx);
                self.async_inner_from_ram(rx, tx_ram_buf).await
            }
            Err(error) => Err(error),
        }
    }

    /// Reads data from the SPI bus without sending anything. Blocks until the buffer has been filled.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.blocking_inner(data, &[])
    }

    /// Simultaneously sends and receives data. Blocks until the transmission is completed.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.blocking_inner(read, write)
    }

    /// Same as [`blocking_transfer`](Spim::blocking_transfer) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_transfer_from_ram(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.blocking_inner(read, write)
    }

    /// Simultaneously sends and receives data.
    /// Places the received data into the same buffer and blocks until the transmission is completed.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.blocking_inner_from_ram(data, data)
    }

    /// Sends data, discarding any received data. Blocks  until the transmission is completed.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.blocking_inner(&mut [], data)
    }

    /// Same as [`blocking_write`](Spim::blocking_write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_write_from_ram(&mut self, data: &[u8]) -> Result<(), Error> {
        self.blocking_inner(&mut [], data)
    }

    /// Reads data from the SPI bus without sending anything.
    pub async fn read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.async_inner(data, &[]).await
    }

    /// Simultaneously sends and receives data.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    pub async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.async_inner(read, write).await
    }

    /// Same as [`transfer`](Spim::transfer) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn transfer_from_ram(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.async_inner_from_ram(read, write).await
    }

    /// Simultaneously sends and receives data. Places the received data into the same buffer.
    pub async fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.async_inner_from_ram(data, data).await
    }

    /// Sends data, discarding any received data.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    pub async fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.async_inner(&mut [], data).await
    }

    /// Same as [`write`](Spim::write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn write_from_ram(&mut self, data: &[u8]) -> Result<(), Error> {
        self.async_inner_from_ram(&mut [], data).await
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

        // Disable all events interrupts
        T::Interrupt::disable();

        trace!("spim drop: done");
    }
}

pub(crate) mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

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

/// SPIM peripheral instance
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
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
            type Interrupt = crate::interrupt::typelevel::$irq;
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
                Self::BufferNotInRAM => embedded_hal_1::spi::ErrorKind::Other,
            }
        }
    }

    impl<'d, T: Instance> embedded_hal_1::spi::ErrorType for Spim<'d, T> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_1::spi::SpiBus<u8> for Spim<'d, T> {
        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_transfer(words, &[])
        }

        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(words)
        }

        fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
            self.blocking_transfer(read, write)
        }

        fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_transfer_in_place(words)
        }
    }
}

#[cfg(all(feature = "unstable-traits", feature = "nightly"))]
mod eha {

    use super::*;

    impl<'d, T: Instance> embedded_hal_async::spi::SpiBus<u8> for Spim<'d, T> {
        async fn flush(&mut self) -> Result<(), Error> {
            Ok(())
        }

        async fn read(&mut self, words: &mut [u8]) -> Result<(), Error> {
            self.read(words).await
        }

        async fn write(&mut self, data: &[u8]) -> Result<(), Error> {
            self.write(data).await
        }

        async fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Error> {
            self.transfer(rx, tx).await
        }

        async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Error> {
            self.transfer_in_place(words).await
        }
    }
}

impl<'d, T: Instance> SetConfig for Spim<'d, T> {
    type Config = Config;
    fn set_config(&mut self, config: &Self::Config) {
        let r = T::regs();
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
    }
}
