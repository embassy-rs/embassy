//! Common types, traits, and helper functions for SPI drivers.

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use maitake_sync::WaitCell;
use paste::paste;

pub use crate::clocks::PoweredClock;
use crate::clocks::periph_helpers::LpspiConfig;
// Re-export clock configuration types for user convenience
pub use crate::clocks::periph_helpers::{Div4, LpspiClockSel};
use crate::clocks::{ClockError, Gate};
use crate::dma::DmaRequest;
use crate::pac::lpspi::vals::{Rrf, Rtf};
use crate::{interrupt, pac};

// =============================================================================
// REGISTER BIT CONSTANTS
// =============================================================================

/// All clearable status flags (TEF, REF, DMF, FCF, WCF, TCF)
pub(super) const LPSPI_ALL_STATUS_FLAGS: u32 = 0x3F00;

/// FIFO size for MCXA276 LPSPI (4 words)
pub(super) const LPSPI_FIFO_SIZE: u8 = 4;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Flush TX and RX FIFOs for a given LPSPI register block
#[inline]
pub(super) fn flush_fifos(spi: pac::lpspi::Lpspi) {
    spi.cr().modify(|w| {
        w.set_rtf(Rtf::TXFIFO_RST);
        w.set_rrf(Rrf::RXFIFO_RST);
    });
}

/// Clear all status flags for a given LPSPI register block
#[inline]
pub(super) fn clear_status_flags(spi: pac::lpspi::Lpspi) {
    spi.sr().write_value(pac::lpspi::regs::Sr(LPSPI_ALL_STATUS_FLAGS));
}

/// Clear NOSTALL bit in CFGR1 (disables "no stall" mode)
#[inline]
pub(super) fn clear_nostall(spi: pac::lpspi::Lpspi) {
    spi.cfgr1().modify(|w| w.set_nostall(false));
}

/// Disable all interrupts by writing reset value (0) to IER
#[inline]
pub(super) fn disable_all_interrupts(spi: pac::lpspi::Lpspi) {
    // IER reset value is 0, writing default clears all interrupts
    spi.ier().write_value(Default::default());
}

/// Read TCR with errata workaround (ERR050606)
#[inline]
pub(super) fn read_tcr_with_errata_workaround(spi: pac::lpspi::Lpspi) -> u32 {
    let mut last = spi.tcr().read().0;
    loop {
        let _ = spi.sr().read();
        let now = spi.tcr().read().0;
        if now == last {
            break now;
        }
        last = now;
    }
}

/// Common setup sequence for async SPI transfers
#[inline]
pub(super) fn prepare_for_transfer(spi: pac::lpspi::Lpspi) {
    spi.cr().modify(|w| w.set_men(false));
    flush_fifos(spi);
    clear_status_flags(spi);
    disable_all_interrupts(spi);
    clear_nostall(spi);
    spi.cr().modify(|w| w.set_men(true));
}

/// Common setup sequence for blocking SPI transfers
#[inline]
pub(super) fn prepare_for_blocking_transfer(spi: pac::lpspi::Lpspi) {
    spi.cr().modify(|w| w.set_men(false));
    flush_fifos(spi);
    clear_status_flags(spi);
    clear_nostall(spi);
    spi.cr().modify(|w| w.set_men(true));
}

// =============================================================================
// PUBLIC TYPES
// =============================================================================

/// Shorthand for `Result<T>`.
pub type Result<T> = core::result::Result<T, Error>;

/// Error information type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Clock configuration error.
    ClockSetup(ClockError),
    /// Transfer error.
    TransferError,
    /// Transmit FIFO error.
    TxFifoError,
    /// Receive FIFO error.
    RxFifoError,
    /// Module busy timeout.
    Timeout,
    /// SPI peripheral is already in use.
    AlreadyInUse,
    /// Other internal errors or unexpected state.
    Other,
}

impl embedded_hal_1::spi::Error for Error {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        embedded_hal_1::spi::ErrorKind::Other
    }
}

/// Maximum number of iterations for busy-wait loops.
pub(super) const SPIN_LIMIT: u32 = 10_000_000;

#[inline]
pub(super) fn spin_wait_while(mut cond: impl FnMut() -> bool) -> Result<()> {
    for _ in 0..SPIN_LIMIT {
        if !cond() {
            return Ok(());
        }
        spin_loop();
    }
    Err(Error::Timeout)
}

// =============================================================================
// SLAVE IRQ-DRIVEN ASYNC STATE
// =============================================================================

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum SlaveIrqOp {
    Idle = 0,
    Rx = 1,
    Tx = 2,
    Transfer = 3,
}

pub(super) struct SlaveIrqStateInner {
    pub op: SlaveIrqOp,
    pub rx_ptr: *mut u8,
    pub rx_len: usize,
    pub rx_pos: usize,
    pub rx_store_len: usize,
    pub tx_ptr: *const u8,
    pub tx_len: usize,
    pub tx_pos: usize,
    pub tx_source_len: usize,
    pub error: Option<Error>,
}

impl SlaveIrqStateInner {
    pub const fn new() -> Self {
        Self {
            op: SlaveIrqOp::Idle,
            rx_ptr: core::ptr::null_mut(),
            rx_len: 0,
            rx_pos: 0,
            rx_store_len: 0,
            tx_ptr: core::ptr::null(),
            tx_len: 0,
            tx_pos: 0,
            tx_source_len: 0,
            error: None,
        }
    }
}

pub(super) struct SlaveIrqState {
    inner: UnsafeCell<SlaveIrqStateInner>,
}

unsafe impl Sync for SlaveIrqState {}

impl SlaveIrqState {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(SlaveIrqStateInner::new()),
        }
    }

    #[inline]
    pub fn with<R>(&'static self, f: impl FnOnce(&mut SlaveIrqStateInner) -> R) -> R {
        critical_section::with(|_| unsafe { f(&mut *self.inner.get()) })
    }
}

// =============================================================================
// INTERRUPT HANDLER
// =============================================================================

#[inline]
pub(super) unsafe fn handle_slave_rx_irq<T: Instance>(regs: pac::lpspi::Lpspi, st: &mut SlaveIrqStateInner) {
    let sr = regs.sr().read();

    if sr.ref_() {
        clear_status_flags(regs);
        st.error = Some(Error::RxFifoError);
        st.op = SlaveIrqOp::Idle;
        disable_all_interrupts(regs);
        T::wait_cell().wake();
        return;
    }

    while st.rx_pos < st.rx_len && regs.fsr().read().rxcount() > 0 {
        let byte = regs.rdr().read().data() as u8;
        unsafe { *st.rx_ptr.add(st.rx_pos) = byte };
        st.rx_pos += 1;
    }

    if st.rx_pos >= st.rx_len {
        st.op = SlaveIrqOp::Idle;
        disable_all_interrupts(regs);
        T::wait_cell().wake();
    }
}

#[inline]
pub(super) unsafe fn handle_slave_tx_irq<T: Instance>(regs: pac::lpspi::Lpspi, st: &mut SlaveIrqStateInner) {
    let sr = regs.sr().read();

    if sr.tef() {
        clear_status_flags(regs);
        st.error = Some(Error::TxFifoError);
        st.op = SlaveIrqOp::Idle;
        disable_all_interrupts(regs);
        T::wait_cell().wake();
        return;
    }

    while st.tx_pos < st.tx_len && regs.fsr().read().txcount() < LPSPI_FIFO_SIZE {
        let byte = unsafe { *st.tx_ptr.add(st.tx_pos) };
        regs.tdr().write(|w| w.set_data(byte as u32));
        st.tx_pos += 1;
    }

    if st.tx_pos >= st.tx_len {
        regs.ier().write(|w| {
            w.set_fcie(true);
            w.set_teie(true);
        });

        let tx_empty = regs.fsr().read().txcount() == 0;
        let sr = regs.sr().read();
        if tx_empty && sr.fcf() {
            regs.sr().write(|w| w.set_fcf(true)); // w1c
            st.op = SlaveIrqOp::Idle;
            disable_all_interrupts(regs);
            T::wait_cell().wake();
        }
    }
}

#[inline]
pub(super) unsafe fn handle_slave_transfer_irq<T: Instance>(regs: pac::lpspi::Lpspi, st: &mut SlaveIrqStateInner) {
    let sr = regs.sr().read();

    if sr.ref_() {
        clear_status_flags(regs);
        st.error = Some(Error::RxFifoError);
        st.op = SlaveIrqOp::Idle;
        disable_all_interrupts(regs);
        T::wait_cell().wake();
        return;
    }

    if sr.tef() {
        clear_status_flags(regs);
        st.error = Some(Error::TxFifoError);
        st.op = SlaveIrqOp::Idle;
        disable_all_interrupts(regs);
        T::wait_cell().wake();
        return;
    }

    while st.rx_pos < st.rx_len && regs.fsr().read().rxcount() > 0 {
        let byte = regs.rdr().read().data() as u8;
        if st.rx_pos < st.rx_store_len {
            unsafe { *st.rx_ptr.add(st.rx_pos) = byte };
        }
        st.rx_pos += 1;
    }

    while st.tx_pos < st.tx_len && regs.fsr().read().txcount() < LPSPI_FIFO_SIZE {
        let byte = if st.tx_pos < st.tx_source_len {
            unsafe { *st.tx_ptr.add(st.tx_pos) }
        } else {
            0
        };
        regs.tdr().write(|w| w.set_data(byte as u32));
        st.tx_pos += 1;
    }

    if st.rx_pos >= st.rx_len {
        st.op = SlaveIrqOp::Idle;
        disable_all_interrupts(regs);
        T::wait_cell().wake();
    }
}

/// Interrupt handler for SPI async operations.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();

        if regs.ier().read().0 == 0 {
            return;
        }

        let mut handled_slave = false;
        T::slave_irq_state().with(|st| match st.op {
            SlaveIrqOp::Rx => {
                handled_slave = true;
                unsafe { handle_slave_rx_irq::<T>(regs, st) };
            }
            SlaveIrqOp::Tx => {
                handled_slave = true;
                unsafe { handle_slave_tx_irq::<T>(regs, st) };
            }
            SlaveIrqOp::Transfer => {
                handled_slave = true;
                unsafe { handle_slave_transfer_irq::<T>(regs, st) };
            }
            SlaveIrqOp::Idle => {}
        });

        if handled_slave {
            return;
        }

        if regs.ier().read().0 != 0 {
            disable_all_interrupts(regs);
            T::wait_cell().wake();
        }
    }
}

// =============================================================================
// SEALED TRAIT AND INSTANCE TRAIT
// =============================================================================

pub(super) mod sealed {
    /// Seal a trait
    pub trait Sealed {}
}

// Seal only the specific pins that can be used with SPI, not all GPIO pins.
// This ensures type safety - only pins that are actually muxed into SPI functions
// can be used with the SPI driver.
impl sealed::Sealed for crate::peripherals::P1_0 {} // LPSPI0_SDO
impl sealed::Sealed for crate::peripherals::P1_1 {} // LPSPI0_SCK
impl sealed::Sealed for crate::peripherals::P1_2 {} // LPSPI0_SDI
impl sealed::Sealed for crate::peripherals::P1_3 {} // LPSPI0_PCS0
impl sealed::Sealed for crate::peripherals::P3_8 {} // LPSPI1_SOUT
impl sealed::Sealed for crate::peripherals::P3_9 {} // LPSPI1_SIN
impl sealed::Sealed for crate::peripherals::P3_10 {} // LPSPI1_SCK
impl sealed::Sealed for crate::peripherals::P3_11 {} // LPSPI1_PCS0

pub(super) trait SealedInstance {
    fn regs() -> pac::lpspi::Lpspi;
    fn wait_cell() -> &'static WaitCell;
    fn slave_irq_state() -> &'static SlaveIrqState;
}

/// SPI Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send + Gate<MrccPeriphConfig = LpspiConfig> {
    /// Interrupt for this SPI instance.
    type Interrupt: interrupt::typelevel::Interrupt;
    /// Clock instance
    const CLOCK_INSTANCE: crate::clocks::periph_helpers::LpspiInstance;
    /// Type-safe DMA request source for TX
    type TxDmaRequest: DmaRequest;
    /// Type-safe DMA request source for RX
    type RxDmaRequest: DmaRequest;
}

macro_rules! impl_instance {
    ($($n:expr),*) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<LPSPI $n>] {
                    fn regs() -> pac::lpspi::Lpspi {
                        pac::[<LPSPI $n>]
                    }

                    fn wait_cell() -> &'static WaitCell {
                        static WAIT_CELL: WaitCell = WaitCell::new();
                        &WAIT_CELL
                    }

                    fn slave_irq_state() -> &'static SlaveIrqState {
                        static SLAVE_IRQ_STATE: SlaveIrqState = SlaveIrqState::new();
                        &SLAVE_IRQ_STATE
                    }
                }

                impl Instance for crate::peripherals::[<LPSPI $n>] {
                    type Interrupt = crate::interrupt::typelevel::[<LPSPI $n>];
                    const CLOCK_INSTANCE: crate::clocks::periph_helpers::LpspiInstance
                        = crate::clocks::periph_helpers::LpspiInstance::[<Lpspi $n>];
                    type TxDmaRequest = crate::dma::[<Lpspi $n TxRequest>];
                    type RxDmaRequest = crate::dma::[<Lpspi $n RxRequest>];
                }
            }
        )*
    };
}

impl_instance!(0, 1);

// =============================================================================
// CONFIGURATION TYPES
// =============================================================================

/// Bit order
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BitOrder {
    /// Most significant bit first
    #[default]
    MsbFirst,
    /// Least significant bit first
    LsbFirst,
}

/// Chip select pin selection
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChipSelect {
    #[default]
    Pcs0 = 0,
    Pcs1 = 1,
    Pcs2 = 2,
    Pcs3 = 3,
}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::Sealed {}

/// Blocking mode.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// Async mode.
pub struct Async;
impl sealed::Sealed for Async {}
impl Mode for Async {}

/// Chip select mode trait.
///
/// This trait distinguishes between hardware-managed CS (PCS signal controlled by LPSPI)
/// and externally-managed CS (user controls CS via GPIO).
#[allow(private_bounds)]
pub trait CsMode: sealed::Sealed {}

/// Hardware-managed chip select mode.
///
/// The LPSPI peripheral controls the PCS (Peripheral Chip Select) signal.
/// Use this when you have a single device on the bus and want automatic CS timing.
pub struct HardwareCs;
impl sealed::Sealed for HardwareCs {}
impl CsMode for HardwareCs {}

/// Externally-managed chip select mode.
///
/// The user controls chip select via GPIO. Use this when:
/// - You have multiple devices on the same SPI bus
/// - You need to use `embassy-embedded-hal::shared_bus::SpiDevice`
/// - You need custom CS timing or behavior
///
/// Only `Spi` instances with `NoCs` implement `embedded_hal::spi::SpiBus`.
pub struct NoCs;
impl sealed::Sealed for NoCs {}
impl CsMode for NoCs {}

/// SPI master configuration
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct Config {
    /// SPI mode (combines polarity and phase).
    /// Use MODE_0, MODE_1, MODE_2, or MODE_3 from embedded_hal.
    pub mode: embedded_hal_02::spi::Mode,
    /// Bit order
    pub bit_order: BitOrder,
    /// Bits per frame.
    ///
    /// Valid range: 1-4096 (register field FRAMESZ is 12 bits, value = bits_per_frame - 1).
    /// Values outside this range are clamped during configuration.
    /// Typical value: 8 for byte-oriented transfers.
    pub bits_per_frame: u16,
    /// Chip select to use
    pub chip_select: ChipSelect,
    /// SCK divider (0-255).
    ///
    /// The SPI clock frequency is: `src_clk / (prescaler.divisor() * (sck_div + 2))`
    ///
    /// This value is also used for timing parameters DBT (delay between transfers),
    /// PCSSCK (PCS-to-SCK delay), and SCKPCS (SCK-to-PCS delay) to maintain symmetric
    /// timing. Adjust these separately if asymmetric timing is needed.
    pub sck_div: u8,
    /// Clock prescaler (Div1 through Div128)
    pub prescaler: Prescaler,
    /// Clock source for the LPSPI peripheral.
    ///
    /// Default: `FroHfDiv` (high-frequency FRO with divider)
    pub clock_source: LpspiClockSel,
    /// Power state for the clock source.
    ///
    /// Default: `NormalEnabledDeepSleepDisabled`
    pub clock_power: PoweredClock,
    /// Pre-divider applied to the clock source before the LPSPI prescaler.
    ///
    /// Default: No division (1:1)
    pub clock_div: Div4,
}

impl Config {
    /// Create a new SPI configuration with default settings (MODE_0).
    pub fn new() -> Self {
        Self {
            mode: embedded_hal_02::spi::MODE_0,
            bit_order: BitOrder::MsbFirst,
            bits_per_frame: 8,
            chip_select: ChipSelect::Pcs0,
            sck_div: 0,
            prescaler: Prescaler::Div1,
            clock_source: LpspiClockSel::FroHfDiv,
            clock_power: PoweredClock::NormalEnabledDeepSleepDisabled,
            clock_div: Div4::no_div(),
        }
    }

    /// Set SPI mode (MODE_0, MODE_1, MODE_2, or MODE_3).
    pub fn mode(&mut self, mode: embedded_hal_02::spi::Mode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Set bit order
    pub fn bit_order(&mut self, order: BitOrder) -> &mut Self {
        self.bit_order = order;
        self
    }

    /// Set bits per frame.
    ///
    /// Valid range: 1-4096. Values outside this range are clamped during configuration.
    pub fn bits_per_frame(&mut self, bits: u16) -> &mut Self {
        self.bits_per_frame = bits;
        self
    }

    /// Set chip select
    pub fn chip_select(&mut self, cs: ChipSelect) -> &mut Self {
        self.chip_select = cs;
        self
    }

    /// Set SCK divider. Baud = src_clk / (prescaler.divisor() * (SCKDIV + 2))
    pub fn sck_div(&mut self, div: u8) -> &mut Self {
        self.sck_div = div;
        self
    }

    /// Set clock prescaler (Div1 through Div128)
    pub fn prescaler(&mut self, prescaler: Prescaler) -> &mut Self {
        self.prescaler = prescaler;
        self
    }

    /// Set clock source for the LPSPI peripheral.
    pub fn clock_source(&mut self, source: LpspiClockSel) -> &mut Self {
        self.clock_source = source;
        self
    }

    /// Set power state for the clock source.
    pub fn clock_power(&mut self, power: PoweredClock) -> &mut Self {
        self.clock_power = power;
        self
    }

    /// Set pre-divider for the clock source.
    pub fn clock_div(&mut self, div: Div4) -> &mut Self {
        self.clock_div = div;
        self
    }

    /// Calculate baud rate parameters for a target frequency.
    pub fn for_frequency(&mut self, src_hz: u32, target_hz: u32) -> &mut Self {
        let (prescaler, sck_div) = compute_baud_params(src_hz, target_hz);
        self.prescaler = prescaler;
        self.sck_div = sck_div;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// SPI clock prescaler values.
/// Maps to the PRESCALE field in the CCR register.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Prescaler {
    /// Divide by 1
    #[default]
    Div1 = 0,
    /// Divide by 2
    Div2 = 1,
    /// Divide by 4
    Div4 = 2,
    /// Divide by 8
    Div8 = 3,
    /// Divide by 16
    Div16 = 4,
    /// Divide by 32
    Div32 = 5,
    /// Divide by 64
    Div64 = 6,
    /// Divide by 128
    Div128 = 7,
}

impl Prescaler {
    /// Get the divisor value for this prescaler setting.
    pub const fn divisor(self) -> u32 {
        1 << (self as u8)
    }

    /// Convert from register value to Prescaler.
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0x7 {
            0 => Self::Div1,
            1 => Self::Div2,
            2 => Self::Div4,
            3 => Self::Div8,
            4 => Self::Div16,
            5 => Self::Div32,
            6 => Self::Div64,
            _ => Self::Div128,
        }
    }
}

/// Compute prescaler and SCKDIV for the desired baud rate.
/// Returns (prescaler, sckdiv) where:
/// - prescaler is a Prescaler enum value
/// - sckdiv is 0-255
///
/// Baud = src_hz / (prescaler.divisor() * (SCKDIV + 2))
pub(super) fn compute_baud_params(src_hz: u32, baud_hz: u32) -> (Prescaler, u8) {
    if baud_hz == 0 {
        return (Prescaler::Div1, 0);
    }

    let prescalers = [
        Prescaler::Div1,
        Prescaler::Div2,
        Prescaler::Div4,
        Prescaler::Div8,
        Prescaler::Div16,
        Prescaler::Div32,
        Prescaler::Div64,
        Prescaler::Div128,
    ];

    let (best_prescaler, best_sckdiv, _) = prescalers.iter().fold(
        (Prescaler::Div1, 0u8, u32::MAX),
        |(best_pre, best_div, best_err), &prescaler| {
            let divisor = prescaler.divisor();
            let denom = divisor.saturating_mul(baud_hz);
            let sckdiv_calc = (src_hz + denom / 2) / denom;
            if sckdiv_calc < 2 {
                return (best_pre, best_div, best_err);
            }
            let sckdiv = (sckdiv_calc - 2).min(255) as u8;
            let actual_baud = src_hz / (divisor * (sckdiv as u32 + 2));
            let error = actual_baud.abs_diff(baud_hz);
            if error < best_err {
                (prescaler, sckdiv, error)
            } else {
                (best_pre, best_div, best_err)
            }
        },
    );

    (best_prescaler, best_sckdiv)
}

/// SPI slave configuration
#[derive(Copy, Clone)]
#[non_exhaustive]
pub struct SlaveConfig {
    /// SPI mode (must match master). Use MODE_0, MODE_1, MODE_2, or MODE_3.
    pub mode: embedded_hal_02::spi::Mode,
    /// Bit order (must match master)
    pub bit_order: BitOrder,
    /// Bits per frame (8 for typical use)
    pub bits_per_frame: u16,
    /// Clock source for the LPSPI peripheral.
    pub clock_source: LpspiClockSel,
    /// Power state for the clock source.
    pub clock_power: PoweredClock,
    /// Pre-divider applied to the clock source.
    pub clock_div: Div4,
}

impl SlaveConfig {
    /// Create a new slave configuration with defaults (MODE_0)
    pub fn new() -> Self {
        Self {
            mode: embedded_hal_02::spi::MODE_0,
            bit_order: BitOrder::MsbFirst,
            bits_per_frame: 8,
            clock_source: LpspiClockSel::FroHfDiv,
            clock_power: PoweredClock::NormalEnabledDeepSleepDisabled,
            clock_div: Div4::no_div(),
        }
    }

    /// Set SPI mode (MODE_0, MODE_1, MODE_2, or MODE_3)
    pub fn mode(&mut self, mode: embedded_hal_02::spi::Mode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Set bit order
    pub fn bit_order(&mut self, order: BitOrder) -> &mut Self {
        self.bit_order = order;
        self
    }

    /// Set bits per frame (valid range: 1-4096, will be clamped)
    pub fn bits_per_frame(&mut self, bits: u16) -> &mut Self {
        self.bits_per_frame = bits;
        self
    }

    /// Set clock source for the LPSPI peripheral.
    pub fn clock_source(&mut self, source: LpspiClockSel) -> &mut Self {
        self.clock_source = source;
        self
    }

    /// Set power state for the clock source.
    pub fn clock_power(&mut self, power: PoweredClock) -> &mut Self {
        self.clock_power = power;
        self
    }

    /// Set pre-divider for the clock source.
    pub fn clock_div(&mut self, div: Div4) -> &mut Self {
        self.clock_div = div;
        self
    }
}

impl Default for SlaveConfig {
    fn default() -> Self {
        Self::new()
    }
}
