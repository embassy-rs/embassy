//! Low-power timer (LPTIMER0 / LPTIMER1).
//!
//! Register programming follows the vendor `tremo_lptimer` driver and the
//! ASR6601 SDK examples (`lptimer_wakeup_stop`, `external_trigger`,
//! `external_clock`, `lptimer_encoder`).
//!
//! PAC field accessors exist for ISR, IER, ICR, CR, CFGR, CSR, SR1,
//! CMP, ARR and CNT and are used below. The numeric masks that remain
//! (`ISR_*`, `CSR_*`, `IT_MASK`, `WKUP_MASK`) are the public flag ABIs
//! and the software pending bitmap, with positions cross-checked against
//! those accessors.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Poll;

use embassy_hal_internal::impl_peripheral;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{self, AlternateFunction, Flex};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt as TypelevelInterrupt};
use crate::rcc::{self, Peripheral as RccPeripheral};
use crate::{Peri, PeripheralType, interrupt, pac, peripherals};

const POLL_LIMIT: u32 = 1_000_000;

pub(crate) const ISR_CMPM: u32 = 1 << 0;
pub(crate) const ISR_ARRM: u32 = 1 << 1;
pub(crate) const ISR_EXTTRIG: u32 = 1 << 2;
pub(crate) const ISR_CMPOK: u32 = 1 << 3;
pub(crate) const ISR_ARROK: u32 = 1 << 4;
pub(crate) const ISR_UP: u32 = 1 << 5;
pub(crate) const ISR_DOWN: u32 = 1 << 6;
pub(crate) const ISR_CFGROK: u32 = 1 << 7;
pub(crate) const ISR_CROK: u32 = 1 << 8;

const IT_MASK: u32 = ISR_CMPM | ISR_ARRM | ISR_EXTTRIG | ISR_CMPOK | ISR_ARROK | ISR_UP | ISR_DOWN;

const CSR_CMPM: u32 = 1 << 0;
const CSR_ARRM: u32 = 1 << 1;
const CSR_EXTTRIG: u32 = 1 << 2;
const CSR_UP: u32 = 1 << 3;
const CSR_DOWN: u32 = 1 << 4;

const WKUP_MASK: u32 = (1 << 25) | (1 << 26) | (1 << 27) | (1 << 28) | (1 << 29) | (1 << 30);

static WAKERS: [AtomicWaker; 2] = [const { AtomicWaker::new() }; 2];
static PENDING: [AtomicU32; 2] = [const { AtomicU32::new(0) }; 2];

/// LPTIMER driver error.
///
/// Deliberately a single type: `Timeout` is returned by both init handshakes
/// (`create`, `configure`) and runtime setters, so an `Error`/`ConfigError`
/// split would duplicate it in both enums for no checking benefit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// A bounded wait for a status bit expired.
    Timeout(TimeoutTarget),
    /// RCC rejected the clock or reset request.
    Rcc(rcc::Error),
}

/// Status bit that failed to become ready within the poll budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
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
    /// No prescaling.
    Div1 = 0x0,
    /// Divide the counter clock by 2.
    Div2 = 0x200,
    /// Divide the counter clock by 4.
    Div4 = 0x400,
    /// Divide the counter clock by 8.
    Div8 = 0x600,
    /// Divide the counter clock by 16.
    Div16 = 0x800,
    /// Divide the counter clock by 32.
    Div32 = 0xa00,
    /// Divide the counter clock by 64.
    Div64 = 0xc00,
    /// Divide the counter clock by 128.
    Div128 = 0xe00,
}

/// External trigger polarity (`CFGR.TRIGEN`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TriggerPolarity {
    /// Software / no edge trigger.
    Software = 0x0,
    /// Clock on rising edge.
    Rising = 0x20000,
    /// Clock on falling edge.
    Falling = 0x40000,
    /// Trigger on either edge.
    RisingFalling = 0x60000,
}

/// External trigger source (`CFGR.TRIGSEL`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TriggerSource {
    /// External trigger ETR pin.
    Etr = 0x0,
    /// Analog comparator 0 output.
    Comp0 = 0x2000,
    /// Analog comparator 1 output.
    Comp1 = 0x4000,
    /// RTC cycle tick.
    RtcCyc = 0x6000,
    /// RTC alarm 0.
    RtcAlarm0 = 0x8000,
    /// RTC alarm 1.
    RtcAlarm1 = 0xa000,
    /// GPIO trigger input 0.
    Gpio0 = 0xc000,
    /// GPIO trigger input 1.
    Gpio1 = 0xe000,
}

/// Trigger digital filter length (`CFGR.TRGFLT`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TriggerFilter {
    /// No digital filtering.
    None = 0x0,
    /// Digital filter over 2 clock cycles.
    Cycles2 = 0x40,
    /// Digital filter over 4 clock cycles.
    Cycles4 = 0x80,
    /// Digital filter over 8 clock cycles.
    Cycles8 = 0xc0,
}

/// External clock digital filter length (`CFGR.CKFLT`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum ClockFilter {
    /// No digital filtering.
    None = 0x0,
    /// Digital filter over 2 clock cycles.
    Cycles2 = 0x8,
    /// Digital filter over 4 clock cycles.
    Cycles4 = 0x10,
    /// Digital filter over 8 clock cycles.
    Cycles8 = 0x18,
}

/// External clock edge polarity (`CFGR.CKPOL`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum ClockPolarity {
    /// Clock on rising edge.
    Rising = 0x0,
    /// Clock on falling edge.
    Falling = 0x2,
    /// Clock on both edges.
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
    /// Compare match.
    pub const CMPM: Self = Self(ISR_CMPM);
    /// Autoreload match (overflow).
    pub const ARRM: Self = Self(ISR_ARRM);
    /// External trigger event.
    pub const EXTTRIG: Self = Self(ISR_EXTTRIG);
    /// Compare register write completed.
    pub const CMPOK: Self = Self(ISR_CMPOK);
    /// Autoreload register write completed.
    pub const ARROK: Self = Self(ISR_ARROK);
    /// Counter direction changed to up.
    pub const UP: Self = Self(ISR_UP);
    /// Counter direction changed to down.
    pub const DOWN: Self = Self(ISR_DOWN);

    /// Empty set.
    /// Empty flag set.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Combine flag sets.
    /// Combine flag sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Bits used by the vendor interrupt API (excludes CFGROK/CROK).
    /// Raw flag bits.
    pub const fn bits(self) -> u32 {
        self.0 & IT_MASK
    }

    /// Whether every bit in `other` is present.
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
    /// Compare match.
    /// Wake on compare match.
    pub const CMPM: Self = Self(1 << 25);
    /// Autoreload match (overflow).
    /// Wake on autoreload match (overflow).
    pub const ARRM: Self = Self(1 << 26);
    /// External trigger event.
    /// Wake on external trigger.
    pub const EXTTRIG: Self = Self(1 << 27);
    /// Counter direction changed to up.
    /// Wake on counter direction change to up.
    pub const UP: Self = Self(1 << 28);
    /// Counter direction changed to down.
    /// Wake on counter direction change to down.
    pub const DOWN: Self = Self(1 << 29);
    /// Wake on timer output edge.
    pub const OUT: Self = Self(1 << 30);

    /// Empty flag set.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Combine flag sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Raw flag bits.
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
#[non_exhaustive]
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

struct Info {
    regs: *const pac::lptim0::RegisterBlock,
    peripheral: RccPeripheral,
    number: usize,
    interrupt: pac::Interrupt,
}

impl Info {
    #[inline]
    fn regs(&self) -> &'static pac::lptim0::RegisterBlock {
        unsafe { &*self.regs }
    }

    fn clock_source_sync(&self) -> bool {
        let rcc = unsafe { pac::Rcc::steal() };
        match self.number {
            0 => rcc.sr1().read().lptim0_clk_en_sync().bit_is_set(),
            _ => rcc.sr1().read().lptim1_clk_en_sync().bit_is_set(),
        }
    }

    fn gate_functional_clock(&self, enable: bool) {
        critical_section::with(|_| {
            let rcc = unsafe { pac::Rcc::steal() };
            match self.number {
                0 => rcc.cgr1().modify(|_, w| w.lptim0_clk_en().bit(enable)),
                _ => rcc.cgr1().modify(|_, w| w.lptim1_clk_en().bit(enable)),
            };
        });
    }

    fn set_clock_source(&self, source: ClockSource) {
        critical_section::with(|_| {
            let rcc = unsafe { pac::Rcc::steal() };
            if self.number == 0 {
                rcc.cr1().modify(|_, w| match source {
                    ClockSource::ExtClk => w.lptim0_ext_clk_sel().set_bit(),
                    ClockSource::Pclk0 => {
                        w.lptim0_ext_clk_sel().clear_bit();
                        w.lptim0_clk_sel().pclk0()
                    }
                    ClockSource::Rco4m => {
                        w.lptim0_ext_clk_sel().clear_bit();
                        w.lptim0_clk_sel().rco4m()
                    }
                    ClockSource::Xo32k => {
                        w.lptim0_ext_clk_sel().clear_bit();
                        w.lptim0_clk_sel().xo32k()
                    }
                    ClockSource::Rco32k => {
                        w.lptim0_ext_clk_sel().clear_bit();
                        w.lptim0_clk_sel().rco32k()
                    }
                });
            } else {
                rcc.cr1().modify(|_, w| match source {
                    ClockSource::ExtClk => w.lptim1_ext_clk_sel().set_bit(),
                    ClockSource::Pclk0 => {
                        w.lptim1_ext_clk_sel().clear_bit();
                        w.lptim1_clk_sel().pclk0()
                    }
                    ClockSource::Rco4m => {
                        w.lptim1_ext_clk_sel().clear_bit();
                        w.lptim1_clk_sel().rco4m()
                    }
                    ClockSource::Xo32k => {
                        w.lptim1_ext_clk_sel().clear_bit();
                        w.lptim1_clk_sel().xo32k()
                    }
                    ClockSource::Rco32k => {
                        w.lptim1_ext_clk_sel().clear_bit();
                        w.lptim1_clk_sel().rco32k()
                    }
                });
            }
        });
    }
}

// SAFETY: `Info` only contains read-only register addresses and instance data.
unsafe impl Sync for Info {}

trait SealedInstance {
    fn info() -> &'static Info;
}

trait SealedAfPin {
    const AF: AlternateFunction;
}

/// LPTIMER peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + Into<AnyLpTimer> + 'static + Send {
    /// NVIC vector for this instance.
    type Interrupt: TypelevelInterrupt;
}

/// Type-erased LPTIMER token.
pub struct AnyLpTimer {
    _private: (),
}

impl_peripheral!(AnyLpTimer);

macro_rules! impl_instance {
    ($name:ident, $pac:ident, $interrupt:ident, $peripheral:ident, $number:expr) => {
        impl SealedInstance for peripherals::$name {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: pac::$pac::PTR,
                    peripheral: RccPeripheral::$peripheral,
                    number: $number,
                    interrupt: pac::Interrupt::$interrupt,
                };
                &INFO
            }
        }

        impl From<peripherals::$name> for AnyLpTimer {
            fn from(_: peripherals::$name) -> Self {
                Self { _private: () }
            }
        }

        impl Instance for peripherals::$name {
            type Interrupt = interrupt::typelevel::$interrupt;
        }
    };
}

impl_instance!(LPTIMER0, Lptim0, LPTIM0, Lptimer0, 0);
impl_instance!(LPTIMER1, Lptim1, LPTIM1, Lptimer1, 1);

/// Interrupt handler for one LPTIMER instance.
///
/// Clears pending `SR1` flags acknowledged by the vendor clear sequence and
/// wakes tasks waiting on [`LpTimer::wait_for`].
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        let regs = info.regs();
        let pending = regs.sr1().read().bits() & regs.ier().read().bits() & IT_MASK;
        if pending == 0 {
            return;
        }

        clear_icr_flags(regs, pending);

        // Vendor wakeup example waits for CSR only for CMPM/ARRM; wait for any
        // flag that has a CSR acknowledge bit. Bounded (`POLL_LIMIT`) so ISR
        // dwell is finite even on wedged hardware; the normal path answers
        // in a few cycles.
        let csr_mask = interrupt_to_csr(pending);
        if csr_mask != 0 {
            for _ in 0..POLL_LIMIT {
                if regs.csr().read().bits() & csr_mask == csr_mask {
                    break;
                }
                core::hint::spin_loop();
            }
        }

        PENDING[info.number].fetch_or(pending, Ordering::Release);
        WAKERS[info.number].wake();
    }
}

/// Owned low-power timer.
pub struct LpTimer<'d> {
    _peri: Peri<'d, AnyLpTimer>,
    info: &'static Info,
}

impl<'d> LpTimer<'d> {
    /// Reset the block, select `config.clock_source`, enable clocks, and apply
    /// the vendor `lptimer_init` fields. The counter remains disabled.
    ///
    /// Does not bind the update interrupt; use [`Self::new`] for
    /// interrupt-driven use.
    pub fn new_blocking<T: Instance>(peri: Peri<'d, T>, config: Config) -> Result<Self, Error> {
        Self::create(peri, config)
    }

    /// Reset the block, select `config.clock_source`, enable clocks, and apply
    /// the vendor `lptimer_init` fields. The counter remains disabled.
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        Self::create(peri, config)
    }

    fn create<T: Instance>(peri: Peri<'d, T>, config: Config) -> Result<Self, Error> {
        let info = T::info();
        // Match the wakeup example: gate functional clock, pulse reset, select
        // source, then re-enable.
        let _ = rcc::disable_peripheral(info.peripheral);
        rcc::reset_peripheral(info.peripheral)?;
        set_instance_clock_source(info, config.clock_source)?;
        rcc::enable_peripheral(info.peripheral)?;

        info.interrupt.disable();
        info.interrupt.unpend();
        PENDING[info.number].store(0, Ordering::Release);

        let this = Self {
            _peri: peri.into(),
            info,
        };
        this.apply_init(config)?;
        Ok(this)
    }

    /// Re-apply the vendor init fields while the counter is stopped.
    pub fn configure(&mut self, config: Config) -> Result<(), Error> {
        set_instance_clock_source(self.info, config.clock_source)?;
        self.apply_init(config)
    }

    fn apply_init(&self, config: Config) -> Result<(), Error> {
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)?;

        self.regs().cfgr().modify(|_, w| {
            if config.count_by_external {
                w.countmode().set_bit()
            } else {
                w.countmode().clear_bit()
            }
        });
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)?;

        self.regs().cfgr().modify(|_, w| match config.prescaler {
            Prescaler::Div1 => w.presc().div1(),
            Prescaler::Div2 => w.presc().div2(),
            Prescaler::Div4 => w.presc().div4(),
            Prescaler::Div8 => w.presc().div8(),
            Prescaler::Div16 => w.presc().div16(),
            Prescaler::Div32 => w.presc().div32(),
            Prescaler::Div64 => w.presc().div64(),
            Prescaler::Div128 => w.presc().div128(),
        });
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)?;

        self.regs().cfgr().modify(|_, w| {
            if config.autoreload_preload {
                w.preload().set_bit()
            } else {
                w.preload().clear_bit()
            }
        });
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)?;

        self.regs().cfgr().modify(|_, w| {
            if config.wavpol_inverted {
                w.wavpol().set_bit()
            } else {
                w.wavpol().clear_bit()
            }
        });
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)?;

        Ok(())
    }

    /// Enable or disable the peripheral (`CR.ENABLE`).
    pub fn set_enabled(&mut self, enabled: bool) -> Result<(), Error> {
        self.regs().cr().modify(|_, w| w.enable().bit(enabled));
        wait_isr(self.info, ISR_CROK, TimeoutTarget::Crok)
    }

    /// Write ARR and wait for `ARROK`.
    ///
    /// The block only latches ARR while enabled: call
    /// [`set_enabled`](Self::set_enabled)`(true)` first, otherwise this
    /// returns [`Error::Timeout`] (`Arrok`).
    pub fn set_arr(&mut self, value: u16) -> Result<(), Error> {
        // `write_with_zero` and the value writer are unsafe on ARR/CMP in
        // the PAC; the value is range-checked by the `u16` type.
        unsafe {
            self.regs().arr().write_with_zero(|w| w.arr().bits(value));
        }
        wait_isr(self.info, ISR_ARROK, TimeoutTarget::Arrok)
    }

    /// Write CMP and wait for `CMPOK`.
    ///
    /// Requires the timer to be enabled first, like [`set_arr`](Self::set_arr).
    pub fn set_cmp(&mut self, value: u16) -> Result<(), Error> {
        unsafe {
            self.regs().cmp().write_with_zero(|w| w.cmp().bits(value));
        }
        wait_isr(self.info, ISR_CMPOK, TimeoutTarget::Cmpok)
    }

    /// Current ARR value.
    pub fn arr(&self) -> u16 {
        self.regs().arr().read().arr().bits()
    }

    /// Current CMP value.
    pub fn cmp(&self) -> u16 {
        self.regs().cmp().read().cmp().bits()
    }

    /// Current counter value.
    pub fn counter(&self) -> u16 {
        self.regs().cnt().read().cnt().bits()
    }

    /// Start counting in single or continuous mode (vendor `lptimer_config_count_mode`).
    pub fn start(&mut self, mode: CountMode) -> Result<(), Error> {
        self.regs().cr().modify(|_, w| match mode {
            CountMode::Single => w.sngstrt().set_bit(),
            CountMode::Continuous => w.cntstrt().set_bit(),
        });
        wait_isr(self.info, ISR_CROK, TimeoutTarget::Crok)
    }

    /// Clear a start-mode bit.
    pub fn stop_mode(&mut self, mode: CountMode) -> Result<(), Error> {
        self.regs().cr().modify(|_, w| match mode {
            CountMode::Single => w.sngstrt().clear_bit(),
            CountMode::Continuous => w.cntstrt().clear_bit(),
        });
        wait_isr(self.info, ISR_CROK, TimeoutTarget::Crok)
    }

    /// Enable or disable timeout mode (`CFGR.TIMEOUT`).
    pub fn set_timeout_enabled(&mut self, enabled: bool) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| w.timeout().bit(enabled));
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Enable or disable wake sources used with STOP modes.
    pub fn set_wakeup(&mut self, flags: WakeupFlags, enabled: bool) -> Result<(), Error> {
        let mask = flags.bits();
        self.regs().cfgr().modify(|_, w| {
            if mask & WakeupFlags::CMPM.bits() != 0 {
                w.cmpm_wkup().bit(enabled);
            }
            if mask & WakeupFlags::ARRM.bits() != 0 {
                w.arrm_wkup().bit(enabled);
            }
            if mask & WakeupFlags::EXTTRIG.bits() != 0 {
                w.exttrig_wkup().bit(enabled);
            }
            if mask & WakeupFlags::UP.bits() != 0 {
                w.up_wkup().bit(enabled);
            }
            if mask & WakeupFlags::DOWN.bits() != 0 {
                w.down_wkup().bit(enabled);
            }
            if mask & WakeupFlags::OUT.bits() != 0 {
                w.out_wkup().bit(enabled);
            }
            w
        });
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Enable or disable waveform generation (`CFGR.WAVE`).
    pub fn set_wave_enabled(&mut self, enabled: bool) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| w.wave().bit(enabled));
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Enable or disable encoder mode (`CFGR.ENC`).
    pub fn set_encoder_enabled(&mut self, enabled: bool) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| w.enc().bit(enabled));
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Configure external trigger polarity.
    pub fn set_trigger_polarity(&mut self, polarity: TriggerPolarity) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| match polarity {
            TriggerPolarity::Software => w.trigen().software(),
            TriggerPolarity::Rising => w.trigen().rising(),
            TriggerPolarity::Falling => w.trigen().falling(),
            TriggerPolarity::RisingFalling => w.trigen().both(),
        });
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Configure external trigger source.
    pub fn set_trigger_source(&mut self, source: TriggerSource) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| match source {
            TriggerSource::Etr => w.trigsel().trig0(),
            TriggerSource::Comp0 => w.trigsel().trig1(),
            TriggerSource::Comp1 => w.trigsel().trig2(),
            TriggerSource::RtcCyc => w.trigsel().trig3(),
            TriggerSource::RtcAlarm0 => w.trigsel().trig4(),
            TriggerSource::RtcAlarm1 => w.trigsel().trig5(),
            TriggerSource::Gpio0 => w.trigsel().trig6(),
            TriggerSource::Gpio1 => w.trigsel().trig7(),
        });
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Configure the trigger digital filter value.
    pub fn set_trigger_filter(&mut self, filter: TriggerFilter) -> Result<(), Error> {
        self.regs()
            .cfgr()
            .modify(|_, w| unsafe { w.trgflt().bits((filter as u32 >> 6) as u8) });
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Enable or disable the trigger digital filter.
    pub fn set_trigger_filter_enabled(&mut self, enabled: bool) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| w.trgflt_en().bit(enabled));
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Configure the external clock digital filter value.
    pub fn set_clock_filter(&mut self, filter: ClockFilter) -> Result<(), Error> {
        self.regs()
            .cfgr()
            .modify(|_, w| unsafe { w.ckflt().bits((filter as u32 >> 3) as u8) });
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Enable or disable the external clock digital filter.
    pub fn set_clock_filter_enabled(&mut self, enabled: bool) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| w.ckflt_en().bit(enabled));
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Configure external clock edge polarity.
    pub fn set_clock_polarity(&mut self, polarity: ClockPolarity) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| match polarity {
            ClockPolarity::Rising => w.ckpol().encoder_mode_1(),
            ClockPolarity::Falling => w.ckpol().encoder_mode_2(),
            ClockPolarity::Both => w.ckpol().encoder_mode_3(),
        });
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Configure the counter clock prescaler.
    pub fn set_prescaler(&mut self, prescaler: Prescaler) -> Result<(), Error> {
        self.regs().cfgr().modify(|_, w| match prescaler {
            Prescaler::Div1 => w.presc().div1(),
            Prescaler::Div2 => w.presc().div2(),
            Prescaler::Div4 => w.presc().div4(),
            Prescaler::Div8 => w.presc().div8(),
            Prescaler::Div16 => w.presc().div16(),
            Prescaler::Div32 => w.presc().div32(),
            Prescaler::Div64 => w.presc().div64(),
            Prescaler::Div128 => w.presc().div128(),
        });
        wait_isr(self.info, ISR_CFGROK, TimeoutTarget::Cfgrok)
    }

    /// Enable or disable interrupt sources and unmask the NVIC vector when any
    /// source is enabled.
    pub fn set_interrupt_enabled(&mut self, flags: InterruptFlags, enabled: bool) {
        let mask = flags.bits();
        self.regs().ier().modify(|r, w| {
            let bits = if enabled { r.bits() | mask } else { r.bits() & !mask };
            unsafe { w.bits(bits) }
        });

        let ier = self.regs().ier().read();
        let any_enabled = ier.cmpmie().bit_is_set()
            || ier.arrmie().bit_is_set()
            || ier.exttrigie().bit_is_set()
            || ier.cmpokie().bit_is_set()
            || ier.arrokie().bit_is_set()
            || ier.upie().bit_is_set()
            || ier.downie().bit_is_set();
        if any_enabled {
            self.info.interrupt.unpend();
            unsafe { self.info.interrupt.enable() };
        } else {
            self.info.interrupt.disable();
        }
    }

    /// Clear interrupt status bits (vendor `lptimer_clear_interrupt` + CSR wait).
    pub fn clear_interrupt(&mut self, flags: InterruptFlags) -> Result<(), Error> {
        let mask = flags.bits();
        clear_icr_flags(self.regs(), mask);

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
    pub async fn wait_for(&mut self, flags: InterruptFlags) -> InterruptFlags {
        let mask = flags.bits();
        if mask == 0 {
            return InterruptFlags::empty();
        }
        self.set_interrupt_enabled(flags, true);
        poll_fn(|cx| {
            WAKERS[self.info.number].register(cx.waker());

            let latched = PENDING[self.info.number].load(Ordering::Acquire) & mask;
            if latched != 0 {
                PENDING[self.info.number].fetch_and(!latched, Ordering::AcqRel);
                return Poll::Ready(InterruptFlags(latched));
            }

            let live = self.regs().sr1().read().bits() & mask;
            if live != 0 {
                // Handler has not run yet; clear in software path as the SDK does.
                let _ = self.clear_interrupt(InterruptFlags(live));
                PENDING[self.info.number].fetch_and(!live, Ordering::AcqRel);
                return Poll::Ready(InterruptFlags(live));
            }

            Poll::Pending
        })
        .await
    }

    /// Mux a documented `IN1` pin to this timer and return the owned GPIO wrap.
    pub fn take_in1_pin<T: Instance>(&self, pin: Peri<'d, impl In1Pin<T>>) -> Flex<'d> {
        configure_af(pin)
    }

    /// Mux a documented `IN2` pin to this timer and return the owned GPIO wrap.
    pub fn take_in2_pin<T: Instance>(&self, pin: Peri<'d, impl In2Pin<T>>) -> Flex<'d> {
        configure_af(pin)
    }

    /// Mux a documented `ETR` pin to this timer and return the owned GPIO wrap.
    pub fn take_etr_pin<T: Instance>(&self, pin: Peri<'d, impl EtrPin<T>>) -> Flex<'d> {
        configure_af(pin)
    }

    #[inline]
    fn regs(&self) -> &'static pac::lptim0::RegisterBlock {
        self.info.regs()
    }
}

impl<'d> embassy_embedded_hal::SetConfig for LpTimer<'d> {
    type Config = Config;
    type ConfigError = Error;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.configure(*config)
    }
}

impl<'d> Drop for LpTimer<'d> {
    fn drop(&mut self) {
        self.info.interrupt.disable();
        let _ = self.set_enabled(false);
        let _ = rcc::disable_peripheral(self.info.peripheral);
        let _ = rcc::reset_peripheral(self.info.peripheral);
    }
}

fn configure_af<'d, P>(pin: Peri<'d, P>) -> Flex<'d>
where
    P: gpio::Pin + SealedAfPin,
{
    let af = P::AF;
    let mut flex = Flex::new(pin);
    flex.set_alternate_function(af);
    flex
}

/// Clear exactly the `pending` event flags through per-flag ICR accessors.
///
/// ICR is write-1-to-clear; clearing flag-by-flag (instead of writing the
/// raw snapshot back) cannot clear a flag that fired after the snapshot.
fn clear_icr_flags(regs: &pac::lptim0::RegisterBlock, pending: u32) {
    // `write_with_zero` is unsafe on W1C registers in the PAC.
    unsafe {
        regs.icr().write_with_zero(|w| {
            if pending & ISR_CMPM != 0 {
                w.cmpmcf().set_bit();
            }
            if pending & ISR_ARRM != 0 {
                w.arrmcf().set_bit();
            }
            if pending & ISR_EXTTRIG != 0 {
                w.exttrigcf().set_bit();
            }
            if pending & ISR_CMPOK != 0 {
                w.cmpokcf().set_bit();
            }
            if pending & ISR_ARROK != 0 {
                w.arrokcf().set_bit();
            }
            if pending & ISR_UP != 0 {
                w.upcf().set_bit();
            }
            if pending & ISR_DOWN != 0 {
                w.downcf().set_bit();
            }
            w
        });
    }
}

fn interrupt_to_csr(flags: u32) -> u32 {
    // ISR and CSR share bit positions for these events (cross-checked
    // against the PAC `isr`/`csr` accessors); the mask stays numeric
    // because it feeds the software pending bitmap and CSR polls.
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

fn wait_isr(info: &'static Info, mask: u32, target: TimeoutTarget) -> Result<(), Error> {
    // Single-flag polls could use the `isr` accessors; the mask form stays
    // to match the vendor polling sequence bit-for-bit.
    for _ in 0..POLL_LIMIT {
        if info.regs().isr().read().bits() & mask == mask {
            return Ok(());
        }
        core::hint::spin_loop();
    }
    if info.regs().isr().read().bits() & mask == mask {
        Ok(())
    } else {
        Err(Error::Timeout(target))
    }
}

fn set_instance_clock_source(info: &'static Info, source: ClockSource) -> Result<(), Error> {
    if info.clock_source_sync() {
        // Vendor disables the functional clock before changing CR1.
        info.gate_functional_clock(false);

        for _ in 0..POLL_LIMIT {
            if !info.clock_source_sync() {
                break;
            }
            core::hint::spin_loop();
        }
        if info.clock_source_sync() {
            return Err(Error::Timeout(TimeoutTarget::ClockSourceSync));
        }
    }

    info.set_clock_source(source);
    Ok(())
}

/// Documented LPTIMER `IN1` pin for an instance.
#[allow(private_bounds)]
pub trait In1Pin<T: Instance>: gpio::Pin + SealedAfPin {}
/// Documented LPTIMER `IN2` pin for an instance.
#[allow(private_bounds)]
pub trait In2Pin<T: Instance>: gpio::Pin + SealedAfPin {}
/// Documented LPTIMER `ETR` pin for an instance.
#[allow(private_bounds)]
pub trait EtrPin<T: Instance>: gpio::Pin + SealedAfPin {}

macro_rules! impl_lptimer0_pin {
    ($trait:ident, $pin:ident, $af:ident) => {
        impl SealedAfPin for peripherals::$pin {
            const AF: AlternateFunction = AlternateFunction::$af;
        }
        impl $trait<peripherals::LPTIMER0> for peripherals::$pin {}
    };
}

// SDK examples:
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
