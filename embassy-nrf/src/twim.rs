//! I2C-compatible Two Wire Interface in master mode (TWIM) driver.

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::Ordering::SeqCst;
use core::sync::atomic::compiler_fence;
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
#[cfg(feature = "time")]
use embassy_time::{Duration, Instant};
use embedded_hal_1::i2c::Operation;
pub use pac::twim::vals::Frequency;

use crate::chip::EASY_DMA_SIZE;
use crate::gpio::Pin as GpioPin;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::gpio::vals as gpiovals;
use crate::pac::twim::vals;
use crate::util::slice_in_ram;
use crate::{gpio, interrupt, pac};

/// TWIM config.
#[non_exhaustive]
pub struct Config {
    /// Frequency
    pub frequency: Frequency,

    /// Enable high drive for the SDA line.
    pub sda_high_drive: bool,

    /// Enable internal pullup for the SDA line.
    ///
    /// Note that using external pullups is recommended for I2C, and
    /// most boards already have them.
    pub sda_pullup: bool,

    /// Enable high drive for the SCL line.
    pub scl_high_drive: bool,

    /// Enable internal pullup for the SCL line.
    ///
    /// Note that using external pullups is recommended for I2C, and
    /// most boards already have them.
    pub scl_pullup: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::K100,
            scl_high_drive: false,
            sda_pullup: false,
            sda_high_drive: false,
            scl_pullup: false,
        }
    }
}

/// TWI error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// TX buffer was too long.
    TxBufferTooLong,
    /// RX buffer was too long.
    RxBufferTooLong,
    /// Data transmit failed.
    Transmit,
    /// Data reception failed.
    Receive,
    /// The buffer is not in data RAM and is larger than the RAM buffer. It's most likely in flash, and nRF's DMA cannot access flash.
    RAMBufferTooSmall,
    /// Didn't receive an ACK bit after the address byte. Address might be wrong, or the i2c device chip might not be connected properly.
    AddressNack,
    /// Didn't receive an ACK bit after a data byte.
    DataNack,
    /// Overrun error.
    Overrun,
    /// Timeout error.
    Timeout,
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        let s = T::state();

        if r.events_suspended().read() != 0 {
            s.end_waker.wake();
            r.intenclr().write(|w| w.set_suspended(true));
        }
        if r.events_stopped().read() != 0 {
            s.end_waker.wake();
            r.intenclr().write(|w| w.set_stopped(true));
        }
        if r.events_error().read() != 0 {
            s.end_waker.wake();
            r.intenclr().write(|w| w.set_error(true));
        }
    }
}

/// TWI driver.
pub struct Twim<'d> {
    r: pac::twim::Twim,
    state: &'static State,
    tx_ram_buffer: &'d mut [u8],
    _p: PhantomData<&'d ()>,
}

impl<'d> Twim<'d> {
    /// Create a new TWI driver.
    ///
    /// `tx_ram_buffer` is required if any write operations will be performed with data that is not in RAM.
    /// Usually this is static data that the compiler locates in flash instead of RAM. The `tx_ram_buffer`
    /// needs to be at least as large as the largest write operation that will be executed with a buffer
    /// that is not in RAM. If all write operations will be performed from RAM, an empty buffer (`&[]`) may
    /// be used.
    pub fn new<T: Instance>(
        _twim: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        sda: Peri<'d, impl GpioPin>,
        scl: Peri<'d, impl GpioPin>,
        config: Config,
        tx_ram_buffer: &'d mut [u8],
    ) -> Self {
        let r = T::regs();

        // Configure pins
        sda.set_high();
        scl.set_high();
        sda.conf().write(|w| {
            w.set_dir(gpiovals::Dir::OUTPUT);
            w.set_input(gpiovals::Input::CONNECT);
            #[cfg(not(feature = "_nrf54l"))]
            w.set_drive(match config.sda_high_drive {
                true => gpiovals::Drive::H0D1,
                false => gpiovals::Drive::S0D1,
            });
            #[cfg(feature = "_nrf54l")]
            {
                w.set_drive0(match config.sda_high_drive {
                    true => gpiovals::Drive::H,
                    false => gpiovals::Drive::S,
                });
                w.set_drive1(gpiovals::Drive::D);
            }
            if config.sda_pullup {
                w.set_pull(gpiovals::Pull::PULLUP);
            }
        });
        scl.conf().write(|w| {
            w.set_dir(gpiovals::Dir::OUTPUT);
            w.set_input(gpiovals::Input::CONNECT);
            #[cfg(not(feature = "_nrf54l"))]
            w.set_drive(match config.scl_high_drive {
                true => gpiovals::Drive::H0D1,
                false => gpiovals::Drive::S0D1,
            });
            #[cfg(feature = "_nrf54l")]
            {
                w.set_drive0(match config.scl_high_drive {
                    true => gpiovals::Drive::H,
                    false => gpiovals::Drive::S,
                });
                w.set_drive1(gpiovals::Drive::D);
            }
            if config.scl_pullup {
                w.set_pull(gpiovals::Pull::PULLUP);
            }
        });

        // Select pins.
        r.psel().sda().write_value(sda.psel_bits());
        r.psel().scl().write_value(scl.psel_bits());

        // Enable TWIM instance.
        r.enable().write(|w| w.set_enable(vals::Enable::ENABLED));

        let mut twim = Self {
            r: T::regs(),
            state: T::state(),
            tx_ram_buffer,
            _p: PhantomData {},
        };

        // Apply runtime peripheral configuration
        Self::set_config(&mut twim, &config).unwrap();

        // Disable all events interrupts
        r.intenclr().write(|w| w.0 = 0xFFFF_FFFF);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        twim
    }

    /// Set TX buffer, checking that it is in RAM and has suitable length.
    unsafe fn set_tx_buffer(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let buffer = if slice_in_ram(buffer) {
            buffer
        } else {
            if buffer.len() > self.tx_ram_buffer.len() {
                return Err(Error::RAMBufferTooSmall);
            }
            trace!("Copying TWIM tx buffer into RAM for DMA");
            let ram_buffer = &mut self.tx_ram_buffer[..buffer.len()];
            ram_buffer.copy_from_slice(buffer);
            &*ram_buffer
        };

        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        let r = self.r;

        // We're giving the register a pointer to the stack. Since we're
        // waiting for the I2C transaction to end before this stack pointer
        // becomes invalid, there's nothing wrong here.
        r.dma().tx().ptr().write_value(buffer.as_ptr() as u32);
        r.dma().tx().maxcnt().write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is 8 bits wide and accepts the full range of
            // values.
            w.set_maxcnt(buffer.len() as _));

        Ok(())
    }

    /// Set RX buffer, checking that it has suitable length.
    unsafe fn set_rx_buffer(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // NOTE: RAM slice check is not necessary, as a mutable
        // slice can only be built from data located in RAM.

        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::RxBufferTooLong);
        }

        let r = self.r;

        // We're giving the register a pointer to the stack. Since we're
        // waiting for the I2C transaction to end before this stack pointer
        // becomes invalid, there's nothing wrong here.
        r.dma().rx().ptr().write_value(buffer.as_mut_ptr() as u32);
        r.dma().rx().maxcnt().write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to the type of maxcnt
            // is also fine.
            //
            // Note that that nrf52840 maxcnt is a wider
            // type than a u8, so we use a `_` cast rather than a `u8` cast.
            // The MAXCNT field is thus at least 8 bits wide and accepts the
            // full range of values that fit in a `u8`.
            w.set_maxcnt(buffer.len() as _));

        Ok(())
    }

    fn clear_errorsrc(&mut self) {
        let r = self.r;
        r.errorsrc().write(|w| {
            w.set_anack(true);
            w.set_dnack(true);
            w.set_overrun(true);
        });
    }

    /// Get Error instance, if any occurred.
    fn check_errorsrc(&mut self) -> Result<(), Error> {
        let r = self.r;

        let err = r.errorsrc().read();
        if err.anack() {
            return Err(Error::AddressNack);
        }
        if err.dnack() {
            return Err(Error::DataNack);
        }
        if err.overrun() {
            return Err(Error::Overrun);
        }
        Ok(())
    }

    fn check_rx(&self, len: usize) -> Result<(), Error> {
        let r = self.r;
        if r.dma().rx().amount().read().0 != len as u32 {
            Err(Error::Receive)
        } else {
            Ok(())
        }
    }

    fn check_tx(&self, len: usize) -> Result<(), Error> {
        let r = self.r;
        if r.dma().tx().amount().read().0 != len as u32 {
            Err(Error::Transmit)
        } else {
            Ok(())
        }
    }

    /// Wait for stop or error
    fn blocking_wait(&mut self) {
        let r = self.r;
        loop {
            if r.events_suspended().read() != 0 || r.events_stopped().read() != 0 {
                r.events_suspended().write_value(0);
                r.events_stopped().write_value(0);
                break;
            }
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
            }
        }
    }

    /// Wait for stop or error
    #[cfg(feature = "time")]
    fn blocking_wait_timeout(&mut self, timeout: Duration) -> Result<(), Error> {
        let r = self.r;
        let deadline = Instant::now() + timeout;
        loop {
            if r.events_suspended().read() != 0 || r.events_stopped().read() != 0 {
                r.events_stopped().write_value(0);
                break;
            }
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
            }
            if Instant::now() > deadline {
                r.tasks_stop().write_value(1);
                return Err(Error::Timeout);
            }
        }

        Ok(())
    }

    /// Wait for stop or error
    async fn async_wait(&mut self) -> Result<(), Error> {
        poll_fn(|cx| {
            let r = self.r;
            let s = self.state;

            s.end_waker.register(cx.waker());
            if r.events_suspended().read() != 0 || r.events_stopped().read() != 0 {
                r.events_stopped().write_value(0);

                return Poll::Ready(Ok(()));
            }

            // stop if an error occurred
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
                if let Err(e) = self.check_errorsrc() {
                    return Poll::Ready(Err(e));
                } else {
                    return Poll::Ready(Err(Error::Timeout));
                }
            }

            Poll::Pending
        })
        .await
    }

    fn setup_operations(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
        last_op: Option<&Operation<'_>>,
        inten: bool,
    ) -> Result<usize, Error> {
        let r = self.r;

        compiler_fence(SeqCst);

        r.address().write(|w| w.set_address(address));

        r.events_suspended().write_value(0);
        r.events_stopped().write_value(0);
        r.events_error().write_value(0);
        self.clear_errorsrc();

        if inten {
            r.intenset().write(|w| {
                w.set_suspended(true);
                w.set_stopped(true);
                w.set_error(true);
            });
        } else {
            r.intenclr().write(|w| {
                w.set_suspended(true);
                w.set_stopped(true);
                w.set_error(true);
            });
        }

        assert!(!operations.is_empty());
        match operations {
            [Operation::Read(_), Operation::Read(_), ..] => {
                panic!("Consecutive read operations are not supported!")
            }
            [Operation::Read(rd_buffer), Operation::Write(wr_buffer), rest @ ..] => {
                let stop = rest.is_empty();

                // Set up DMA buffers.
                unsafe {
                    self.set_tx_buffer(wr_buffer)?;
                    self.set_rx_buffer(rd_buffer)?;
                }

                r.shorts().write(|w| {
                    w.set_lastrx_dma_tx_start(true);
                    if stop {
                        w.set_lasttx_stop(true);
                    } else {
                        w.set_lasttx_suspend(true);
                    }
                });

                // Start read+write operation.
                r.tasks_dma().rx().start().write_value(1);
                if last_op.is_some() {
                    r.tasks_resume().write_value(1);
                }

                // TODO: Handle empty write buffer
                if rd_buffer.is_empty() {
                    // With a zero-length buffer, LASTRX doesn't fire (because there's no last byte!), so do the STARTTX ourselves.
                    r.tasks_dma().tx().start().write_value(1);
                }

                Ok(2)
            }
            [Operation::Read(buffer)] => {
                // Set up DMA buffers.
                unsafe {
                    self.set_rx_buffer(buffer)?;
                }

                r.shorts().write(|w| w.set_lastrx_stop(true));

                // Start read operation.
                r.tasks_dma().rx().start().write_value(1);
                if last_op.is_some() {
                    r.tasks_resume().write_value(1);
                }

                if buffer.is_empty() {
                    // With a zero-length buffer, LASTRX doesn't fire (because there's no last byte!), so do the STOP ourselves.
                    r.tasks_stop().write_value(1);
                }

                Ok(1)
            }
            [Operation::Write(wr_buffer), Operation::Read(rd_buffer)]
                if !wr_buffer.is_empty() && !rd_buffer.is_empty() =>
            {
                // Set up DMA buffers.
                unsafe {
                    self.set_tx_buffer(wr_buffer)?;
                    self.set_rx_buffer(rd_buffer)?;
                }

                // Start write+read operation.
                r.shorts().write(|w| {
                    w.set_lasttx_dma_rx_start(true);
                    w.set_lastrx_stop(true);
                });

                r.tasks_dma().tx().start().write_value(1);
                if last_op.is_some() {
                    r.tasks_resume().write_value(1);
                }

                Ok(2)
            }
            [Operation::Write(buffer), rest @ ..] => {
                let stop = rest.is_empty();

                // Set up DMA buffers.
                unsafe {
                    self.set_tx_buffer(buffer)?;
                }

                // Start write operation.
                r.shorts().write(|w| {
                    if stop {
                        w.set_lasttx_stop(true);
                    } else {
                        w.set_lasttx_suspend(true);
                    }
                });

                r.tasks_dma().tx().start().write_value(1);
                if last_op.is_some() {
                    r.tasks_resume().write_value(1);
                }

                if buffer.is_empty() {
                    // With a zero-length buffer, LASTTX doesn't fire (because there's no last byte!), so do the STOP/SUSPEND ourselves.
                    if stop {
                        r.tasks_stop().write_value(1);
                    } else {
                        r.tasks_suspend().write_value(1);
                    }
                }

                Ok(1)
            }
            [] => unreachable!(),
        }
    }

    fn check_operations(&mut self, operations: &[Operation<'_>]) -> Result<(), Error> {
        compiler_fence(SeqCst);
        self.check_errorsrc()?;

        assert!(operations.len() == 1 || operations.len() == 2);
        match operations {
            [Operation::Read(rd_buffer), Operation::Write(wr_buffer)]
            | [Operation::Write(wr_buffer), Operation::Read(rd_buffer)] => {
                self.check_rx(rd_buffer.len())?;
                self.check_tx(wr_buffer.len())?;
            }
            [Operation::Read(buffer)] => {
                self.check_rx(buffer.len())?;
            }
            [Operation::Write(buffer), ..] => {
                self.check_tx(buffer.len())?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    // ===========================================

    /// Execute the provided operations on the I2C bus.
    ///
    /// Each buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    ///
    /// Consecutive `Operation::Read`s are not supported due to hardware
    /// limitations.
    ///
    /// An `Operation::Write` following an `Operation::Read` must have a
    /// non-empty buffer.
    pub fn blocking_transaction(&mut self, address: u8, mut operations: &mut [Operation<'_>]) -> Result<(), Error> {
        let mut last_op = None;
        while !operations.is_empty() {
            let ops = self.setup_operations(address, operations, last_op, false)?;
            let (in_progress, rest) = operations.split_at_mut(ops);
            self.blocking_wait();
            self.check_operations(in_progress)?;
            last_op = in_progress.last();
            operations = rest;
        }
        Ok(())
    }

    /// Execute the provided operations on the I2C bus with timeout.
    ///
    /// See [Self::blocking_transaction].
    #[cfg(feature = "time")]
    pub fn blocking_transaction_timeout(
        &mut self,
        address: u8,
        mut operations: &mut [Operation<'_>],
        timeout: Duration,
    ) -> Result<(), Error> {
        let mut last_op = None;
        while !operations.is_empty() {
            let ops = self.setup_operations(address, operations, last_op, false)?;
            let (in_progress, rest) = operations.split_at_mut(ops);
            self.blocking_wait_timeout(timeout)?;
            self.check_operations(in_progress)?;
            last_op = in_progress.last();
            operations = rest;
        }
        Ok(())
    }

    /// Execute the provided operations on the I2C bus.
    ///
    /// Each buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    ///
    /// Consecutive `Operation::Read`s are not supported due to hardware
    /// limitations.
    ///
    /// An `Operation::Write` following an `Operation::Read` must have a
    /// non-empty buffer.
    pub async fn transaction(&mut self, address: u8, mut operations: &mut [Operation<'_>]) -> Result<(), Error> {
        let mut last_op = None;
        while !operations.is_empty() {
            let ops = self.setup_operations(address, operations, last_op, true)?;
            let (in_progress, rest) = operations.split_at_mut(ops);
            self.async_wait().await?;
            self.check_operations(in_progress)?;
            last_op = in_progress.last();
            operations = rest;
        }
        Ok(())
    }

    // ===========================================

    /// Write to an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn blocking_write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.blocking_transaction(address, &mut [Operation::Write(buffer)])
    }

    /// Read from an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn blocking_read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.blocking_transaction(address, &mut [Operation::Read(buffer)])
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    ///
    /// The buffers must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn blocking_write_read(&mut self, address: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Error> {
        self.blocking_transaction(address, &mut [Operation::Write(wr_buffer), Operation::Read(rd_buffer)])
    }

    // ===========================================

    /// Write to an I2C slave with timeout.
    ///
    /// See [Self::blocking_write].
    #[cfg(feature = "time")]
    pub fn blocking_write_timeout(&mut self, address: u8, buffer: &[u8], timeout: Duration) -> Result<(), Error> {
        self.blocking_transaction_timeout(address, &mut [Operation::Write(buffer)], timeout)
    }

    /// Read from an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    #[cfg(feature = "time")]
    pub fn blocking_read_timeout(&mut self, address: u8, buffer: &mut [u8], timeout: Duration) -> Result<(), Error> {
        self.blocking_transaction_timeout(address, &mut [Operation::Read(buffer)], timeout)
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    ///
    /// The buffers must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    #[cfg(feature = "time")]
    pub fn blocking_write_read_timeout(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
        timeout: Duration,
    ) -> Result<(), Error> {
        self.blocking_transaction_timeout(
            address,
            &mut [Operation::Write(wr_buffer), Operation::Read(rd_buffer)],
            timeout,
        )
    }

    // ===========================================

    /// Read from an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.transaction(address, &mut [Operation::Read(buffer)]).await
    }

    /// Write to an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub async fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.transaction(address, &mut [Operation::Write(buffer)]).await
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    ///
    /// The buffers must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub async fn write_read(&mut self, address: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Error> {
        self.transaction(address, &mut [Operation::Write(wr_buffer), Operation::Read(rd_buffer)])
            .await
    }
}

impl<'a> Drop for Twim<'a> {
    fn drop(&mut self) {
        trace!("twim drop");

        // TODO: check for abort

        // disable!
        let r = self.r;
        r.enable().write(|w| w.set_enable(vals::Enable::DISABLED));

        gpio::deconfigure_pin(r.psel().sda().read());
        gpio::deconfigure_pin(r.psel().scl().read());

        trace!("twim drop: done");
    }
}

pub(crate) struct State {
    end_waker: AtomicWaker,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            end_waker: AtomicWaker::new(),
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::twim::Twim;
    fn state() -> &'static State;
}

/// TWIM peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_twim {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::twim::SealedInstance for peripherals::$type {
            fn regs() -> pac::twim::Twim {
                pac::$pac_type
            }
            fn state() -> &'static crate::twim::State {
                static STATE: crate::twim::State = crate::twim::State::new();
                &STATE
            }
        }
        impl crate::twim::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

// ====================

mod eh02 {
    use super::*;

    impl<'a> embedded_hal_02::blocking::i2c::Write for Twim<'a> {
        type Error = Error;

        fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
            self.blocking_write(addr, bytes)
        }
    }

    impl<'a> embedded_hal_02::blocking::i2c::Read for Twim<'a> {
        type Error = Error;

        fn read(&mut self, addr: u8, bytes: &mut [u8]) -> Result<(), Error> {
            self.blocking_read(addr, bytes)
        }
    }

    impl<'a> embedded_hal_02::blocking::i2c::WriteRead for Twim<'a> {
        type Error = Error;

        fn write_read<'w>(&mut self, addr: u8, bytes: &'w [u8], buffer: &'w mut [u8]) -> Result<(), Error> {
            self.blocking_write_read(addr, bytes, buffer)
        }
    }
}

impl embedded_hal_1::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match *self {
            Self::TxBufferTooLong => embedded_hal_1::i2c::ErrorKind::Other,
            Self::RxBufferTooLong => embedded_hal_1::i2c::ErrorKind::Other,
            Self::Transmit => embedded_hal_1::i2c::ErrorKind::Other,
            Self::Receive => embedded_hal_1::i2c::ErrorKind::Other,
            Self::RAMBufferTooSmall => embedded_hal_1::i2c::ErrorKind::Other,
            Self::AddressNack => {
                embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Address)
            }
            Self::DataNack => {
                embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Data)
            }
            Self::Overrun => embedded_hal_1::i2c::ErrorKind::Overrun,
            Self::Timeout => embedded_hal_1::i2c::ErrorKind::Other,
        }
    }
}

impl<'d> embedded_hal_1::i2c::ErrorType for Twim<'d> {
    type Error = Error;
}

impl<'d> embedded_hal_1::i2c::I2c for Twim<'d> {
    fn transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Self::Error> {
        self.blocking_transaction(address, operations)
    }
}

impl<'d> embedded_hal_async::i2c::I2c for Twim<'d> {
    async fn transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Self::Error> {
        self.transaction(address, operations).await
    }
}

impl<'d> SetConfig for Twim<'d> {
    type Config = Config;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        let r = self.r;
        r.frequency().write(|w| w.set_frequency(config.frequency));

        Ok(())
    }
}
