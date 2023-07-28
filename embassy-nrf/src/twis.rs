//! I2C-compatible Two Wire Interface in slave mode (TWIM) driver.

#![macro_use]

use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
#[cfg(feature = "time")]
use embassy_time::{Duration, Instant};

use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::gpio::Pin as GpioPin;
use crate::interrupt::typelevel::Interrupt;
use crate::util::slice_in_ram_or;
use crate::{gpio, interrupt, pac, Peripheral};

/// TWIS config.
#[non_exhaustive]
pub struct Config {
    /// First address
    pub address0: u8,

    /// Second address, optional.
    pub address1: Option<u8>,

    /// Overread character.
    ///
    /// If the master keeps clocking the bus after all the bytes in the TX buffer have
    /// already been transmitted, this byte will be constantly transmitted.
    pub orc: u8,

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
            address0: 0x55,
            address1: None,
            orc: 0x00,
            scl_high_drive: false,
            sda_pullup: false,
            sda_high_drive: false,
            scl_pullup: false,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum Status {
    Read,
    Write,
}

/// TWIS error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// TX buffer was too long.
    TxBufferTooLong,
    /// RX buffer was too long.
    RxBufferTooLong,
    /// Didn't receive an ACK bit after a data byte.
    DataNack,
    /// Bus error.
    Bus,
    /// The buffer is not in data RAM. It's most likely in flash, and nRF's DMA cannot access flash.
    BufferNotInRAM,
    /// Overflow
    Overflow,
    /// Overread
    OverRead,
    /// Timeout
    Timeout,
}

/// Received command
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    /// Read
    Read,
    /// Write+read
    WriteRead(usize),
    /// Write
    Write(usize),
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        let s = T::state();

        if r.events_read.read().bits() != 0 || r.events_write.read().bits() != 0 {
            s.waker.wake();
            r.intenclr.modify(|_r, w| w.read().clear().write().clear());
        }
        if r.events_stopped.read().bits() != 0 {
            s.waker.wake();
            r.intenclr.modify(|_r, w| w.stopped().clear());
        }
        if r.events_error.read().bits() != 0 {
            s.waker.wake();
            r.intenclr.modify(|_r, w| w.error().clear());
        }
    }
}

/// TWIS driver.
pub struct Twis<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Twis<'d, T> {
    /// Create a new TWIS driver.
    pub fn new(
        twis: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        sda: impl Peripheral<P = impl GpioPin> + 'd,
        scl: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(twis, sda, scl);

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

        // Enable TWIS instance.
        r.enable.write(|w| w.enable().enabled());

        // Disable all events interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        // Set address
        r.address[0].write(|w| unsafe { w.address().bits(config.address0) });
        r.config.write(|w| w.address0().enabled());
        if let Some(address1) = config.address1 {
            r.address[1].write(|w| unsafe { w.address().bits(address1) });
            r.config.modify(|_r, w| w.address1().enabled());
        }

        // Set over-read character
        r.orc.write(|w| unsafe { w.orc().bits(config.orc) });

        // Generate suspend on read event
        r.shorts.write(|w| w.read_suspend().enabled());

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self { _p: twis }
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
            .write(|w| w.overflow().bit(true).overread().bit(true).dnack().bit(true));
    }

    /// Returns matched address for latest command.
    pub fn address_match(&self) -> u8 {
        let r = T::regs();
        r.address[r.match_.read().bits() as usize].read().address().bits()
    }

    /// Returns the index of the address matched in the latest command.
    pub fn address_match_index(&self) -> usize {
        T::regs().match_.read().bits() as _
    }

    /// Wait for read, write, stop or error
    fn blocking_listen_wait(&mut self) -> Result<Status, Error> {
        let r = T::regs();
        loop {
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                while r.events_stopped.read().bits() == 0 {}
                return Err(Error::Overflow);
            }
            if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                return Err(Error::Bus);
            }
            if r.events_read.read().bits() != 0 {
                r.events_read.reset();
                return Ok(Status::Read);
            }
            if r.events_write.read().bits() != 0 {
                r.events_write.reset();
                return Ok(Status::Write);
            }
        }
    }

    /// Wait for stop, repeated start or error
    fn blocking_listen_wait_end(&mut self, status: Status) -> Result<Command, Error> {
        let r = T::regs();
        loop {
            // stop if an error occurred
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                return Err(Error::Overflow);
            } else if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                return match status {
                    Status::Read => Ok(Command::Read),
                    Status::Write => {
                        let n = r.rxd.amount.read().bits() as usize;
                        Ok(Command::Write(n))
                    }
                };
            } else if r.events_read.read().bits() != 0 {
                r.events_read.reset();
                let n = r.rxd.amount.read().bits() as usize;
                return Ok(Command::WriteRead(n));
            }
        }
    }

    /// Wait for stop or error
    fn blocking_wait(&mut self) -> Result<usize, Error> {
        let r = T::regs();
        loop {
            // stop if an error occurred
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                let errorsrc = r.errorsrc.read();
                if errorsrc.overread().is_detected() {
                    return Err(Error::OverRead);
                } else if errorsrc.dnack().is_received() {
                    return Err(Error::DataNack);
                } else {
                    return Err(Error::Bus);
                }
            } else if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                let n = r.txd.amount.read().bits() as usize;
                return Ok(n);
            }
        }
    }

    /// Wait for stop or error with timeout
    #[cfg(feature = "time")]
    fn blocking_wait_timeout(&mut self, timeout: Duration) -> Result<usize, Error> {
        let r = T::regs();
        let deadline = Instant::now() + timeout;
        loop {
            // stop if an error occurred
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                let errorsrc = r.errorsrc.read();
                if errorsrc.overread().is_detected() {
                    return Err(Error::OverRead);
                } else if errorsrc.dnack().is_received() {
                    return Err(Error::DataNack);
                } else {
                    return Err(Error::Bus);
                }
            } else if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                let n = r.txd.amount.read().bits() as usize;
                return Ok(n);
            } else if Instant::now() > deadline {
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                return Err(Error::Timeout);
            }
        }
    }

    /// Wait for read, write, stop or error with timeout
    #[cfg(feature = "time")]
    fn blocking_listen_wait_timeout(&mut self, timeout: Duration) -> Result<Status, Error> {
        let r = T::regs();
        let deadline = Instant::now() + timeout;
        loop {
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                while r.events_stopped.read().bits() == 0 {}
                return Err(Error::Overflow);
            }
            if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                return Err(Error::Bus);
            }
            if r.events_read.read().bits() != 0 {
                r.events_read.reset();
                return Ok(Status::Read);
            }
            if r.events_write.read().bits() != 0 {
                r.events_write.reset();
                return Ok(Status::Write);
            }
            if Instant::now() > deadline {
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                return Err(Error::Timeout);
            }
        }
    }

    /// Wait for stop, repeated start or error with timeout
    #[cfg(feature = "time")]
    fn blocking_listen_wait_end_timeout(&mut self, status: Status, timeout: Duration) -> Result<Command, Error> {
        let r = T::regs();
        let deadline = Instant::now() + timeout;
        loop {
            // stop if an error occurred
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                return Err(Error::Overflow);
            } else if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                return match status {
                    Status::Read => Ok(Command::Read),
                    Status::Write => {
                        let n = r.rxd.amount.read().bits() as usize;
                        Ok(Command::Write(n))
                    }
                };
            } else if r.events_read.read().bits() != 0 {
                r.events_read.reset();
                let n = r.rxd.amount.read().bits() as usize;
                return Ok(Command::WriteRead(n));
            } else if Instant::now() > deadline {
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                return Err(Error::Timeout);
            }
        }
    }

    /// Wait for stop or error
    fn async_wait(&mut self) -> impl Future<Output = Result<usize, Error>> {
        poll_fn(move |cx| {
            let r = T::regs();
            let s = T::state();

            s.waker.register(cx.waker());

            // stop if an error occurred
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                let errorsrc = r.errorsrc.read();
                if errorsrc.overread().is_detected() {
                    return Poll::Ready(Err(Error::OverRead));
                } else if errorsrc.dnack().is_received() {
                    return Poll::Ready(Err(Error::DataNack));
                } else {
                    return Poll::Ready(Err(Error::Bus));
                }
            } else if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                let n = r.txd.amount.read().bits() as usize;
                return Poll::Ready(Ok(n));
            }

            Poll::Pending
        })
    }

    /// Wait for read or write
    fn async_listen_wait(&mut self) -> impl Future<Output = Result<Status, Error>> {
        poll_fn(move |cx| {
            let r = T::regs();
            let s = T::state();

            s.waker.register(cx.waker());

            // stop if an error occurred
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                return Poll::Ready(Err(Error::Overflow));
            } else if r.events_read.read().bits() != 0 {
                r.events_read.reset();
                return Poll::Ready(Ok(Status::Read));
            } else if r.events_write.read().bits() != 0 {
                r.events_write.reset();
                return Poll::Ready(Ok(Status::Write));
            } else if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                return Poll::Ready(Err(Error::Bus));
            }
            Poll::Pending
        })
    }

    /// Wait for stop, repeated start or error
    fn async_listen_wait_end(&mut self, status: Status) -> impl Future<Output = Result<Command, Error>> {
        poll_fn(move |cx| {
            let r = T::regs();
            let s = T::state();

            s.waker.register(cx.waker());

            // stop if an error occurred
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
                return Poll::Ready(Err(Error::Overflow));
            } else if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                return match status {
                    Status::Read => Poll::Ready(Ok(Command::Read)),
                    Status::Write => {
                        let n = r.rxd.amount.read().bits() as usize;
                        Poll::Ready(Ok(Command::Write(n)))
                    }
                };
            } else if r.events_read.read().bits() != 0 {
                r.events_read.reset();
                let n = r.rxd.amount.read().bits() as usize;
                return Poll::Ready(Ok(Command::WriteRead(n)));
            }
            Poll::Pending
        })
    }

    fn setup_respond_from_ram(&mut self, buffer: &[u8], inten: bool) -> Result<(), Error> {
        let r = T::regs();

        compiler_fence(SeqCst);

        // Set up the DMA write.
        unsafe { self.set_tx_buffer(buffer)? };

        // Clear events
        r.events_stopped.reset();
        r.events_error.reset();
        self.clear_errorsrc();

        if inten {
            r.intenset.write(|w| w.stopped().set().error().set());
        } else {
            r.intenclr.write(|w| w.stopped().clear().error().clear());
        }

        // Start write operation.
        r.tasks_preparetx.write(|w| unsafe { w.bits(1) });
        r.tasks_resume.write(|w| unsafe { w.bits(1) });
        Ok(())
    }

    fn setup_respond(&mut self, wr_buffer: &[u8], inten: bool) -> Result<(), Error> {
        match self.setup_respond_from_ram(wr_buffer, inten) {
            Ok(_) => Ok(()),
            Err(Error::BufferNotInRAM) => {
                trace!("Copying TWIS tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..wr_buffer.len()];
                tx_ram_buf.copy_from_slice(wr_buffer);
                self.setup_respond_from_ram(&tx_ram_buf, inten)
            }
            Err(error) => Err(error),
        }
    }

    fn setup_listen(&mut self, buffer: &mut [u8], inten: bool) -> Result<(), Error> {
        let r = T::regs();
        compiler_fence(SeqCst);

        // Set up the DMA read.
        unsafe { self.set_rx_buffer(buffer)? };

        // Clear events
        r.events_read.reset();
        r.events_write.reset();
        r.events_stopped.reset();
        r.events_error.reset();
        self.clear_errorsrc();

        if inten {
            r.intenset
                .write(|w| w.stopped().set().error().set().read().set().write().set());
        } else {
            r.intenclr
                .write(|w| w.stopped().clear().error().clear().read().clear().write().clear());
        }

        // Start read operation.
        r.tasks_preparerx.write(|w| unsafe { w.bits(1) });

        Ok(())
    }

    fn setup_listen_end(&mut self, inten: bool) -> Result<(), Error> {
        let r = T::regs();
        compiler_fence(SeqCst);

        // Clear events
        r.events_read.reset();
        r.events_write.reset();
        r.events_stopped.reset();
        r.events_error.reset();
        self.clear_errorsrc();

        if inten {
            r.intenset.write(|w| w.stopped().set().error().set().read().set());
        } else {
            r.intenclr.write(|w| w.stopped().clear().error().clear().read().clear());
        }

        Ok(())
    }

    /// Wait for commands from an I2C master.
    /// `buffer` is provided in case master does a 'write' and is unused for 'read'.
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    /// To know which one of the addresses were matched, call `address_match` or `address_match_index`
    pub fn blocking_listen(&mut self, buffer: &mut [u8]) -> Result<Command, Error> {
        self.setup_listen(buffer, false)?;
        let status = self.blocking_listen_wait()?;
        if status == Status::Write {
            self.setup_listen_end(false)?;
            let command = self.blocking_listen_wait_end(status)?;
            return Ok(command);
        }
        Ok(Command::Read)
    }

    /// Respond to an I2C master READ command.
    /// Returns the number of bytes written.
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn blocking_respond_to_read(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        self.setup_respond(buffer, false)?;
        self.blocking_wait()
    }

    /// Same as [`blocking_respond_to_read`](Twis::blocking_respond_to_read) but will fail instead of copying data into RAM.
    /// Consult the module level documentation to learn more.
    pub fn blocking_respond_to_read_from_ram(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        self.setup_respond_from_ram(buffer, false)?;
        self.blocking_wait()
    }

    // ===========================================

    /// Wait for commands from an I2C master, with timeout.
    /// `buffer` is provided in case master does a 'write' and is unused for 'read'.
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    /// To know which one of the addresses were matched, call `address_match` or `address_match_index`
    #[cfg(feature = "time")]
    pub fn blocking_listen_timeout(&mut self, buffer: &mut [u8], timeout: Duration) -> Result<Command, Error> {
        self.setup_listen(buffer, false)?;
        let status = self.blocking_listen_wait_timeout(timeout)?;
        if status == Status::Write {
            self.setup_listen_end(false)?;
            let command = self.blocking_listen_wait_end_timeout(status, timeout)?;
            return Ok(command);
        }
        Ok(Command::Read)
    }

    /// Respond to an I2C master READ command with timeout.
    /// Returns the number of bytes written.
    /// See [`blocking_respond_to_read`].
    #[cfg(feature = "time")]
    pub fn blocking_respond_to_read_timeout(&mut self, buffer: &[u8], timeout: Duration) -> Result<usize, Error> {
        self.setup_respond(buffer, false)?;
        self.blocking_wait_timeout(timeout)
    }

    /// Same as [`blocking_respond_to_read_timeout`](Twis::blocking_respond_to_read_timeout) but will fail instead of copying data into RAM.
    /// Consult the module level documentation to learn more.
    #[cfg(feature = "time")]
    pub fn blocking_respond_to_read_from_ram_timeout(
        &mut self,
        buffer: &[u8],
        timeout: Duration,
    ) -> Result<usize, Error> {
        self.setup_respond_from_ram(buffer, false)?;
        self.blocking_wait_timeout(timeout)
    }

    // ===========================================

    /// Wait asynchronously for commands from an I2C master.
    /// `buffer` is provided in case master does a 'write' and is unused for 'read'.
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    /// To know which one of the addresses were matched, call `address_match` or `address_match_index`
    pub async fn listen(&mut self, buffer: &mut [u8]) -> Result<Command, Error> {
        self.setup_listen(buffer, true)?;
        let status = self.async_listen_wait().await?;
        if status == Status::Write {
            self.setup_listen_end(true)?;
            let command = self.async_listen_wait_end(status).await?;
            return Ok(command);
        }
        Ok(Command::Read)
    }

    /// Respond to an I2C master READ command, asynchronously.
    /// Returns the number of bytes written.
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub async fn respond_to_read(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        self.setup_respond(buffer, true)?;
        self.async_wait().await
    }

    /// Same as [`respond_to_read`](Twis::respond_to_read) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn respond_to_read_from_ram(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        self.setup_respond_from_ram(buffer, true)?;
        self.async_wait().await
    }
}

impl<'a, T: Instance> Drop for Twis<'a, T> {
    fn drop(&mut self) {
        trace!("twis drop");

        // TODO: check for abort

        // disable!
        let r = T::regs();
        r.enable.write(|w| w.enable().disabled());

        gpio::deconfigure_pin(r.psel.sda.read().bits());
        gpio::deconfigure_pin(r.psel.scl.read().bits());

        trace!("twis drop: done");
    }
}

pub(crate) mod sealed {
    use super::*;

    pub struct State {
        pub waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static pac::twis0::RegisterBlock;
        fn state() -> &'static State;
    }
}

/// TWIS peripheral instance.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_twis {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::twis::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::twis0::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::twis::sealed::State {
                static STATE: crate::twis::sealed::State = crate::twis::sealed::State::new();
                &STATE
            }
        }
        impl crate::twis::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
