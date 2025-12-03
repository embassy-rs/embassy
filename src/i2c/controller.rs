//! LPI2C controller driver

use core::future::Future;
use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use mcxa_pac::lpi2c0::mtdr::Cmd;

use super::{Async, Blocking, Error, Instance, InterruptHandler, Mode, Result, SclPin, SdaPin};
use crate::clocks::periph_helpers::{Div4, Lpi2cClockSel, Lpi2cConfig};
use crate::clocks::{enable_and_reset, PoweredClock};
use crate::interrupt::typelevel::Interrupt;
use crate::AnyPin;

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
pub struct I2c<'d, T: Instance, M: Mode> {
    _peri: Peri<'d, T>,
    _scl: Peri<'d, AnyPin>,
    _sda: Peri<'d, AnyPin>,
    _phantom: PhantomData<M>,
    is_hs: bool,
}

impl<'d, T: Instance> I2c<'d, T, Blocking> {
    /// Create a new blocking instance of the I2C Controller bus driver.
    pub fn new_blocking(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
    ) -> Result<Self> {
        Self::new_inner(peri, scl, sda, config)
    }
}

impl<'d, T: Instance, M: Mode> I2c<'d, T, M> {
    fn new_inner(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
    ) -> Result<Self> {
        let (power, source, div) = Self::clock_config(config.speed);

        // Enable clocks
        let conf = Lpi2cConfig {
            power,
            source,
            div,
            instance: T::CLOCK_INSTANCE,
        };

        _ = unsafe { enable_and_reset::<T>(&conf).map_err(Error::ClockSetup)? };

        scl.mux();
        sda.mux();

        let _scl = scl.into();
        let _sda = sda.into();

        Self::set_config(&config)?;

        Ok(Self {
            _peri,
            _scl,
            _sda,
            _phantom: PhantomData,
            is_hs: config.speed == Speed::UltraFast,
        })
    }

    fn set_config(config: &Config) -> Result<()> {
        // Disable the controller.
        critical_section::with(|_| T::regs().mcr().modify(|_, w| w.men().disabled()));

        // Soft-reset the controller, read and write FIFOs.
        critical_section::with(|_| {
            T::regs()
                .mcr()
                .modify(|_, w| w.rst().reset().rtf().reset().rrf().reset());
            // According to Reference Manual section 40.7.1.4, "There
            // is no minimum delay required before clearing the
            // software reset", therefore we clear it immediately.
            T::regs().mcr().modify(|_, w| w.rst().not_reset());

            T::regs().mcr().modify(|_, w| w.dozen().clear_bit().dbgen().clear_bit());
        });

        let (clklo, clkhi, sethold, datavd) = config.speed.into();

        critical_section::with(|_| {
            T::regs().mccr0().modify(|_, w| unsafe {
                w.clklo()
                    .bits(clklo)
                    .clkhi()
                    .bits(clkhi)
                    .sethold()
                    .bits(sethold)
                    .datavd()
                    .bits(datavd)
            })
        });

        // Enable the controller.
        critical_section::with(|_| T::regs().mcr().modify(|_, w| w.men().enabled()));

        // Clear all flags
        T::regs().msr().write(|w| {
            w.epf()
                .clear_bit_by_one()
                .sdf()
                .clear_bit_by_one()
                .ndf()
                .clear_bit_by_one()
                .alf()
                .clear_bit_by_one()
                .fef()
                .clear_bit_by_one()
                .pltf()
                .clear_bit_by_one()
                .dmf()
                .clear_bit_by_one()
                .stf()
                .clear_bit_by_one()
        });

        Ok(())
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

    fn is_tx_fifo_full(&mut self) -> bool {
        let txfifo_size = 1 << T::regs().param().read().mtxfifo().bits();
        T::regs().mfsr().read().txcount().bits() == txfifo_size
    }

    fn is_tx_fifo_empty(&mut self) -> bool {
        T::regs().mfsr().read().txcount() == 0
    }

    fn is_rx_fifo_empty(&mut self) -> bool {
        T::regs().mfsr().read().rxcount() == 0
    }

    fn status(&mut self) -> Result<()> {
        let msr = T::regs().msr().read();
        T::regs().msr().write(|w| {
            w.epf()
                .clear_bit_by_one()
                .sdf()
                .clear_bit_by_one()
                .ndf()
                .clear_bit_by_one()
                .alf()
                .clear_bit_by_one()
                .fef()
                .clear_bit_by_one()
                .fef()
                .clear_bit_by_one()
                .pltf()
                .clear_bit_by_one()
                .dmf()
                .clear_bit_by_one()
                .stf()
                .clear_bit_by_one()
        });

        if msr.ndf().bit_is_set() {
            Err(Error::AddressNack)
        } else if msr.alf().bit_is_set() {
            Err(Error::ArbitrationLoss)
        } else if msr.fef().bit_is_set() {
            Err(Error::FifoError)
        } else {
            Ok(())
        }
    }

    fn send_cmd(&mut self, cmd: Cmd, data: u8) {
        #[cfg(feature = "defmt")]
        defmt::trace!(
            "Sending cmd '{}' ({}) with data '{:08x}' MSR: {:08x}",
            cmd,
            cmd as u8,
            data,
            T::regs().msr().read().bits()
        );

        T::regs()
            .mtdr()
            .write(|w| unsafe { w.data().bits(data) }.cmd().variant(cmd));
    }

    fn start(&mut self, address: u8, read: bool) -> Result<()> {
        if address >= 0x80 {
            return Err(Error::AddressOutOfRange(address));
        }

        // Wait until we have space in the TxFIFO
        while self.is_tx_fifo_full() {}

        let addr_rw = address << 1 | if read { 1 } else { 0 };
        self.send_cmd(if self.is_hs { Cmd::StartHs } else { Cmd::Start }, addr_rw);

        // Wait for TxFIFO to be drained
        while !self.is_tx_fifo_empty() {}

        // Check controller status
        self.status()
    }

    fn stop(&mut self) -> Result<()> {
        // Wait until we have space in the TxFIFO
        while self.is_tx_fifo_full() {}

        self.send_cmd(Cmd::Stop, 0);

        // Wait for TxFIFO to be drained
        while !self.is_tx_fifo_empty() {}

        self.status()
    }

    fn blocking_read_internal(&mut self, address: u8, read: &mut [u8], send_stop: SendStop) -> Result<()> {
        if read.is_empty() {
            return Err(Error::InvalidReadBufferLength);
        }

        for chunk in read.chunks_mut(256) {
            self.start(address, true)?;

            // Wait until we have space in the TxFIFO
            while self.is_tx_fifo_full() {}

            self.send_cmd(Cmd::Receive, (chunk.len() - 1) as u8);

            for byte in chunk.iter_mut() {
                // Wait until there's data in the RxFIFO
                while self.is_rx_fifo_empty() {}

                *byte = T::regs().mrdr().read().data().bits();
            }
        }

        if send_stop == SendStop::Yes {
            self.stop()?;
        }

        Ok(())
    }

    fn blocking_write_internal(&mut self, address: u8, write: &[u8], send_stop: SendStop) -> Result<()> {
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

            self.send_cmd(Cmd::Transmit, *byte);
        }

        if send_stop == SendStop::Yes {
            self.stop()?;
        }

        Ok(())
    }

    // Public API: Blocking

    /// Read from address into buffer blocking caller until done.
    pub fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<()> {
        self.blocking_read_internal(address, read, SendStop::Yes)
        // Automatic Stop
    }

    /// Write to address from buffer blocking caller until done.
    pub fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<()> {
        self.blocking_write_internal(address, write, SendStop::Yes)
    }

    /// Write to address from bytes and read from address into buffer blocking caller until done.
    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<()> {
        self.blocking_write_internal(address, write, SendStop::No)?;
        self.blocking_read_internal(address, read, SendStop::Yes)
    }
}

impl<'d, T: Instance> I2c<'d, T, Async> {
    /// Create a new async instance of the I2C Controller bus driver.
    pub fn new_async(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self> {
        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        Self::new_inner(peri, scl, sda, config)
    }

    fn enable_rx_ints(&mut self) {
        T::regs().mier().write(|w| {
            w.rdie()
                .enabled()
                .ndie()
                .enabled()
                .alie()
                .enabled()
                .feie()
                .enabled()
                .pltie()
                .enabled()
        });
    }

    fn enable_tx_ints(&mut self) {
        T::regs().mier().write(|w| {
            w.tdie()
                .enabled()
                .ndie()
                .enabled()
                .alie()
                .enabled()
                .feie()
                .enabled()
                .pltie()
                .enabled()
        });
    }

    async fn async_start(&mut self, address: u8, read: bool) -> Result<()> {
        if address >= 0x80 {
            return Err(Error::AddressOutOfRange(address));
        }

        // send the start command
        let addr_rw = address << 1 | if read { 1 } else { 0 };
        self.send_cmd(if self.is_hs { Cmd::StartHs } else { Cmd::Start }, addr_rw);

        T::wait_cell()
            .wait_for(|| {
                // enable interrupts
                self.enable_tx_ints();
                // if the command FIFO is empty, we're done sending start
                self.is_tx_fifo_empty()
            })
            .await
            .map_err(|_| Error::Other)?;

        self.status()
    }

    async fn async_stop(&mut self) -> Result<()> {
        // send the stop command
        self.send_cmd(Cmd::Stop, 0);

        T::wait_cell()
            .wait_for(|| {
                // enable interrupts
                self.enable_tx_ints();
                // if the command FIFO is empty, we're done sending stop
                self.is_tx_fifo_empty()
            })
            .await
            .map_err(|_| Error::Other)?;

        self.status()
    }

    async fn async_read_internal(&mut self, address: u8, read: &mut [u8], send_stop: SendStop) -> Result<()> {
        if read.is_empty() {
            return Err(Error::InvalidReadBufferLength);
        }

        for chunk in read.chunks_mut(256) {
            self.async_start(address, true).await?;

            // send receive command
            self.send_cmd(Cmd::Receive, (chunk.len() - 1) as u8);

            T::wait_cell()
                .wait_for(|| {
                    // enable interrupts
                    self.enable_tx_ints();
                    // if the command FIFO is empty, we're done sending start
                    self.is_tx_fifo_empty()
                })
                .await
                .map_err(|_| Error::Other)?;

            for byte in chunk.iter_mut() {
                T::wait_cell()
                    .wait_for(|| {
                        // enable interrupts
                        self.enable_rx_ints();
                        // it the rx FIFO is not empty, we need to read a byte
                        !self.is_rx_fifo_empty()
                    })
                    .await
                    .map_err(|_| Error::ReadFail)?;

                *byte = T::regs().mrdr().read().data().bits();
            }
        }

        if send_stop == SendStop::Yes {
            self.async_stop().await?;
        }

        Ok(())
    }

    async fn async_write_internal(&mut self, address: u8, write: &[u8], send_stop: SendStop) -> Result<()> {
        self.async_start(address, false).await?;

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
            T::wait_cell()
                .wait_for(|| {
                    // enable interrupts
                    self.enable_tx_ints();
                    // initiate trasmit
                    self.send_cmd(Cmd::Transmit, *byte);
                    // it the rx FIFO is not empty, we're done transmiting
                    self.is_tx_fifo_empty()
                })
                .await
                .map_err(|_| Error::WriteFail)?;
        }

        if send_stop == SendStop::Yes {
            self.async_stop().await?;
        }

        Ok(())
    }

    // Public API: Async

    /// Read from address into buffer asynchronously.
    pub fn async_read<'a>(
        &mut self,
        address: u8,
        read: &'a mut [u8],
    ) -> impl Future<Output = Result<()>> + use<'_, 'a, 'd, T> {
        self.async_read_internal(address, read, SendStop::Yes)
    }

    /// Write to address from buffer asynchronously.
    pub fn async_write<'a>(
        &mut self,
        address: u8,
        write: &'a [u8],
    ) -> impl Future<Output = Result<()>> + use<'_, 'a, 'd, T> {
        self.async_write_internal(address, write, SendStop::Yes)
    }

    /// Write to address from bytes and read from address into buffer asynchronously.
    pub async fn async_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<()> {
        self.async_write_internal(address, write, SendStop::No).await?;
        self.async_read_internal(address, read, SendStop::Yes).await
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::Read for I2c<'d, T, M> {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<()> {
        self.blocking_read(address, buffer)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::Write for I2c<'d, T, M> {
    type Error = Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<()> {
        self.blocking_write(address, bytes)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::WriteRead for I2c<'d, T, M> {
    type Error = Error;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<()> {
        self.blocking_write_read(address, bytes, buffer)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::Transactional for I2c<'d, T, M> {
    type Error = Error;

    fn exec(&mut self, address: u8, operations: &mut [embedded_hal_02::blocking::i2c::Operation<'_>]) -> Result<()> {
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

impl embedded_hal_1::i2c::Error for Error {
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

impl<'d, T: Instance, M: Mode> embedded_hal_1::i2c::ErrorType for I2c<'d, T, M> {
    type Error = Error;
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::i2c::I2c for I2c<'d, T, M> {
    fn transaction(&mut self, address: u8, operations: &mut [embedded_hal_1::i2c::Operation<'_>]) -> Result<()> {
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

impl<'d, T: Instance> embedded_hal_async::i2c::I2c for I2c<'d, T, Async> {
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<()> {
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

impl<'d, T: Instance, M: Mode> embassy_embedded_hal::SetConfig for I2c<'d, T, M> {
    type Config = Config;
    type ConfigError = Error;

    fn set_config(&mut self, config: &Self::Config) -> Result<()> {
        Self::set_config(config)
    }
}
