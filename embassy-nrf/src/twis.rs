//! I2C-compatible Two Wire Interface in slave mode (TWIM) driver.

#![macro_use]

use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
#[cfg(feature = "time")]
use embassy_time::{Duration, Instant};

use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::gpio::Pin as GpioPin;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::gpio::vals as gpiovals;
use crate::pac::twis::vals;
use crate::util::slice_in_ram_or;
use crate::{gpio, interrupt, pac};

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

        if r.events_read().read() != 0 || r.events_write().read() != 0 {
            s.waker.wake();
            r.intenclr().write(|w| {
                w.set_read(true);
                w.set_write(true);
            });
        }
        if r.events_stopped().read() != 0 {
            s.waker.wake();
            r.intenclr().write(|w| w.set_stopped(true));
        }
        if r.events_error().read() != 0 {
            s.waker.wake();
            r.intenclr().write(|w| w.set_error(true));
        }
    }
}

/// TWIS driver.
pub struct Twis<'d, T: Instance> {
    _p: Peri<'d, T>,
}

impl<'d, T: Instance> Twis<'d, T> {
    /// Create a new TWIS driver.
    pub fn new(
        twis: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        sda: Peri<'d, impl GpioPin>,
        scl: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        let r = T::regs();

        // Configure pins
        sda.conf().write(|w| {
            w.set_dir(gpiovals::Dir::INPUT);
            w.set_input(gpiovals::Input::CONNECT);
            w.set_drive(match config.sda_high_drive {
                true => gpiovals::Drive::H0D1,
                false => gpiovals::Drive::S0D1,
            });
            if config.sda_pullup {
                w.set_pull(gpiovals::Pull::PULLUP);
            }
        });
        scl.conf().write(|w| {
            w.set_dir(gpiovals::Dir::INPUT);
            w.set_input(gpiovals::Input::CONNECT);
            w.set_drive(match config.scl_high_drive {
                true => gpiovals::Drive::H0D1,
                false => gpiovals::Drive::S0D1,
            });
            if config.sda_pullup {
                w.set_pull(gpiovals::Pull::PULLUP);
            }
        });

        // Select pins.
        r.psel().sda().write_value(sda.psel_bits());
        r.psel().scl().write_value(scl.psel_bits());

        // Enable TWIS instance.
        r.enable().write(|w| w.set_enable(vals::Enable::ENABLED));

        // Disable all events interrupts
        r.intenclr().write(|w| w.0 = 0xFFFF_FFFF);

        // Set address
        r.address(0).write(|w| w.set_address(config.address0));
        r.config().write(|w| w.set_address0(true));
        if let Some(address1) = config.address1 {
            r.address(1).write(|w| w.set_address(address1));
            r.config().modify(|w| w.set_address1(true));
        }

        // Set over-read character
        r.orc().write(|w| w.set_orc(config.orc));

        // Generate suspend on read event
        r.shorts().write(|w| w.set_read_suspend(true));

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

        // We're giving the register a pointer to the stack. Since we're
        // waiting for the I2C transaction to end before this stack pointer
        // becomes invalid, there's nothing wrong here.
        r.txd().ptr().write_value(buffer.as_ptr() as u32);
        r.txd().maxcnt().write(|w|
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

        let r = T::regs();

        // We're giving the register a pointer to the stack. Since we're
        // waiting for the I2C transaction to end before this stack pointer
        // becomes invalid, there's nothing wrong here.
        r.rxd().ptr().write_value(buffer.as_mut_ptr() as u32);
        r.rxd().maxcnt().write(|w|
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
        let r = T::regs();
        r.errorsrc().write(|w| {
            w.set_overflow(true);
            w.set_overread(true);
            w.set_dnack(true);
        });
    }

    /// Returns matched address for latest command.
    pub fn address_match(&self) -> u8 {
        let r = T::regs();
        r.address(r.match_().read().0 as usize).read().address()
    }

    /// Returns the index of the address matched in the latest command.
    pub fn address_match_index(&self) -> usize {
        T::regs().match_().read().0 as _
    }

    /// Wait for read, write, stop or error
    fn blocking_listen_wait(&mut self) -> Result<Status, Error> {
        let r = T::regs();
        loop {
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
                while r.events_stopped().read() == 0 {}
                return Err(Error::Overflow);
            }
            if r.events_stopped().read() != 0 {
                r.events_stopped().write_value(0);
                return Err(Error::Bus);
            }
            if r.events_read().read() != 0 {
                r.events_read().write_value(0);
                return Ok(Status::Read);
            }
            if r.events_write().read() != 0 {
                r.events_write().write_value(0);
                return Ok(Status::Write);
            }
        }
    }

    /// Wait for stop, repeated start or error
    fn blocking_listen_wait_end(&mut self, status: Status) -> Result<Command, Error> {
        let r = T::regs();
        loop {
            // stop if an error occurred
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
                return Err(Error::Overflow);
            } else if r.events_stopped().read() != 0 {
                r.events_stopped().write_value(0);
                return match status {
                    Status::Read => Ok(Command::Read),
                    Status::Write => {
                        let n = r.rxd().amount().read().0 as usize;
                        Ok(Command::Write(n))
                    }
                };
            } else if r.events_read().read() != 0 {
                r.events_read().write_value(0);
                let n = r.rxd().amount().read().0 as usize;
                return Ok(Command::WriteRead(n));
            }
        }
    }

    /// Wait for stop or error
    fn blocking_wait(&mut self) -> Result<usize, Error> {
        let r = T::regs();
        loop {
            // stop if an error occurred
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
                let errorsrc = r.errorsrc().read();
                if errorsrc.overread() {
                    return Err(Error::OverRead);
                } else if errorsrc.dnack() {
                    return Err(Error::DataNack);
                } else {
                    return Err(Error::Bus);
                }
            } else if r.events_stopped().read() != 0 {
                r.events_stopped().write_value(0);
                let n = r.txd().amount().read().0 as usize;
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
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
                let errorsrc = r.errorsrc().read();
                if errorsrc.overread() {
                    return Err(Error::OverRead);
                } else if errorsrc.dnack() {
                    return Err(Error::DataNack);
                } else {
                    return Err(Error::Bus);
                }
            } else if r.events_stopped().read() != 0 {
                r.events_stopped().write_value(0);
                let n = r.txd().amount().read().0 as usize;
                return Ok(n);
            } else if Instant::now() > deadline {
                r.tasks_stop().write_value(1);
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
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
                while r.events_stopped().read() == 0 {}
                return Err(Error::Overflow);
            }
            if r.events_stopped().read() != 0 {
                r.events_stopped().write_value(0);
                return Err(Error::Bus);
            }
            if r.events_read().read() != 0 {
                r.events_read().write_value(0);
                return Ok(Status::Read);
            }
            if r.events_write().read() != 0 {
                r.events_write().write_value(0);
                return Ok(Status::Write);
            }
            if Instant::now() > deadline {
                r.tasks_stop().write_value(1);
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
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
                return Err(Error::Overflow);
            } else if r.events_stopped().read() != 0 {
                r.events_stopped().write_value(0);
                return match status {
                    Status::Read => Ok(Command::Read),
                    Status::Write => {
                        let n = r.rxd().amount().read().0 as usize;
                        Ok(Command::Write(n))
                    }
                };
            } else if r.events_read().read() != 0 {
                r.events_read().write_value(0);
                let n = r.rxd().amount().read().0 as usize;
                return Ok(Command::WriteRead(n));
            } else if Instant::now() > deadline {
                r.tasks_stop().write_value(1);
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
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
                let errorsrc = r.errorsrc().read();
                if errorsrc.overread() {
                    return Poll::Ready(Err(Error::OverRead));
                } else if errorsrc.dnack() {
                    return Poll::Ready(Err(Error::DataNack));
                } else {
                    return Poll::Ready(Err(Error::Bus));
                }
            } else if r.events_stopped().read() != 0 {
                r.events_stopped().write_value(0);
                let n = r.txd().amount().read().0 as usize;
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
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
                return Poll::Ready(Err(Error::Overflow));
            } else if r.events_read().read() != 0 {
                r.events_read().write_value(0);
                return Poll::Ready(Ok(Status::Read));
            } else if r.events_write().read() != 0 {
                r.events_write().write_value(0);
                return Poll::Ready(Ok(Status::Write));
            } else if r.events_stopped().read() != 0 {
                r.events_stopped().write_value(0);
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
            if r.events_error().read() != 0 {
                r.events_error().write_value(0);
                r.tasks_stop().write_value(1);
                return Poll::Ready(Err(Error::Overflow));
            } else if r.events_stopped().read() != 0 {
                r.events_stopped().write_value(0);
                return match status {
                    Status::Read => Poll::Ready(Ok(Command::Read)),
                    Status::Write => {
                        let n = r.rxd().amount().read().0 as usize;
                        Poll::Ready(Ok(Command::Write(n)))
                    }
                };
            } else if r.events_read().read() != 0 {
                r.events_read().write_value(0);
                let n = r.rxd().amount().read().0 as usize;
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
        r.events_stopped().write_value(0);
        r.events_error().write_value(0);
        self.clear_errorsrc();

        if inten {
            r.intenset().write(|w| {
                w.set_stopped(true);
                w.set_error(true);
            });
        } else {
            r.intenclr().write(|w| {
                w.set_stopped(true);
                w.set_error(true);
            });
        }

        // Start write operation.
        r.tasks_preparetx().write_value(1);
        r.tasks_resume().write_value(1);
        Ok(())
    }

    fn setup_respond(&mut self, wr_buffer: &[u8], inten: bool) -> Result<(), Error> {
        match self.setup_respond_from_ram(wr_buffer, inten) {
            Ok(_) => Ok(()),
            Err(Error::BufferNotInRAM) => {
                trace!("Copying TWIS tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..wr_buffer.len()];
                tx_ram_buf.copy_from_slice(wr_buffer);
                self.setup_respond_from_ram(tx_ram_buf, inten)
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
        r.events_read().write_value(0);
        r.events_write().write_value(0);
        r.events_stopped().write_value(0);
        r.events_error().write_value(0);
        self.clear_errorsrc();

        if inten {
            r.intenset().write(|w| {
                w.set_stopped(true);
                w.set_error(true);
                w.set_read(true);
                w.set_write(true);
            });
        } else {
            r.intenclr().write(|w| {
                w.set_stopped(true);
                w.set_error(true);
                w.set_read(true);
                w.set_write(true);
            });
        }

        // Start read operation.
        r.tasks_preparerx().write_value(1);

        Ok(())
    }

    fn setup_listen_end(&mut self, inten: bool) -> Result<(), Error> {
        let r = T::regs();
        compiler_fence(SeqCst);

        // Clear events
        r.events_read().write_value(0);
        r.events_write().write_value(0);
        r.events_stopped().write_value(0);
        r.events_error().write_value(0);
        self.clear_errorsrc();

        if inten {
            r.intenset().write(|w| {
                w.set_stopped(true);
                w.set_error(true);
                w.set_read(true);
            });
        } else {
            r.intenclr().write(|w| {
                w.set_stopped(true);
                w.set_error(true);
                w.set_read(true);
            });
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
        r.enable().write(|w| w.set_enable(vals::Enable::DISABLED));

        gpio::deconfigure_pin(r.psel().sda().read());
        gpio::deconfigure_pin(r.psel().scl().read());

        trace!("twis drop: done");
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
    fn regs() -> pac::twis::Twis;
    fn state() -> &'static State;
}

/// TWIS peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_twis {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::twis::SealedInstance for peripherals::$type {
            fn regs() -> pac::twis::Twis {
                pac::$pac_type
            }
            fn state() -> &'static crate::twis::State {
                static STATE: crate::twis::State = crate::twis::State::new();
                &STATE
            }
        }
        impl crate::twis::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
