//! Master SPI driver for the ASR6601 SSP0–SSP2 (ARM PL022) blocks.
//!
//! Register programming follows the vendor `tremo_spi` driver. The SVD/PAC
//! currently exposes SSP registers without field accessors, so this module uses
//! the documented PL022 bit layouts from the SDK headers.
//!
//! Alternate-function numbers come from datasheet Table 4-4 (Fun=4 for every
//! SSP signal listed there). Constructors either take typed pins that encode
//! that AF, or accept an explicit [`AlternateFunction`] per pin.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU8, Ordering};
use core::task::Poll;

use embassy_futures::join::join;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;
use embedded_hal::spi::{Mode as SpiMode, Phase, Polarity};

use crate::dma::{
    self, AddressMode, ChannelInstance, ControllerInstance, DataWidth as DmaWidth, Request, TransferConfig,
    TransferOptions,
};
use crate::gpio::{AlternateFunction, Flex, Pin as GpioPin, Pull};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt as TypelevelInterrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::pac::ssp0::RegisterBlock;
use crate::rcc::{self, Peripheral};
use crate::{Peri, PeripheralType, interrupt, pac, peripherals};

// PL022 / tremo_spi register bits (fields missing from the PAC).
const CR0_FRF_SHIFT: u32 = 4;
const CR0_SPO: u32 = 1 << 6;
const CR0_SPH: u32 = 1 << 7;
const CR0_SCR_SHIFT: u32 = 8;
const CR0_SCR_MASK: u32 = 0xff << CR0_SCR_SHIFT;

const CR1_SSE: u32 = 1 << 1;
const CR1_MS: u32 = 1 << 2;

const SR_TFE: u32 = 1 << 0;
const SR_TNF: u32 = 1 << 1;
const SR_RNE: u32 = 1 << 2;
const SR_BSY: u32 = 1 << 4;

const INT_ROR: u32 = 1 << 0;
const INT_RT: u32 = 1 << 1;
const INT_RX: u32 = 1 << 2;
const INT_TX: u32 = 1 << 3;

const DMA_RX: u32 = 1 << 0;
const DMA_TX: u32 = 1 << 1;

const DSS_4BIT: u32 = 0x3;
const DSS_8BIT: u32 = 0x7;
const DSS_16BIT: u32 = 0xf;

const FRF_MOTOROLA: u32 = 0x0;
const FRF_TI: u32 = 0x1;
const FRF_MICROWIRE: u32 = 0x2;

const RESULT_OK: u8 = 0;
const RESULT_OVERRUN: u8 = 1;

/// SPI driver errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// RX FIFO overrun (ROR).
    Overrun,
    /// Requested SCLK cannot be formed from the current PCLK dividers.
    InvalidFrequency,
    /// LSB-first transfers are not supported by the PL022 Motorola frame format.
    UnsupportedBitOrder,
    /// Embedded-hal byte transfers require [`DataWidth::Bits8`].
    InvalidDataWidth,
    /// Peripheral clocks are not published yet (`rcc::init` has not completed).
    ClocksNotInitialized,
    /// DMA transfer failed.
    Dma(dma::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Overrun => f.write_str("SPI RX overrun"),
            Self::InvalidFrequency => f.write_str("SPI frequency is out of range"),
            Self::UnsupportedBitOrder => f.write_str("SPI LSB-first is unsupported"),
            Self::InvalidDataWidth => f.write_str("SPI data width is not 8 bit"),
            Self::ClocksNotInitialized => f.write_str("RCC clocks are not initialized"),
            Self::Dma(err) => write!(f, "SPI DMA error: {err}"),
        }
    }
}

impl core::error::Error for Error {}

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
/// [`BitOrder::LsbFirst`] returns [`Error::UnsupportedBitOrder`].
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

impl FrameFormat {
    const fn cr0_bits(self) -> u32 {
        match self {
            Self::Motorola => FRF_MOTOROLA,
            Self::Ti => FRF_TI,
            Self::Microwire => FRF_MICROWIRE,
        }
    }
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
    const fn dss(self) -> u32 {
        match self {
            Self::Bits4 => DSS_4BIT,
            Self::Bits8 => DSS_8BIT,
            Self::Bits16 => DSS_16BIT,
        }
    }

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
    /// Desired SCLK frequency in hertz.
    pub frequency: u32,
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
            frequency: 1_000_000,
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
            self.frequency,
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
        let mis = regs.mis().read().bits();

        if mis & INT_ROR != 0 {
            info.state.result.store(RESULT_OVERRUN, Ordering::Release);
            regs.icr().write_with_zero(|w| w.bits(INT_ROR));
        }
        if mis & INT_RT != 0 {
            regs.icr().write_with_zero(|w| w.bits(INT_RT));
        }

        // Mask level-sensitive FIFO interrupts; the waiter re-enables as needed.
        let mask = mis & (INT_TX | INT_RX);
        if mask != 0 {
            regs.imsc().modify(|r, w| w.bits(r.bits() & !mask));
        }

        info.state.waker.wake();
    }
}

/// Master SPI driver.
pub struct Spi<'d, M: Mode> {
    info: &'static Info,
    _sck: Option<Flex<'d>>,
    _mosi: Option<Flex<'d>>,
    _miso: Option<Flex<'d>>,
    _nss: Option<Flex<'d>>,
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

fn calc_dividers(pclk: u32, freq: u32) -> Result<(u8, u8), Error> {
    if freq == 0 || pclk == 0 {
        return Err(Error::InvalidFrequency);
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

    best.map(|(c, s, _)| (c, s)).ok_or(Error::InvalidFrequency)
}

impl<'d, M: Mode> Spi<'d, M> {
    fn new_inner<T: Instance>(
        _spi: Peri<'d, T>,
        sck: Option<Flex<'d>>,
        mosi: Option<Flex<'d>>,
        miso: Option<Flex<'d>>,
        nss: Option<Flex<'d>>,
        tx_dma: Option<dma::Channel<'d, Async>>,
        rx_dma: Option<dma::Channel<'d, Async>>,
        config: Config,
        enable_irq: bool,
    ) -> Result<Self, Error> {
        let info = T::info();

        let _ = rcc::enable_peripheral(info.peripheral);
        let _ = rcc::reset_peripheral(info.peripheral);

        let mut spi = Self {
            info,
            _sck: sck,
            _mosi: mosi,
            _miso: miso,
            _nss: nss,
            tx_dma,
            rx_dma,
            data_width: config.data_width,
            _mode: PhantomData,
        };

        spi.configure(&config)?;

        info.state.result.store(RESULT_OK, Ordering::Release);
        unsafe {
            let regs = &*info.regs;
            regs.imsc().write_with_zero(|w| w.bits(0));
            regs.icr().write_with_zero(|w| w.bits(INT_ROR | INT_RT));
        }

        if enable_irq {
            info.interrupt.unpend();
            unsafe {
                info.interrupt.enable();
            }
        }

        spi.set_enabled(true);
        Ok(spi)
    }

    fn configure(&mut self, config: &Config) -> Result<(), Error> {
        if config.bit_order == BitOrder::LsbFirst {
            return Err(Error::UnsupportedBitOrder);
        }

        let clocks = rcc::clocks().ok_or(Error::ClocksNotInitialized)?;
        let pclk = match self.info.pclk {
            PclkSel::Pclk0 => clocks.pclk0_hz,
            PclkSel::Pclk1 => clocks.pclk1_hz,
        };
        let (cpsdvsr, scr) = calc_dividers(pclk, config.frequency)?;

        let mut cr0 = config.data_width.dss();
        cr0 |= config.frame_format.cr0_bits() << CR0_FRF_SHIFT;
        if config.mode.polarity == Polarity::IdleHigh {
            cr0 |= CR0_SPO;
        }
        if config.mode.phase == Phase::CaptureOnSecondTransition {
            cr0 |= CR0_SPH;
        }
        cr0 |= (scr as u32) << CR0_SCR_SHIFT;

        self.set_enabled(false);
        unsafe {
            let regs = &*self.info.regs;
            regs.cpsr().write_with_zero(|w| w.bits(cpsdvsr as u32));
            regs.cr0().write_with_zero(|w| w.bits(cr0));
            // Master mode, SSP disabled until set_enabled(true).
            regs.cr1().write_with_zero(|w| w.bits(0));
            let mut dma_cr = 0;
            if self.tx_dma.is_some() {
                dma_cr |= DMA_TX;
            }
            if self.rx_dma.is_some() {
                dma_cr |= DMA_RX;
            }
            regs.dma_cr().write_with_zero(|w| w.bits(dma_cr));
        }
        self.data_width = config.data_width;
        Ok(())
    }

    fn set_enabled(&mut self, enabled: bool) {
        unsafe {
            let regs = &*self.info.regs;
            regs.cr1().modify(|r, w| {
                let bits = if enabled {
                    (r.bits() & !CR1_MS) | CR1_SSE
                } else {
                    r.bits() & !CR1_SSE
                };
                w.bits(bits)
            });
        }
    }

    fn sr(&self) -> u32 {
        unsafe { (*self.info.regs).sr().read().bits() }
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
            while regs.sr().read().bits() & SR_RNE != 0 {
                let _ = regs.dr().read().bits();
            }
            regs.icr().write_with_zero(|w| w.bits(INT_ROR | INT_RT));
        }
        self.info.state.result.store(RESULT_OK, Ordering::Release);
    }

    fn write_frame(&mut self, frame: u16) {
        unsafe {
            (*self.info.regs).dr().write_with_zero(|w| w.bits(frame as u32));
        }
    }

    fn read_frame(&mut self) -> u16 {
        unsafe { (*self.info.regs).dr().read().bits() as u16 }
    }

    /// Reconfigure the SPI peripheral.
    pub fn set_config(&mut self, config: &Config) -> Result<(), Error> {
        self.configure(config)?;
        self.set_enabled(true);
        Ok(())
    }

    /// Change only the SCLK frequency.
    pub fn set_frequency(&mut self, frequency: u32) -> Result<(), Error> {
        let clocks = rcc::clocks().ok_or(Error::ClocksNotInitialized)?;
        let pclk = match self.info.pclk {
            PclkSel::Pclk0 => clocks.pclk0_hz,
            PclkSel::Pclk1 => clocks.pclk1_hz,
        };
        let (cpsdvsr, scr) = calc_dividers(pclk, frequency)?;
        self.set_enabled(false);
        unsafe {
            let regs = &*self.info.regs;
            regs.cpsr().write_with_zero(|w| w.bits(cpsdvsr as u32));
            regs.cr0()
                .modify(|r, w| w.bits((r.bits() & !CR0_SCR_MASK) | ((scr as u32) << CR0_SCR_SHIFT)));
        }
        self.set_enabled(true);
        Ok(())
    }

    /// Block until the SSP is idle.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while self.sr() & SR_BSY != 0 {}
        self.take_error()
    }

    /// Blocking write. Received bytes are discarded.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.ensure_u8()?;
        for &b in data {
            while self.sr() & SR_TNF == 0 {}
            self.write_frame(b as u16);
            while self.sr() & SR_RNE == 0 {}
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
            while self.sr() & SR_TNF == 0 {}
            self.write_frame(tx as u16);
            while self.sr() & SR_RNE == 0 {}
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
            while self.sr() & SR_TNF == 0 {}
            self.write_frame(*b as u16);
            while self.sr() & SR_RNE == 0 {}
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
            while self.sr() & SR_TNF == 0 {}
            self.write_frame(w);
            while self.sr() & SR_RNE == 0 {}
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
            while self.sr() & SR_TNF == 0 {}
            self.write_frame(0);
            while self.sr() & SR_RNE == 0 {}
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
    ) -> Result<Self, Error> {
        let pull = sck_pull(config.mode);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            Some(configure_output_af(mosi, Mosi::AF, Pull::None)),
            Some(configure_input_af(miso, Miso::AF, Pull::None)),
            None,
            None,
            None,
            config,
            false,
        )
    }

    /// Create a blocking master with explicit alternate-function numbers.
    pub fn new_blocking_af<T: Instance>(
        spi: Peri<'d, T>,
        sck: Peri<'d, impl GpioPin + 'd>,
        sck_af: AlternateFunction,
        mosi: Peri<'d, impl GpioPin + 'd>,
        mosi_af: AlternateFunction,
        miso: Peri<'d, impl GpioPin + 'd>,
        miso_af: AlternateFunction,
        config: Config,
    ) -> Result<Self, Error> {
        let pull = sck_pull(config.mode);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, sck_af, pull)),
            Some(configure_output_af(mosi, mosi_af, Pull::None)),
            Some(configure_input_af(miso, miso_af, Pull::None)),
            None,
            None,
            None,
            config,
            false,
        )
    }

    /// Blocking transmit-only master (MOSI + SCK).
    pub fn new_blocking_txonly<T: Instance, Sck: SckPin<T>, Mosi: MosiPin<T>>(
        spi: Peri<'d, T>,
        sck: Peri<'d, Sck>,
        mosi: Peri<'d, Mosi>,
        config: Config,
    ) -> Result<Self, Error> {
        let pull = sck_pull(config.mode);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            Some(configure_output_af(mosi, Mosi::AF, Pull::None)),
            None,
            None,
            None,
            None,
            config,
            false,
        )
    }

    /// Blocking receive-only master (MISO + SCK).
    pub fn new_blocking_rxonly<T: Instance, Sck: SckPin<T>, Miso: MisoPin<T>>(
        spi: Peri<'d, T>,
        sck: Peri<'d, Sck>,
        miso: Peri<'d, Miso>,
        config: Config,
    ) -> Result<Self, Error> {
        let pull = sck_pull(config.mode);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            None,
            Some(configure_input_af(miso, Miso::AF, Pull::None)),
            None,
            None,
            None,
            config,
            false,
        )
    }

    /// Blocking master that also muxes the hardware NSS/FSS pin.
    pub fn new_blocking_with_nss<T: Instance, Sck: SckPin<T>, Mosi: MosiPin<T>, Miso: MisoPin<T>, Nss: NssPin<T>>(
        spi: Peri<'d, T>,
        sck: Peri<'d, Sck>,
        mosi: Peri<'d, Mosi>,
        miso: Peri<'d, Miso>,
        nss: Peri<'d, Nss>,
        config: Config,
    ) -> Result<Self, Error> {
        let pull = sck_pull(config.mode);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            Some(configure_output_af(mosi, Mosi::AF, Pull::None)),
            Some(configure_input_af(miso, Miso::AF, Pull::None)),
            Some(configure_output_af(nss, Nss::AF, Pull::Up)),
            None,
            None,
            config,
            false,
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
    ) -> Result<Self, Error> {
        let pull = sck_pull(config.mode);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            Some(configure_output_af(mosi, Mosi::AF, Pull::None)),
            Some(configure_input_af(miso, Miso::AF, Pull::None)),
            None,
            None,
            None,
            config,
            true,
        )
    }

    /// Create an async master with explicit alternate-function numbers.
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
    ) -> Result<Self, Error> {
        let pull = sck_pull(config.mode);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, sck_af, pull)),
            Some(configure_output_af(mosi, mosi_af, Pull::None)),
            Some(configure_input_af(miso, miso_af, Pull::None)),
            None,
            None,
            None,
            config,
            true,
        )
    }

    /// Create an async master that uses DMA for transfers.
    ///
    /// Bind both the SSP interrupt (overrun / timeout) and the DMA controller
    /// interrupt(s) used by the supplied channels.
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
        + Copy
        + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        let pull = sck_pull(config.mode);
        let tx = dma::Channel::new(tx_dma, irq);
        let rx = dma::Channel::new(rx_dma, irq);
        Self::new_inner::<T>(
            spi,
            Some(configure_output_af(sck, Sck::AF, pull)),
            Some(configure_output_af(mosi, Mosi::AF, Pull::None)),
            Some(configure_input_af(miso, Miso::AF, Pull::None)),
            None,
            Some(tx),
            Some(rx),
            config,
            true,
        )
    }

    async fn wait_sr(&mut self, mask: u32) -> Result<(), Error> {
        poll_fn(|cx| {
            if let Err(e) = self.take_error() {
                return Poll::Ready(Err(e));
            }
            if self.sr() & mask != 0 {
                return Poll::Ready(Ok(()));
            }
            self.info.state.waker.register(cx.waker());
            unsafe {
                let regs = &*self.info.regs;
                let ie = if mask & SR_TNF != 0 { INT_TX } else { 0 }
                    | if mask & (SR_RNE | SR_TFE) != 0 {
                        INT_RX | INT_RT | INT_TX
                    } else {
                        0
                    }
                    | INT_ROR;
                regs.imsc().modify(|r, w| w.bits(r.bits() | ie));
            }
            if self.sr() & mask != 0 {
                return Poll::Ready(Ok(()));
            }
            if let Err(e) = self.take_error() {
                return Poll::Ready(Err(e));
            }
            Poll::Pending
        })
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
            self.wait_sr(SR_TNF).await?;
            self.write_frame(b as u16);
            self.wait_sr(SR_RNE).await?;
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
            self.wait_sr(SR_TNF).await?;
            self.write_frame(tx as u16);
            self.wait_sr(SR_RNE).await?;
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
            self.wait_sr(SR_TNF).await?;
            self.write_frame(*b as u16);
            self.wait_sr(SR_RNE).await?;
            *b = self.read_frame() as u8;
        }
        self.flush_async().await
    }

    async fn flush_async(&mut self) -> Result<(), Error> {
        while self.sr() & SR_BSY != 0 {
            self.wait_sr(SR_TFE).await?;
            if self.sr() & SR_BSY == 0 {
                break;
            }
        }
        self.take_error()
    }

    async fn write_dma(&mut self, data: &[u8]) -> Result<(), Error> {
        let dr = self.dr_ptr();
        let request = self.info.dma_tx;
        let tx = self.tx_dma.as_mut().unwrap();
        unsafe { tx.write(data, dr.cast(), request, TransferOptions::default()) }
            .map_err(Error::Dma)?
            .await
            .map_err(Error::Dma)?;

        while self.sr() & SR_BSY != 0 {}
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
        let rx_xfer =
            unsafe { rx.read(dr.cast::<u8>(), read, rx_req, TransferOptions::default()) }.map_err(Error::Dma)?;

        let tx_future = async {
            if tx_len == 0 {
                let dummy = 0u8;
                let mut config = TransferConfig::memory_to_peripheral(DmaWidth::Bits8, tx_req);
                config.source_address = AddressMode::Fixed;
                unsafe { tx.transfer_raw(&dummy as *const u8, dr.cast(), rx_len, config) }
                    .map_err(Error::Dma)?
                    .await
                    .map_err(Error::Dma)?;
            } else if tx_len >= rx_len {
                unsafe { tx.write(write, dr.cast(), tx_req, TransferOptions::default()) }
                    .map_err(Error::Dma)?
                    .await
                    .map_err(Error::Dma)?;
            } else {
                unsafe { tx.write(&write[..tx_len], dr.cast(), tx_req, TransferOptions::default()) }
                    .map_err(Error::Dma)?
                    .await
                    .map_err(Error::Dma)?;

                let dummy = 0u8;
                let mut config = TransferConfig::memory_to_peripheral(DmaWidth::Bits8, tx_req);
                config.source_address = AddressMode::Fixed;
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
            while self.sr() & SR_BSY != 0 {}
            self.drain_rx();
        }
        self.take_error()
    }
}

impl<'d, M: Mode> Drop for Spi<'d, M> {
    fn drop(&mut self) {
        self.set_enabled(false);
        unsafe {
            let regs = &*self.info.regs;
            regs.imsc().write_with_zero(|w| w.bits(0));
            regs.dma_cr().write_with_zero(|w| w.bits(0));
            regs.icr().write_with_zero(|w| w.bits(INT_ROR | INT_RT));
        }
        self.info.interrupt.disable();
    }
}

mod sealed {
    use super::Info;

    pub trait Instance {
        fn info() -> &'static Info;
    }
}

/// SSP instance.
#[allow(private_bounds)]
pub trait Instance: sealed::Instance + PeripheralType + 'static {
    /// NVIC vector for this SSP.
    type Interrupt: TypelevelInterrupt;
}

macro_rules! impl_instance {
    ($peri:ident, $pac:ident, $rcc:ident, $pclk:ident, $tx:ident, $rx:ident, $state:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$peri {
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

/// NSS / SSP_FSS pin for an SSP instance.
pub trait NssPin<T: Instance>: GpioPin {
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
impl_pin!(PA1, SSP0, NssPin, Function4);
impl_pin!(PA2, SSP0, MosiPin, Function4);
impl_pin!(PA3, SSP0, MisoPin, Function4);
impl_pin!(PB7, SSP0, MisoPin, Function4);
impl_pin!(PC12, SSP0, SckPin, Function4);
impl_pin!(PC13, SSP0, NssPin, Function4);
impl_pin!(PC15, SSP0, MisoPin, Function4);

impl_pin!(PA4, SSP1, SckPin, Function4);
impl_pin!(PA5, SSP1, NssPin, Function4);
impl_pin!(PA6, SSP1, MosiPin, Function4);
impl_pin!(PA7, SSP1, MisoPin, Function4);
impl_pin!(PB8, SSP1, SckPin, Function4);
impl_pin!(PB9, SSP1, NssPin, Function4);
impl_pin!(PB10, SSP1, MosiPin, Function4);
impl_pin!(PB11, SSP1, MisoPin, Function4);

impl_pin!(PA8, SSP2, SckPin, Function4);
impl_pin!(PA9, SSP2, NssPin, Function4);
impl_pin!(PA10, SSP2, MosiPin, Function4);
impl_pin!(PA11, SSP2, MisoPin, Function4);
impl_pin!(PB12, SSP2, SckPin, Function4);
impl_pin!(PB13, SSP2, NssPin, Function4);
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
