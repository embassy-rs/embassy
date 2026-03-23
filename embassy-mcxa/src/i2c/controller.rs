//! # LPI2C Controller Driver
//!
//! This module provides a driver for the Low-Power Inter-Integrated
//! Circuit (LPI2C) controller, supporting blocking,
//! interrupt-only async, and DMA async modes of operation.
//!
//! The driver support all transfer speeds except for Fast Mode+.
//!
//! ## Features
//!
//! - **Blocking and Asynchronous Modes**: Supports both blocking and
//! async APIs for flexibility in different runtime environments.
//! - **DMA Support**: Enables high-performance data transfers using
//! DMA.
//! - **Configurable Bus Speeds**: Supports standard (100 kHz), fast
//! (400 kHz), and fast-plus (1 MHz) modes. Ultra-fast (3.4 MHz) mode
//! is not yet implemented.
//! - **Error Handling**: Comprehensive error reporting, including
//! FIFO errors, arbitration loss, and address NACK conditions.
//! - **Embedded HAL Compatibility**: Implements traits from
//! `embedded-hal` and `embedded-hal-async` for interoperability with
//! other libraries.
//!
//! ### Error Types
//!
//! - `SetupError`: Errors related to hardware initialization, such as
//! clock configuration issues.
//! - `IOError`: Errors during I2C operations, including FIFO errors,
//! arbitration loss, and invalid buffer lengths.
//!
//! ## Example
//!
//! ```rust,no_run
//! #![no_std]
//! #![no_main]
//!
//! # extern crate panic_halt;
//! # extern crate embassy_mcxa;
//! # extern crate embassy_executor;
//! # use panic_halt as _;
//! use embassy_executor::Spawner;
//! use embassy_mcxa::clocks::config::Div8;
//! use embassy_mcxa::config::Config;
//! use embassy_mcxa::i2c::controller::{self, I2c, Speed};
//!
//! #[embassy_executor::main]
//! async fn main(_spawner: Spawner) {
//!     let mut config = Config::default();
//!     config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);
//!
//!     let p = embassy_mcxa::init(config);
//!
//!     let mut i2c = I2c::new_blocking(p.LPI2C2, p.P1_9, p.P1_8, Default::default()).unwrap();
//!
//!     // Write data
//!     i2c.blocking_write(0x50, &[0x01, 0x02, 0x03]).unwrap();
//!
//!     // Read data
//!     let mut buffer = [0u8; 3];
//!     i2c.blocking_read(0x50, &mut buffer).unwrap();
//! }
//! ```

use core::future::Future;
use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;

use super::{Async, AsyncMode, Blocking, Dma, Info, Instance, Mode, SclPin, SdaPin};
use crate::clocks::periph_helpers::{Div4, Lpi2cClockSel, Lpi2cConfig};
use crate::clocks::{ClockError, PoweredClock, WakeGuard, enable_and_reset};
use crate::dma::{Channel, DMA_MAX_TRANSFER_SIZE, DmaChannel, TransferOptions};
use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::lpi2c::{Alf, Cmd, Dmf, Dozen, Epf, McrRrf, McrRtf, Msr, MsrFef, MsrSdf, Ndf, Pltf, Stf};

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
    /// FIFO Error, the command in the FIFO queue expected the controller to be in a STARTed state, but it was not.
    ///
    /// Even though a START could have been issued earlier, the controller might now be in a different state.
    /// For example, a NAK condition was detected and the controller automatically issued a STOP.
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

impl From<crate::dma::InvalidParameters> for IOError {
    fn from(_value: crate::dma::InvalidParameters) -> Self {
        IOError::Other
    }
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

    /// Clock configuration
    pub clock_config: ClockConfig,
}

/// I2C controller clock configuration
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct ClockConfig {
    /// Powered clock configuration
    pub power: PoweredClock,
    /// LPI2C clock source
    pub source: Lpi2cClockSel,
    /// LPI2C pre-divider
    pub div: Div4,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: Lpi2cClockSel::FroLfDiv,
            div: const { Div4::no_div() },
        }
    }
}

/// I2C Controller Driver.
pub struct I2c<'d, M: Mode> {
    info: &'static Info,
    _scl: Peri<'d, AnyPin>,
    _sda: Peri<'d, AnyPin>,
    mode: M,
    is_hs: bool,
    _wg: Option<WakeGuard>,
}

impl<'d> I2c<'d, Blocking> {
    /// Creates a new blocking instance of the I2C Controller bus driver.
    ///
    /// This method initializes the I2C controller in blocking mode, allowing
    /// synchronous read and write operations.  The I2C bus is configured based
    /// on the provided `Config` structure, which specifies parameters such as
    /// bus speed and clock settings.
    ///
    /// # Arguments
    ///
    /// - `peri`: The peripheral instance representing the I2C controller hardware.
    /// - `scl`: The pin to be used for the I2C clock line (SCL).
    /// - `sda`: The pin to be used for the I2C data line (SDA).
    /// - `config`: A `Config` structure specifying the desired I2C configuration, including bus speed and clock settings.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: A new instance of the I2C driver in blocking mode if initialization is successful.
    /// - `Err(SetupError)`: An error if the initialization fails, such as due to invalid clock configuration.
    ///
    /// # Behavior
    ///
    /// - The I2C controller is configured and enabled based on the provided `Config`.
    /// - Any external pins used for SCL and SDA will be placed into a disabled state when the driver instance is dropped.
    ///
    /// # Errors
    ///
    /// - `SetupError::ClockSetup`: If there is an issue with the clock configuration.
    /// - `SetupError::Other`: For other unexpected initialization errors.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
    ) -> Result<Self, SetupError> {
        Self::new_inner(peri, scl, sda, config, Blocking)
    }
}

impl<'d, M: Mode> I2c<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
        mode: M,
    ) -> Result<Self, SetupError> {
        let ClockConfig { power, source, div } = config.clock_config;

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
            mode,
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

    fn remediation(&self) {
        #[cfg(feature = "defmt")]
        defmt::trace!("Future dropped, issuing stop",);

        // if the FIFO is not empty, drop its contents.
        if !self.is_tx_fifo_empty_or_error() {
            self.reset_fifos();
        }

        // send a stop command
        let _ = self.stop();
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

    /// Checks whether the TX FIFO or if there is an error condition active.
    fn is_tx_fifo_empty_or_error(&self) -> bool {
        self.is_tx_fifo_empty() || self.status().is_err()
    }

    /// Checks whether the RX FIFO is empty.
    fn is_rx_fifo_empty(&self) -> bool {
        self.info.regs().mfsr().read().rxcount() == 0
    }

    /// Parses the controller status producing an
    /// appropriate `Result<(), Error>` variant.
    fn parse_status(&self, msr: &Msr) -> Result<(), IOError> {
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

    /// Reads, parses and clears the controller status producing an
    /// appropriate `Result<(), Error>` variant.
    ///
    /// Will also send a STOP command if the tx_fifo is empty.
    fn status_and_act(&self) -> Result<(), IOError> {
        let msr = self.info.regs().msr().read();
        self.info.regs().msr().write(|w| *w = msr);

        let status = self.parse_status(&msr);

        if let Err(IOError::AddressNack) = status {
            // According to the Reference Manual, section 40.7.1.5
            // Controller Status (MSR), the controller will
            // automatically send a STOP condition if
            // `MCFGR1[AUTOSTOP]` is enabled or if the transmit FIFO
            // is *not* empty.
            //
            // If neither of those conditions is true, we will send a
            // STOP ourselves.
            if !self.info.regs().mcfgr1().read().autostop() && self.is_tx_fifo_empty() {
                self.remediation();
            }
        }

        status
    }

    /// Reads and parses the controller status producing an
    /// appropriate `Result<(), Error>` variant.
    fn status(&self) -> Result<(), IOError> {
        self.parse_status(&self.info.regs().msr().read())
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
        while !self.is_tx_fifo_empty_or_error() {}

        // Check controller status
        self.status_and_act()
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
        while !self.is_tx_fifo_empty_or_error() {}

        self.status_and_act()
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
        if write.is_empty() {
            #[cfg(feature = "defmt")]
            defmt::trace!("Empty write, write probing?");
            if send_stop == SendStop::Yes {
                self.stop()?;
            }
            return Ok(());
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

    /// Reads data from the specified I2C address into the provided buffer.
    ///
    /// This method blocks the caller until the operation is complete.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `read`: A mutable buffer to store the data read from the device.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the read operation is successful.
    /// - `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    ///
    /// # Notes
    ///
    /// The driver will attempt to fill the buffer with data. If the
    /// buffer length exceeds the maximum transfer size of the
    /// controller, the read operation will be performed in multiple
    /// chunks. This will be transparent to the caller.
    pub fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), IOError> {
        self.blocking_read_internal(address, read, SendStop::Yes)
    }

    /// Writes data to the specified I2C address from the provided buffer.
    ///
    /// This method blocks the caller until the operation is complete.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `write`: A buffer containing the data to be written to the device.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the write operation is successful.
    /// - `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    pub fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<(), IOError> {
        self.blocking_write_internal(address, write, SendStop::Yes)
    }

    /// Performs a combined write and read operation on the specified I2C
    /// address.
    ///
    /// This method first writes data to the device, then reads data from the
    /// device into the provided buffer.  The caller is blocked until the
    /// operation is complete.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `write`: A buffer containing the data to be written to the device.
    /// - `read`: A mutable buffer to store the data read from the device.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the write-read operation is successful.
    /// - `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), IOError> {
        self.blocking_write_internal(address, write, SendStop::No)?;
        self.blocking_read_internal(address, read, SendStop::Yes)
    }
}

#[allow(private_bounds)]
impl<'d, M: AsyncMode> I2c<'d, M>
where
    Self: AsyncEngine,
{
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

    /// Schedule sending a START command and await it being pulled from the FIFO.
    ///
    /// Does not indicate that the command was responded to.
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
                self.is_tx_fifo_empty_or_error()
            })
            .await
            .map_err(|_| IOError::Other)?;

        // Note: the START + ACK/NACK have not necessarily been finished here.
        // thus this might return Ok(()), but might at a later state result in NAK or FifoError.
        self.status_and_act()
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
                self.is_tx_fifo_empty_or_error()
            })
            .await
            .map_err(|_| IOError::Other)?;

        self.status_and_act()
    }

    // Public API: Async

    /// Reads data from the specified I2C address into the provided buffer asynchronously.
    ///
    /// This method performs the read operation without blocking the caller,
    /// returning a `Future` that resolves when the operation is complete.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `read`: A mutable buffer to store the data read from the device.
    ///
    /// # Returns
    ///
    /// - A `Future` that resolves to `Ok(())` if the read operation is successful.
    /// - Resolves to `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    pub fn async_read<'a>(
        &'a mut self,
        address: u8,
        read: &'a mut [u8],
    ) -> impl Future<Output = Result<(), IOError>> + 'a {
        <Self as AsyncEngine>::async_read_internal(self, address, read, SendStop::Yes)
    }

    /// Writes data to the specified I2C address from the provided buffer asynchronously.
    ///
    /// This method performs the write operation without blocking the caller, returning a `Future` that resolves when the operation is complete.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `write`: A buffer containing the data to be written to the device.
    ///
    /// # Returns
    ///
    /// - A `Future` that resolves to `Ok(())` if the write operation is successful.
    /// - Resolves to `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    pub fn async_write<'a>(
        &'a mut self,
        address: u8,
        write: &'a [u8],
    ) -> impl Future<Output = Result<(), IOError>> + 'a {
        <Self as AsyncEngine>::async_write_internal(self, address, write, SendStop::Yes)
    }

    /// Performs a combined write and read operation on the specified I2C
    /// address asynchronously.
    ///
    /// This method first writes data to the device, then reads data from the
    /// device into the provided buffer. The operation is performed without
    /// blocking the caller.
    ///
    /// # Arguments
    ///
    /// - `address`: The 7-bit I2C address of the target device.
    /// - `write`: A buffer containing the data to be written to the device.
    /// - `read`: A mutable buffer to store the data read from the device.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the write-read operation is successful.
    /// - `Err(IOError)` if an error occurs during the operation, such as an address NACK or FIFO error.
    ///
    /// # Errors
    ///
    /// - `IOError::AddressNack`: If the device does not acknowledge the address.
    /// - `IOError::FifoError`: If there is an issue with the FIFO queue.
    /// - Other variants of `IOError` for specific I2C errors.
    pub async fn async_write_read<'a>(
        &'a mut self,
        address: u8,
        write: &'a [u8],
        read: &'a mut [u8],
    ) -> Result<(), IOError> {
        <Self as AsyncEngine>::async_write_internal(self, address, write, SendStop::No).await?;
        <Self as AsyncEngine>::async_read_internal(self, address, read, SendStop::Yes).await
    }
}

trait AsyncEngine {
    fn async_read_internal<'a>(
        &'a mut self,
        address: u8,
        read: &'a mut [u8],
        send_stop: SendStop,
    ) -> impl Future<Output = Result<(), IOError>> + 'a;

    fn async_write_internal<'a>(
        &'a mut self,
        address: u8,
        write: &'a [u8],
        send_stop: SendStop,
    ) -> impl Future<Output = Result<(), IOError>> + 'a;
}

impl<'d> I2c<'d, Async> {
    /// Creates a new interrupt-only asynchronous instance of the I2C Controller
    /// bus driver.
    ///
    /// This method initializes the I2C controller in asynchronous mode,
    /// enabling non-blocking operations using futures.  The I2C bus is
    /// configured based on the provided `Config` structure, which specifies
    /// parameters such as bus speed and clock settings.
    ///
    /// # Arguments
    ///
    /// - `peri`: The peripheral instance representing the I2C controller hardware.
    /// - `scl`: The pin to be used for the I2C clock line (SCL).
    /// - `sda`: The pin to be used for the I2C data line (SDA).
    /// - `_irq`: The interrupt binding for the I2C controller, ensuring that an interrupt handler is registered.
    /// - `config`: A `Config` structure specifying the desired I2C configuration, including bus speed and clock settings.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: A new instance of the I2C driver in asynchronous mode if initialization is successful.
    /// - `Err(SetupError)`: An error if the initialization fails, such as due to invalid clock configuration.
    ///
    /// # Behavior
    ///
    /// - The I2C controller is configured and enabled based on the provided `Config`.
    /// - The interrupt for the I2C controller is enabled to support asynchronous operations.
    /// - Any external pins used for SCL and SDA will be placed into a disabled state when the driver instance is dropped.
    ///
    /// # Errors
    ///
    /// - `SetupError::ClockSetup`: If there is an issue with the clock configuration.
    /// - `SetupError::Other`: For other unexpected initialization errors.
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

        Self::new_inner(peri, scl, sda, config, Async)
    }
}

impl<'d> AsyncEngine for I2c<'d, Async> {
    async fn async_read_internal(&mut self, address: u8, read: &mut [u8], send_stop: SendStop) -> Result<(), IOError> {
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
                    self.is_tx_fifo_empty_or_error()
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

    async fn async_write_internal(&mut self, address: u8, write: &[u8], send_stop: SendStop) -> Result<(), IOError> {
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
        if write.is_empty() {
            #[cfg(feature = "defmt")]
            defmt::trace!("Empty write, write probing?");
            if send_stop == SendStop::Yes {
                self.async_stop().await?;
            }
            return Ok(());
        }

        for byte in write {
            // initiate transmit
            self.send_cmd(Cmd::TRANSMIT, *byte);

            self.info
                .wait_cell()
                .wait_for(|| {
                    // enable interrupts
                    self.enable_tx_ints();
                    // if the tx FIFO is empty, we're done transmiting
                    self.is_tx_fifo_empty_or_error()
                })
                .await
                .map_err(|_| IOError::WriteFail)?;

            self.status_and_act()?;
        }

        if send_stop == SendStop::Yes {
            self.async_stop().await?;
        }

        // defuse it if the future is not dropped
        on_drop.defuse();

        Ok(())
    }
}

impl<'d> I2c<'d, Dma<'d>> {
    /// Creates a new asynchronous instance of the I2C Controller bus driver with DMA support.
    ///
    /// This method initializes the I2C controller in asynchronous mode with
    /// Direct Memory Access (DMA) support, enabling efficient non-blocking
    /// operations for large data transfers.  The I2C bus is configured based on
    /// the provided `Config` structure, which specifies parameters such as bus
    /// speed and clock settings.
    ///
    /// # Arguments
    ///
    /// - `peri`: The peripheral instance representing the I2C controller hardware.
    /// - `scl`: The pin to be used for the I2C clock line (SCL).
    /// - `sda`: The pin to be used for the I2C data line (SDA).
    /// - `tx_dma`: The DMA channel to be used for transmitting data.
    /// - `rx_dma`: The DMA channel to be used for receiving data.
    /// - `_irq`: The interrupt binding for the I2C controller, ensuring that an interrupt handler is registered.
    /// - `config`: A `Config` structure specifying the desired I2C configuration, including bus speed and clock settings.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: A new instance of the I2C driver in asynchronous mode with DMA support if initialization is successful.
    /// - `Err(SetupError)`: An error if the initialization fails, such as due to invalid clock configuration.
    ///
    /// # Behavior
    ///
    /// - The I2C controller is configured and enabled based on the provided `Config`.
    /// - The interrupt for the I2C controller is enabled to support asynchronous operations.
    /// - The specified DMA channels are initialized and their interrupts are enabled.
    /// - Any external pins used for SCL and SDA will be placed into a disabled state when the driver instance is dropped.
    ///
    /// # Errors
    ///
    /// - `SetupError::ClockSetup`: If there is an issue with the clock configuration.
    /// - `SetupError::Other`: For other unexpected initialization errors.
    pub fn new_async_with_dma<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, SetupError> {
        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        // enable this channel's interrupt
        let tx_dma = DmaChannel::new(tx_dma);
        let rx_dma = DmaChannel::new(rx_dma);

        tx_dma.enable_interrupt();
        rx_dma.enable_interrupt();

        Self::new_inner(
            peri,
            scl,
            sda,
            config,
            Dma {
                tx_dma,
                rx_dma,
                tx_request: T::TX_DMA_REQUEST,
                rx_request: T::RX_DMA_REQUEST,
            },
        )
    }
}

impl<'d> AsyncEngine for I2c<'d, Dma<'d>> {
    async fn async_read_internal(&mut self, address: u8, read: &mut [u8], send_stop: SendStop) -> Result<(), IOError> {
        if read.is_empty() {
            return Err(IOError::InvalidReadBufferLength);
        }

        // perform corrective action if the future is dropped
        let on_drop = OnDrop::new(|| {
            self.remediation();
            self.info.regs().mder().modify(|w| w.set_rdde(false));
        });

        for chunk in read.chunks_mut(256) {
            self.async_start(address, true).await?;

            // send receive command
            self.send_cmd(Cmd::RECEIVE, (chunk.len() - 1) as u8);

            let peri_addr = self.info.regs().mrdr().as_ptr() as *const u8;

            // _rx_dma is guaranteed to be Some
            unsafe {
                // Clean up channel state
                self.mode.rx_dma.disable_request();
                self.mode.rx_dma.clear_done();
                self.mode.rx_dma.clear_interrupt();

                // Set DMA request source from instance type (type-safe)
                self.mode.rx_dma.set_request_source(self.mode.rx_request);

                // Configure TCD for peripheral-to-memory transfer
                self.mode.rx_dma.setup_read_from_peripheral(
                    peri_addr,
                    chunk,
                    false,
                    TransferOptions::COMPLETE_INTERRUPT,
                )?;

                // Enable I2C RX DMA request
                self.info.regs().mder().modify(|w| w.set_rdde(true));

                // Enable DMA channel request
                self.mode.rx_dma.enable_request();
            }

            // Wait for completion asynchronously
            core::future::poll_fn(|cx| {
                self.mode.rx_dma.waker().register(cx.waker());
                if self.mode.rx_dma.is_done() {
                    core::task::Poll::Ready(())
                } else {
                    core::task::Poll::Pending
                }
            })
            .await;

            // Ensure DMA writes are visible to CPU
            cortex_m::asm::dsb();
            // Cleanup
            self.info.regs().mder().modify(|w| w.set_rdde(false));
            unsafe {
                self.mode.rx_dma.disable_request();
                self.mode.rx_dma.clear_done();
            }
        }

        if send_stop == SendStop::Yes {
            self.async_stop().await?;
        }

        // defuse it if the future is not dropped
        on_drop.defuse();

        Ok(())
    }

    async fn async_write_internal(&mut self, address: u8, write: &[u8], send_stop: SendStop) -> Result<(), IOError> {
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
        if write.is_empty() {
            #[cfg(feature = "defmt")]
            defmt::trace!("Empty write, write probing?");
            if send_stop == SendStop::Yes {
                self.async_stop().await?;
            }
            return Ok(());
        }

        // perform corrective action if the future is dropped
        let on_drop = OnDrop::new(|| {
            self.remediation();
            self.info.regs().mder().modify(|w| w.set_tdde(false));
        });

        for chunk in write.chunks(DMA_MAX_TRANSFER_SIZE) {
            let peri_addr = self.info.regs().mtdr().as_ptr() as *mut u8;

            unsafe {
                // Clean up channel state
                self.mode.tx_dma.disable_request();
                self.mode.tx_dma.clear_done();
                self.mode.tx_dma.clear_interrupt();

                // Set DMA request source from instance type (type-safe)
                self.mode.tx_dma.set_request_source(self.mode.tx_request);

                // Configure TCD for memory-to-peripheral transfer
                self.mode.tx_dma.setup_write_to_peripheral(
                    chunk,
                    peri_addr,
                    false,
                    TransferOptions::COMPLETE_INTERRUPT,
                )?;

                // Enable I2C TX DMA request
                self.info.regs().mder().modify(|w| w.set_tdde(true));

                // Enable DMA channel request
                self.mode.tx_dma.enable_request();
            }

            // Wait for completion asynchronously
            core::future::poll_fn(|cx| {
                self.mode.tx_dma.waker().register(cx.waker());
                if self.mode.tx_dma.is_done() {
                    core::task::Poll::Ready(())
                } else {
                    core::task::Poll::Pending
                }
            })
            .await;

            // Ensure DMA writes are visible to CPU
            cortex_m::asm::dsb();
            // Cleanup
            self.info.regs().mder().modify(|w| w.set_tdde(false));
            unsafe {
                self.mode.tx_dma.disable_request();
                self.mode.tx_dma.clear_done();
            }
        }

        if send_stop == SendStop::Yes {
            self.async_stop().await?;
        }

        // defuse it if the future is not dropped
        on_drop.defuse();

        Ok(())
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

impl<'d, M: AsyncMode> embedded_hal_async::i2c::I2c for I2c<'d, M>
where
    I2c<'d, M>: AsyncEngine,
{
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        if let Some((last, rest)) = operations.split_last_mut() {
            for op in rest {
                match op {
                    embedded_hal_async::i2c::Operation::Read(buf) => {
                        <Self as AsyncEngine>::async_read_internal(self, address, buf, SendStop::No).await?
                    }
                    embedded_hal_async::i2c::Operation::Write(buf) => {
                        <Self as AsyncEngine>::async_write_internal(self, address, buf, SendStop::No).await?
                    }
                }
            }

            match last {
                embedded_hal_async::i2c::Operation::Read(buf) => {
                    <Self as AsyncEngine>::async_read_internal(self, address, buf, SendStop::Yes).await
                }
                embedded_hal_async::i2c::Operation::Write(buf) => {
                    <Self as AsyncEngine>::async_write_internal(self, address, buf, SendStop::Yes).await
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
