//! LPI2C target driver

use core::marker::PhantomData;
use core::ops::Range;
use core::sync::atomic::{Ordering, fence};

use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;

use super::{Async, AsyncMode, Blocking, Dma, Info, Instance, Mode, RxDma, SclPin, SdaPin, TxDma};
pub use crate::clocks::PoweredClock;
pub use crate::clocks::periph_helpers::{Div4, Lpi2cClockSel, Lpi2cConfig};
use crate::clocks::{ClockError, WakeGuard, enable_and_reset};
use crate::dma::{DMA_MAX_TRANSFER_SIZE, DmaChannel, EnableInterrupt};
use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::lpi2c::vals::{Addrcfg, Filtdz, ScrRrf, ScrRtf};

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
#[derive(Clone, Default)]
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
                w.set_filtdz(Filtdz::FILTER_DISABLED);
                w.set_filten(false);
            });

            self.info.regs().scfgr1().modify(|w| {
                w.set_rxstall(true);
                w.set_txdstall(true);
            });

            // Configure address matching
            match config.address {
                Address::Single(addr) => {
                    self.info.regs().samr().write(|w| w.set_addr0(addr));
                    self.info.regs().scfgr1().modify(|w| {
                        w.set_addrcfg(if (0x00..=0x7f).contains(&addr) {
                            Addrcfg::ADDRESS_MATCH0_7_BIT
                        } else {
                            Addrcfg::ADDRESS_MATCH0_10_BIT
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
                            Addrcfg::ADDRESS_MATCH0_7_BIT_OR_ADDRESS_MATCH1_7_BIT
                        } else {
                            Addrcfg::ADDRESS_MATCH0_10_BIT_OR_ADDRESS_MATCH1_10_BIT
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
                            Addrcfg::FROM_ADDRESS_MATCH0_7_BIT_TO_ADDRESS_MATCH1_7_BIT
                        } else {
                            Addrcfg::FROM_ADDRESS_MATCH0_10_BIT_TO_ADDRESS_MATCH1_10_BIT
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
                w.set_rtf(ScrRtf::NOW_EMPTY);
                w.set_rrf(ScrRrf::NOW_EMPTY);
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

        if ssr.avf() {
            let sasr = self.info.regs().sasr().read();
            let addr = sasr.raddr();
            Ok(Event::AddressValid(addr))
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
        } else if ssr.bef() {
            Err(IOError::BitError)
        } else if ssr.fef() {
            Err(IOError::FifoError)
        } else if ssr.gcf() {
            Ok(Event::GeneralCall)
        } else if ssr.sarf() {
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
            let avr = ssr.avf();
            let sarf = ssr.sarf();
            let gcf = ssr.gcf();

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
                let tdf = ssr.tdf();
                let sdf = ssr.sdf();
                let rsf = ssr.rsf();

                if tdf || sdf || rsf {
                    break ssr;
                }
            };

            // If we see a STOP or REPEATED START, break out
            if ssr.sdf() || ssr.rsf() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Send routine. STOP or Repeated-start received");
                break;
            } else {
                self.info.regs().stdr().write(|w| w.set_data(*byte));
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
                let rdf = ssr.rdf();
                let sdf = ssr.sdf();
                let rsf = ssr.rsf();

                if rdf || sdf || rsf {
                    break ssr;
                }
            };

            // If we see a STOP or REPEATED START, break out
            if ssr.sdf() || ssr.rsf() {
                #[cfg(feature = "defmt")]
                defmt::trace!("Early stop of Target Receive routine. STOP or Repeated-start received");
                break;
            } else {
                *byte = self.info.regs().srdr().read().data();
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
        Self::new_inner(peri, scl, sda, config, Blocking)
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

        Self::new_inner(peri, scl, sda, config, Async)
    }
}

impl<'d> I2c<'d, Dma<'d>> {
    /// Create a new async instance of the I2C Controller bus driver with DMA support.
    ///
    /// Any external pin will be placed into Disabled state upon Drop,
    /// additionally, the DMA channel is disabled.
    pub fn new_async_with_dma<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        tx_dma: Peri<'d, impl TxDma<T>>,
        rx_dma: Peri<'d, impl RxDma<T>>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, SetupError> {
        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        // grab request numbers
        let tx_request_number = tx_dma.request_number();
        let rx_request_number = rx_dma.request_number();

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
                tx_request_number,
                rx_request_number,
            },
        )
    }

    async fn read_dma_chunk(&mut self, data: &mut [u8]) -> Result<usize, IOError> {
        let peri_addr = self.info.regs().srdr().as_ptr() as *const u8;

        self.clear_status();

        unsafe {
            // Clean up channel state
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();
            self.mode.rx_dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            self.mode.rx_dma.set_request_source(self.mode.rx_request_number);

            // Configure TCD for memory-to-peripheral transfer
            self.mode
                .rx_dma
                .setup_read_from_peripheral(peri_addr, data, EnableInterrupt::Yes);

            // Enable I2C RX DMA request
            self.info.regs().sder().modify(|w| w.set_rdde(true));

            // Enable DMA channel request
            self.mode.rx_dma.enable_request();
        }

        // Wait until STOP or REPEATED START
        self.info
            .wait_cell()
            .wait_for(|| {
                self.info.regs().sier().write(|w| {
                    w.set_feie(true);
                    w.set_beie(true);
                    w.set_sdie(true);
                    w.set_rsie(true);
                });
                let ssr = self.info.regs().ssr().read();
                ssr.fef() || ssr.bef() || ssr.sdf() || ssr.rsf()
            })
            .await
            .map_err(|_| IOError::Other)?;

        // Cleanup
        self.info.regs().sder().modify(|w| w.set_rdde(false));
        unsafe {
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();
        }

        // Ensure all writes by DMA are visible to the CPU
        // TODO: ensure this is done internal to the DMA methods so individual drivers
        // don't need to handle this?
        fence(Ordering::Acquire);

        let ssr = self.info.regs().ssr().read();

        if ssr.fef() {
            Err(IOError::FifoError)
        } else if ssr.bef() {
            Err(IOError::BitError)
        } else if ssr.sdf() || ssr.rsf() {
            Ok(self.mode.rx_dma.transferred_bytes())
        } else {
            Err(IOError::Other)
        }
    }

    async fn write_dma_chunk(&mut self, data: &[u8]) -> Result<usize, IOError> {
        let peri_addr = self.info.regs().stdr().as_ptr() as *mut u8;

        self.clear_status();

        unsafe {
            // Clean up channel state
            self.mode.tx_dma.disable_request();
            self.mode.tx_dma.clear_done();
            self.mode.tx_dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            self.mode.tx_dma.set_request_source(self.mode.tx_request_number);

            // Configure TCD for memory-to-peripheral transfer
            self.mode
                .tx_dma
                .setup_write_to_peripheral(data, peri_addr, EnableInterrupt::No);

            // Ensure all writes by DMA are visible to the CPU
            // TODO: ensure this is done internal to the DMA methods so individual drivers
            // don't need to handle this?
            fence(Ordering::Release);

            // Enable I2C TX DMA request
            self.info.regs().sder().modify(|w| w.set_tdde(true));

            // Enable DMA channel request
            self.mode.tx_dma.enable_request();
        }

        // Wait until STOP or REPEATED START
        self.info
            .wait_cell()
            .wait_for(|| {
                self.info.regs().sier().write(|w| {
                    w.set_feie(true);
                    w.set_beie(true);
                    w.set_sdie(true);
                    w.set_rsie(true);
                });
                let ssr = self.info.regs().ssr().read();
                ssr.fef() || ssr.bef() || ssr.sdf() || ssr.rsf()
            })
            .await
            .map_err(|_| IOError::Other)?;

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
        } else if ssr.sdf() || ssr.rsf() {
            Ok(self.mode.tx_dma.transferred_bytes())
        } else {
            Err(IOError::Other)
        }
    }
}

#[allow(private_bounds)]
impl<'d, M: AsyncMode> I2c<'d, M>
where
    Self: AsyncEngine,
{
    fn enable_ints(&self) {
        self.info.regs().sier().write(|w| {
            w.set_sarie(self.smbus_alert.clone().into());
            w.set_gcie(self.general_call.clone().into());
            w.set_am1ie(true);
            w.set_am0ie(true);
            w.set_feie(true);
            w.set_beie(true);
            w.set_sdie(true);
            w.set_rsie(true);
            w.set_taie(true);
            w.set_avie(true);
            w.set_rdie(true);
            w.set_tdie(true);
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
                self.info.regs().ssr().read().avf()
                    || self.info.regs().ssr().read().sarf()
                    || self.info.regs().ssr().read().gcf()
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
    pub fn async_respond_to_read<'a>(&'a mut self, buf: &'a [u8]) -> impl Future<Output = Result<usize, IOError>> + 'a {
        <Self as AsyncEngine>::async_respond_to_read_internal(self, buf)
    }

    /// Asynchronously receive data from the I2C controller into
    /// `buf`.
    ///
    /// Care is taken to guarantee that we receive at most `buf.len()`
    /// bytes. On success returns `Ok(usize)` containing the number of
    /// bytes received or an `Error`.
    pub fn async_respond_to_write<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<usize, IOError>> + 'a {
        <Self as AsyncEngine>::async_respond_to_write_internal(self, buf)
    }
}

trait AsyncEngine {
    fn async_respond_to_read_internal<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<usize, IOError>> + 'a;

    fn async_respond_to_write_internal<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<usize, IOError>> + 'a;
}

impl<'d> AsyncEngine for I2c<'d, Async> {
    fn async_respond_to_read_internal<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<usize, IOError>> + 'a {
        async move {
            let mut count = 0;

            self.clear_status();

            for byte in buf.iter() {
                // Wait until we can send data
                self.info
                    .wait_cell()
                    .wait_for(|| {
                        self.enable_ints();
                        let ssr = self.info.regs().ssr().read();
                        ssr.tdf() || ssr.sdf() || ssr.rsf()
                    })
                    .await
                    .map_err(|_| IOError::Other)?;

                // If we see a STOP or REPEATED START, break out
                let ssr = self.info.regs().ssr().read();
                if ssr.sdf() || ssr.rsf() {
                    #[cfg(feature = "defmt")]
                    defmt::trace!("Early stop of Target Send routine. STOP or Repeated-start received");
                    self.reset_fifos();
                    break;
                } else {
                    self.info.regs().stdr().write(|w| w.set_data(*byte));
                    count += 1;
                }
            }

            Ok(count)
        }
    }

    fn async_respond_to_write_internal<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<usize, IOError>> + 'a {
        async move {
            let mut count = 0;

            self.clear_status();

            for byte in buf.iter_mut() {
                self.info
                    .wait_cell()
                    .wait_for(|| {
                        self.enable_ints();
                        let ssr = self.info.regs().ssr().read();
                        ssr.rdf() || ssr.sdf() || ssr.rsf()
                    })
                    .await
                    .map_err(|_| IOError::Other)?;

                // If we see a STOP or REPEATED START, break out
                let ssr = self.info.regs().ssr().read();
                if ssr.sdf() || ssr.rsf() {
                    #[cfg(feature = "defmt")]
                    defmt::trace!("Early stop of Target Receive routine. STOP or Repeated-start received");
                    self.reset_fifos();
                    break;
                } else {
                    *byte = self.info.regs().srdr().read().data();
                    count += 1;
                }
            }

            Ok(count)
        }
    }
}

impl<'d> AsyncEngine for I2c<'d, Dma<'d>> {
    fn async_respond_to_read_internal<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<usize, IOError>> + 'a {
        async move {
            let mut count = 0;

            self.clear_status();

            // perform corrective action if the future is dropped
            let on_drop = OnDrop::new(|| {
                self.info.regs().sder().modify(|w| w.set_tdde(false));
            });

            for chunk in buf.chunks(DMA_MAX_TRANSFER_SIZE) {
                count += self.write_dma_chunk(chunk).await?;
            }

            // defuse it if the future is not dropped
            on_drop.defuse();

            Ok(count)
        }
    }

    fn async_respond_to_write_internal<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<usize, IOError>> + 'a {
        async move {
            let mut count = 0;

            self.clear_status();

            // perform corrective action if the future is dropped
            let on_drop = OnDrop::new(|| {
                self.info.regs().sder().modify(|w| w.set_rdde(false));
            });

            for chunk in buf.chunks_mut(DMA_MAX_TRANSFER_SIZE) {
                count += self.read_dma_chunk(chunk).await?;
            }

            // defuse it if the future is not dropped
            on_drop.defuse();

            Ok(count)
        }
    }
}

impl<'d, M: Mode> Drop for I2c<'d, M> {
    fn drop(&mut self) {
        self._scl.set_as_disabled();
        self._sda.set_as_disabled();
    }
}
