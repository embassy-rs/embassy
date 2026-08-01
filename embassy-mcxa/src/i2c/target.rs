//! LPI2C Target Driver
//!
//! This module provides an implementation of an I2C target (slave)
//! driver. It supports both blocking and asynchronous modes of
//! operation, as well as DMA-based transfers. The driver allows the
//! target device to respond to requests from an I2C controller
//! (master), including reading and writing data, handling general
//! calls, and responding to SMBus alerts.
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
//! use embassy_mcxa::i2c::target;
//!
//! #[embassy_executor::main]
//! async fn main(_spawner: Spawner) {
//!     let mut config = Config::default();
//!     config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);
//!
//!     let p = embassy_mcxa::init(config);
//!
//!     let mut config = target::Config::default();
//!     config.address = target::Address::Dual(0x2a, 0x31);
//!     let mut i2c = target::I2c::new_blocking(p.LPI2C3, p.P3_27, p.P3_28, config).unwrap();
//!     let mut buf = [0u8; 32];
//!
//!     loop {
//!         let request = i2c.blocking_listen().unwrap();
//!         match request {
//!             target::Request::Read(addr) => {
//!                 // Controller wants to read from us at `addr`
//!                 buf.fill(0x55);
//!                 let _status = i2c.blocking_respond_to_read(&buf).unwrap();
//!             }
//!             target::Request::Write(_addr) => {
//!                 // Controller wants to write to us at `addr`
//!                 let _status = i2c.blocking_respond_to_write(&mut buf).unwrap();
//!             }
//!             target::Request::Stop(_addr) => {
//!                 // Controller issued a STOP condition for `addr`
//!             }
//!             target::Request::GeneralCall => {
//!                 // Controller issued a General Call (broadcast write
//!                 // to address 0x00). Drain the payload via the
//!                 // normal write-response path.
//!                 let _status = i2c.blocking_respond_to_write(&mut buf).unwrap();
//!             }
//!             target::Request::SmbusAlert => {
//!                 // Controller issued an SMBus Alert
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//! ```

use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::Range;
use core::sync::atomic::{Ordering, fence};
use core::task::Poll;

use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;

use super::{Async, AsyncMode, Blocking, Dma, Info, Instance, Mode, SclPin, SdaPin};
pub use crate::clocks::PoweredClock;
pub use crate::clocks::periph_helpers::{Div4, Lpi2cClockSel, Lpi2cConfig};
use crate::clocks::{ClockError, WakeGuard, enable_and_reset};
use crate::dma::{Channel, DMA_MAX_TRANSFER_SIZE, DmaChannel, TransferOptions};
use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::lpi2c::{Addrcfg, Filtdz, ScrRrf, ScrRtf};

/// Errors exclusive to hardware Initialization
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Clock configuration error.
    ClockSetup(ClockError),
    /// Invalid Address
    InvalidAddress,
    /// Other internal errors or unexpected state.
    Other,
}

/// Errors exclusive to I/O
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum IOError {
    /// Busy Busy
    BusBusy,
    /// Target Busy
    TargetBusy,
    /// FIFO Error
    FifoError,
    /// Bit Error
    BitError,
    /// Other internal errors or unexpected state.
    Other,
}

impl From<crate::dma::InvalidParameters> for IOError {
    fn from(_value: crate::dma::InvalidParameters) -> Self {
        IOError::Other
    }
}

/// Outcome of a `respond_to_read` call.
///
/// The `usize` in every variant counts bytes consumed from the supplied
/// buffer, i.e. bytes the controller actually clocked out and ACKed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ReadStatus {
    /// Controller terminated the read with NACK + STOP exactly when the
    /// supplied buffer was exhausted.
    Complete(usize),
    /// Buffer was fully consumed but the controller is still asking for
    /// more bytes (it ACKed the last byte). Caller should call
    /// `respond_to_read` again with additional bytes, or accept that the
    /// bus will clock-stretch (with TXDSTALL enabled) until something
    /// else terminates the transfer.
    NeedMore(usize),
    /// Controller issued an early STOP or repeated START before the
    /// buffer was exhausted.
    EarlyStop(usize),
}

/// Outcome of a `respond_to_write` call.
///
/// The `usize` in every variant counts bytes written into the supplied
/// buffer, i.e. bytes the target ACKed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum WriteStatus {
    /// Controller issued STOP.
    Stopped(usize),
    /// Controller issued a repeated START. The next `listen` call will
    /// report the direction/address of the new sub-transaction.
    Restarted(usize),
    /// The supplied buffer filled before the controller terminated the
    /// transfer. Caller should call `respond_to_write` again with more
    /// buffer space, or accept that the bus will clock-stretch (with
    /// RXSTALL enabled) until something else terminates the transfer.
    BufferFull(usize),
}

/// I2C interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        if T::info().regs().sier().read().0 != 0 {
            T::info().regs().sier().write(|w| {
                w.set_tdie(false);
                w.set_rdie(false);
                w.set_avie(false);
                w.set_taie(false);
                w.set_rsie(false);
                w.set_sdie(false);
                w.set_beie(false);
                w.set_feie(false);
                w.set_am0ie(false);
                w.set_am1ie(false);
                w.set_gcie(false);
                w.set_sarie(false);
            });

            T::PERF_INT_WAKE_INCR();
            T::info().wait_cell().wake();
        }
    }
}

/// I2C target addresses.
#[derive(Clone)]
pub enum Address {
    /// Single address
    Single(u16),
    /// Two addresses
    Dual(u16, u16),
    /// Range of addresses
    Range(Range<u16>),
}

impl Default for Address {
    fn default() -> Self {
        Self::Single(0x2a)
    }
}

/// Enable or disable feature
#[derive(Copy, Clone, Default)]
pub enum Status {
    #[default]
    Disabled,
    Enabled,
}

impl From<Status> for bool {
    fn from(value: Status) -> Self {
        match value {
            Status::Disabled => false,
            Status::Enabled => true,
        }
    }
}

/// I2C target configuration
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct Config {
    /// Addresses to respond to
    pub address: Address,

    /// Enable SMBus alert
    pub smbus_alert: Status,

    /// Enable general call support
    pub general_call: Status,

    /// Clock configuration
    pub clock_config: ClockConfig,
}

/// I2C target clock configuration
#[derive(Clone)]
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

/// I2C target events
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Request {
    /// Controller wants to write data to this Target
    Write(u16),
    /// Controller wants to read data from this Target
    Read(u16),
    /// Controller issued Stop condition for this Target
    Stop(u16),
    /// Controller issued a General Call
    GeneralCall,
    /// Controller issued SMBUS Alert
    SmbusAlert,
}

/// I2C target events
#[derive(Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Event {
    SmbusAlert,
    GeneralCall,
    Address0Match(u16),
    Address1Match(u16),
    Stop(u16),
    RepeatedStart(u16),
    TransmitAck,
    AddressValid(u16),
    ReceiveData,
    TransmitData,
}

/// I2C Target Driver.
pub struct I2c<'d, M: Mode> {
    info: &'static Info,
    _scl: Peri<'d, AnyPin>,
    _sda: Peri<'d, AnyPin>,
    smbus_alert: Status,
    general_call: Status,
    mode: M,
    _wg: Option<WakeGuard>,
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
            smbus_alert: config.smbus_alert.clone(),
            general_call: config.general_call.clone(),
            mode,
            _wg: parts.wake_guard,
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    fn set_configuration(&self, config: &Config) -> Result<(), SetupError> {
        critical_section::with(|_| {
            // Disable the target.
            self.info.regs().scr().modify(|w| w.set_sen(false));

            // Soft-reset the target, read and write FIFOs.
            self.reset_fifos();
            self.info.regs().scr().modify(|w| w.set_rst(true));
            // According to Reference Manual section 40.7.1.4, "There
            // is no minimum delay required before clearing the
            // software reset", therefore we clear it immediately.
            self.info.regs().scr().modify(|w| w.set_rst(false));

            self.info.regs().scr().modify(|w| {
                w.set_filtdz(Filtdz::FilterDisabled);
                w.set_filten(false);
            });

            self.info.regs().scfgr1().modify(|w| {
                w.set_adrstall(true);
                w.set_rxstall(true);
                w.set_txdstall(true);
                w.set_gcen(config.general_call.into());
                w.set_saen(config.smbus_alert.into());
            });

            // Configure address matching
            match config.address {
                Address::Single(addr) => {
                    self.info.regs().samr().write(|w| w.set_addr0(addr));
                    self.info.regs().scfgr1().modify(|w| {
                        w.set_addrcfg(if (0x00..=0x7f).contains(&addr) {
                            Addrcfg::AddressMatch07Bit
                        } else {
                            Addrcfg::AddressMatch010Bit
                        })
                    });
                }

                Address::Dual(addr0, addr1) => {
                    // Either both a 7-bit or both are 10-bit
                    if ((0x00..=0x7f).contains(&addr0) ^ (0x00..=0x7f).contains(&addr1))
                        || ((0x80..=0x3ff).contains(&addr0) ^ (0x80..=0x3ff).contains(&addr1))
                    {
                        return Err(SetupError::InvalidAddress);
                    }

                    self.info.regs().samr().write(|w| {
                        w.set_addr0(addr0);
                        w.set_addr1(addr1);
                    });
                    self.info.regs().scfgr1().modify(|w| {
                        w.set_addrcfg(if (0x00..=0x7f).contains(&addr0) {
                            Addrcfg::AddressMatch07BitOrAddressMatch17Bit
                        } else {
                            Addrcfg::AddressMatch010BitOrAddressMatch110Bit
                        })
                    });
                }

                Address::Range(Range { start, end }) => {
                    if ((0x00..=0x7f).contains(&start) ^ (0x00..=0x7f).contains(&end))
                        || ((0x80..=0x3ff).contains(&start) ^ (0x80..=0x3ff).contains(&end))
                    {
                        return Err(SetupError::InvalidAddress);
                    }

                    self.info.regs().samr().write(|w| {
                        w.set_addr0(start);
                        w.set_addr1(end - 1);
                    });
                    self.info.regs().scfgr1().modify(|w| {
                        w.set_addrcfg(if (0x00..=0x7f).contains(&start) {
                            Addrcfg::FromAddressMatch07BitToAddressMatch17Bit
                        } else {
                            Addrcfg::FromAddressMatch010BitToAddressMatch110Bit
                        })
                    });
                }
            }

            // Enable the target.
            self.info.regs().scr().modify(|w| w.set_sen(true));

            // Clear all flags
            self.info.regs().ssr().write(|w| {
                w.set_rsf(true);
                w.set_sdf(true);
                w.set_bef(true);
                w.set_fef(true);
            });

            Ok(())
        })
    }

    /// Resets both TX and RX FIFOs dropping their contents.
    fn reset_fifos(&self) {
        // The critical section is needed to prevent an interrupt from
        // modifying SCR while we're in the middle of our
        // read-modify-write operation.
        critical_section::with(|_| {
            self.info.regs().scr().modify(|w| {
                w.set_rtf(ScrRtf::NowEmpty);
                w.set_rrf(ScrRrf::NowEmpty);
            });
        });
    }

    fn clear_status(&self) {
        self.info.regs().ssr().write(|w| {
            w.set_rsf(true);
            w.set_sdf(true);
            w.set_bef(true);
            w.set_fef(true);
        });
    }

    /// Reads and parses the target status producing an
    /// appropriate `Result<(), Error>` variant.
    fn status(&self) -> Result<Event, IOError> {
        let ssr = self.info.regs().ssr().read();
        self.clear_status();

        if ssr.bef() {
            Err(IOError::BitError)
        } else if ssr.fef() {
            Err(IOError::FifoError)
        } else if ssr.avf() || ssr.gcf() || ssr.sarf() {
            // GCF/SARF are address-classification tags on the
            // address-valid event. We must read SASR to consume
            // the address-valid state regardless of which tag
            // triggered the match.
            let is_gc = ssr.gcf();
            let is_alert = ssr.sarf();
            let sasr = self.info.regs().sasr().read();
            let addr = sasr.raddr();
            if is_gc {
                Ok(Event::GeneralCall)
            } else if is_alert {
                Ok(Event::SmbusAlert)
            } else {
                Ok(Event::AddressValid(addr))
            }
        } else if ssr.taf() {
            Ok(Event::TransmitAck)
        } else if ssr.rsf() {
            let sasr = self.info.regs().sasr().read();
            let addr = sasr.raddr();
            Ok(Event::RepeatedStart(addr))
        } else if ssr.sdf() {
            let sasr = self.info.regs().sasr().read();
            let addr = sasr.raddr();
            Ok(Event::Stop(addr))
        } else {
            Err(IOError::Other)
        }
    }

    // Public API: Blocking

    /// Block waiting for new events.
    ///
    /// This function blocks the caller until a new I2C event is received. It returns the
    /// type of request made by the I2C controller.
    ///
    /// # Returns
    ///
    /// - `Ok(Request)` on success.
    /// - `Err(IOError)` if an error occurs.
    pub fn blocking_listen(&mut self) -> Result<Request, IOError> {
        self.clear_status();

        // Wait for Address Valid
        loop {
            let ssr = self.info.regs().ssr().read();
            let avr = ssr.avf();
            let sarf = ssr.sarf();
            let gcf = ssr.gcf();
            let sdf = ssr.sdf();

            if avr || sarf || gcf || sdf {
                break;
            }
        }

        let event = self.status()?;

        match event {
            Event::SmbusAlert => Ok(Request::SmbusAlert),
            Event::GeneralCall => Ok(Request::GeneralCall),
            Event::Stop(addr) => Ok(Request::Stop(addr >> 1)),
            Event::RepeatedStart(addr) | Event::AddressValid(addr) => {
                if addr & 1 != 0 {
                    Ok(Request::Read(addr >> 1))
                } else {
                    Ok(Request::Write(addr >> 1))
                }
            }
            _ => Err(IOError::Other),
        }
    }

    /// Transmit data to the I2C controller.
    ///
    /// Sends the contents of the provided buffer to the I2C controller. The
    /// call services the transfer to a clean termination point (STOP,
    /// repeated START, or buffer exhausted) before returning.
    ///
    /// # Parameters
    ///
    /// - `buf`: The buffer containing the data to transmit.
    ///
    /// # Returns
    ///
    /// - `Ok(ReadStatus)` describing how the transfer ended and how many
    ///   bytes the controller ACKed.
    /// - `Err(IOError)` if an error occurs.
    pub fn blocking_respond_to_read(&mut self, buf: &[u8]) -> Result<ReadStatus, IOError> {
        let mut count = 0;

        self.clear_status();

        for byte in buf.iter() {
            // Wait until we can send data
            let ssr = loop {
                let ssr = self.info.regs().ssr().read();
                if ssr.tdf() || ssr.sdf() || ssr.rsf() {
                    break ssr;
                }
            };

            if ssr.sdf() || ssr.rsf() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Send routine. STOP or Repeated-start received");
                return Ok(ReadStatus::EarlyStop(count));
            }

            self.info.regs().stdr().write(|w| w.set_data(*byte));
            count += 1;
        }

        // All caller bytes pushed. Wait briefly to determine whether the
        // controller is done (NACK + STOP/RSTART) or whether it wants more.
        let ssr = loop {
            let ssr = self.info.regs().ssr().read();
            if ssr.tdf() || ssr.sdf() || ssr.rsf() {
                break ssr;
            }
        };

        if ssr.sdf() || ssr.rsf() {
            Ok(ReadStatus::Complete(count))
        } else {
            // tdf set: TX FIFO empty during a transmit transfer means the
            // controller is still clocking and wants another byte.
            Ok(ReadStatus::NeedMore(count))
        }
    }

    /// Receive data from the I2C controller.
    ///
    /// Reads bytes the controller writes into the provided buffer. The call
    /// services the transfer to a clean termination point (STOP, repeated
    /// START, or buffer filled) before returning.
    ///
    /// # Parameters
    ///
    /// - `buf`: The buffer to store the received data.
    ///
    /// # Returns
    ///
    /// - `Ok(WriteStatus)` describing how the transfer ended and how many
    ///   bytes the target received.
    /// - `Err(IOError)` if an error occurs.
    pub fn blocking_respond_to_write(&mut self, buf: &mut [u8]) -> Result<WriteStatus, IOError> {
        let mut count = 0;

        self.clear_status();

        for byte in buf.iter_mut() {
            // Wait until we have data to read
            let ssr = loop {
                let ssr = self.info.regs().ssr().read();
                if ssr.rdf() || ssr.sdf() || ssr.rsf() {
                    break ssr;
                }
            };

            if ssr.sdf() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Receive routine. STOP received");
                return Ok(WriteStatus::Stopped(count));
            }
            if ssr.rsf() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Receive routine. Repeated-start received");
                return Ok(WriteStatus::Restarted(count));
            }

            *byte = self.info.regs().srdr().read().data();
            count += 1;
        }

        Ok(WriteStatus::BufferFull(count))
    }
}

impl<'d> I2c<'d, Blocking> {
    /// Create a new blocking instance of the I2C Target bus driver.
    ///
    /// This function initializes the I2C target driver in blocking mode. It configures the
    /// I2C peripheral, sets up the clock, and prepares the pins for operation. Any external
    /// pin will be placed into the Disabled state upon `Drop`.
    ///
    /// # Parameters
    ///
    /// - `peri`: The I2C peripheral instance.
    /// - `scl`: The SCL pin.
    /// - `sda`: The SDA pin.
    /// - `config`: The configuration for the I2C target.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` on success.
    /// - `Err(SetupError)` if initialization fails.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
    ) -> Result<Self, SetupError> {
        Self::new_inner(peri, scl, sda, config, Blocking)
    }
}

impl<'d> I2c<'d, Async> {
    /// Create a new asynchronous instance of the I2C Target bus driver.
    ///
    /// This function initializes the I2C target driver in asynchronous mode. It configures the
    /// I2C peripheral, sets up the clock, and prepares the pins for operation. Any external
    /// pin will be placed into the Disabled state upon `Drop`.
    ///
    /// # Parameters
    ///
    /// - `peri`: The I2C peripheral instance.
    /// - `scl`: The SCL pin.
    /// - `sda`: The SDA pin.
    /// - `_irq`: The interrupt binding for the I2C peripheral.
    /// - `config`: The configuration for the I2C target.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` on success.
    /// - `Err(SetupError)` if initialization fails.
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

/// Internal outcome of a single DMA TX chunk transfer (target -> controller).
#[derive(Copy, Clone)]
enum TxChunkOutcome {
    /// Controller issued STOP. `usize` is bytes transferred from this chunk.
    Stopped(usize),
    /// Controller issued repeated START. `usize` is bytes transferred from
    /// this chunk.
    Restarted(usize),
    /// DMA exhausted the chunk and the controller is still asking for more
    /// bytes. `usize` equals the chunk length.
    NeedMore(usize),
}

/// Internal outcome of a single DMA RX chunk transfer (controller -> target).
#[derive(Copy, Clone)]
enum RxChunkOutcome {
    /// Controller issued STOP. `usize` is bytes received into this chunk.
    Stopped(usize),
    /// Controller issued repeated START. `usize` is bytes received into
    /// this chunk.
    Restarted(usize),
    /// DMA filled the chunk before the controller terminated the transfer.
    /// `usize` equals the chunk length.
    Filled(usize),
}

impl<'d> I2c<'d, Dma<'d>> {
    /// Create a new asynchronous instance of the I2C Target bus driver with DMA support.
    ///
    /// This function initializes the I2C target driver in asynchronous mode with DMA support.
    /// It configures the I2C peripheral, sets up the clock, and prepares the pins for operation.
    /// Any external pin will be placed into the Disabled state upon `Drop`, and the DMA channels
    /// are also disabled.
    ///
    /// # Parameters
    ///
    /// - `peri`: The I2C peripheral instance.
    /// - `scl`: The SCL pin.
    /// - `sda`: The SDA pin.
    /// - `tx_dma`: The DMA channel for transmitting data.
    /// - `rx_dma`: The DMA channel for receiving data.
    /// - `_irq`: The interrupt binding for the I2C peripheral.
    /// - `config`: The configuration for the I2C target.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` on success.
    /// - `Err(SetupError)` if initialization fails.
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

    async fn read_dma_chunk(&mut self, data: &mut [u8]) -> Result<RxChunkOutcome, IOError> {
        let peri_addr = self.info.regs().srdr().as_ptr() as *const u8;
        let chunk_len = data.len();

        self.clear_status();

        unsafe {
            // Clean up channel state
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();
            self.mode.rx_dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            self.mode.rx_dma.set_request_source(self.mode.rx_request);

            // Configure TCD for peripheral-to-memory transfer
            self.mode
                .rx_dma
                .setup_read_from_peripheral(peri_addr, data, false, TransferOptions::COMPLETE_INTERRUPT)?;

            // Enable I2C RX DMA request
            self.info.regs().sder().modify(|w| w.set_rdde(true));

            // Enable DMA channel request
            self.mode.rx_dma.enable_request();
        }

        // Wait for any of:
        //  - I2C end-of-transfer flag (sdf, rsf) -> controller terminated
        //  - I2C error flag (fef, bef) -> bus problem
        //  - DMA channel completion -> chunk filled before controller stopped
        //
        // The DMA done interrupt wakes the DMA's wait_cell; I2C status
        // changes wake the I2C wait_cell. Register on both.
        poll_fn(|cx| {
            let _ = self.mode.rx_dma.wait_cell().poll_wait(cx);
            let _ = self.info.wait_cell().poll_wait(cx);

            self.info.regs().sier().write(|w| {
                w.set_feie(true);
                w.set_beie(true);
                w.set_sdie(true);
                w.set_rsie(true);
            });

            let ssr = self.info.regs().ssr().read();
            if ssr.fef() || ssr.bef() || ssr.sdf() || ssr.rsf() || self.mode.rx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // Cleanup
        self.info.regs().sder().modify(|w| w.set_rdde(false));
        unsafe {
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();
        }

        // Ensure all writes by DMA are visible to the CPU
        fence(Ordering::Acquire);

        let ssr = self.info.regs().ssr().read();

        if ssr.fef() {
            Err(IOError::FifoError)
        } else if ssr.bef() {
            Err(IOError::BitError)
        } else if ssr.sdf() {
            Ok(RxChunkOutcome::Stopped(self.mode.rx_dma.transferred_bytes()))
        } else if ssr.rsf() {
            Ok(RxChunkOutcome::Restarted(self.mode.rx_dma.transferred_bytes()))
        } else {
            // DMA done with no end-of-transfer flag: chunk filled, controller
            // may want to write more bytes.
            Ok(RxChunkOutcome::Filled(chunk_len))
        }
    }

    async fn write_dma_chunk(&mut self, data: &[u8]) -> Result<TxChunkOutcome, IOError> {
        let peri_addr = self.info.regs().stdr().as_ptr() as *mut u8;
        let chunk_len = data.len();

        self.clear_status();

        unsafe {
            // Clean up channel state
            self.mode.tx_dma.disable_request();
            self.mode.tx_dma.clear_done();
            self.mode.tx_dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            self.mode.tx_dma.set_request_source(self.mode.tx_request);

            // Configure TCD for memory-to-peripheral transfer. Use
            // COMPLETE_INTERRUPT so the channel wakes us on DMA exhaustion
            // (which lets us return NeedMore when the controller wants more
            // bytes than the chunk holds).
            self.mode
                .tx_dma
                .setup_write_to_peripheral(data, peri_addr, false, TransferOptions::COMPLETE_INTERRUPT)?;

            // Ensure all writes by DMA are visible to the CPU
            fence(Ordering::Release);

            // Enable I2C TX DMA request
            self.info.regs().sder().modify(|w| w.set_tdde(true));

            // Enable DMA channel request
            self.mode.tx_dma.enable_request();
        }

        // Wait for any of:
        //  - I2C end-of-transfer flag (sdf, rsf) -> controller terminated
        //  - I2C error flag (fef, bef) -> bus problem
        //  - DMA channel completion -> chunk exhausted; if controller still
        //    clocking, caller may want to call again (NeedMore)
        poll_fn(|cx| {
            let _ = self.mode.tx_dma.wait_cell().poll_wait(cx);
            let _ = self.info.wait_cell().poll_wait(cx);

            self.info.regs().sier().write(|w| {
                w.set_feie(true);
                w.set_beie(true);
                w.set_sdie(true);
                w.set_rsie(true);
            });

            let ssr = self.info.regs().ssr().read();
            if ssr.fef() || ssr.bef() || ssr.sdf() || ssr.rsf() || self.mode.tx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // Cleanup
        self.info.regs().sder().modify(|w| w.set_tdde(false));
        unsafe {
            self.mode.tx_dma.disable_request();
            self.mode.tx_dma.clear_done();
        }

        let ssr = self.info.regs().ssr().read();

        if ssr.fef() {
            Err(IOError::FifoError)
        } else if ssr.bef() {
            Err(IOError::BitError)
        } else if ssr.sdf() {
            Ok(TxChunkOutcome::Stopped(self.mode.tx_dma.transferred_bytes()))
        } else if ssr.rsf() {
            Ok(TxChunkOutcome::Restarted(self.mode.tx_dma.transferred_bytes()))
        } else {
            // DMA done with no end-of-transfer flag: chunk exhausted,
            // controller still expects more bytes.
            Ok(TxChunkOutcome::NeedMore(chunk_len))
        }
    }
}

#[allow(private_bounds)]
impl<'d, M: AsyncMode> I2c<'d, M>
where
    Self: AsyncEngine,
{
    /// Enable only the interrupts relevant to listening for a new address match.
    fn enable_listen_ints(&self) {
        self.info.regs().sier().write(|w| {
            w.set_sarie(self.smbus_alert.clone().into());
            w.set_gcie(self.general_call.clone().into());
            w.set_am1ie(true);
            w.set_am0ie(true);
            w.set_feie(true);
            w.set_beie(true);
            w.set_sdie(true);
            w.set_rsie(true);
            w.set_avie(true);
        });
    }

    /// Enable only the interrupts relevant to receiving data (respond_to_write).
    fn enable_rx_ints(&self) {
        self.info.regs().sier().write(|w| {
            w.set_feie(true);
            w.set_beie(true);
            w.set_sdie(true);
            w.set_rsie(true);
            w.set_rdie(true);
        });
    }

    /// Enable only the interrupts relevant to transmitting data (respond_to_read).
    fn enable_tx_ints(&self) {
        self.info.regs().sier().write(|w| {
            w.set_feie(true);
            w.set_beie(true);
            w.set_sdie(true);
            w.set_rsie(true);
            w.set_tdie(true);
        });
    }

    // Public API: Async

    /// Asynchronously wait for new events.
    ///
    /// This function waits asynchronously for a new I2C event and returns the type of
    /// request made by the I2C controller.
    ///
    /// # Returns
    ///
    /// - `Ok(Request)` on success.
    /// - `Err(IOError)` if an error occurs.
    pub async fn async_listen(&mut self) -> Result<Request, IOError> {
        self.clear_status();

        self.info
            .wait_cell()
            .wait_for(|| {
                self.enable_listen_ints();
                let status = self.info.regs().ssr().read();
                status.avf() || status.sarf() || status.gcf() || status.sdf()
            })
            .await
            .map_err(|_| IOError::Other)?;

        let event = self.status()?;

        match event {
            Event::SmbusAlert => Ok(Request::SmbusAlert),
            Event::GeneralCall => Ok(Request::GeneralCall),
            Event::Stop(addr) => Ok(Request::Stop(addr >> 1)),
            Event::RepeatedStart(addr) | Event::AddressValid(addr) => {
                if addr & 1 != 0 {
                    Ok(Request::Read(addr >> 1))
                } else {
                    Ok(Request::Write(addr >> 1))
                }
            }
            _ => Err(IOError::Other),
        }
    }

    /// Asynchronously transmit data to the I2C controller.
    ///
    /// Sends the contents of the provided buffer to the I2C controller.
    /// The future services the transfer to a clean termination point
    /// (STOP, repeated START, or buffer exhausted) before resolving.
    ///
    /// If the controller continues clocking after the buffer has been
    /// fully transmitted (for example, an I2C-HID host that reads a fixed
    /// block size larger than the prepared response), this call resolves
    /// with [`ReadStatus::NeedMore`] so the caller can decide what to do:
    /// call `async_respond_to_read` again with more bytes (or fill data),
    /// or let the bus clock-stretch (with TXDSTALL enabled) until the
    /// controller eventually terminates the transfer.
    ///
    /// # Parameters
    ///
    /// - `buf`: The buffer containing the data to transmit.
    ///
    /// # Returns
    ///
    /// - `Ok(ReadStatus)` describing how the transfer ended and how many
    ///   bytes the controller ACKed.
    /// - `Err(IOError)` if an error occurs.
    pub fn async_respond_to_read<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<ReadStatus, IOError>> + 'a {
        <Self as AsyncEngine>::async_respond_to_read_internal(self, buf)
    }

    /// Asynchronously receive data from the I2C controller.
    ///
    /// Reads bytes the controller writes into the provided buffer. The
    /// future services the transfer to a clean termination point (STOP,
    /// repeated START, or buffer filled) before resolving.
    ///
    /// # Parameters
    ///
    /// - `buf`: The buffer to store the received data.
    ///
    /// # Returns
    ///
    /// - `Ok(WriteStatus)` describing how the transfer ended and how many
    ///   bytes the target received.
    /// - `Err(IOError)` if an error occurs.
    pub fn async_respond_to_write<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<WriteStatus, IOError>> + 'a {
        <Self as AsyncEngine>::async_respond_to_write_internal(self, buf)
    }
}

trait AsyncEngine {
    fn async_respond_to_read_internal<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<ReadStatus, IOError>> + 'a;

    fn async_respond_to_write_internal<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<WriteStatus, IOError>> + 'a;
}

impl<'d> AsyncEngine for I2c<'d, Async> {
    async fn async_respond_to_read_internal(&mut self, buf: &[u8]) -> Result<ReadStatus, IOError> {
        let mut count = 0;

        self.clear_status();

        for byte in buf.iter() {
            // Wait until we can send data
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.enable_tx_ints();
                    let ssr = self.info.regs().ssr().read();
                    ssr.tdf() || ssr.sdf() || ssr.rsf()
                })
                .await
                .map_err(|_| IOError::Other)?;

            let ssr = self.info.regs().ssr().read();
            if ssr.sdf() || ssr.rsf() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Send routine. STOP or Repeated-start received");
                self.reset_fifos();
                return Ok(ReadStatus::EarlyStop(count));
            }

            self.info.regs().stdr().write(|w| w.set_data(*byte));
            count += 1;
        }

        // All caller bytes pushed. Wait briefly to determine whether the
        // controller is done (NACK + STOP/RSTART) or whether it wants more.
        // We do NOT auto-pad here: doing so blocks the firmware for the
        // duration of the controller's extra reads, which causes us to fall
        // behind on subsequent back-to-back transactions. The caller
        // receives ReadStatus::NeedMore and decides how to proceed.
        self.info
            .wait_cell()
            .wait_for(|| {
                self.enable_tx_ints();
                let ssr = self.info.regs().ssr().read();
                ssr.tdf() || ssr.sdf() || ssr.rsf()
            })
            .await
            .map_err(|_| IOError::Other)?;

        let ssr = self.info.regs().ssr().read();
        if ssr.sdf() || ssr.rsf() {
            self.reset_fifos();
            Ok(ReadStatus::Complete(count))
        } else {
            // tdf set: TX FIFO empty during a transmit transfer means the
            // controller is still clocking and wants another byte.
            Ok(ReadStatus::NeedMore(count))
        }
    }

    async fn async_respond_to_write_internal(&mut self, buf: &mut [u8]) -> Result<WriteStatus, IOError> {
        let mut count = 0;

        self.clear_status();

        for byte in buf.iter_mut() {
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.enable_rx_ints();
                    let ssr = self.info.regs().ssr().read();
                    ssr.rdf() || ssr.sdf() || ssr.rsf()
                })
                .await
                .map_err(|_| IOError::Other)?;

            let ssr = self.info.regs().ssr().read();
            if ssr.sdf() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Receive routine. STOP received");
                self.reset_fifos();
                return Ok(WriteStatus::Stopped(count));
            }
            if ssr.rsf() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Receive routine. Repeated-start received");
                self.reset_fifos();
                return Ok(WriteStatus::Restarted(count));
            }

            *byte = self.info.regs().srdr().read().data();
            count += 1;
        }

        Ok(WriteStatus::BufferFull(count))
    }
}

impl<'d> AsyncEngine for I2c<'d, Dma<'d>> {
    async fn async_respond_to_read_internal(&mut self, buf: &[u8]) -> Result<ReadStatus, IOError> {
        let mut count = 0;

        self.clear_status();

        // perform corrective action if the future is dropped
        let on_drop = OnDrop::new(|| {
            self.info.regs().sder().modify(|w| w.set_tdde(false));
        });

        let total = buf.len();
        let mut chunks = buf.chunks(DMA_MAX_TRANSFER_SIZE).peekable();
        while let Some(chunk) = chunks.next() {
            let is_last = chunks.peek().is_none();
            match self.write_dma_chunk(chunk).await? {
                TxChunkOutcome::Stopped(n) => {
                    count += n;
                    on_drop.defuse();
                    return Ok(if is_last && count == total {
                        ReadStatus::Complete(count)
                    } else {
                        ReadStatus::EarlyStop(count)
                    });
                }
                TxChunkOutcome::Restarted(n) => {
                    count += n;
                    on_drop.defuse();
                    return Ok(if is_last && count == total {
                        ReadStatus::Complete(count)
                    } else {
                        ReadStatus::EarlyStop(count)
                    });
                }
                TxChunkOutcome::NeedMore(n) => {
                    count += n;
                    if is_last {
                        on_drop.defuse();
                        return Ok(ReadStatus::NeedMore(count));
                    }
                    // Non-last chunk completed normally: proceed to next
                    // chunk. The bus will clock-stretch briefly between
                    // chunks while we reprogram the TCD.
                }
            }
        }

        // Reached only when buf was empty.
        on_drop.defuse();
        Ok(ReadStatus::NeedMore(count))
    }

    async fn async_respond_to_write_internal<'a>(&'a mut self, buf: &'a mut [u8]) -> Result<WriteStatus, IOError> {
        let mut count = 0;

        self.clear_status();

        // perform corrective action if the future is dropped
        let on_drop = OnDrop::new(|| {
            self.info.regs().sder().modify(|w| w.set_rdde(false));
        });

        let total = buf.len();
        let mut chunks = buf.chunks_mut(DMA_MAX_TRANSFER_SIZE).peekable();
        while let Some(chunk) = chunks.next() {
            let is_last = chunks.peek().is_none();
            match self.read_dma_chunk(chunk).await? {
                RxChunkOutcome::Stopped(n) => {
                    count += n;
                    on_drop.defuse();
                    return Ok(WriteStatus::Stopped(count));
                }
                RxChunkOutcome::Restarted(n) => {
                    count += n;
                    on_drop.defuse();
                    return Ok(WriteStatus::Restarted(count));
                }
                RxChunkOutcome::Filled(n) => {
                    count += n;
                    if is_last {
                        on_drop.defuse();
                        return Ok(WriteStatus::BufferFull(count));
                    }
                    // Non-last chunk filled: proceed to next chunk.
                }
            }
        }

        // Reached only when buf was empty.
        on_drop.defuse();
        let _ = total;
        Ok(WriteStatus::BufferFull(count))
    }
}

impl<'d, M: Mode> Drop for I2c<'d, M> {
    fn drop(&mut self) {
        self._scl.set_as_disabled();
        self._sda.set_as_disabled();
    }
}

// ---------------------------------------------------------------------------
// `embedded-mcu-hal` I2C target trait implementations
// ---------------------------------------------------------------------------
//
// These adapt the inherent blocking/async target API above to the generic
// `embedded_mcu_hal::i2c::target` traits, gated behind the `embedded-mcu-hal`
// cargo feature.
//
// Both 7-bit (`SevenBitAddress`) and 10-bit (`TenBitAddress`) address modes
// are supported. The blocking trait is implemented once, generic over the
// address width; the async trait is implemented once, generic over both the
// address width and the async mode (`Async` and `Dma`).
//
// Known deviations from the trait contract (see the module docs and the
// trait docs in `embedded_mcu_hal::i2c::target`):
//
//   * `listen` never returns `Request::RepeatedStart`. A repeated START is
//     folded into the next `Read`/`Write` event and additionally surfaced
//     through `WriteStatus::Restarted` / `ReadStatus::EarlyStop`. The
//     listen -> respond -> re-listen loop still behaves correctly; faithful
//     edge emission would require tracking the previous sub-transaction
//     address at the driver level.
//   * The `SevenBitAddress` implementation truncates the matched address to
//     seven bits (`as u8`). Use the `TenBitAddress` implementation for a
//     lossless 10-bit address.

#[cfg(feature = "embedded-mcu-hal")]
use embedded_mcu_hal::i2c::target as emh;
#[cfg(feature = "embedded-mcu-hal")]
use embedded_mcu_hal::i2c::{AddressMode, SevenBitAddress, TenBitAddress};

/// Map this driver's [`IOError`] onto the generic target [`emh::ErrorKind`].
#[cfg(feature = "embedded-mcu-hal")]
impl emh::Error for IOError {
    fn kind(&self) -> emh::ErrorKind {
        match self {
            // FIFO over/underrun maps precisely onto `Overrun`.
            IOError::FifoError => emh::ErrorKind::Overrun,
            // A bit error is an illegal state on the bus lines.
            IOError::BitError => emh::ErrorKind::Bus,
            // A busy bus is a bus-level condition.
            IOError::BusBusy => emh::ErrorKind::Bus,
            IOError::TargetBusy => emh::ErrorKind::Other,
            IOError::Other => emh::ErrorKind::Other,
        }
    }
}

#[cfg(feature = "embedded-mcu-hal")]
impl<'d, M: Mode> emh::ErrorType for I2c<'d, M> {
    type Error = IOError;
}

#[cfg(feature = "embedded-mcu-hal")]
impl From<ReadStatus> for emh::ReadStatus {
    fn from(value: ReadStatus) -> Self {
        match value {
            ReadStatus::Complete(n) => emh::ReadStatus::Complete(n),
            ReadStatus::NeedMore(n) => emh::ReadStatus::NeedMore(n),
            ReadStatus::EarlyStop(n) => emh::ReadStatus::EarlyStop(n),
        }
    }
}

#[cfg(feature = "embedded-mcu-hal")]
impl From<WriteStatus> for emh::WriteStatus {
    fn from(value: WriteStatus) -> Self {
        match value {
            WriteStatus::Stopped(n) => emh::WriteStatus::Stopped(n),
            WriteStatus::Restarted(n) => emh::WriteStatus::Restarted(n),
            WriteStatus::BufferFull(n) => emh::WriteStatus::BufferFull(n),
        }
    }
}

/// 7-bit view of a matched-address [`Request`]. The matched address is
/// truncated to seven bits (`as u8`).
///
/// The inherent `listen` methods never produce a repeated-start event, so
/// this conversion never yields [`emh::Request::RepeatedStart`].
#[cfg(feature = "embedded-mcu-hal")]
impl From<Request> for emh::Request<SevenBitAddress> {
    fn from(value: Request) -> Self {
        match value {
            Request::Read(addr) => emh::Request::Read(addr as u8),
            Request::Write(addr) => emh::Request::Write(addr as u8),
            Request::Stop(addr) => emh::Request::Stop(addr as u8),
            Request::GeneralCall => emh::Request::GeneralCall,
            Request::SmbusAlert => emh::Request::SmbusAlert,
        }
    }
}

/// 10-bit (lossless) view of a matched-address [`Request`].
///
/// The inherent `listen` methods never produce a repeated-start event, so
/// this conversion never yields [`emh::Request::RepeatedStart`].
#[cfg(feature = "embedded-mcu-hal")]
impl From<Request> for emh::Request<TenBitAddress> {
    fn from(value: Request) -> Self {
        match value {
            Request::Read(addr) => emh::Request::Read(addr),
            Request::Write(addr) => emh::Request::Write(addr),
            Request::Stop(addr) => emh::Request::Stop(addr),
            Request::GeneralCall => emh::Request::GeneralCall,
            Request::SmbusAlert => emh::Request::SmbusAlert,
        }
    }
}

#[cfg(feature = "embedded-mcu-hal")]
impl<'d, M: Mode> I2c<'d, M> {
    /// Bring the target back to a known-clean baseline while preserving the
    /// configured addressing, general-call / SMBus-alert settings, and
    /// clocking.
    ///
    /// This is the shared implementation behind the blocking and async
    /// `recover` trait methods. All work is synchronous register access
    /// performed inside a single critical section, which makes the async
    /// wrapper trivially cancellation-safe and re-entrant.
    fn recover_inner(&self) {
        critical_section::with(|_| {
            // Stop driving SCL/SDA by disabling the target.
            self.info.regs().scr().modify(|w| w.set_sen(false));

            // Drop any in-flight FIFO bytes.
            self.info.regs().scr().modify(|w| {
                w.set_rtf(ScrRtf::NowEmpty);
                w.set_rrf(ScrRrf::NowEmpty);
            });

            // Disable any DMA request enables left set by a cancelled DMA
            // respond future.
            self.info.regs().sder().modify(|w| {
                w.set_tdde(false);
                w.set_rdde(false);
            });

            // Mask all target interrupts (write 0). The async wait helpers
            // re-enable exactly the interrupts they need before awaiting.
            self.info.regs().sier().write(|_| {});

            // Clear latched bus-event status.
            self.clear_status();

            // Re-enable the target. Addressing (SAMR/SCFGR1) and clocking are
            // untouched, so the next `listen` accepts a fresh transaction
            // without re-initialising the driver.
            self.info.regs().scr().modify(|w| w.set_sen(true));
        });
    }
}

#[cfg(feature = "embedded-mcu-hal")]
impl<'d, A: AddressMode> emh::blocking::I2c<A> for I2c<'d, Blocking>
where
    Request: Into<emh::Request<A>>,
{
    fn recover(&mut self) -> Result<(), Self::Error> {
        self.recover_inner();
        Ok(())
    }

    fn listen(&mut self) -> Result<emh::Request<A>, Self::Error> {
        self.blocking_listen().map(Into::into)
    }

    fn respond_to_read(&mut self, buf: &[u8]) -> Result<emh::ReadStatus, Self::Error> {
        self.blocking_respond_to_read(buf).map(Into::into)
    }

    fn respond_to_write(&mut self, buf: &mut [u8]) -> Result<emh::WriteStatus, Self::Error> {
        self.blocking_respond_to_write(buf).map(Into::into)
    }
}

#[cfg(feature = "embedded-mcu-hal")]
#[allow(private_bounds)]
impl<'d, M: AsyncMode, A: AddressMode> emh::asynch::I2c<A> for I2c<'d, M>
where
    Self: AsyncEngine,
    Request: Into<emh::Request<A>>,
{
    async fn recover(&mut self) -> Result<(), Self::Error> {
        self.recover_inner();
        Ok(())
    }

    async fn listen(&mut self) -> Result<emh::Request<A>, Self::Error> {
        self.async_listen().await.map(Into::into)
    }

    async fn respond_to_read(&mut self, buf: &[u8]) -> Result<emh::ReadStatus, Self::Error> {
        self.async_respond_to_read(buf).await.map(Into::into)
    }

    async fn respond_to_write(&mut self, buf: &mut [u8]) -> Result<emh::WriteStatus, Self::Error> {
        self.async_respond_to_write(buf).await.map(Into::into)
    }
}
