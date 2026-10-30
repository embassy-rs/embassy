//! Master SPI driver for the ASR6601 SSP0–SSP2 (ARM PL022) blocks.
//!
//! Register programming follows the vendor `tremo_spi` driver. The PL022
//! register layout is fully described by the PAC field accessors.
//!
//! Alternate-function numbers come from datasheet Table 4-4 (Fun=4 for every
//! SSP signal listed there). Constructors either take typed pins that encode
//! that AF, or accept an explicit [`AlternateFunction`] per pin.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU8, Ordering};
use core::task::Poll;

use embassy_futures::join::join;
use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;
use embedded_hal::spi::{Mode as SpiMode, Phase, Polarity};

use crate::dma::{
    self, AddressMode, BurstSize, ChannelInstance, ControllerInstance, DataWidth as DmaWidth, Request, TransferConfig,
    TransferOptions,
};
use crate::gpio::{AlternateFunction, Flex, Pin as GpioPin, Pull};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt as TypelevelInterrupt};
use crate::mode::{Async, Blocking, Mode};

// The SSP RX DMA request is generated on four-byte FIFO boundaries. A
// one-item burst leaves the final three bytes in the FIFO, so use four-item
// bursts for both directions. This also works for lengths below four.
const DMA_OPTIONS: TransferOptions = TransferOptions {
    source_burst: BurstSize::Four,
    destination_burst: BurstSize::Four,
};
use crate::pac::ssp0::RegisterBlock;
use crate::rcc::{self, Peripheral};
use crate::time::Hertz;
use crate::{Peri, PeripheralType, interrupt, pac, peripherals};

// PL022 / tremo_spi register layout is fully described by the PAC field
// accessors used below; no hand-written bit positions remain in this driver.

const RESULT_OK: u8 = 0;
const RESULT_OVERRUN: u8 = 1;

/// SPI driver errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// RX FIFO overrun (ROR).
    Overrun,
    /// Embedded-hal byte transfers require [`DataWidth::Bits8`].
    InvalidDataWidth,
    /// DMA transfer failed.
    Dma(dma::Error),
}

/// SPI configuration error (returned by constructors and `set_config`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// Requested SCLK cannot be formed from the current PCLK dividers.
    InvalidFrequency,
    /// LSB-first transfers are not supported by the PL022 Motorola frame format.
    UnsupportedBitOrder,
    /// Peripheral clocks are not published yet (`rcc::init` has not completed).
    ClocksNotInitialized,
    /// RCC rejected the clock or reset request.
    Rcc(rcc::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Overrun => f.write_str("SPI RX overrun"),
            Self::InvalidDataWidth => f.write_str("SPI data width is not 8 bit"),
            Self::Dma(err) => write!(f, "SPI DMA error: {err}"),
        }
    }
}

impl core::error::Error for Error {}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFrequency => f.write_str("SPI frequency is out of range"),
            Self::UnsupportedBitOrder => f.write_str("SPI LSB-first is unsupported"),
            Self::ClocksNotInitialized => f.write_str("RCC clocks are not initialized"),
            Self::Rcc(err) => write!(f, "SPI RCC error: {err}"),
        }
    }
}

impl core::error::Error for ConfigError {}

impl embedded_hal::spi::Error for Error {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        match self {
            Self::Overrun => embedded_hal::spi::ErrorKind::Overrun,
            _ => embedded_hal::spi::ErrorKind::Other,
        }
    }
}

/// Bit order.
///
/// The ASR6601 SSP Motorola frame format only shifts MSB first. Selecting
/// [`BitOrder::LsbFirst`] returns [`ConfigError::UnsupportedBitOrder`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BitOrder {
    /// Most significant bit first (hardware default).
    MsbFirst,
    /// Least significant bit first (unsupported on this hardware).
    LsbFirst,
}

/// Frame format programmed into `CR0.FRF`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FrameFormat {
    /// Motorola SPI.
    Motorola,
    /// Texas Instruments synchronous serial.
    Ti,
    /// National Microwire.
    Microwire,
}

/// SSP data size (`CR0.DSS`), matching the vendor SDK constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DataWidth {
    /// 4-bit frames.
    Bits4,
    /// 8-bit frames.
    Bits8,
    /// 16-bit frames.
    Bits16,
}

impl DataWidth {
    const fn is_u16(self) -> bool {
        matches!(self, Self::Bits16)
    }
}

/// SPI configuration.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// SPI mode (clock polarity / phase).
    pub mode: SpiMode,
    /// Desired SCLK frequency.
    pub frequency: Hertz,
    /// Bit order. Only [`BitOrder::MsbFirst`] is accepted.
    pub bit_order: BitOrder,
    /// Frame data width.
    pub data_width: DataWidth,
    /// Frame format.
    pub frame_format: FrameFormat,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: SpiMode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            frequency: Hertz(1_000_000),
            bit_order: BitOrder::MsbFirst,
            data_width: DataWidth::Bits8,
            frame_format: FrameFormat::Motorola,
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Config {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Config {{ frequency: {=u32}, bit_order: {}, data_width: {}, frame_format: {} }}",
            self.frequency.hz(),
            self.bit_order,
            self.data_width,
            self.frame_format,
        );
    }
}

#[derive(Clone, Copy)]
enum PclkSel {
    Pclk0,
    Pclk1,
}

struct State {
    waker: AtomicWaker,
    result: AtomicU8,
}

impl State {
    const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            result: AtomicU8::new(RESULT_OK),
        }
    }
}

static STATE_SSP0: State = State::new();
static STATE_SSP1: State = State::new();
static STATE_SSP2: State = State::new();

struct Info {
    regs: *const RegisterBlock,
    peripheral: Peripheral,
    pclk: PclkSel,
    dma_tx: Request,
    dma_rx: Request,
    state: &'static State,
    interrupt: pac::Interrupt,
}

// Register blocks are Sync; the pointer is unique to the owned SSP instance.
unsafe impl Sync for Info {}

/// Interrupt handler for one SSP instance.
///
/// Clears RX overrun / timeout, masks TX/RX FIFO interrupts, and wakes any
/// async waiter registered by the driver.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        let regs = &*info.regs;
        let mis = regs.mis().read();

        if mis.rormis().bit_is_set() {
            info.state.result.store(RESULT_OVERRUN, Ordering::Release);
            unsafe {
                regs.icr().write_with_zero(|w| w.roric().set_bit());
            }
        }
        if mis.rtmis().bit_is_set() {
            unsafe {
                regs.icr().write_with_zero(|w| w.rtic().set_bit());
            }
        }

        // Mask level-sensitive FIFO interrupts; the waiter re-enables as needed.
        regs.imsc().modify(|_, w| {
            w.txim().clear_bit();
            w.rxim().clear_bit()
        });

        info.state.waker.wake();
    }
}

/// Master SPI driver.
pub struct Spi<'d, M: Mode> {
    info: &'static Info,
    _sck: Option<Flex<'d>>,
    _mosi: Option<Flex<'d>>,
    _miso: Option<Flex<'d>>,
    tx_dma: Option<dma::Channel<'d, Async>>,
    rx_dma: Option<dma::Channel<'d, Async>>,
    data_width: DataWidth,
    _mode: PhantomData<&'d mut M>,
}

fn configure_output_af<'d>(pin: Peri<'d, impl GpioPin>, af: AlternateFunction, pull: Pull) -> Flex<'d> {
    let mut flex = Flex::new(pin);
    flex.set_as_output();
    flex.set_pull(pull);
    flex.set_alternate_function(af);
    flex
}

fn configure_input_af<'d>(pin: Peri<'d, impl GpioPin>, af: AlternateFunction, pull: Pull) -> Flex<'d> {
    let mut flex = Flex::new(pin);
    flex.set_as_input();
    flex.set_pull(pull);
    flex.set_alternate_function(af);
    flex
}

fn sck_pull(mode: SpiMode) -> Pull {
    match mode.polarity {
        Polarity::IdleLow => Pull::Down,
        Polarity::IdleHigh => Pull::Up,
    }
}

fn calc_dividers(pclk: u32, freq: u32) -> Result<(u8, u8), ConfigError> {
    if freq == 0 || pclk == 0 {
        return Err(ConfigError::InvalidFrequency);
    }

    // SCLK = PCLK / (CPSDVSR * (SCR + 1)), CPSDVSR even in 2..=254, SCR in 0..=255.
    let mut best: Option<(u8, u8, u32)> = None;
    for cpsdvsr in (2u32..=254).step_by(2) {
        let base = pclk / cpsdvsr;
        if base < freq {
            continue;
        }
        let scr_plus = base / freq;
        if scr_plus == 0 || scr_plus > 256 {
            continue;
        }
        for candidate in (scr_plus.saturating_sub(1)..=(scr_plus + 1).min(256)).rev() {
            if candidate == 0 {
                continue;
            }
            let scr = candidate - 1;
            let actual = base / candidate;
            if actual == 0 {
                continue;
            }
            let err = actual.abs_diff(freq);
            match best {
                Some((_, _, best_err)) if err >= best_err => {}
                _ => best = Some((cpsdvsr as u8, scr as u8, err)),
            }
            if err == 0 {
                return Ok((cpsdvsr as u8, scr as u8));
            }
        }
    }

    best.map(|(c, s, _)| (c, s)).ok_or(ConfigError::InvalidFrequency)
}

impl<'d, M: Mode> Spi<'d, M> {
    /// Arm the NVIC vector for interrupt-driven use.
    ///
    /// Called by the async constructors only; blocking instances never
    /// enable the vector.
    fn enable_interrupts(&self) {
        self.info.interrupt.unpend();
        unsafe {
            self.info.interrupt.enable();
        }
    }

    fn new_inner<T: Instance>(
        _spi: Peri<'d, T>,
        sck: Option<Flex<'d>>,
        mosi: Option<Flex<'d>>,
        miso: Option<Flex<'d>>,
        tx_dma: Option<dma::Channel<'d, Async>>,
        rx_dma: Option<dma::Channel<'d, Async>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let info = T::info();

        rcc::enable_peripheral(info.peripheral).map_err(ConfigError::Rcc)?;
        rcc::reset_peripheral(info.peripheral).map_err(ConfigError::Rcc)?;

        let mut spi = Self {
            info,
            _sck: sck,
            _mosi: mosi,
            _miso: miso,
            tx_dma,
            rx_dma,
            data_width: config.data_width,
            _mode: PhantomData,
        };

        spi.configure(&config)?;

        info.state.result.store(RESULT_OK, Ordering::Release);
        unsafe {
            let regs = &*info.regs;
            regs.imsc().write_with_zero(|w| w);
            regs.icr().write_with_zero(|w| {
                w.roric().set_bit();
                w.rtic().set_bit()
            });
        }

        spi.set_enabled(true);
        Ok(spi)
    }

    fn configure(&mut self, config: &Config) -> Result<(), ConfigError> {
        if config.bit_order == BitOrder::LsbFirst {
            return Err(ConfigError::UnsupportedBitOrder);
        }

        let clocks = rcc::clocks().ok_or(ConfigError::ClocksNotInitialized)?;
        let pclk = match self.info.pclk {
            PclkSel::Pclk0 => clocks.pclk0_hz,
            PclkSel::Pclk1 => clocks.pclk1_hz,
        };
        let (cpsdvsr, scr) = calc_dividers(pclk, config.frequency.hz())?;

        let dss = match config.data_width {
            DataWidth::Bits4 => pac::ssp0::cr0::Dss::Value4,
            DataWidth::Bits8 => pac::ssp0::cr0::Dss::Value8,
            DataWidth::Bits16 => pac::ssp0::cr0::Dss::Value16,
        };
        let frf = match config.frame_format {
            FrameFormat::Motorola => pac::ssp0::cr0::Frf::Motorola,
            FrameFormat::Ti => pac::ssp0::cr0::Frf::Ti,
            FrameFormat::Microwire => pac::ssp0::cr0::Frf::Microwire,
        };

        self.set_enabled(false);
        unsafe {
            let regs = &*self.info.regs;
            regs.cpsr().write_with_zero(|w| w.cpsdvsr().bits(cpsdvsr));
            regs.cr0().write_with_zero(|w| {
                w.dss().variant(dss);
                w.frf().variant(frf);
                w.spo().bit(config.mode.polarity == Polarity::IdleHigh);
                w.sph().bit(config.mode.phase == Phase::CaptureOnSecondTransition);
                w.scr().bits(scr)
            });
            // Master mode, SSP disabled until set_enabled(true).
            regs.cr1().write_with_zero(|w| w);
            regs.dmacr().write_with_zero(|w| {
                w.txdmae().bit(self.tx_dma.is_some());
                w.rxdmae().bit(self.rx_dma.is_some())
            });
        }
        self.data_width = config.data_width;
        Ok(())
    }

    fn set_enabled(&mut self, enabled: bool) {
        let regs = unsafe { &*self.info.regs };
        regs.cr1().modify(|_, w| {
            // Master mode is fixed; enabling sets SSE, disabling clears it.
            w.ms().clear_bit();
            w.sse().bit(enabled)
        });
    }

    fn tx_fifo_not_full(&self) -> bool {
        unsafe { (*self.info.regs).sr().read().tnf().bit_is_set() }
    }

    fn rx_fifo_not_empty(&self) -> bool {
        unsafe { (*self.info.regs).sr().read().rne().bit_is_set() }
    }

    fn tx_fifo_empty(&self) -> bool {
        unsafe { (*self.info.regs).sr().read().tfe().bit_is_set() }
    }

    fn busy(&self) -> bool {
        unsafe { (*self.info.regs).sr().read().bsy().bit_is_set() }
    }

    fn take_error(&self) -> Result<(), Error> {
        match self.info.state.result.swap(RESULT_OK, Ordering::AcqRel) {
            RESULT_OVERRUN => Err(Error::Overrun),
            _ => Ok(()),
        }
    }

    fn drain_rx(&mut self) {
        unsafe {
            let regs = &*self.info.regs;
            while regs.sr().read().rne().bit_is_set() {
                let _ = regs.dr().read().data().bits();
            }
            regs.icr().write_with_zero(|w| {
                w.roric().set_bit();
                w.rtic().set_bit()
            });
        }
        self.info.state.result.store(RESULT_OK, Ordering::Release);
    }

    fn write_frame(&mut self, frame: u16) {
        unsafe {
            (*self.info.regs).dr().write_with_zero(|w| w.data().bits(frame));
        }
    }

    fn read_frame(&mut self) -> u16 {
        unsafe { (*self.info.regs).dr().read().data().bits() }
    }

    /// Reconfigure the SPI peripheral.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        self.configure(config)?;
        self.set_enabled(true);
        Ok(())
    }

    /// Change only the SCLK frequency.
    pub fn set_frequency(&mut self, frequency: Hertz) -> Result<(), ConfigError> {
        let clocks = rcc::clocks().ok_or(ConfigError::ClocksNotInitialized)?;
        let pclk = match self.info.pclk {
            PclkSel::Pclk0 => clocks.pclk0_hz,
            PclkSel::Pclk1 => clocks.pclk1_hz,
        };
        let (cpsdvsr, scr) = calc_dividers(pclk, frequency.hz())?;
        self.set_enabled(false);
        unsafe {
            let regs = &*self.info.regs;
            regs.cpsr().write_with_zero(|w| w.cpsdvsr().bits(cpsdvsr));
            regs.cr0().modify(|_, w| w.scr().bits(scr));
        }
        self.set_enabled(true);
        Ok(())
    }

    /// Block until the SSP is idle.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while self.busy() {}
        self.take_error()
    }

    /// Blocking write. Received bytes are discarded.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.ensure_u8()?;
        for &b in data {
            while !self.tx_fifo_not_full() {}
            self.write_frame(b as u16);
            while !self.rx_fifo_not_empty() {}
            let _ = self.read_frame();
        }
        self.blocking_flush()?;
        self.drain_rx();
        Ok(())
    }

    /// Blocking read. Dummy `0x00` frames are transmitted.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.blocking_transfer(data, &[])
    }

    /// Blocking full-duplex transfer.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.ensure_u8()?;
        let len = read.len().max(write.len());
        for i in 0..len {
            let tx = write.get(i).copied().unwrap_or(0);
            while !self.tx_fifo_not_full() {}
            self.write_frame(tx as u16);
            while !self.rx_fifo_not_empty() {}
            let rx = self.read_frame() as u8;
            if let Some(slot) = read.get_mut(i) {
                *slot = rx;
            }
        }
        self.blocking_flush()
    }

    /// Blocking in-place full-duplex transfer.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.ensure_u8()?;
        for b in data {
            while !self.tx_fifo_not_full() {}
            self.write_frame(*b as u16);
            while !self.rx_fifo_not_empty() {}
            *b = self.read_frame() as u8;
        }
        self.blocking_flush()
    }

    /// Blocking write of 16-bit frames. Requires [`DataWidth::Bits16`].
    pub fn blocking_write_u16(&mut self, data: &[u16]) -> Result<(), Error> {
        if !self.data_width.is_u16() {
            return Err(Error::InvalidDataWidth);
        }
        for &w in data {
            while !self.tx_fifo_not_full() {}
            self.write_frame(w);
            while !self.rx_fifo_not_empty() {}
            let _ = self.read_frame();
        }
        self.blocking_flush()?;
        self.drain_rx();
        Ok(())
    }

    /// Blocking read of 16-bit frames. Requires [`DataWidth::Bits16`].
    pub fn blocking_read_u16(&mut self, data: &mut [u16]) -> Result<(), Error> {
        if !self.data_width.is_u16() {
            return Err(Error::InvalidDataWidth);
        }
        for slot in data.iter_mut() {
            while !self.tx_fifo_not_full() {}
            self.write_frame(0);
            while !self.rx_fifo_not_empty() {}
            *slot = self.read_frame();
        }
        self.blocking_flush()
    }

    fn ensure_u8(&self) -> Result<(), Error> {
        if self.data_width == DataWidth::Bits8 {
            Ok(())
        } else {
            Err(Error::InvalidDataWidth)
        }
    }

    fn dr_ptr(&self) -> *mut u32 {
        unsafe { (*self.info.regs).dr().as_ptr() }
    }
}

impl<'d> Spi<'d, Blocking> {
    /// Create a blocking master with SCK, MOSI, and MISO.
    pub fn new_blocking<T: Instance, Sck: SckPin<T>, Mosi: MosiPin<T>, Miso: MisoPin<T>>(
        spi: Peri<'d, T>,
        sck: Peri<'d, Sck>,
        mosi: Peri<'d, Mosi>,
        miso: Peri<'d, Miso>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let pull = sck_pull(config.mode);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            Some(configure_output_af(mosi, Mosi::AF, Pull::None)),
            Some(configure_input_af(miso, Miso::AF, Pull::None)),
            None,
            None,
            config,
        )
    }

    /// Create a blocking master with explicit alternate-function numbers.
    #[allow(clippy::too_many_arguments)]
    pub fn new_blocking_af<T: Instance>(
        spi: Peri<'d, T>,
        sck: Peri<'d, impl GpioPin + 'd>,
        sck_af: AlternateFunction,
        mosi: Peri<'d, impl GpioPin + 'd>,
        mosi_af: AlternateFunction,
        miso: Peri<'d, impl GpioPin + 'd>,
        miso_af: AlternateFunction,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let pull = sck_pull(config.mode);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, sck_af, pull)),
            Some(configure_output_af(mosi, mosi_af, Pull::None)),
            Some(configure_input_af(miso, miso_af, Pull::None)),
            None,
            None,
            config,
        )
    }

    /// Blocking transmit-only master (MOSI + SCK).
    pub fn new_blocking_txonly<T: Instance, Sck: SckPin<T>, Mosi: MosiPin<T>>(
        spi: Peri<'d, T>,
        sck: Peri<'d, Sck>,
        mosi: Peri<'d, Mosi>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let pull = sck_pull(config.mode);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            Some(configure_output_af(mosi, Mosi::AF, Pull::None)),
            None,
            None,
            None,
            config,
        )
    }

    /// Blocking receive-only master (MISO + SCK).
    pub fn new_blocking_rxonly<T: Instance, Sck: SckPin<T>, Miso: MisoPin<T>>(
        spi: Peri<'d, T>,
        sck: Peri<'d, Sck>,
        miso: Peri<'d, Miso>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let pull = sck_pull(config.mode);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            None,
            Some(configure_input_af(miso, Miso::AF, Pull::None)),
            None,
            None,
            config,
        )
    }
}

impl<'d> Spi<'d, Async> {
    /// Create an async master using SSP FIFO interrupts (no DMA).
    pub fn new<T: Instance, Sck: SckPin<T>, Mosi: MosiPin<T>, Miso: MisoPin<T>>(
        spi: Peri<'d, T>,
        sck: Peri<'d, Sck>,
        mosi: Peri<'d, Mosi>,
        miso: Peri<'d, Miso>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let pull = sck_pull(config.mode);
        let this = Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            Some(configure_output_af(mosi, Mosi::AF, Pull::None)),
            Some(configure_input_af(miso, Miso::AF, Pull::None)),
            None,
            None,
            config,
        )?;
        this.enable_interrupts();
        Ok(this)
    }

    /// Create an async master with explicit alternate-function numbers.
    #[allow(clippy::too_many_arguments)]
    pub fn new_af<T: Instance>(
        spi: Peri<'d, T>,
        sck: Peri<'d, impl GpioPin + 'd>,
        sck_af: AlternateFunction,
        mosi: Peri<'d, impl GpioPin + 'd>,
        mosi_af: AlternateFunction,
        miso: Peri<'d, impl GpioPin + 'd>,
        miso_af: AlternateFunction,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let pull = sck_pull(config.mode);
        let this = Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, sck_af, pull)),
            Some(configure_output_af(mosi, mosi_af, Pull::None)),
            Some(configure_input_af(miso, miso_af, Pull::None)),
            None,
            None,
            config,
        )?;
        this.enable_interrupts();
        Ok(this)
    }

    /// Create an async transmit-only master (MOSI + SCK only).
    pub fn new_txonly<T: Instance, Sck: SckPin<T>, Mosi: MosiPin<T>>(
        spi: Peri<'d, T>,
        sck: Peri<'d, Sck>,
        mosi: Peri<'d, Mosi>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let pull = sck_pull(config.mode);
        let this = Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            Some(configure_output_af(mosi, Mosi::AF, Pull::None)),
            None,
            None,
            None,
            config,
        )?;
        this.enable_interrupts();
        Ok(this)
    }

    /// Create an async receive-only master (MISO + SCK only).
    pub fn new_rxonly<T: Instance, Sck: SckPin<T>, Miso: MisoPin<T>>(
        spi: Peri<'d, T>,
        sck: Peri<'d, Sck>,
        miso: Peri<'d, Miso>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let pull = sck_pull(config.mode);
        let this = Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            None,
            Some(configure_input_af(miso, Miso::AF, Pull::None)),
            None,
            None,
            config,
        )?;
        this.enable_interrupts();
        Ok(this)
    }

    /// Create an async master that uses DMA for transfers.
    ///
    /// Bind both the SSP interrupt (overrun / timeout) and the DMA controller
    /// interrupt(s) used by the supplied channels.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_dma<
        T: Instance,
        Sck: SckPin<T>,
        Mosi: MosiPin<T>,
        Miso: MisoPin<T>,
        TxDma: ChannelInstance,
        RxDma: ChannelInstance,
    >(
        spi: Peri<'d, T>,
        sck: Peri<'d, Sck>,
        mosi: Peri<'d, Mosi>,
        miso: Peri<'d, Miso>,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>>
        + Binding<<TxDma::Controller as ControllerInstance>::Interrupt, dma::InterruptHandler<TxDma::Controller>>
        + Binding<<RxDma::Controller as ControllerInstance>::Interrupt, dma::InterruptHandler<RxDma::Controller>>
        + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let pull = sck_pull(config.mode);
        let tx = dma::Channel::new(tx_dma, irq);
        let rx = dma::Channel::new(rx_dma, irq);
        let this = Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            Some(configure_output_af(mosi, Mosi::AF, Pull::None)),
            Some(configure_input_af(miso, Miso::AF, Pull::None)),
            Some(tx),
            Some(rx),
            config,
        )?;
        this.enable_interrupts();
        Ok(this)
    }

    /// Wait for a FIFO status condition, arming its interrupts.
    ///
    /// `ready` tests the condition; `enable_irqs` arms exactly the sources
    /// that can produce it (plus overrun). The waker registers before every
    /// check so no event is lost.
    async fn wait_fifo(
        &mut self,
        ready: impl Fn(&Self) -> bool,
        enable_irqs: impl Fn(&RegisterBlock, bool),
    ) -> Result<(), Error> {
        let regs = unsafe { &*self.info.regs };
        // If dropped mid-wait the FIFO interrupts would stay armed with no
        // waiter; disarm them. Fast paths never arm, so defuse is a no-op.
        let irq_guard = OnDrop::new(|| enable_irqs(regs, false));
        let result = poll_fn(|cx| {
            self.info.state.waker.register(cx.waker());
            if let Err(e) = self.take_error() {
                return Poll::Ready(Err(e));
            }
            if ready(self) {
                return Poll::Ready(Ok(()));
            }
            enable_irqs(regs, true);
            if ready(self) {
                return Poll::Ready(Ok(()));
            }
            if let Err(e) = self.take_error() {
                return Poll::Ready(Err(e));
            }
            Poll::Pending
        })
        .await;
        irq_guard.defuse();
        result
    }

    async fn wait_tnf(&mut self) -> Result<(), Error> {
        self.wait_fifo(
            |spi| spi.tx_fifo_not_full(),
            |regs, enable| {
                regs.imsc().modify(|_, w| {
                    w.txim().bit(enable);
                    w.rorim().bit(enable);
                    w
                });
            },
        )
        .await
    }

    async fn wait_rne(&mut self) -> Result<(), Error> {
        self.wait_fifo(
            |spi| spi.rx_fifo_not_empty(),
            |regs, enable| {
                regs.imsc().modify(|_, w| {
                    w.rxim().bit(enable);
                    w.rtim().bit(enable);
                    w.txim().bit(enable);
                    w.rorim().bit(enable);
                    w
                });
            },
        )
        .await
    }

    async fn wait_tfe(&mut self) -> Result<(), Error> {
        self.wait_fifo(
            |spi| spi.tx_fifo_empty(),
            |regs, enable| {
                regs.imsc().modify(|_, w| {
                    w.rxim().bit(enable);
                    w.rtim().bit(enable);
                    w.txim().bit(enable);
                    w.rorim().bit(enable);
                    w
                });
            },
        )
        .await
    }

    /// Async write using FIFO interrupts (or DMA when configured).
    pub async fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.ensure_u8()?;
        if data.is_empty() {
            return Ok(());
        }
        if self.tx_dma.is_some() {
            return self.write_dma(data).await;
        }
        for &b in data {
            self.wait_tnf().await?;
            self.write_frame(b as u16);
            self.wait_rne().await?;
            let _ = self.read_frame();
        }
        self.flush_async().await
    }

    /// Async read using FIFO interrupts (or DMA when configured).
    pub async fn read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.transfer(data, &[]).await
    }

    /// Async full-duplex transfer.
    pub async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.ensure_u8()?;
        if read.is_empty() && write.is_empty() {
            return Ok(());
        }
        if self.tx_dma.is_some() && self.rx_dma.is_some() {
            return self.transfer_dma(read, write).await;
        }

        let len = read.len().max(write.len());
        for i in 0..len {
            let tx = write.get(i).copied().unwrap_or(0);
            self.wait_tnf().await?;
            self.write_frame(tx as u16);
            self.wait_rne().await?;
            let rx = self.read_frame() as u8;
            if let Some(slot) = read.get_mut(i) {
                *slot = rx;
            }
        }
        self.flush_async().await
    }

    /// Async in-place full-duplex transfer.
    pub async fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.ensure_u8()?;
        if self.tx_dma.is_some() && self.rx_dma.is_some() {
            // SAFETY: exclusive borrow of `data`; DMA reads TX from the same
            // buffer while writing RX back into it, matching full-duplex SPI.
            let write = unsafe { core::slice::from_raw_parts(data.as_ptr(), data.len()) };
            return self.transfer_dma(data, write).await;
        }
        for b in data.iter_mut() {
            self.wait_tnf().await?;
            self.write_frame(*b as u16);
            self.wait_rne().await?;
            *b = self.read_frame() as u8;
        }
        self.flush_async().await
    }

    async fn flush_async(&mut self) -> Result<(), Error> {
        while self.busy() {
            self.wait_tfe().await?;
            if !self.busy() {
                break;
            }
        }
        self.take_error()
    }

    async fn write_dma(&mut self, data: &[u8]) -> Result<(), Error> {
        let dr = self.dr_ptr();
        let request = self.info.dma_tx;
        let tx = self.tx_dma.as_mut().unwrap();
        unsafe { tx.write(data, dr.cast(), request, DMA_OPTIONS) }
            .map_err(Error::Dma)?
            .await
            .map_err(Error::Dma)?;

        self.flush_async().await?;
        self.drain_rx();
        self.take_error()
    }

    async fn transfer_dma(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        let dr = self.dr_ptr();
        let tx_req = self.info.dma_tx;
        let rx_req = self.info.dma_rx;
        let rx_len = read.len();
        let tx_len = write.len();

        if rx_len == 0 {
            return self.write_dma(write).await;
        }

        let tx = self.tx_dma.as_mut().unwrap();
        let rx = self.rx_dma.as_mut().unwrap();

        // Start RX before TX so early clocks are not lost.
        let rx_xfer = unsafe { rx.read(dr.cast::<u8>(), read, rx_req, DMA_OPTIONS) }.map_err(Error::Dma)?;

        let tx_future = async {
            if tx_len == 0 {
                let dummy = 0u8;
                let mut config = TransferConfig::memory_to_peripheral(DmaWidth::Bits8, tx_req);
                config.source_address = AddressMode::Fixed;
                config.source_burst = BurstSize::Four;
                config.destination_burst = BurstSize::Four;
                unsafe { tx.transfer_raw(&dummy as *const u8, dr.cast(), rx_len, config) }
                    .map_err(Error::Dma)?
                    .await
                    .map_err(Error::Dma)?;
            } else if tx_len >= rx_len {
                unsafe { tx.write(write, dr.cast(), tx_req, DMA_OPTIONS) }
                    .map_err(Error::Dma)?
                    .await
                    .map_err(Error::Dma)?;
            } else {
                unsafe { tx.write(&write[..tx_len], dr.cast(), tx_req, DMA_OPTIONS) }
                    .map_err(Error::Dma)?
                    .await
                    .map_err(Error::Dma)?;

                let dummy = 0u8;
                let mut config = TransferConfig::memory_to_peripheral(DmaWidth::Bits8, tx_req);
                config.source_address = AddressMode::Fixed;
                config.source_burst = BurstSize::Four;
                config.destination_burst = BurstSize::Four;
                let remaining = rx_len - tx_len;
                unsafe { tx.transfer_raw(&dummy as *const u8, dr.cast(), remaining, config) }
                    .map_err(Error::Dma)?
                    .await
                    .map_err(Error::Dma)?;
            }
            Ok::<(), Error>(())
        };

        let (tx_res, rx_res) = join(tx_future, rx_xfer).await;
        tx_res?;
        rx_res.map_err(Error::Dma)?;

        if tx_len > rx_len {
            self.flush_async().await?;
            self.drain_rx();
        }
        self.take_error()
    }
}

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for Spi<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d, M: Mode> Drop for Spi<'d, M> {
    fn drop(&mut self) {
        self.set_enabled(false);
        unsafe {
            let regs = &*self.info.regs;
            regs.imsc().write_with_zero(|w| w);
            regs.dmacr().modify(|_, w| w.txdmae().clear_bit().rxdmae().clear_bit());
            regs.icr().write_with_zero(|w| {
                w.roric().set_bit();
                w.rtic().set_bit()
            });
        }
        // Disabling an already-disabled vector is idempotent, so this is
        // harmless for `Blocking` instances that never enabled it.
        self.info.interrupt.disable();
    }
}

trait SealedInstance {
    fn info() -> &'static Info;
}

/// SSP instance.
#[allow(private_bounds, private_interfaces)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// NVIC vector for this SSP.
    type Interrupt: TypelevelInterrupt;
}

macro_rules! impl_instance {
    ($peri:ident, $pac:ident, $rcc:ident, $pclk:ident, $tx:ident, $rx:ident, $state:ident, $irq:ident) => {
        #[allow(private_interfaces)]
        impl SealedInstance for peripherals::$peri {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: pac::$pac::PTR,
                    peripheral: Peripheral::$rcc,
                    pclk: PclkSel::$pclk,
                    dma_tx: Request::$tx,
                    dma_rx: Request::$rx,
                    state: &$state,
                    interrupt: pac::Interrupt::$irq,
                };
                &INFO
            }
        }

        impl Instance for peripherals::$peri {
            type Interrupt = interrupt::typelevel::$irq;
        }
    };
}

impl_instance!(SSP0, Ssp0, Ssp0, Pclk0, Ssp0Tx, Ssp0Rx, STATE_SSP0, SSP0);
impl_instance!(SSP1, Ssp1, Ssp1, Pclk1, Ssp1Tx, Ssp1Rx, STATE_SSP1, SSP1);
impl_instance!(SSP2, Ssp2, Ssp2, Pclk1, Ssp2Tx, Ssp2Rx, STATE_SSP2, SSP2);

/// SCK pin for an SSP instance.
pub trait SckPin<T: Instance>: GpioPin {
    /// Datasheet alternate-function number for this pin/signal.
    const AF: AlternateFunction;
}

/// MOSI / SSP_TX pin for an SSP instance.
pub trait MosiPin<T: Instance>: GpioPin {
    /// Datasheet alternate-function number for this pin/signal.
    const AF: AlternateFunction;
}

/// MISO / SSP_RX pin for an SSP instance.
pub trait MisoPin<T: Instance>: GpioPin {
    /// Datasheet alternate-function number for this pin/signal.
    const AF: AlternateFunction;
}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $trait:ident, $af:ident) => {
        impl $trait<peripherals::$instance> for peripherals::$pin {
            const AF: AlternateFunction = AlternateFunction::$af;
        }
    };
}

// Table 4-4: every documented SSP remap uses Fun=4.
impl_pin!(PA0, SSP0, SckPin, Function4);
impl_pin!(PA2, SSP0, MosiPin, Function4);
impl_pin!(PA3, SSP0, MisoPin, Function4);
impl_pin!(PB7, SSP0, MisoPin, Function4);
impl_pin!(PC12, SSP0, SckPin, Function4);
impl_pin!(PC15, SSP0, MisoPin, Function4);

impl_pin!(PA4, SSP1, SckPin, Function4);
impl_pin!(PA6, SSP1, MosiPin, Function4);
impl_pin!(PA7, SSP1, MisoPin, Function4);
impl_pin!(PB8, SSP1, SckPin, Function4);
impl_pin!(PB10, SSP1, MosiPin, Function4);
impl_pin!(PB11, SSP1, MisoPin, Function4);

impl_pin!(PA8, SSP2, SckPin, Function4);
impl_pin!(PA10, SSP2, MosiPin, Function4);
impl_pin!(PA11, SSP2, MisoPin, Function4);
impl_pin!(PB12, SSP2, SckPin, Function4);
impl_pin!(PB14, SSP2, MosiPin, Function4);
impl_pin!(PB15, SSP2, MisoPin, Function4);

impl<'d, M: Mode> embedded_hal::spi::ErrorType for Spi<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_hal::spi::SpiBus<u8> for Spi<'d, M> {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(words)
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_transfer(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_transfer_in_place(words)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d> embedded_hal_async::spi::SpiBus<u8> for Spi<'d, Async> {
    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.read(words).await
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.write(words).await
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.transfer(read, write).await
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.transfer_in_place(words).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush_async().await
    }
}

// --- embedded-hal 0.2 blocking traits ----------------------------------------

impl<'d, M: Mode> embedded_hal_02::blocking::spi::Transfer<u8> for Spi<'d, M> {
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.blocking_transfer_in_place(words)?;
        Ok(words)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::spi::Write<u8> for Spi<'d, M> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }
}
