//! Low-power timer (LPTIMER0 / LPTIMER1).
//!
//! Register programming follows the vendor `tremo_lptimer` driver and the
//! ASR6601 SDK examples (`lptimer_wakeup_stop`, `external_trigger`,
//! `external_clock`, `lptimer_encoder`).
//!
//! Many multi-bit CFGR/CR fields and the IER/ICR/CMP/ARR/CNT/SR1 field layouts
//! are missing from the PAC/SVD. Those bits are programmed with the masks from
//! `tremo_lptimer.h`.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Poll;

use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{self, AlternateFunction, Flex};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt as TypelevelInterrupt};
use crate::rcc::{self, Peripheral as RccPeripheral};
use crate::{Peri, PeripheralType, interrupt, pac, peripherals};

const POLL_LIMIT: u32 = 1_000_000;

const ISR_CMPM: u32 = 1 << 0;
const ISR_ARRM: u32 = 1 << 1;
const ISR_EXTTRIG: u32 = 1 << 2;
const ISR_CMPOK: u32 = 1 << 3;
const ISR_ARROK: u32 = 1 << 4;
const ISR_UP: u32 = 1 << 5;
const ISR_DOWN: u32 = 1 << 6;
const ISR_CFGROK: u32 = 1 << 7;
const ISR_CROK: u32 = 1 << 8;

const IT_MASK: u32 = ISR_CMPM | ISR_ARRM | ISR_EXTTRIG | ISR_CMPOK | ISR_ARROK | ISR_UP | ISR_DOWN;

const CSR_CMPM: u32 = 1 << 0;
const CSR_ARRM: u32 = 1 << 1;
const CSR_EXTTRIG: u32 = 1 << 2;
const CSR_UP: u32 = 1 << 3;
const CSR_DOWN: u32 = 1 << 4;

const CFGR_CKPOL_MASK: u32 = 0x6;
const CFGR_CKFLT_MASK: u32 = 0x18;
const CFGR_TRGFLT_MASK: u32 = 0xc0;
const CFGR_PRESC_MASK: u32 = 0xe00;
const CFGR_TRIGSEL_MASK: u32 = 0xe000;
const CFGR_TRIGEN_MASK: u32 = 0x60000;

const CR_SNGSTRT: u32 = 1 << 1;
const CR_CNTSTRT: u32 = 1 << 2;

const WKUP_MASK: u32 = (1 << 25) | (1 << 26) | (1 << 27) | (1 << 28) | (1 << 29) | (1 << 30);

static WAKERS: [AtomicWaker; 2] = [const { AtomicWaker::new() }; 2];
static PENDING: [AtomicU32; 2] = [const { AtomicU32::new(0) }; 2];

/// LPTIMER driver error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// A bounded wait for a status bit expired.
    Timeout(TimeoutTarget),
    /// RCC rejected the clock or reset request.
    Rcc(rcc::Error),
}

/// Status bit that failed to become ready within the poll budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TimeoutTarget {
    /// `ISR.CFGROK` after a CFGR write.
    Cfgrok,
    /// `ISR.CROK` after a CR write.
    Crok,
    /// `ISR.ARROK` after an ARR write.
    Arrok,
    /// `ISR.CMPOK` after a CMP write.
    Cmpok,
    /// `CSR` acknowledge after an interrupt clear.
    ClearStatus,
    /// RCC functional-clock sync clear while changing the LPTIMER source.
    ClockSourceSync,
}

impl From<rcc::Error> for Error {
    fn from(value: rcc::Error) -> Self {
        Self::Rcc(value)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Timeout(target) => write!(f, "LPTIMER timeout waiting for {target:?}"),
            Self::Rcc(err) => write!(f, "LPTIMER RCC error: {err:?}"),
        }
    }
}

impl core::error::Error for Error {}

/// Kernel clock source selected in `RCC.CR1`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockSource {
    /// PCLK0.
    Pclk0,
    /// Internal ~4 MHz RC oscillator.
    Rco4m,
    /// External 32.768 kHz crystal oscillator.
    Xo32k,
    /// Internal 32.768 kHz RC oscillator.
    Rco32k,
    /// External clock input (RCC `EXTCLK_SEL`).
    ExtClk,
}

/// Counter clock prescaler (`CFGR.PRESC`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum Prescaler {
    Div1 = 0x0,
    Div2 = 0x200,
    Div4 = 0x400,
    Div8 = 0x600,
    Div16 = 0x800,
    Div32 = 0xa00,
    Div64 = 0xc00,
    Div128 = 0xe00,
}

/// External trigger polarity (`CFGR.TRIGEN`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TriggerPolarity {
    /// Software / no edge trigger.
    Software = 0x0,
    Rising = 0x20000,
    Falling = 0x40000,
    RisingFalling = 0x60000,
}

/// External trigger source (`CFGR.TRIGSEL`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TriggerSource {
    /// External trigger ETR pin.
    Etr = 0x0,
    Comp0 = 0x2000,
    Comp1 = 0x4000,
    RtcCyc = 0x6000,
    RtcAlarm0 = 0x8000,
    RtcAlarm1 = 0xa000,
    Gpio0 = 0xc000,
    Gpio1 = 0xe000,
}

/// Trigger digital filter length (`CFGR.TRGFLT`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TriggerFilter {
    None = 0x0,
    Cycles2 = 0x40,
    Cycles4 = 0x80,
    Cycles8 = 0xc0,
}

/// External clock digital filter length (`CFGR.CKFLT`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum ClockFilter {
    None = 0x0,
    Cycles2 = 0x8,
    Cycles4 = 0x10,
    Cycles8 = 0x18,
}

/// External clock edge polarity (`CFGR.CKPOL`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum ClockPolarity {
    Rising = 0x0,
    Falling = 0x2,
    Both = 0x4,
}

/// Count start mode programmed in `CR`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CountMode {
    /// Single-shot (`SNGSTRT`).
    Single,
    /// Continuous (`CNTSTRT`).
    Continuous,
}

/// Interrupt / status flags that map to `IER` / `SR1` / `ICR` bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterruptFlags(u32);

impl InterruptFlags {
    pub const CMPM: Self = Self(ISR_CMPM);
    pub const ARRM: Self = Self(ISR_ARRM);
    pub const EXTTRIG: Self = Self(ISR_EXTTRIG);
    pub const CMPOK: Self = Self(ISR_CMPOK);
    pub const ARROK: Self = Self(ISR_ARROK);
    pub const UP: Self = Self(ISR_UP);
    pub const DOWN: Self = Self(ISR_DOWN);

    /// Empty set.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Combine flag sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Bits used by the vendor interrupt API (excludes CFGROK/CROK).
    pub const fn bits(self) -> u32 {
        self.0 & IT_MASK
    }

    /// Whether every bit in `other` is present.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl core::ops::BitOr for InterruptFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl core::ops::BitOrAssign for InterruptFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

/// Wake-from-stop sources programmed in `CFGR` (vendor `lptimer_wkup_t`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WakeupFlags(u32);

impl WakeupFlags {
    pub const CMPM: Self = Self(1 << 25);
    pub const ARRM: Self = Self(1 << 26);
    pub const EXTTRIG: Self = Self(1 << 27);
    pub const UP: Self = Self(1 << 28);
    pub const DOWN: Self = Self(1 << 29);
    pub const OUT: Self = Self(1 << 30);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn bits(self) -> u32 {
        self.0 & WKUP_MASK
    }
}

impl core::ops::BitOr for WakeupFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl core::ops::BitOrAssign for WakeupFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

/// LPTIMER initialization configuration (vendor `lptimer_init_t` plus RCC source).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Kernel clock selected in `RCC.CR1`.
    pub clock_source: ClockSource,
    /// Count external edges (`CFGR.COUNTMODE`).
    pub count_by_external: bool,
    /// Counter clock prescaler.
    pub prescaler: Prescaler,
    /// ARR/CMP preload (`CFGR.PRELOAD`).
    pub autoreload_preload: bool,
    /// Invert waveform polarity (`CFGR.WAVPOL`).
    pub wavpol_inverted: bool,
}

impl Config {
    /// Defaults matching the SDK wakeup example (XO32K, /1, no preload).
    pub const fn new() -> Self {
        Self {
            clock_source: ClockSource::Xo32k,
            count_by_external: false,
            prescaler: Prescaler::Div1,
            autoreload_preload: false,
            wavpol_inverted: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> &'static pac::lptimer0::RegisterBlock;
        fn rcc_peripheral() -> RccPeripheral;
        fn nv_interrupt() -> pac::Interrupt;
        const INDEX: usize;
        fn clock_source_sync() -> bool;
        fn gate_functional_clock(enable: bool);
        fn set_clock_source(source: ClockSource);
    }

    pub trait AfPin {
        const AF: AlternateFunction;
    }
}

/// LPTIMER peripheral instance.
#[allow(private_bounds)]
pub trait Instance: sealed::Instance + PeripheralType + 'static {
    /// NVIC vector for this instance.
    type Interrupt: TypelevelInterrupt;
}

impl sealed::Instance for peripherals::LPTIMER0 {
    #[inline]
    fn regs() -> &'static pac::lptimer0::RegisterBlock {
        unsafe { &*pac::Lptimer0::ptr() }
    }

    #[inline]
    fn rcc_peripheral() -> RccPeripheral {
        RccPeripheral::Lptimer0
    }

    #[inline]
    fn nv_interrupt() -> pac::Interrupt {
        pac::Interrupt::LPTIMER0
    }

    const INDEX: usize = 0;

    fn clock_source_sync() -> bool {
        unsafe { pac::Rcc::steal() }
            .sr1()
            .read()
            .lptimer0_clk_en_sync()
            .bit_is_set()
    }

    fn gate_functional_clock(enable: bool) {
        critical_section::with(|_| {
            unsafe { pac::Rcc::steal() }
                .cgr1()
                .modify(|_, w| w.lptimer0_clk_en().bit(enable));
        });
    }

    fn set_clock_source(source: ClockSource) {
        critical_section::with(|_| {
            let rcc = unsafe { pac::Rcc::steal() };
            rcc.cr1().modify(|_, w| match source {
                ClockSource::ExtClk => w.lptimer0_extclk_sel().set_bit(),
                ClockSource::Pclk0 => {
                    w.lptimer0_extclk_sel().clear_bit();
                    w.lptimer0_clk_sel().pclk0()
                }
                ClockSource::Rco4m => {
                    w.lptimer0_extclk_sel().clear_bit();
                    w.lptimer0_clk_sel().rco4m()
                }
                ClockSource::Xo32k => {
                    w.lptimer0_extclk_sel().clear_bit();
                    w.lptimer0_clk_sel().xo32k()
                }
                ClockSource::Rco32k => {
                    w.lptimer0_extclk_sel().clear_bit();
                    w.lptimer0_clk_sel().rco32k()
                }
            });
        });
    }
}

impl Instance for peripherals::LPTIMER0 {
    type Interrupt = interrupt::typelevel::LPTIMER0;
}

impl sealed::Instance for peripherals::LPTIMER1 {
    #[inline]
    fn regs() -> &'static pac::lptimer0::RegisterBlock {
        unsafe { &*pac::Lptimer1::ptr() }
    }

    #[inline]
    fn rcc_peripheral() -> RccPeripheral {
        RccPeripheral::Lptimer1
    }

    #[inline]
    fn nv_interrupt() -> pac::Interrupt {
        pac::Interrupt::LPTIMER1
    }

    const INDEX: usize = 1;

    fn clock_source_sync() -> bool {
        unsafe { pac::Rcc::steal() }
            .sr1()
            .read()
            .lptimer1_clk_en_sync()
            .bit_is_set()
    }

    fn gate_functional_clock(enable: bool) {
        critical_section::with(|_| {
            unsafe { pac::Rcc::steal() }
                .cgr1()
                .modify(|_, w| w.lptimer1_clk_en().bit(enable));
        });
    }

    fn set_clock_source(source: ClockSource) {
        critical_section::with(|_| {
            let rcc = unsafe { pac::Rcc::steal() };
            rcc.cr1().modify(|_, w| match source {
                ClockSource::ExtClk => w.lptimer1_extclk_sel().set_bit(),
                ClockSource::Pclk0 => {
                    w.lptimer1_extclk_sel().clear_bit();
                    w.lptimer1_clk_sel().pclk0()
                }
                ClockSource::Rco4m => {
                    w.lptimer1_extclk_sel().clear_bit();
                    w.lptimer1_clk_sel().rco4m()
                }
                ClockSource::Xo32k => {
                    w.lptimer1_extclk_sel().clear_bit();
                    w.lptimer1_clk_sel().xo32k()
                }
                ClockSource::Rco32k => {
                    w.lptimer1_extclk_sel().clear_bit();
                    w.lptimer1_clk_sel().rco32k()
                }
            });
        });
    }
}

impl Instance for peripherals::LPTIMER1 {
    type Interrupt = interrupt::typelevel::LPTIMER1;
}

/// Interrupt handler for one LPTIMER instance.
///
/// Clears pending `SR1` flags acknowledged by the vendor clear sequence and
/// wakes tasks waiting on [`LpTimer::wait_for`].
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        let pending = regs.sr1().read().bits() & regs.ier().read().bits() & IT_MASK;
        if pending == 0 {
            return;
        }

        unsafe {
            regs.icr().write_with_zero(|w| w.bits(pending));
        }

        // Vendor wakeup example waits for CSR only for CMPM/ARRM; wait for any
        // flag that has a CSR acknowledge bit.
        let csr_mask = interrupt_to_csr(pending);
        if csr_mask != 0 {
            for _ in 0..POLL_LIMIT {
                if regs.csr().read().bits() & csr_mask == csr_mask {
                    break;
                }
                core::hint::spin_loop();
            }
        }

        PENDING[T::INDEX].fetch_or(pending, Ordering::Release);
        WAKERS[T::INDEX].wake();
    }
}

/// Owned low-power timer.
pub struct LpTimer<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> LpTimer<'d, T> {
    /// Reset the block, select `config.clock_source`, enable clocks, and apply
    /// the vendor `lptimer_init` fields. The counter remains disabled.
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        // Match the wakeup example: gate functional clock, pulse reset, select
        // source, then re-enable.
        let _ = rcc::disable_peripheral(T::rcc_peripheral());
        rcc::reset_peripheral(T::rcc_peripheral())?;
        set_instance_clock_source::<T>(config.clock_source)?;
        rcc::enable_peripheral(T::rcc_peripheral())?;

        T::nv_interrupt().disable();
        T::nv_interrupt().unpend();
        PENDING[T::INDEX].store(0, Ordering::Release);

        let this = Self { _peri: peri };
        this.apply_init(config)?;
        Ok(this)
    }

    /// Re-apply the vendor init fields while the counter is stopped.
    pub fn configure(&self, config: Config) -> Result<(), Error> {
        set_instance_clock_source::<T>(config.clock_source)?;
        self.apply_init(config)
    }

    fn apply_init(&self, config: Config) -> Result<(), Error> {
        wait_isr::<T>(ISR_CFGROK, TimeoutTarget::Cfgrok)?;

        self.regs().cfgr().modify(|_, w| {
            if config.count_by_external {
                w.countmode().set_bit()
            } else {
                w.countmode().clear_bit()
            }
        });
        wait_isr::<T>(ISR_CFGROK, TimeoutTarget::Cfgrok)?;

        self.modify_cfgr_bits(CFGR_PRESC_MASK, 0)?;
        self.modify_cfgr_bits(CFGR_PRESC_MASK, config.prescaler as u32)?;

        self.regs().cfgr().modify(|_, w| {
            if config.autoreload_preload {
                w.preload().set_bit()
            } else {
                w.preload().clear_bit()
            }
        });
        wait_isr::<T>(ISR_CFGROK, TimeoutTarget::Cfgrok)?;

        self.regs().cfgr().modify(|_, w| {
            if config.wavpol_inverted {
                w.wavpol().set_bit()
            } else {
                w.wavpol().clear_bit()
            }
        });
        wait_isr::<T>(ISR_CFGROK, TimeoutTarget::Cfgrok)?;

        Ok(())
    }

    /// Enable or disable the peripheral (`CR.ENABLE`).
    pub fn set_enabled(&self, enabled: bool) -> Result<(), Error> {
        self.regs().cr().modify(|_, w| w.enable().bit(enabled));
        wait_isr::<T>(ISR_CROK, TimeoutTarget::Crok)
    }

    /// Write ARR and wait for `ARROK`.
    pub fn set_arr(&self, value: u16) -> Result<(), Error> {
        unsafe {
            self.regs().arr().write_with_zero(|w| w.bits(u32::from(value)));
        }
        wait_isr::<T>(ISR_ARROK, TimeoutTarget::Arrok)
    }

    /// Write CMP and wait for `CMPOK`.
    pub fn set_cmp(&self, value: u16) -> Result<(), Error> {
        unsafe {
            self.regs().cmp().write_with_zero(|w| w.bits(u32::from(value)));
        }
        wait_isr::<T>(ISR_CMPOK, TimeoutTarget::Cmpok)
    }

    /// Current ARR value.
    pub fn arr(&self) -> u16 {
        self.regs().arr().read().bits() as u16
    }

    /// Current CMP value.
    pub fn cmp(&self) -> u16 {
        self.regs().cmp().read().bits() as u16
    }

    /// Current counter value.
    pub fn counter(&self) -> u16 {
        self.regs().cnt().read().bits() as u16
    }

    /// Start counting in single or continuous mode (vendor `lptimer_config_count_mode`).
    pub fn start(&self, mode: CountMode) -> Result<(), Error> {
        let bit = match mode {
            CountMode::Single => CR_SNGSTRT,
            CountMode::Continuous => CR_CNTSTRT,
        };
        self.regs().cr().modify(|r, w| unsafe { w.bits(r.bits() | bit) });
        wait_isr::<T>(ISR_CROK, TimeoutTarget::Crok)
    }

    /// Clear a start-mode bit.
    pub fn stop_mode(&self, mode: CountMode) -> Result<(), Error> {
        let bit = match mode {
            CountMode::Single => CR_SNGSTRT,
            CountMode::Continuous => CR_CNTSTRT,
        };
        self.regs().cr().modify(|r, w| unsafe { w.bits(r.bits() & !bit) });
        wait_isr::<T>(ISR_CROK, TimeoutTarget::Crok)
    }

    /// Enable or disable timeout mode (`CFGR.TIMEOUT`).
    pub fn set_timeout_enabled(&self, enabled: bool) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| w.timeout().bit(enabled));
        wait_isr::<T>(ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Enable or disable wake sources used with STOP modes.
    pub fn set_wakeup(&self, flags: WakeupFlags, enabled: bool) -> Result<(), Error> {
        let mask = flags.bits();
        self.regs().cfgr().modify(|r, w| unsafe {
            let bits = if enabled { r.bits() | mask } else { r.bits() & !mask };
            w.bits(bits)
        });
        wait_isr::<T>(ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Enable or disable waveform generation (`CFGR.WAVE`).
    pub fn set_wave_enabled(&self, enabled: bool) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| w.wave().bit(enabled));
        wait_isr::<T>(ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Enable or disable encoder mode (`CFGR.ENC`).
    pub fn set_encoder_enabled(&self, enabled: bool) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| w.enc().bit(enabled));
        wait_isr::<T>(ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Configure external trigger polarity.
    pub fn set_trigger_polarity(&self, polarity: TriggerPolarity) -> Result<(), Error> {
        self.modify_cfgr_bits(CFGR_TRIGEN_MASK, 0)?;
        self.modify_cfgr_bits(CFGR_TRIGEN_MASK, polarity as u32)
    }

    /// Configure external trigger source.
    pub fn set_trigger_source(&self, source: TriggerSource) -> Result<(), Error> {
        self.modify_cfgr_bits(CFGR_TRIGSEL_MASK, 0)?;
        self.modify_cfgr_bits(CFGR_TRIGSEL_MASK, source as u32)
    }

    /// Configure the trigger digital filter value.
    pub fn set_trigger_filter(&self, filter: TriggerFilter) -> Result<(), Error> {
        self.modify_cfgr_bits(CFGR_TRGFLT_MASK, 0)?;
        self.modify_cfgr_bits(CFGR_TRGFLT_MASK, filter as u32)
    }

    /// Enable or disable the trigger digital filter.
    pub fn set_trigger_filter_enabled(&self, enabled: bool) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| w.trgflt_en().bit(enabled));
        wait_isr::<T>(ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Configure the external clock digital filter value.
    pub fn set_clock_filter(&self, filter: ClockFilter) -> Result<(), Error> {
        self.modify_cfgr_bits(CFGR_CKFLT_MASK, 0)?;
        self.modify_cfgr_bits(CFGR_CKFLT_MASK, filter as u32)
    }

    /// Enable or disable the external clock digital filter.
    pub fn set_clock_filter_enabled(&self, enabled: bool) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| w.ckflt_en().bit(enabled));
        wait_isr::<T>(ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Configure external clock edge polarity.
    pub fn set_clock_polarity(&self, polarity: ClockPolarity) -> Result<(), Error> {
        self.modify_cfgr_bits(CFGR_CKPOL_MASK, 0)?;
        self.modify_cfgr_bits(CFGR_CKPOL_MASK, polarity as u32)
    }

    /// Configure the counter clock prescaler.
    pub fn set_prescaler(&self, prescaler: Prescaler) -> Result<(), Error> {
        self.modify_cfgr_bits(CFGR_PRESC_MASK, 0)?;
        self.modify_cfgr_bits(CFGR_PRESC_MASK, prescaler as u32)
    }

    /// Enable or disable interrupt sources and unmask the NVIC vector when any
    /// source is enabled.
    pub fn set_interrupt_enabled(&self, flags: InterruptFlags, enabled: bool) {
        let mask = flags.bits();
        self.regs().ier().modify(|r, w| unsafe {
            let bits = if enabled { r.bits() | mask } else { r.bits() & !mask };
            w.bits(bits)
        });

        if self.regs().ier().read().bits() & IT_MASK != 0 {
            T::nv_interrupt().unpend();
            unsafe { T::nv_interrupt().enable() };
        } else {
            T::nv_interrupt().disable();
        }
    }

    /// Clear interrupt status bits (vendor `lptimer_clear_interrupt` + CSR wait).
    pub fn clear_interrupt(&self, flags: InterruptFlags) -> Result<(), Error> {
        let mask = flags.bits();
        unsafe {
            self.regs().icr().write_with_zero(|w| w.bits(mask));
        }

        let csr_mask = interrupt_to_csr(mask);
        if csr_mask == 0 {
            return Ok(());
        }

        for _ in 0..POLL_LIMIT {
            if self.regs().csr().read().bits() & csr_mask == csr_mask {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(Error::Timeout(TimeoutTarget::ClearStatus))
    }

    /// Raw `SR1` interrupt status bits that are set.
    pub fn interrupt_status(&self) -> InterruptFlags {
        InterruptFlags(self.regs().sr1().read().bits() & IT_MASK)
    }

    /// Raw sticky `ISR` flags (includes CFGROK/CROK/CMPOK/ARROK).
    pub fn status(&self) -> u32 {
        self.regs().isr().read().bits()
    }

    /// Wait until any of `flags` is reported by [`InterruptHandler`].
    ///
    /// The handler clears the hardware status; this future consumes the latched
    /// software pending bits for `flags`.
    pub async fn wait_for(&self, flags: InterruptFlags) -> InterruptFlags {
        let mask = flags.bits();
        poll_fn(|cx| {
            WAKERS[T::INDEX].register(cx.waker());

            let latched = PENDING[T::INDEX].load(Ordering::Acquire) & mask;
            if latched != 0 {
                PENDING[T::INDEX].fetch_and(!latched, Ordering::AcqRel);
                return Poll::Ready(InterruptFlags(latched));
            }

            let live = self.regs().sr1().read().bits() & mask;
            if live != 0 {
                // Handler has not run yet; clear in software path as the SDK does.
                let _ = self.clear_interrupt(InterruptFlags(live));
                PENDING[T::INDEX].fetch_and(!live, Ordering::AcqRel);
                return Poll::Ready(InterruptFlags(live));
            }

            Poll::Pending
        })
        .await
    }

    /// Mux a documented `IN1` pin to this timer and return the owned GPIO wrap.
    pub fn take_in1_pin(&self, pin: Peri<'d, impl In1Pin<T>>) -> Flex<'d> {
        configure_af(pin)
    }

    /// Mux a documented `IN2` pin to this timer and return the owned GPIO wrap.
    pub fn take_in2_pin(&self, pin: Peri<'d, impl In2Pin<T>>) -> Flex<'d> {
        configure_af(pin)
    }

    /// Mux a documented `ETR` pin to this timer and return the owned GPIO wrap.
    pub fn take_etr_pin(&self, pin: Peri<'d, impl EtrPin<T>>) -> Flex<'d> {
        configure_af(pin)
    }

    fn modify_cfgr_bits(&self, mask: u32, value: u32) -> Result<(), Error> {
        self.regs()
            .cfgr()
            .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | (value & mask)) });
        wait_isr::<T>(ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    #[inline]
    fn regs(&self) -> &'static pac::lptimer0::RegisterBlock {
        T::regs()
    }
}

impl<'d, T: Instance> Drop for LpTimer<'d, T> {
    fn drop(&mut self) {
        T::nv_interrupt().disable();
        let _ = self.set_enabled(false);
        let _ = rcc::disable_peripheral(T::rcc_peripheral());
        let _ = rcc::reset_peripheral(T::rcc_peripheral());
    }
}

fn configure_af<'d, P>(pin: Peri<'d, P>) -> Flex<'d>
where
    P: gpio::Pin + sealed::AfPin,
{
    let af = P::AF;
    let mut flex = Flex::new(pin);
    flex.set_alternate_function(af);
    flex
}

fn interrupt_to_csr(flags: u32) -> u32 {
    let mut csr = 0;
    if flags & ISR_CMPM != 0 {
        csr |= CSR_CMPM;
    }
    if flags & ISR_ARRM != 0 {
        csr |= CSR_ARRM;
    }
    if flags & ISR_EXTTRIG != 0 {
        csr |= CSR_EXTTRIG;
    }
    if flags & ISR_UP != 0 {
        csr |= CSR_UP;
    }
    if flags & ISR_DOWN != 0 {
        csr |= CSR_DOWN;
    }
    csr
}

fn wait_isr<T: Instance>(mask: u32, target: TimeoutTarget) -> Result<(), Error> {
    for _ in 0..POLL_LIMIT {
        if T::regs().isr().read().bits() & mask == mask {
            return Ok(());
        }
        core::hint::spin_loop();
    }
    if T::regs().isr().read().bits() & mask == mask {
        Ok(())
    } else {
        Err(Error::Timeout(target))
    }
}

fn set_instance_clock_source<T: Instance>(source: ClockSource) -> Result<(), Error> {
    if T::clock_source_sync() {
        // Vendor disables the functional clock before changing CR1.
        T::gate_functional_clock(false);

        for _ in 0..POLL_LIMIT {
            if !T::clock_source_sync() {
                break;
            }
            core::hint::spin_loop();
        }
        if T::clock_source_sync() {
            return Err(Error::Timeout(TimeoutTarget::ClockSourceSync));
        }
    }

    T::set_clock_source(source);
    Ok(())
}

/// Documented LPTIMER `IN1` pin for an instance.
pub trait In1Pin<T: Instance>: gpio::Pin + sealed::AfPin {}
/// Documented LPTIMER `IN2` pin for an instance.
pub trait In2Pin<T: Instance>: gpio::Pin + sealed::AfPin {}
/// Documented LPTIMER `ETR` pin for an instance.
pub trait EtrPin<T: Instance>: gpio::Pin + sealed::AfPin {}

macro_rules! impl_lptimer0_pin {
    ($trait:ident, $pin:ident, $af:ident) => {
        impl sealed::AfPin for peripherals::$pin {
            const AF: AlternateFunction = AlternateFunction::$af;
        }
        impl $trait<peripherals::LPTIMER0> for peripherals::$pin {}
    };
}

// SDK examples (ASR6601SE/CB EVAL):
//   GPIOD PIN10 AF2 = IN1 (GP58)
//   GPIOD PIN12 AF5 = IN2 (GP60)
//   GPIOD PIN14 AF3 = ETR (GP62)
// No LPTIMER1 or OUT routes are documented by the vendor examples.
impl_lptimer0_pin!(In1Pin, PD10, Function2);
impl_lptimer0_pin!(In2Pin, PD12, Function5);
impl_lptimer0_pin!(EtrPin, PD14, Function3);

/// Alternate-function numbers documented by the LPTIMER0 SDK examples.
pub mod af {
    use crate::gpio::AlternateFunction;

    /// `PD10` / GP58 as LPTIMER0 IN1.
    pub const LPTIMER0_IN1: AlternateFunction = AlternateFunction::Function2;
    /// `PD12` / GP60 as LPTIMER0 IN2.
    pub const LPTIMER0_IN2: AlternateFunction = AlternateFunction::Function5;
    /// `PD14` / GP62 as LPTIMER0 ETR.
    pub const LPTIMER0_ETR: AlternateFunction = AlternateFunction::Function3;
}
