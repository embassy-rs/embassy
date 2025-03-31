//! Serial Peripheral Instance in slave mode (SPIS) driver.

#![macro_use]
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
pub use embedded_hal_02::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
pub use pac::spis::vals::Order as BitOrder;

use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::gpio::{self, convert_drive, AnyPin, OutputDrive, Pin as GpioPin, SealedPin as _};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::gpio::vals as gpiovals;
use crate::pac::spis::vals;
use crate::util::slice_in_ram_or;
use crate::{interrupt, pac};

/// SPIS error
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

/// SPIS configuration.
#[non_exhaustive]
pub struct Config {
    /// SPI mode
    pub mode: Mode,

    /// Bit order
    pub bit_order: BitOrder,

    /// Overread character.
    ///
    /// If the master keeps clocking the bus after all the bytes in the TX buffer have
    /// already been transmitted, this byte will be constantly transmitted in the MISO line.
    pub orc: u8,

    /// Default byte.
    ///
    /// This is the byte clocked out in the MISO line for ignored transactions (if the master
    /// sets CSN low while the semaphore is owned by the firmware)
    pub def: u8,

    /// Automatically make the firmware side acquire the semaphore on transfer end.
    pub auto_acquire: bool,

    /// Drive strength for the MISO line.
    pub miso_drive: OutputDrive,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: MODE_0,
            bit_order: BitOrder::MSB_FIRST,
            orc: 0x00,
            def: 0x00,
            auto_acquire: true,
            miso_drive: OutputDrive::HighDrive,
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

        if r.events_end().read() != 0 {
            s.waker.wake();
            r.intenclr().write(|w| w.set_end(true));
        }

        if r.events_acquired().read() != 0 {
            s.waker.wake();
            r.intenclr().write(|w| w.set_acquired(true));
        }
    }
}

/// SPIS driver.
pub struct Spis<'d, T: Instance> {
    _p: Peri<'d, T>,
}

impl<'d, T: Instance> Spis<'d, T> {
    /// Create a new SPIS driver.
    pub fn new(
        spis: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        cs: Peri<'d, impl GpioPin>,
        sck: Peri<'d, impl GpioPin>,
        miso: Peri<'d, impl GpioPin>,
        mosi: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            spis,
            cs.into(),
            Some(sck.into()),
            Some(miso.into()),
            Some(mosi.into()),
            config,
        )
    }

    /// Create a new SPIS driver, capable of TX only (MISO only).
    pub fn new_txonly(
        spis: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        cs: Peri<'d, impl GpioPin>,
        sck: Peri<'d, impl GpioPin>,
        miso: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(spis, cs.into(), Some(sck.into()), Some(miso.into()), None, config)
    }

    /// Create a new SPIS driver, capable of RX only (MOSI only).
    pub fn new_rxonly(
        spis: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        cs: Peri<'d, impl GpioPin>,
        sck: Peri<'d, impl GpioPin>,
        mosi: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(spis, cs.into(), Some(sck.into()), None, Some(mosi.into()), config)
    }

    /// Create a new SPIS driver, capable of TX only (MISO only) without SCK pin.
    pub fn new_txonly_nosck(
        spis: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        cs: Peri<'d, impl GpioPin>,
        miso: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(spis, cs.into(), None, Some(miso.into()), None, config)
    }

    fn new_inner(
        spis: Peri<'d, T>,
        cs: Peri<'d, AnyPin>,
        sck: Option<Peri<'d, AnyPin>>,
        miso: Option<Peri<'d, AnyPin>>,
        mosi: Option<Peri<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        compiler_fence(Ordering::SeqCst);

        let r = T::regs();

        // Configure pins.
        cs.conf().write(|w| w.set_input(gpiovals::Input::CONNECT));
        r.psel().csn().write_value(cs.psel_bits());
        if let Some(sck) = &sck {
            sck.conf().write(|w| w.set_input(gpiovals::Input::CONNECT));
            r.psel().sck().write_value(sck.psel_bits());
        }
        if let Some(mosi) = &mosi {
            mosi.conf().write(|w| w.set_input(gpiovals::Input::CONNECT));
            r.psel().mosi().write_value(mosi.psel_bits());
        }
        if let Some(miso) = &miso {
            miso.conf().write(|w| {
                w.set_dir(gpiovals::Dir::OUTPUT);
                convert_drive(w, config.miso_drive);
            });
            r.psel().miso().write_value(miso.psel_bits());
        }

        // Enable SPIS instance.
        r.enable().write(|w| w.set_enable(vals::Enable::ENABLED));

        let mut spis = Self { _p: spis };

        // Apply runtime peripheral configuration
        Self::set_config(&mut spis, &config).unwrap();

        // Disable all events interrupts.
        r.intenclr().write(|w| w.0 = 0xFFFF_FFFF);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        spis
    }

    fn prepare(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
        slice_in_ram_or(tx, Error::BufferNotInRAM)?;
        // NOTE: RAM slice check for rx is not necessary, as a mutable
        // slice can only be built from data located in RAM.

        compiler_fence(Ordering::SeqCst);

        let r = T::regs();

        // Set up the DMA write.
        if tx.len() > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }
        r.txd().ptr().write_value(tx as *const u8 as _);
        r.txd().maxcnt().write(|w| w.set_maxcnt(tx.len() as _));

        // Set up the DMA read.
        if rx.len() > EASY_DMA_SIZE {
            return Err(Error::RxBufferTooLong);
        }
        r.rxd().ptr().write_value(rx as *mut u8 as _);
        r.rxd().maxcnt().write(|w| w.set_maxcnt(rx.len() as _));

        // Reset end event.
        r.events_end().write_value(0);

        // Release the semaphore.
        r.tasks_release().write_value(1);

        Ok(())
    }

    fn blocking_inner_from_ram(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(usize, usize), Error> {
        compiler_fence(Ordering::SeqCst);
        let r = T::regs();

        // Acquire semaphore.
        if r.semstat().read().0 != 1 {
            r.events_acquired().write_value(0);
            r.tasks_acquire().write_value(1);
            // Wait until CPU has acquired the semaphore.
            while r.semstat().read().0 != 1 {}
        }

        self.prepare(rx, tx)?;

        // Wait for 'end' event.
        while r.events_end().read() == 0 {}

        let n_rx = r.rxd().amount().read().0 as usize;
        let n_tx = r.txd().amount().read().0 as usize;

        compiler_fence(Ordering::SeqCst);

        Ok((n_rx, n_tx))
    }

    fn blocking_inner(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(usize, usize), Error> {
        match self.blocking_inner_from_ram(rx, tx) {
            Ok(n) => Ok(n),
            Err(Error::BufferNotInRAM) => {
                trace!("Copying SPIS tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..tx.len()];
                tx_ram_buf.copy_from_slice(tx);
                self.blocking_inner_from_ram(rx, tx_ram_buf)
            }
            Err(error) => Err(error),
        }
    }

    async fn async_inner_from_ram(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(usize, usize), Error> {
        let r = T::regs();
        let s = T::state();

        // Clear status register.
        r.status().write(|w| {
            w.set_overflow(true);
            w.set_overread(true);
        });

        // Acquire semaphore.
        if r.semstat().read().0 != 1 {
            // Reset and enable the acquire event.
            r.events_acquired().write_value(0);
            r.intenset().write(|w| w.set_acquired(true));

            // Request acquiring the SPIS semaphore.
            r.tasks_acquire().write_value(1);

            // Wait until CPU has acquired the semaphore.
            poll_fn(|cx| {
                s.waker.register(cx.waker());
                if r.events_acquired().read() == 1 {
                    r.events_acquired().write_value(0);
                    return Poll::Ready(());
                }
                Poll::Pending
            })
            .await;
        }

        self.prepare(rx, tx)?;

        // Wait for 'end' event.
        r.intenset().write(|w| w.set_end(true));
        poll_fn(|cx| {
            s.waker.register(cx.waker());
            if r.events_end().read() != 0 {
                r.events_end().write_value(0);
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        let n_rx = r.rxd().amount().read().0 as usize;
        let n_tx = r.txd().amount().read().0 as usize;

        compiler_fence(Ordering::SeqCst);

        Ok((n_rx, n_tx))
    }

    async fn async_inner(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(usize, usize), Error> {
        match self.async_inner_from_ram(rx, tx).await {
            Ok(n) => Ok(n),
            Err(Error::BufferNotInRAM) => {
                trace!("Copying SPIS tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..tx.len()];
                tx_ram_buf.copy_from_slice(tx);
                self.async_inner_from_ram(rx, tx_ram_buf).await
            }
            Err(error) => Err(error),
        }
    }

    /// Reads data from the SPI bus without sending anything. Blocks until `cs` is deasserted.
    /// Returns number of bytes read.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        self.blocking_inner(data, &[]).map(|n| n.0)
    }

    /// Simultaneously sends and receives data. Blocks until the transmission is completed.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    /// Returns number of bytes transferred `(n_rx, n_tx)`.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(usize, usize), Error> {
        self.blocking_inner(read, write)
    }

    /// Same as [`blocking_transfer`](Spis::blocking_transfer) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    /// Returns number of bytes transferred `(n_rx, n_tx)`.
    pub fn blocking_transfer_from_ram(&mut self, read: &mut [u8], write: &[u8]) -> Result<(usize, usize), Error> {
        self.blocking_inner_from_ram(read, write)
    }

    /// Simultaneously sends and receives data.
    /// Places the received data into the same buffer and blocks until the transmission is completed.
    /// Returns number of bytes transferred.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        self.blocking_inner_from_ram(data, data).map(|n| n.0)
    }

    /// Sends data, discarding any received data. Blocks  until the transmission is completed.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    /// Returns number of bytes written.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.blocking_inner(&mut [], data).map(|n| n.1)
    }

    /// Same as [`blocking_write`](Spis::blocking_write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    /// Returns number of bytes written.
    pub fn blocking_write_from_ram(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.blocking_inner_from_ram(&mut [], data).map(|n| n.1)
    }

    /// Reads data from the SPI bus without sending anything.
    /// Returns number of bytes read.
    pub async fn read(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        self.async_inner(data, &[]).await.map(|n| n.0)
    }

    /// Simultaneously sends and receives data.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    /// Returns number of bytes transferred `(n_rx, n_tx)`.
    pub async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(usize, usize), Error> {
        self.async_inner(read, write).await
    }

    /// Same as [`transfer`](Spis::transfer) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    /// Returns number of bytes transferred `(n_rx, n_tx)`.
    pub async fn transfer_from_ram(&mut self, read: &mut [u8], write: &[u8]) -> Result<(usize, usize), Error> {
        self.async_inner_from_ram(read, write).await
    }

    /// Simultaneously sends and receives data. Places the received data into the same buffer.
    /// Returns number of bytes transferred.
    pub async fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        self.async_inner_from_ram(data, data).await.map(|n| n.0)
    }

    /// Sends data, discarding any received data.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    /// Returns number of bytes written.
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.async_inner(&mut [], data).await.map(|n| n.1)
    }

    /// Same as [`write`](Spis::write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    /// Returns number of bytes written.
    pub async fn write_from_ram(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.async_inner_from_ram(&mut [], data).await.map(|n| n.1)
    }

    /// Checks if last transaction overread.
    pub fn is_overread(&mut self) -> bool {
        T::regs().status().read().overread()
    }

    /// Checks if last transaction overflowed.
    pub fn is_overflow(&mut self) -> bool {
        T::regs().status().read().overflow()
    }
}

impl<'d, T: Instance> Drop for Spis<'d, T> {
    fn drop(&mut self) {
        trace!("spis drop");

        // Disable
        let r = T::regs();
        r.enable().write(|w| w.set_enable(vals::Enable::DISABLED));

        gpio::deconfigure_pin(r.psel().sck().read());
        gpio::deconfigure_pin(r.psel().csn().read());
        gpio::deconfigure_pin(r.psel().miso().read());
        gpio::deconfigure_pin(r.psel().mosi().read());

        trace!("spis drop: done");
    }
}

pub(crate) struct State {
    waker: AtomicWaker,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::spis::Spis;
    fn state() -> &'static State;
}

/// SPIS peripheral instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_spis {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::spis::SealedInstance for peripherals::$type {
            fn regs() -> pac::spis::Spis {
                pac::$pac_type
            }
            fn state() -> &'static crate::spis::State {
                static STATE: crate::spis::State = crate::spis::State::new();
                &STATE
            }
        }
        impl crate::spis::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

// ====================

impl<'d, T: Instance> SetConfig for Spis<'d, T> {
    type Config = Config;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        let r = T::regs();
        // Configure mode.
        let mode = config.mode;
        r.config().write(|w| {
            w.set_order(config.bit_order);
            match mode {
                MODE_0 => {
                    w.set_cpol(vals::Cpol::ACTIVE_HIGH);
                    w.set_cpha(vals::Cpha::LEADING);
                }
                MODE_1 => {
                    w.set_cpol(vals::Cpol::ACTIVE_HIGH);
                    w.set_cpha(vals::Cpha::TRAILING);
                }
                MODE_2 => {
                    w.set_cpol(vals::Cpol::ACTIVE_LOW);
                    w.set_cpha(vals::Cpha::LEADING);
                }
                MODE_3 => {
                    w.set_cpol(vals::Cpol::ACTIVE_LOW);
                    w.set_cpha(vals::Cpha::TRAILING);
                }
            }
        });

        // Set over-read character.
        let orc = config.orc;
        r.orc().write(|w| w.set_orc(orc));

        // Set default character.
        let def = config.def;
        r.def().write(|w| w.set_def(def));

        // Configure auto-acquire on 'transfer end' event.
        let auto_acquire = config.auto_acquire;
        r.shorts().write(|w| w.set_end_acquire(auto_acquire));

        Ok(())
    }
}
