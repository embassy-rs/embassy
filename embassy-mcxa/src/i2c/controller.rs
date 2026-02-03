//! LPI2C controller driver

use core::future::Future;
use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;

use super::{Async, Blocking, Info, Instance, Mode, SclPin, SdaPin};
use crate::clocks::periph_helpers::{Div4, Lpi2cClockSel, Lpi2cConfig};
use crate::clocks::{ClockError, PoweredClock, WakeGuard, enable_and_reset};
use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::lpi2c::vals::{Alf, Cmd, Dmf, Dozen, Epf, McrRrf, McrRtf, MsrFef, MsrSdf, Ndf, Pltf, Stf};

/// Errors exclusive to HW initialization
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Clock configuration error.
    ClockSetup(ClockError),
    /// Other internal errors or unexpected state.
    Other,
}

/// I/O Errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum IOError {
    /// FIFO Error
    FifoError,
    /// Reading for I2C failed.
    ReadFail,
    /// Writing to I2C failed.
    WriteFail,
    /// I2C address NAK condition.
    AddressNack,
    /// Bus level arbitration loss.
    ArbitrationLoss,
    /// Address out of range.
    AddressOutOfRange(u8),
    /// Invalid write buffer length.
    InvalidWriteBufferLength,
    /// Invalid read buffer length.
    InvalidReadBufferLength,
    /// Other internal errors or unexpected state.
    Other,
}

/// I2C interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        if T::info().regs().mier().read().0 != 0 {
            T::info().regs().mier().write(|w| {
                w.set_tdie(false);
                w.set_rdie(false);
                w.set_epie(false);
                w.set_sdie(false);
                w.set_ndie(false);
                w.set_alie(false);
                w.set_feie(false);
                w.set_pltie(false);
                w.set_dmie(false);
                w.set_stie(false);
            });

            T::PERF_INT_WAKE_INCR();
            T::info().wait_cell().wake();
        }
    }
}

/// Bus speed (nominal SCL, no clock stretching)
#[derive(Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Speed {
    #[default]
    /// 100 kbit/sec
    Standard,
    /// 400 kbit/sec
    Fast,
    /// 1 Mbit/sec
    FastPlus,
    /// 3.4 Mbit/sec
    UltraFast,
}

impl From<Speed> for u32 {
    fn from(val: Speed) -> Self {
        match val {
            Speed::Standard => 100_000,
            Speed::Fast => 400_000,
            Speed::FastPlus => 1_000_000,
            Speed::UltraFast => 3_400_000,
        }
    }
}

impl From<Speed> for (u8, u8, u8, u8) {
    fn from(value: Speed) -> (u8, u8, u8, u8) {
        match value {
            Speed::Standard => (0x3d, 0x37, 0x3b, 0x1d),
            Speed::Fast => (0x0e, 0x0c, 0x0d, 0x06),
            Speed::FastPlus => (0x04, 0x03, 0x03, 0x02),

            // UltraFast is "special". Leaving it unimplemented until
            // the driver and the clock API is further stabilized.
            Speed::UltraFast => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum SendStop {
    No,
    Yes,
}

/// I2C controller configuration
#[derive(Clone, Copy, Default)]
#[non_exhaustive]
pub struct Config {
    /// Bus speed
    pub speed: Speed,
}

/// I2C Controller Driver.
pub struct I2c<'d, M: Mode> {
    info: &'static Info,
    _scl: Peri<'d, AnyPin>,
    _sda: Peri<'d, AnyPin>,
    _phantom: PhantomData<M>,
    is_hs: bool,
    _wg: Option<WakeGuard>,
}

impl<'d> I2c<'d, Blocking> {
    /// Create a new blocking instance of the I2C Controller bus driver.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
    ) -> Result<Self, SetupError> {
        Self::new_inner(peri, scl, sda, config)
    }
}

impl<'d, M: Mode> I2c<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
    ) -> Result<Self, SetupError> {
        let (power, source, div) = Self::clock_config(config.speed);

        // Enable clocks
        let conf = Lpi2cConfig {
            power,
            source,
            div,
            instance: T::CLOCK_INSTANCE,
        };

        let parts = unsafe { enable_and_reset::<T>(&conf).map_err(SetupError::ClockSetup)? };

        scl.mux();
        sda.mux();

        let _scl = scl.into();
        let _sda = sda.into();

        let inst = Self {
            info: T::info(),
            _scl,
            _sda,
            _phantom: PhantomData,
            is_hs: config.speed == Speed::UltraFast,
            _wg: parts.wake_guard,
        };

        inst.set_configuration(&config);

        Ok(inst)
    }

    fn set_configuration(&self, config: &Config) {
        // Disable the controller.
        critical_section::with(|_| self.info.regs().mcr().modify(|w| w.set_men(false)));

        // Soft-reset the controller, read and write FIFOs.
        self.reset_fifos();
        critical_section::with(|_| {
            self.info.regs().mcr().modify(|w| w.set_rst(true));
            // According to Reference Manual section 40.7.1.4, "There
            // is no minimum delay required before clearing the
            // software reset", therefore we clear it immediately.
            self.info.regs().mcr().modify(|w| w.set_rst(false));

            self.info.regs().mcr().modify(|w| {
                w.set_dozen(Dozen::ENABLED);
                w.set_dbgen(false);
            });
        });

        let (clklo, clkhi, sethold, datavd) = config.speed.into();

        critical_section::with(|_| {
            self.info.regs().mccr0().modify(|w| {
                w.set_clklo(clklo);
                w.set_clkhi(clkhi);
                w.set_sethold(sethold);
                w.set_datavd(datavd);
            })
        });

        // Enable the controller.
        critical_section::with(|_| self.info.regs().mcr().modify(|w| w.set_men(true)));

        // Clear all flags
        self.info.regs().msr().write(|w| {
            w.set_epf(Epf::INT_YES);
            w.set_sdf(MsrSdf::INT_YES);
            w.set_ndf(Ndf::INT_YES);
            w.set_alf(Alf::INT_YES);
            w.set_fef(MsrFef::INT_YES);
            w.set_pltf(Pltf::INT_YES);
            w.set_dmf(Dmf::INT_YES);
            w.set_stf(Stf::INT_YES);
        });
    }

    // REVISIT: turn this into a function of the speed parameter
    fn clock_config(speed: Speed) -> (PoweredClock, Lpi2cClockSel, Div4) {
        match speed {
            Speed::Standard | Speed::Fast | Speed::FastPlus => (
                PoweredClock::NormalEnabledDeepSleepDisabled,
                Lpi2cClockSel::FroLfDiv,
                const { Div4::no_div() },
            ),
            Speed::UltraFast => (
                PoweredClock::NormalEnabledDeepSleepDisabled,
                Lpi2cClockSel::FroHfDiv,
                const { Div4::no_div() },
            ),
        }
    }

    /// Resets both TX and RX FIFOs dropping their contents.
    fn reset_fifos(&self) {
        critical_section::with(|_| {
            self.info.regs().mcr().modify(|w| {
                w.set_rtf(McrRtf::RESET);
                w.set_rrf(McrRrf::RESET);
            });
        });
    }

    /// Checks whether the TX FIFO is full
    fn is_tx_fifo_full(&self) -> bool {
        let txfifo_size = 1 << self.info.regs().param().read().mtxfifo();
        self.info.regs().mfsr().read().txcount() == txfifo_size
    }

    /// Checks whether the TX FIFO is empty
    fn is_tx_fifo_empty(&self) -> bool {
        self.info.regs().mfsr().read().txcount() == 0
    }

    /// Checks whether the RX FIFO is empty.
    fn is_rx_fifo_empty(&self) -> bool {
        self.info.regs().mfsr().read().rxcount() == 0
    }

    /// Reads and parses the controller status producing an
    /// appropriate `Result<(), Error>` variant.
    fn status(&self) -> Result<(), IOError> {
        let msr = self.info.regs().msr().read();
        self.info.regs().msr().write(|w| {
            w.set_epf(Epf::INT_YES);
            w.set_sdf(MsrSdf::INT_YES);
            w.set_ndf(Ndf::INT_YES);
            w.set_alf(Alf::INT_YES);
            w.set_fef(MsrFef::INT_YES);
            w.set_pltf(Pltf::INT_YES);
            w.set_dmf(Dmf::INT_YES);
            w.set_stf(Stf::INT_YES);
        });

        if msr.ndf() == Ndf::INT_YES {
            Err(IOError::AddressNack)
        } else if msr.alf() == Alf::INT_YES {
            Err(IOError::ArbitrationLoss)
        } else if msr.fef() == MsrFef::INT_YES {
            Err(IOError::FifoError)
        } else {
            Ok(())
        }
    }

    /// Inserts the given command into the outgoing FIFO.
    ///
    /// Caller must ensure there is space in the FIFO for the new
    /// command.
    fn send_cmd(&self, cmd: Cmd, data: u8) {
        #[cfg(feature = "defmt")]
        defmt::trace!(
            "Sending cmd '{}' ({}) with data '{:08x}' MSR: {:08x}",
            cmd,
            cmd as u8,
            data,
            self.info.regs().msr().read()
        );

        self.info.regs().mtdr().write(|w| {
            w.set_data(data);
            w.set_cmd(cmd);
        });
    }

    /// Prepares an appropriate Start condition on bus by issuing a
    /// `Start` command together with the device address and R/w bit.
    ///
    /// Blocks waiting for space in the FIFO to become available, then
    /// sends the command and blocks waiting for the FIFO to become
    /// empty ensuring the command was sent.
    fn start(&self, address: u8, read: bool) -> Result<(), IOError> {
        if address >= 0x80 {
            return Err(IOError::AddressOutOfRange(address));
        }

        // Wait until we have space in the TxFIFO
        while self.is_tx_fifo_full() {}

        let addr_rw = address << 1 | if read { 1 } else { 0 };
        self.send_cmd(if self.is_hs { Cmd::START_HS } else { Cmd::START }, addr_rw);

        // Wait for TxFIFO to be drained
        while !self.is_tx_fifo_empty() {}

        // Check controller status
        self.status()
    }

    /// Prepares a Stop condition on the bus.
    ///
    /// Analogous to `start`, this blocks waiting for space in the
    /// FIFO to become available, then sends the command and blocks
    /// waiting for the FIFO to become empty ensuring the command was
    /// sent.
    fn stop(&self) -> Result<(), IOError> {
        // Wait until we have space in the TxFIFO
        while self.is_tx_fifo_full() {}

        self.send_cmd(Cmd::STOP, 0);

        // Wait for TxFIFO to be drained
        while !self.is_tx_fifo_empty() {}

        self.status()
    }

    fn blocking_read_internal(&self, address: u8, read: &mut [u8], send_stop: SendStop) -> Result<(), IOError> {
        if read.is_empty() {
            return Err(IOError::InvalidReadBufferLength);
        }

        for chunk in read.chunks_mut(256) {
            self.start(address, true)?;

            // Wait until we have space in the TxFIFO
            while self.is_tx_fifo_full() {}

            self.send_cmd(Cmd::RECEIVE, (chunk.len() - 1) as u8);

            for byte in chunk.iter_mut() {
                // Wait until there's data in the RxFIFO
                while self.is_rx_fifo_empty() {}

                *byte = self.info.regs().mrdr().read().data();
            }
        }

        if send_stop == SendStop::Yes {
            self.stop()?;
        }

        Ok(())
    }

    fn blocking_write_internal(&self, address: u8, write: &[u8], send_stop: SendStop) -> Result<(), IOError> {
        self.start(address, false)?;

        // Usually, embassy HALs error out with an empty write,
        // however empty writes are useful for writing I2C scanning
        // logic through write probing. That is, we send a start with
        // R/w bit cleared, but instead of writing any data, just send
        // the stop onto the bus. This has the effect of checking if
        // the resulting address got an ACK but causing no
        // side-effects to the device on the other end.
        //
        // Because of this, we are not going to error out in case of
        // empty writes.
        #[cfg(feature = "defmt")]
        if write.is_empty() {
            defmt::trace!("Empty write, write probing?");
        }

        for byte in write {
            // Wait until we have space in the TxFIFO
            while self.is_tx_fifo_full() {}

            self.send_cmd(Cmd::TRANSMIT, *byte);
        }

        if send_stop == SendStop::Yes {
            self.stop()?;
        }

        Ok(())
    }

    // Public API: Blocking

    /// Read from address into buffer blocking caller until done.
    pub fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), IOError> {
        self.blocking_read_internal(address, read, SendStop::Yes)
    }

    /// Write to address from buffer blocking caller until done.
    pub fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<(), IOError> {
        self.blocking_write_internal(address, write, SendStop::Yes)
    }

    /// Write to address from bytes and read from address into buffer blocking caller until done.
    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), IOError> {
        self.blocking_write_internal(address, write, SendStop::No)?;
        self.blocking_read_internal(address, read, SendStop::Yes)
    }
}

impl<'d> I2c<'d, Async> {
    /// Create a new async instance of the I2C Controller bus driver.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_async<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, SetupError> {
        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        Self::new_inner(peri, scl, sda, config)
    }

    fn remediation(&self) {
        #[cfg(feature = "defmt")]
        defmt::trace!("Future dropped, issuing stop",);

        // if the FIFO is not empty, drop its contents.
        if !self.is_tx_fifo_empty() {
            self.reset_fifos();
        }

        // send a stop command
        let _ = self.stop();
    }

    fn enable_rx_ints(&self) {
        self.info.regs().mier().write(|w| {
            w.set_rdie(true);
            w.set_ndie(true);
            w.set_alie(true);
            w.set_feie(true);
            w.set_pltie(true);
        });
    }

    fn enable_tx_ints(&self) {
        self.info.regs().mier().write(|w| {
            w.set_tdie(true);
            w.set_ndie(true);
            w.set_alie(true);
            w.set_feie(true);
            w.set_pltie(true);
        });
    }

    async fn async_start(&self, address: u8, read: bool) -> Result<(), IOError> {
        if address >= 0x80 {
            return Err(IOError::AddressOutOfRange(address));
        }

        // send the start command
        let addr_rw = address << 1 | if read { 1 } else { 0 };
        self.send_cmd(if self.is_hs { Cmd::START_HS } else { Cmd::START }, addr_rw);

        self.info
            .wait_cell()
            .wait_for(|| {
                // enable interrupts
                self.enable_tx_ints();
                // if the command FIFO is empty, we're done sending start
                self.is_tx_fifo_empty()
            })
            .await
            .map_err(|_| IOError::Other)?;

        self.status()
    }

    async fn async_stop(&self) -> Result<(), IOError> {
        // send the stop command
        self.send_cmd(Cmd::STOP, 0);

        self.info
            .wait_cell()
            .wait_for(|| {
                // enable interrupts
                self.enable_tx_ints();
                // if the command FIFO is empty, we're done sending stop
                self.is_tx_fifo_empty()
            })
            .await
            .map_err(|_| IOError::Other)?;

        self.status()
    }

    async fn async_read_internal(&self, address: u8, read: &mut [u8], send_stop: SendStop) -> Result<(), IOError> {
        if read.is_empty() {
            return Err(IOError::InvalidReadBufferLength);
        }

        // perform corrective action if the future is dropped
        let on_drop = OnDrop::new(|| self.remediation());

        for chunk in read.chunks_mut(256) {
            self.async_start(address, true).await?;

            // send receive command
            self.send_cmd(Cmd::RECEIVE, (chunk.len() - 1) as u8);

            self.info
                .wait_cell()
                .wait_for(|| {
                    // enable interrupts
                    self.enable_tx_ints();
                    // if the command FIFO is empty, we're done sending start
                    self.is_tx_fifo_empty()
                })
                .await
                .map_err(|_| IOError::Other)?;

            for byte in chunk.iter_mut() {
                self.info
                    .wait_cell()
                    .wait_for(|| {
                        // enable interrupts
                        self.enable_rx_ints();
                        // if the rx FIFO is not empty, we need to read a byte
                        !self.is_rx_fifo_empty()
                    })
                    .await
                    .map_err(|_| IOError::ReadFail)?;

                *byte = self.info.regs().mrdr().read().data();
            }
        }

        if send_stop == SendStop::Yes {
            self.async_stop().await?;
        }

        // defuse it if the future is not dropped
        on_drop.defuse();

        Ok(())
    }

    async fn async_write_internal(&self, address: u8, write: &[u8], send_stop: SendStop) -> Result<(), IOError> {
        self.async_start(address, false).await?;

        // perform corrective action if the future is dropped
        let on_drop = OnDrop::new(|| self.remediation());

        // Usually, embassy HALs error out with an empty write,
        // however empty writes are useful for writing I2C scanning
        // logic through write probing. That is, we send a start with
        // R/w bit cleared, but instead of writing any data, just send
        // the stop onto the bus. This has the effect of checking if
        // the resulting address got an ACK but causing no
        // side-effects to the device on the other end.
        //
        // Because of this, we are not going to error out in case of
        // empty writes.
        #[cfg(feature = "defmt")]
        if write.is_empty() {
            defmt::trace!("Empty write, write probing?");
        }

        for byte in write {
            self.info
                .wait_cell()
                .wait_for(|| {
                    // enable interrupts
                    self.enable_tx_ints();
                    // initiate transmit
                    self.send_cmd(Cmd::TRANSMIT, *byte);
                    // if the tx FIFO is empty, we're done transmiting
                    self.is_tx_fifo_empty()
                })
                .await
                .map_err(|_| IOError::WriteFail)?;
        }

        if send_stop == SendStop::Yes {
            self.async_stop().await?;
        }

        // defuse it if the future is not dropped
        on_drop.defuse();

        Ok(())
    }

    // Public API: Async

    /// Read from address into buffer asynchronously.
    pub fn async_read<'a>(
        &mut self,
        address: u8,
        read: &'a mut [u8],
    ) -> impl Future<Output = Result<(), IOError>> + use<'_, 'a, 'd> {
        self.async_read_internal(address, read, SendStop::Yes)
    }

    /// Write to address from buffer asynchronously.
    pub fn async_write<'a>(
        &mut self,
        address: u8,
        write: &'a [u8],
    ) -> impl Future<Output = Result<(), IOError>> + use<'_, 'a, 'd> {
        self.async_write_internal(address, write, SendStop::Yes)
    }

    /// Write to address from bytes and read from address into buffer asynchronously.
    pub async fn async_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), IOError> {
        self.async_write_internal(address, write, SendStop::No).await?;
        self.async_read_internal(address, read, SendStop::Yes).await
    }
}

impl<'d, M: Mode> Drop for I2c<'d, M> {
    fn drop(&mut self) {
        self._scl.set_as_disabled();
        self._sda.set_as_disabled();
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Read for I2c<'d, M> {
    type Error = IOError;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, buffer)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Write for I2c<'d, M> {
    type Error = IOError;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, bytes)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::WriteRead for I2c<'d, M> {
    type Error = IOError;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(address, bytes, buffer)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Transactional for I2c<'d, M> {
    type Error = IOError;

    fn exec(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_02::blocking::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        if let Some((last, rest)) = operations.split_last_mut() {
            for op in rest {
                match op {
                    embedded_hal_02::blocking::i2c::Operation::Read(buf) => {
                        self.blocking_read_internal(address, buf, SendStop::No)?
                    }
                    embedded_hal_02::blocking::i2c::Operation::Write(buf) => {
                        self.blocking_write_internal(address, buf, SendStop::No)?
                    }
                }
            }

            match last {
                embedded_hal_02::blocking::i2c::Operation::Read(buf) => {
                    self.blocking_read_internal(address, buf, SendStop::Yes)
                }
                embedded_hal_02::blocking::i2c::Operation::Write(buf) => {
                    self.blocking_write_internal(address, buf, SendStop::Yes)
                }
            }
        } else {
            Ok(())
        }
    }
}

impl embedded_hal_1::i2c::Error for IOError {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match *self {
            Self::ArbitrationLoss => embedded_hal_1::i2c::ErrorKind::ArbitrationLoss,
            Self::AddressNack => {
                embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Address)
            }
            _ => embedded_hal_1::i2c::ErrorKind::Other,
        }
    }
}

impl<'d, M: Mode> embedded_hal_1::i2c::ErrorType for I2c<'d, M> {
    type Error = IOError;
}

impl<'d, M: Mode> embedded_hal_1::i2c::I2c for I2c<'d, M> {
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        if let Some((last, rest)) = operations.split_last_mut() {
            for op in rest {
                match op {
                    embedded_hal_1::i2c::Operation::Read(buf) => {
                        self.blocking_read_internal(address, buf, SendStop::No)?
                    }
                    embedded_hal_1::i2c::Operation::Write(buf) => {
                        self.blocking_write_internal(address, buf, SendStop::No)?
                    }
                }
            }

            match last {
                embedded_hal_1::i2c::Operation::Read(buf) => self.blocking_read_internal(address, buf, SendStop::Yes),
                embedded_hal_1::i2c::Operation::Write(buf) => self.blocking_write_internal(address, buf, SendStop::Yes),
            }
        } else {
            Ok(())
        }
    }
}

impl<'d> embedded_hal_async::i2c::I2c for I2c<'d, Async> {
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        if let Some((last, rest)) = operations.split_last_mut() {
            for op in rest {
                match op {
                    embedded_hal_async::i2c::Operation::Read(buf) => {
                        self.async_read_internal(address, buf, SendStop::No).await?
                    }
                    embedded_hal_async::i2c::Operation::Write(buf) => {
                        self.async_write_internal(address, buf, SendStop::No).await?
                    }
                }
            }

            match last {
                embedded_hal_async::i2c::Operation::Read(buf) => {
                    self.async_read_internal(address, buf, SendStop::Yes).await
                }
                embedded_hal_async::i2c::Operation::Write(buf) => {
                    self.async_write_internal(address, buf, SendStop::Yes).await
                }
            }
        } else {
            Ok(())
        }
    }
}

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for I2c<'d, M> {
    type Config = Config;
    type ConfigError = SetupError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), SetupError> {
        self.set_configuration(config);
        Ok(())
    }
}
