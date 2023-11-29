//! I2C-compatible Two Wire Interface in master mode (TWIM) driver.

#![macro_use]

use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
#[cfg(feature = "time")]
use embassy_time::{Duration, Instant};

use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::gpio::Pin as GpioPin;
use crate::interrupt::typelevel::Interrupt;
use crate::util::{slice_in_ram, slice_in_ram_or};
use crate::{gpio, interrupt, pac, Peripheral};

/// TWI frequency
#[derive(Clone, Copy)]
pub enum Frequency {
    /// 100 kbps
    K100 = 26738688,
    /// 250 kbps
    K250 = 67108864,
    /// 400 kbps
    K400 = 104857600,
}

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
    /// The buffer is not in data RAM. It's most likely in flash, and nRF's DMA cannot access flash.
    BufferNotInRAM,
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

        if r.events_stopped.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.stopped().clear());
        }
        if r.events_error.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.error().clear());
        }
    }
}

/// TWI driver.
pub struct Twim<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Twim<'d, T> {
    /// Create a new TWI driver.
    pub fn new(
        twim: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        sda: impl Peripheral<P = impl GpioPin> + 'd,
        scl: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(twim, sda, scl);

        let r = T::regs();

        // Configure pins
        sda.conf().write(|w| {
            w.dir().input();
            w.input().connect();
            if config.sda_high_drive {
                w.drive().h0d1();
            } else {
                w.drive().s0d1();
            }
            if config.sda_pullup {
                w.pull().pullup();
            }
            w
        });
        scl.conf().write(|w| {
            w.dir().input();
            w.input().connect();
            if config.scl_high_drive {
                w.drive().h0d1();
            } else {
                w.drive().s0d1();
            }
            if config.scl_pullup {
                w.pull().pullup();
            }
            w
        });

        // Select pins.
        r.psel.sda.write(|w| unsafe { w.bits(sda.psel_bits()) });
        r.psel.scl.write(|w| unsafe { w.bits(scl.psel_bits()) });

        // Enable TWIM instance.
        r.enable.write(|w| w.enable().enabled());

        let mut twim = Self { _p: twim };

        // Apply runtime peripheral configuration
        Self::set_config(&mut twim, &config).unwrap();

        // Disable all events interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        twim
    }

    /// Set TX buffer, checking that it is in RAM and has suitable length.
    unsafe fn set_tx_buffer(&mut self, buffer: &[u8]) -> Result<(), Error> {
        slice_in_ram_or(buffer, Error::BufferNotInRAM)?;

        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        let r = T::regs();

        r.txd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the I2C transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            w.ptr().bits(buffer.as_ptr() as u32));
        r.txd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is 8 bits wide and accepts the full range of
            // values.
            w.maxcnt().bits(buffer.len() as _));

        Ok(())
    }

    /// Set RX buffer, checking that it has suitable length.
    unsafe fn set_rx_buffer(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // NOTE: RAM slice check is not necessary, as a mutable
        // slice can only be built from data located in RAM.

        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::RxBufferTooLong);
        }

        let r = T::regs();

        r.rxd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the I2C transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            w.ptr().bits(buffer.as_mut_ptr() as u32));
        r.rxd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to the type of maxcnt
            // is also fine.
            //
            // Note that that nrf52840 maxcnt is a wider
            // type than a u8, so we use a `_` cast rather than a `u8` cast.
            // The MAXCNT field is thus at least 8 bits wide and accepts the
            // full range of values that fit in a `u8`.
            w.maxcnt().bits(buffer.len() as _));

        Ok(())
    }

    fn clear_errorsrc(&mut self) {
        let r = T::regs();
        r.errorsrc
            .write(|w| w.anack().bit(true).dnack().bit(true).overrun().bit(true));
    }

    /// Get Error instance, if any occurred.
    fn check_errorsrc(&self) -> Result<(), Error> {
        let r = T::regs();

        let err = r.errorsrc.read();
        if err.anack().is_received() {
            return Err(Error::AddressNack);
        }
        if err.dnack().is_received() {
            return Err(Error::DataNack);
        }
        if err.overrun().is_received() {
            return Err(Error::Overrun);
        }
        Ok(())
    }

    fn check_rx(&self, len: usize) -> Result<(), Error> {
        let r = T::regs();
        if r.rxd.amount.read().bits() != len as u32 {
            Err(Error::Receive)
        } else {
            Ok(())
        }
    }

    fn check_tx(&self, len: usize) -> Result<(), Error> {
        let r = T::regs();
        if r.txd.amount.read().bits() != len as u32 {
            Err(Error::Transmit)
        } else {
            Ok(())
        }
    }

    /// Wait for stop or error
    fn blocking_wait(&mut self) {
        let r = T::regs();
        loop {
            if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                break;
            }
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
            }
        }
    }

    /// Wait for stop or error
    #[cfg(feature = "time")]
    fn blocking_wait_timeout(&mut self, timeout: Duration) -> Result<(), Error> {
        let r = T::regs();
        let deadline = Instant::now() + timeout;
        loop {
            if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                break;
            }
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
            }
            if Instant::now() > deadline {
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                return Err(Error::Timeout);
            }
        }

        Ok(())
    }

    /// Wait for stop or error
    fn async_wait(&mut self) -> impl Future<Output = ()> {
        poll_fn(move |cx| {
            let r = T::regs();
            let s = T::state();

            s.end_waker.register(cx.waker());
            if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();

                return Poll::Ready(());
            }

            // stop if an error occurred
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
            }

            Poll::Pending
        })
    }

    fn setup_write_from_ram(&mut self, address: u8, buffer: &[u8], inten: bool) -> Result<(), Error> {
        let r = T::regs();

        compiler_fence(SeqCst);

        r.address.write(|w| unsafe { w.address().bits(address) });

        // Set up the DMA write.
        unsafe { self.set_tx_buffer(buffer)? };

        // Clear events
        r.events_stopped.reset();
        r.events_error.reset();
        r.events_lasttx.reset();
        self.clear_errorsrc();

        if inten {
            r.intenset.write(|w| w.stopped().set().error().set());
        } else {
            r.intenclr.write(|w| w.stopped().clear().error().clear());
        }

        // Start write operation.
        r.shorts.write(|w| w.lasttx_stop().enabled());
        r.tasks_starttx.write(|w| unsafe { w.bits(1) });
        if buffer.len() == 0 {
            // With a zero-length buffer, LASTTX doesn't fire (because there's no last byte!), so do the STOP ourselves.
            r.tasks_stop.write(|w| unsafe { w.bits(1) });
        }
        Ok(())
    }

    fn setup_read(&mut self, address: u8, buffer: &mut [u8], inten: bool) -> Result<(), Error> {
        let r = T::regs();

        compiler_fence(SeqCst);

        r.address.write(|w| unsafe { w.address().bits(address) });

        // Set up the DMA read.
        unsafe { self.set_rx_buffer(buffer)? };

        // Clear events
        r.events_stopped.reset();
        r.events_error.reset();
        self.clear_errorsrc();

        if inten {
            r.intenset.write(|w| w.stopped().set().error().set());
        } else {
            r.intenclr.write(|w| w.stopped().clear().error().clear());
        }

        // Start read operation.
        r.shorts.write(|w| w.lastrx_stop().enabled());
        r.tasks_startrx.write(|w| unsafe { w.bits(1) });
        if buffer.len() == 0 {
            // With a zero-length buffer, LASTRX doesn't fire (because there's no last byte!), so do the STOP ourselves.
            r.tasks_stop.write(|w| unsafe { w.bits(1) });
        }
        Ok(())
    }

    fn setup_write_read_from_ram(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
        inten: bool,
    ) -> Result<(), Error> {
        let r = T::regs();

        compiler_fence(SeqCst);

        r.address.write(|w| unsafe { w.address().bits(address) });

        // Set up DMA buffers.
        unsafe {
            self.set_tx_buffer(wr_buffer)?;
            self.set_rx_buffer(rd_buffer)?;
        }

        // Clear events
        r.events_stopped.reset();
        r.events_error.reset();
        self.clear_errorsrc();

        if inten {
            r.intenset.write(|w| w.stopped().set().error().set());
        } else {
            r.intenclr.write(|w| w.stopped().clear().error().clear());
        }

        // Start write+read operation.
        r.shorts.write(|w| {
            w.lasttx_startrx().enabled();
            w.lastrx_stop().enabled();
            w
        });
        r.tasks_starttx.write(|w| unsafe { w.bits(1) });
        if wr_buffer.len() == 0 && rd_buffer.len() == 0 {
            // With a zero-length buffer, LASTRX/LASTTX doesn't fire (because there's no last byte!), so do the STOP ourselves.
            // TODO handle when only one of the buffers is zero length
            r.tasks_stop.write(|w| unsafe { w.bits(1) });
        }

        Ok(())
    }

    fn setup_write_read(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
        inten: bool,
    ) -> Result<(), Error> {
        match self.setup_write_read_from_ram(address, wr_buffer, rd_buffer, inten) {
            Ok(_) => Ok(()),
            Err(Error::BufferNotInRAM) => {
                trace!("Copying TWIM tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..wr_buffer.len()];
                tx_ram_buf.copy_from_slice(wr_buffer);
                self.setup_write_read_from_ram(address, &tx_ram_buf, rd_buffer, inten)
            }
            Err(error) => Err(error),
        }
    }

    fn setup_write(&mut self, address: u8, wr_buffer: &[u8], inten: bool) -> Result<(), Error> {
        match self.setup_write_from_ram(address, wr_buffer, inten) {
            Ok(_) => Ok(()),
            Err(Error::BufferNotInRAM) => {
                trace!("Copying TWIM tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..wr_buffer.len()];
                tx_ram_buf.copy_from_slice(wr_buffer);
                self.setup_write_from_ram(address, &tx_ram_buf, inten)
            }
            Err(error) => Err(error),
        }
    }

    /// Write to an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn blocking_write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.setup_write(address, buffer, false)?;
        self.blocking_wait();
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(buffer.len())?;
        Ok(())
    }

    /// Same as [`blocking_write`](Twim::blocking_write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_write_from_ram(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.setup_write_from_ram(address, buffer, false)?;
        self.blocking_wait();
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(buffer.len())?;
        Ok(())
    }

    /// Read from an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn blocking_read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.setup_read(address, buffer, false)?;
        self.blocking_wait();
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_rx(buffer.len())?;
        Ok(())
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    ///
    /// The buffers must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn blocking_write_read(&mut self, address: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Error> {
        self.setup_write_read(address, wr_buffer, rd_buffer, false)?;
        self.blocking_wait();
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(wr_buffer.len())?;
        self.check_rx(rd_buffer.len())?;
        Ok(())
    }

    /// Same as [`blocking_write_read`](Twim::blocking_write_read) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_write_read_from_ram(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.setup_write_read_from_ram(address, wr_buffer, rd_buffer, false)?;
        self.blocking_wait();
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(wr_buffer.len())?;
        self.check_rx(rd_buffer.len())?;
        Ok(())
    }

    // ===========================================

    /// Write to an I2C slave with timeout.
    ///
    /// See [`blocking_write`].
    #[cfg(feature = "time")]
    pub fn blocking_write_timeout(&mut self, address: u8, buffer: &[u8], timeout: Duration) -> Result<(), Error> {
        self.setup_write(address, buffer, false)?;
        self.blocking_wait_timeout(timeout)?;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(buffer.len())?;
        Ok(())
    }

    /// Same as [`blocking_write`](Twim::blocking_write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    #[cfg(feature = "time")]
    pub fn blocking_write_from_ram_timeout(
        &mut self,
        address: u8,
        buffer: &[u8],
        timeout: Duration,
    ) -> Result<(), Error> {
        self.setup_write_from_ram(address, buffer, false)?;
        self.blocking_wait_timeout(timeout)?;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(buffer.len())?;
        Ok(())
    }

    /// Read from an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    #[cfg(feature = "time")]
    pub fn blocking_read_timeout(&mut self, address: u8, buffer: &mut [u8], timeout: Duration) -> Result<(), Error> {
        self.setup_read(address, buffer, false)?;
        self.blocking_wait_timeout(timeout)?;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_rx(buffer.len())?;
        Ok(())
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
        self.setup_write_read(address, wr_buffer, rd_buffer, false)?;
        self.blocking_wait_timeout(timeout)?;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(wr_buffer.len())?;
        self.check_rx(rd_buffer.len())?;
        Ok(())
    }

    /// Same as [`blocking_write_read`](Twim::blocking_write_read) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    #[cfg(feature = "time")]
    pub fn blocking_write_read_from_ram_timeout(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
        timeout: Duration,
    ) -> Result<(), Error> {
        self.setup_write_read_from_ram(address, wr_buffer, rd_buffer, false)?;
        self.blocking_wait_timeout(timeout)?;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(wr_buffer.len())?;
        self.check_rx(rd_buffer.len())?;
        Ok(())
    }

    // ===========================================

    /// Read from an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.setup_read(address, buffer, true)?;
        self.async_wait().await;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_rx(buffer.len())?;
        Ok(())
    }

    /// Write to an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub async fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.setup_write(address, buffer, true)?;
        self.async_wait().await;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(buffer.len())?;
        Ok(())
    }

    /// Same as [`write`](Twim::write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn write_from_ram(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.setup_write_from_ram(address, buffer, true)?;
        self.async_wait().await;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(buffer.len())?;
        Ok(())
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    ///
    /// The buffers must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub async fn write_read(&mut self, address: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Error> {
        self.setup_write_read(address, wr_buffer, rd_buffer, true)?;
        self.async_wait().await;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(wr_buffer.len())?;
        self.check_rx(rd_buffer.len())?;
        Ok(())
    }

    /// Same as [`write_read`](Twim::write_read) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn write_read_from_ram(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.setup_write_read_from_ram(address, wr_buffer, rd_buffer, true)?;
        self.async_wait().await;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(wr_buffer.len())?;
        self.check_rx(rd_buffer.len())?;
        Ok(())
    }
}

impl<'a, T: Instance> Drop for Twim<'a, T> {
    fn drop(&mut self) {
        trace!("twim drop");

        // TODO: check for abort

        // disable!
        let r = T::regs();
        r.enable.write(|w| w.enable().disabled());

        gpio::deconfigure_pin(r.psel.sda.read().bits());
        gpio::deconfigure_pin(r.psel.scl.read().bits());

        trace!("twim drop: done");
    }
}

pub(crate) mod sealed {
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
        fn regs() -> &'static pac::twim0::RegisterBlock;
        fn state() -> &'static State;
    }
}

/// TWIM peripheral instance.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_twim {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::twim::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::twim0::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::twim::sealed::State {
                static STATE: crate::twim::sealed::State = crate::twim::sealed::State::new();
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

    impl<'a, T: Instance> embedded_hal_02::blocking::i2c::Write for Twim<'a, T> {
        type Error = Error;

        fn write<'w>(&mut self, addr: u8, bytes: &'w [u8]) -> Result<(), Error> {
            if slice_in_ram(bytes) {
                self.blocking_write(addr, bytes)
            } else {
                let buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..];
                for chunk in bytes.chunks(FORCE_COPY_BUFFER_SIZE) {
                    buf[..chunk.len()].copy_from_slice(chunk);
                    self.blocking_write(addr, &buf[..chunk.len()])?;
                }
                Ok(())
            }
        }
    }

    impl<'a, T: Instance> embedded_hal_02::blocking::i2c::Read for Twim<'a, T> {
        type Error = Error;

        fn read<'w>(&mut self, addr: u8, bytes: &'w mut [u8]) -> Result<(), Error> {
            self.blocking_read(addr, bytes)
        }
    }

    impl<'a, T: Instance> embedded_hal_02::blocking::i2c::WriteRead for Twim<'a, T> {
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
            Self::BufferNotInRAM => embedded_hal_1::i2c::ErrorKind::Other,
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

impl<'d, T: Instance> embedded_hal_1::i2c::ErrorType for Twim<'d, T> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_hal_1::i2c::I2c for Twim<'d, T> {
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, buffer)
    }

    fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, buffer)
    }

    fn write_read(&mut self, address: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(address, wr_buffer, rd_buffer)
    }

    fn transaction<'a>(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal_1::i2c::Operation<'a>],
    ) -> Result<(), Self::Error> {
        todo!();
    }
}

impl<'d, T: Instance> embedded_hal_async::i2c::I2c for Twim<'d, T> {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.read(address, read).await
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.write(address, write).await
    }
    async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.write_read(address, write, read).await
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let _ = address;
        let _ = operations;
        todo!()
    }
}

impl<'d, T: Instance> SetConfig for Twim<'d, T> {
    type Config = Config;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        let r = T::regs();
        r.frequency
            .write(|w| unsafe { w.frequency().bits(config.frequency as u32) });

        Ok(())
    }
}
