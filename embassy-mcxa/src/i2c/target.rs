//! LPI2C target driver

use core::marker::PhantomData;
use core::ops::Range;

use embassy_hal_internal::Peri;
use mcxa_pac::lpi2c0::scfgr1::Addrcfg;
use mcxa_pac::lpi2c0::sier::{Gcie, Sarie};

use super::{Async, Blocking, Info, Instance, Mode, SclPin, SdaPin};
pub use crate::clocks::PoweredClock;
pub use crate::clocks::periph_helpers::{Div4, Lpi2cClockSel, Lpi2cConfig};
use crate::clocks::{ClockError, WakeGuard, enable_and_reset};
use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;

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

/// I2C interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        if T::info().regs().sier().read().bits() != 0 {
            T::info().regs().sier().write(|w| {
                w.tdie()
                    .disabled()
                    .rdie()
                    .disabled()
                    .avie()
                    .disabled()
                    .taie()
                    .disabled()
                    .rsie()
                    .disabled()
                    .sdie()
                    .disabled()
                    .beie()
                    .disabled()
                    .feie()
                    .disabled()
                    .am0ie()
                    .disabled()
                    .am1ie()
                    .disabled()
                    .gcie()
                    .disabled()
                    .sarie()
                    .disabled()
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
#[derive(Clone, Default)]
pub enum Status {
    #[default]
    Disabled,
    Enabled,
}

impl From<Status> for Sarie {
    fn from(value: Status) -> Self {
        match value {
            Status::Disabled => Self::Disabled,
            Status::Enabled => Self::Enabled,
        }
    }
}

impl From<Status> for Gcie {
    fn from(value: Status) -> Self {
        match value {
            Status::Disabled => Self::Disabled,
            Status::Enabled => Self::Enabled,
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
#[derive(Clone)]
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
    _phantom: PhantomData<M>,
    smbus_alert: Status,
    general_call: Status,
    _wg: Option<WakeGuard>,
}

impl<'d, M: Mode> I2c<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
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
            _phantom: PhantomData,
            smbus_alert: config.smbus_alert.clone(),
            general_call: config.general_call.clone(),
            _wg: parts.wake_guard,
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    fn set_configuration(&self, config: &Config) -> Result<(), SetupError> {
        critical_section::with(|_| {
            // Disable the target.
            self.info.regs().scr().modify(|_, w| w.sen().disabled());

            // Soft-reset the target, read and write FIFOs.
            self.reset_fifos();
            self.info.regs().scr().modify(|_, w| w.rst().reset());
            // According to Reference Manual section 40.7.1.4, "There
            // is no minimum delay required before clearing the
            // software reset", therefore we clear it immediately.
            self.info.regs().scr().modify(|_, w| w.rst().not_reset());

            self.info
                .regs()
                .scr()
                .modify(|_, w| w.filtdz().filter_disabled().filten().disable());

            self.info
                .regs()
                .scfgr1()
                .modify(|_, w| w.rxstall().enabled().txdstall().enabled());

            // Configure address matching
            match config.address {
                Address::Single(addr) => {
                    self.info.regs().samr().write(|w| unsafe { w.addr0().bits(addr) });
                    self.info.regs().scfgr1().modify(|_, w| {
                        w.addrcfg().variant(if (0x00..=0x7f).contains(&addr) {
                            Addrcfg::AddressMatch0_7Bit
                        } else {
                            Addrcfg::AddressMatch0_10Bit
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

                    self.info
                        .regs()
                        .samr()
                        .write(|w| unsafe { w.addr0().bits(addr0).addr1().bits(addr1) });
                    self.info.regs().scfgr1().modify(|_, w| {
                        w.addrcfg().variant(if (0x00..=0x7f).contains(&addr0) {
                            Addrcfg::AddressMatch0_7BitOrAddressMatch1_7Bit
                        } else {
                            Addrcfg::AddressMatch0_10BitOrAddressMatch1_10Bit
                        })
                    });
                }

                Address::Range(Range { start, end }) => {
                    if ((0x00..=0x7f).contains(&start) ^ (0x00..=0x7f).contains(&end))
                        || ((0x80..=0x3ff).contains(&start) ^ (0x80..=0x3ff).contains(&end))
                    {
                        return Err(SetupError::InvalidAddress);
                    }

                    self.info
                        .regs()
                        .samr()
                        .write(|w| unsafe { w.addr0().bits(start).addr1().bits(end) });
                    self.info.regs().scfgr1().modify(|_, w| {
                        w.addrcfg().variant(if (0x00..=0x7f).contains(&start) {
                            Addrcfg::FromAddressMatch0_7BitToAddressMatch1_7Bit
                        } else {
                            Addrcfg::FromAddressMatch0_10BitToAddressMatch1_10Bit
                        })
                    });
                }
            }

            // Enable the target.
            self.info.regs().scr().modify(|_, w| w.sen().enabled());

            // Clear all flags
            self.info.regs().ssr().write(|w| {
                w.rsf()
                    .clear_bit_by_one()
                    .sdf()
                    .clear_bit_by_one()
                    .bef()
                    .clear_bit_by_one()
                    .fef()
                    .clear_bit_by_one()
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
            self.info
                .regs()
                .scr()
                .modify(|_, w| w.rtf().now_empty().rrf().now_empty());
        });
    }

    fn clear_status(&self) {
        self.info.regs().ssr().write(|w| {
            w.rsf()
                .clear_bit_by_one()
                .sdf()
                .clear_bit_by_one()
                .bef()
                .clear_bit_by_one()
                .fef()
                .clear_bit_by_one()
        });
    }

    /// Reads and parses the target status producing an
    /// appropriate `Result<(), Error>` variant.
    fn status(&self) -> Result<Event, IOError> {
        let ssr = self.info.regs().ssr().read();
        self.clear_status();

        if ssr.avf().bit_is_set() {
            let sasr = self.info.regs().sasr().read();
            let addr = sasr.raddr().bits();
            Ok(Event::AddressValid(addr))
        } else if ssr.taf().bit_is_set() {
            Ok(Event::TransmitAck)
        } else if ssr.rsf().bit_is_set() {
            let sasr = self.info.regs().sasr().read();
            let addr = sasr.raddr().bits();
            Ok(Event::RepeatedStart(addr))
        } else if ssr.sdf().bit_is_set() {
            let sasr = self.info.regs().sasr().read();
            let addr = sasr.raddr().bits();
            Ok(Event::Stop(addr))
        } else if ssr.bef().bit_is_set() {
            Err(IOError::BitError)
        } else if ssr.fef().bit_is_set() {
            Err(IOError::FifoError)
        } else if ssr.gcf().bit_is_set() {
            Ok(Event::GeneralCall)
        } else if ssr.sarf().bit_is_set() {
            Ok(Event::SmbusAlert)
        } else {
            Err(IOError::Other)
        }
    }

    // Public API: Blocking

    /// Block waiting for new events
    pub fn blocking_listen(&mut self) -> Result<Request, IOError> {
        self.clear_status();

        // Wait for Address Valid
        loop {
            let ssr = self.info.regs().ssr().read();
            let avr = ssr.avf().bit_is_set();
            let sarf = ssr.sarf().bit_is_set();
            let gcf = ssr.gcf().bit_is_set();

            if avr || sarf || gcf {
                break;
            }
        }

        let event = self.status()?;

        match event {
            Event::SmbusAlert => Ok(Request::SmbusAlert),
            Event::GeneralCall => Ok(Request::GeneralCall),
            Event::Stop(addr) => return Ok(Request::Stop(addr >> 1)),
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

    /// Transmit the contents of `buf` to the I2C controller.
    ///
    /// Returns either an `Ok(usize)` containing the number of bytes
    /// transmitted, or an `Error`.
    pub fn blocking_respond_to_read(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        let mut count = 0;

        self.clear_status();

        for byte in buf.iter() {
            // Wait until we can send data
            let ssr = loop {
                let ssr = self.info.regs().ssr().read();
                let tdf = ssr.tdf().bit_is_set();
                let sdf = ssr.sdf().bit_is_set();
                let rsf = ssr.rsf().bit_is_set();

                if tdf || sdf || rsf {
                    break ssr;
                }
            };

            // If we see a STOP or REPEATED START, break out
            if ssr.sdf().bit_is_set() || ssr.rsf().bit_is_set() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Send routine. STOP or Repeated-start received");
                break;
            } else {
                self.info.regs().stdr().write(|w| unsafe { w.data().bits(*byte) });
                count += 1;
            }
        }

        Ok(count)
    }

    /// Receive data from the I2C controller into `buf`.
    ///
    /// Care is taken to guarantee that we receive at most `buf.len()`
    /// bytes. On success returns `Ok(usize)` containing the number of
    /// bytes received or an `Error`.
    pub fn blocking_respond_to_write(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        let mut count = 0;

        self.clear_status();

        for byte in buf.iter_mut() {
            // Wait until we have data to read
            let ssr = loop {
                let ssr = self.info.regs().ssr().read();
                let rdf = ssr.rdf().bit_is_set();
                let sdf = ssr.sdf().bit_is_set();
                let rsf = ssr.rsf().bit_is_set();

                if rdf || sdf || rsf {
                    break ssr;
                }
            };

            // If we see a STOP or REPEATED START, break out
            if ssr.sdf().bit_is_set() || ssr.rsf().bit_is_set() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Receive routine. STOP or Repeated-start received");
                break;
            } else {
                *byte = self.info.regs().srdr().read().data().bits();
                count += 1;
            }
        }

        Ok(count)
    }
}

impl<'d> I2c<'d, Blocking> {
    /// Create a new blocking instance of the I2C Target bus driver.
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

impl<'d> I2c<'d, Async> {
    /// Create a new blocking instance of the I2C Target bus driver.
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

    fn enable_ints(&self) {
        self.info.regs().sier().write(|w| {
            w.sarie()
                .variant(self.smbus_alert.clone().into())
                .gcie()
                .variant(self.general_call.clone().into())
                .am1ie()
                .enabled()
                .am0ie()
                .enabled()
                .feie()
                .enabled()
                .beie()
                .enabled()
                .sdie()
                .enabled()
                .rsie()
                .enabled()
                .taie()
                .enabled()
                .avie()
                .enabled()
                .rdie()
                .enabled()
                .tdie()
                .enabled()
        });
    }

    // Public API: Async

    /// Asynchronously wait for new events
    pub async fn async_listen(&mut self) -> Result<Request, IOError> {
        self.clear_status();

        self.info
            .wait_cell()
            .wait_for(|| {
                self.enable_ints();
                self.info.regs().ssr().read().avf().bit_is_set()
                    || self.info.regs().ssr().read().sarf().bit_is_set()
                    || self.info.regs().ssr().read().gcf().bit_is_set()
            })
            .await
            .map_err(|_| IOError::Other)?;

        let event = self.status()?;

        match event {
            Event::SmbusAlert => Ok(Request::SmbusAlert),
            Event::GeneralCall => Ok(Request::GeneralCall),
            Event::Stop(addr) => return Ok(Request::Stop(addr >> 1)),
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

    /// Asynchronously transmit the contents of `buf` to the I2C
    /// controller.
    ///
    /// Returns either an `Ok(usize)` containing the number of bytes
    /// transmitted, or an `Error`.
    pub async fn async_respond_to_read(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        let mut count = 0;

        self.clear_status();

        for byte in buf.iter() {
            // Wait until we can send data
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.enable_ints();
                    let ssr = self.info.regs().ssr().read();
                    ssr.tdf().bit_is_set() || ssr.sdf().bit_is_set() || ssr.rsf().bit_is_set()
                })
                .await
                .map_err(|_| IOError::Other)?;

            // If we see a STOP or REPEATED START, break out
            let ssr = self.info.regs().ssr().read();
            if ssr.sdf().bit_is_set() || ssr.rsf().bit_is_set() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Send routine. STOP or Repeated-start received");
                self.reset_fifos();
                break;
            } else {
                self.info.regs().stdr().write(|w| unsafe { w.data().bits(*byte) });
                count += 1;
            }
        }

        Ok(count)
    }

    /// Asynchronously receive data from the I2C controller into
    /// `buf`.
    ///
    /// Care is taken to guarantee that we receive at most `buf.len()`
    /// bytes. On success returns `Ok(usize)` containing the number of
    /// bytes received or an `Error`.
    pub async fn async_respond_to_write(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        let mut count = 0;

        self.clear_status();

        for byte in buf.iter_mut() {
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.enable_ints();
                    let ssr = self.info.regs().ssr().read();
                    ssr.rdf().bit_is_set() || ssr.sdf().bit_is_set() || ssr.rsf().bit_is_set()
                })
                .await
                .map_err(|_| IOError::Other)?;

            // If we see a STOP or REPEATED START, break out
            let ssr = self.info.regs().ssr().read();
            if ssr.sdf().bit_is_set() || ssr.rsf().bit_is_set() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Receive routine. STOP or Repeated-start received");
                self.reset_fifos();
                break;
            } else {
                *byte = self.info.regs().srdr().read().data().bits();
                count += 1;
            }
        }

        Ok(count)
    }
}

impl<'d, M: Mode> Drop for I2c<'d, M> {
    fn drop(&mut self) {
        self._scl.set_as_disabled();
        self._sda.set_as_disabled();
    }
}
