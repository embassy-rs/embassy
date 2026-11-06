//! General-purpose timers (TIMER0–TIMER3 / GPTimer).
//!
//! Register programming follows the vendor `tremo_timer` driver and the
//! ASR6601 SDK `gptimer` examples (`simple_timer`, `pwm`, `input_capture`).
//!
//! This is an Embassy-style subset: up-counting, PWM output, input capture,
//! update/CC interrupts, and typed alternate-function pins from the datasheet
//! mux tables. Encoder, slave-mode, DMA burst, and OR remaps are omitted.
//!
//! PAC field accessors exist for DIER, SR, EGR, CCER, CR1, CR2, SMCR, CCMR1/2,
//! CNT, PSC, ARR and CCRx and are used below. `SR` flags are modeled as
//! zero-to-clear in the PAC, so clears go through the per-flag accessors.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Poll;

use embassy_hal_internal::impl_peripheral;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AlternateFunction, Flex, Pin as GpioPin, Pull};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt as TypelevelInterrupt};
use crate::rcc::{self, Peripheral as RccPeripheral};
use crate::time::Hertz;
use crate::{Peri, PeripheralType, interrupt, pac, peripherals};

const SR_IT_MASK: u32 = 0x5f;
const DIER_IT_MASK: u32 = 0x5f;

static WAKERS: [AtomicWaker; 4] = [const { AtomicWaker::new() }; 4];
static PENDING: [AtomicU32; 4] = [const { AtomicU32::new(0) }; 4];

/// TIMER configuration error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// Requested frequency cannot be represented with 16-bit PSC/ARR.
    InvalidFrequency,
    /// Kernel clock is unknown (RCC not initialized) or zero.
    ClockUnavailable,
    /// RCC rejected the clock or reset request.
    Rcc(rcc::Error),
}

impl From<rcc::Error> for ConfigError {
    fn from(value: rcc::Error) -> Self {
        Self::Rcc(value)
    }
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFrequency => write!(f, "TIMER frequency cannot be represented"),
            Self::ClockUnavailable => write!(f, "TIMER kernel clock unavailable"),
            Self::Rcc(err) => write!(f, "TIMER RCC error: {err:?}"),
        }
    }
}

impl core::error::Error for ConfigError {}

/// Capture/compare channel (vendor 0-based indexing).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Channel {
    /// Capture/compare channel 0.
    Ch0 = 0,
    /// Capture/compare channel 1.
    Ch1 = 1,
    /// Capture/compare channel 2.
    Ch2 = 2,
    /// Capture/compare channel 3.
    Ch3 = 3,
}

impl Channel {
    /// Channel index `0..=3`.
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Channel 0 marker.
pub enum Ch0 {}
/// Channel 1 marker.
pub enum Ch1 {}
/// Channel 2 marker.
pub enum Ch2 {}
/// Channel 3 marker.
pub enum Ch3 {}

enum Pclk {
    Pclk0,
    Pclk1,
}

struct Info {
    regs: *const pac::gptim0::RegisterBlock,
    peripheral: RccPeripheral,
    interrupt: pac::Interrupt,
    index: usize,
    pclk: Pclk,
}

impl Info {
    #[inline]
    fn regs(&self) -> &'static pac::gptim0::RegisterBlock {
        unsafe { &*self.regs }
    }

    fn kernel_clock_hz(&self) -> Option<u32> {
        rcc::clocks().map(|c| match self.pclk {
            Pclk::Pclk0 => c.pclk0_hz,
            Pclk::Pclk1 => c.pclk1_hz,
        })
    }
}

// SAFETY: `Info` only contains read-only register addresses and instance data.
unsafe impl Sync for Info {}

trait SealedInstance {
    fn info() -> &'static Info;
}

trait SealedTimerChannel {}

/// Compile-time timer channel.
#[allow(private_bounds)]
pub trait TimerChannel: SealedTimerChannel {
    /// Runtime channel value.
    const CHANNEL: Channel;
}

impl SealedTimerChannel for Ch0 {}
impl SealedTimerChannel for Ch1 {}
impl SealedTimerChannel for Ch2 {}
impl SealedTimerChannel for Ch3 {}

impl TimerChannel for Ch0 {
    const CHANNEL: Channel = Channel::Ch0;
}
impl TimerChannel for Ch1 {
    const CHANNEL: Channel = Channel::Ch1;
}
impl TimerChannel for Ch2 {
    const CHANNEL: Channel = Channel::Ch2;
}
impl TimerChannel for Ch3 {
    const CHANNEL: Channel = Channel::Ch3;
}

/// Dead-time sampling clock division (`CR1.CKD`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
#[non_exhaustive]
pub enum ClockDivision {
    /// No prescaling.
    Div1 = 0x0,
    /// Divide the timer clock by 2.
    Div2 = 0x100,
    /// Divide the timer clock by 4.
    Div4 = 0x200,
}

/// PWM output compare mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum PwmMode {
    /// PWM mode 1.
    Mode1,
    /// PWM mode 2.
    Mode2,
}

/// Output polarity for PWM / OC.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputPolarity {
    /// Active high (`high_level = true` in the vendor OC init).
    ActiveHigh,
    /// Active low.
    ActiveLow,
}

/// Input-capture edge polarity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CapturePolarity {
    /// Capture on rising edge.
    Rising,
    /// Capture on falling edge.
    Falling,
    /// Capture on both edges.
    Both,
}

/// Interrupt / status flags (`DIER` / `SR`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterruptFlags(u32);

impl InterruptFlags {
    /// Counter overflow/underflow update event.
    pub const UPDATE: Self = Self(1 << 0);
    /// Capture/compare channel 0 event.
    pub const CC0: Self = Self(1 << 1);
    /// Capture/compare channel 1 event.
    pub const CC1: Self = Self(1 << 2);
    /// Capture/compare channel 2 event.
    pub const CC2: Self = Self(1 << 3);
    /// Capture/compare channel 3 event.
    pub const CC3: Self = Self(1 << 4);
    /// Trigger input event.
    pub const TRIGGER: Self = Self(1 << 6);

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
        self.0 & SR_IT_MASK
    }

    /// Whether every bit in `other` is present.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Capture/compare flag for `channel`.
    pub const fn cc(channel: Channel) -> Self {
        Self(1 << (channel as u32 + 1))
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

/// Basic up-counter configuration (vendor `timer_init_t`, up mode only).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// Prescaler loaded into `PSC` (`0..=0xFFFF`).
    pub prescaler: u16,
    /// Auto-reload period loaded into `ARR` (`0..=0xFFFF`).
    pub period: u16,
    /// Dead-time sampling clock division.
    pub clock_division: ClockDivision,
    /// Auto-reload preload enable.
    pub autoreload_preload: bool,
}

impl Config {
    /// Defaults matching the simple-timer example shape (caller sets PSC/ARR).
    pub const fn new() -> Self {
        Self {
            prescaler: 0,
            period: 0xffff,
            clock_division: ClockDivision::Div1,
            autoreload_preload: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// PWM configuration shared by all channels of one timer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct PwmConfig {
    /// Output compare mode.
    pub mode: PwmMode,
    /// Output polarity.
    pub polarity: OutputPolarity,
    /// Auto-reload preload enable.
    pub autoreload_preload: bool,
}

impl PwmConfig {
    /// PWM mode 1, active-high, no ARR preload.
    pub const fn new() -> Self {
        Self {
            mode: PwmMode::Mode1,
            polarity: OutputPolarity::ActiveHigh,
            autoreload_preload: false,
        }
    }
}

impl Default for PwmConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// GPTimer peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + Into<AnyTimer> + 'static + Send {
    /// NVIC vector for this instance.
    type Interrupt: TypelevelInterrupt;
}

/// Type-erased GPTimer token.
pub struct AnyTimer {
    _private: (),
}

impl_peripheral!(AnyTimer);

macro_rules! impl_timer {
    ($name:ident, $pac:ident, $rcc:ident, $irq:ident, $index:expr, $pclk:ident) => {
        impl SealedInstance for peripherals::$name {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    // GPTIM1-3 have their own PAC modules only because their
                    // OR remap layouts differ; the register block layout is
                    // otherwise identical to GPTIM0.
                    regs: pac::$pac::PTR.cast::<pac::gptim0::RegisterBlock>(),
                    peripheral: RccPeripheral::$rcc,
                    interrupt: pac::Interrupt::$irq,
                    index: $index,
                    pclk: Pclk::$pclk,
                };
                &INFO
            }
        }

        impl From<peripherals::$name> for AnyTimer {
            fn from(_: peripherals::$name) -> Self {
                Self { _private: () }
            }
        }

        impl Instance for peripherals::$name {
            type Interrupt = interrupt::typelevel::$irq;
        }
    };
}

// TIMER0/2 sit on the 0x4000_xxxx bus (PCLK0); TIMER1/3 on 0x4001_xxxx (PCLK1).
impl_timer!(TIMER0, Gptim0, Timer0, GPTIM0, 0, Pclk0);
impl_timer!(TIMER1, Gptim1, Timer1, GPTIM1, 1, Pclk1);
impl_timer!(TIMER2, Gptim2, Timer2, GPTIM2, 2, Pclk0);
impl_timer!(TIMER3, Gptim3, Timer3, GPTIM3, 3, Pclk1);

/// Interrupt handler for one GPTimer instance.
///
/// Clears pending `SR` flags that are enabled in `DIER` and wakes tasks waiting
/// on [`Timer::wait_for`], [`SimplePwm::wait_for`], or [`InputCapture::wait_for`].
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        let regs = info.regs();
        let sr = regs.sr().read().bits();
        let dier = regs.dier().read().bits();
        let pending = sr & dier & SR_IT_MASK;
        if pending == 0 {
            return;
        }

        clear_sr(info, pending);

        PENDING[info.index].fetch_or(pending, Ordering::Release);
        WAKERS[info.index].wake();
    }
}

/// Clear the selected `SR` flags.
///
/// `SR` is zero-to-clear: only the selected flags are written as 0, while all
/// other flags are written as 1 and keep their state.
fn clear_sr(info: &'static Info, mask: u32) {
    info.regs().sr().modify(|_, w| {
        if mask & InterruptFlags::UPDATE.bits() != 0 {
            w.uif().clear_bit();
        }
        if mask & InterruptFlags::CC0.bits() != 0 {
            w.cc0if().clear_bit();
        }
        if mask & InterruptFlags::CC1.bits() != 0 {
            w.cc1if().clear_bit();
        }
        if mask & InterruptFlags::CC2.bits() != 0 {
            w.cc2if().clear_bit();
        }
        if mask & InterruptFlags::CC3.bits() != 0 {
            w.cc3if().clear_bit();
        }
        if mask & InterruptFlags::TRIGGER.bits() != 0 {
            w.tif().clear_bit();
        }
        w
    });
}

fn enable_instance(info: &'static Info) -> Result<(), ConfigError> {
    rcc::enable_peripheral(info.peripheral)?;
    rcc::reset_peripheral(info.peripheral)?;
    info.interrupt.disable();
    info.interrupt.unpend();
    PENDING[info.index].store(0, Ordering::Release);
    Ok(())
}

fn apply_basic_config(info: &'static Info, config: Config) {
    let regs = info.regs();
    regs.cr1().modify(|_, w| {
        w.dir().clear_bit();
        w.cms().edge_aligned();
        w.arpe().bit(config.autoreload_preload);
        w.cen().clear_bit();
        match config.clock_division {
            ClockDivision::Div1 => w.ckd().div1(),
            ClockDivision::Div2 => w.ckd().div2(),
            ClockDivision::Div4 => w.ckd().div4(),
        }
    });

    unsafe {
        regs.arr().write_with_zero(|w| w.arr().bits(config.period));
        regs.psc().write_with_zero(|w| w.psc().bits(config.prescaler));
        regs.egr().write_with_zero(|w| w.ug().set_bit());
    }
    clear_sr(info, InterruptFlags::UPDATE.bits());
}

fn set_enabled(info: &'static Info, enabled: bool) {
    info.regs().cr1().modify(|_, w| w.cen().bit(enabled));
}

fn set_interrupt_enabled(info: &'static Info, flags: InterruptFlags, enabled: bool) {
    let mask = flags.bits() & DIER_IT_MASK;
    let regs = info.regs();
    regs.dier().modify(|r, w| {
        let bits = if enabled { r.bits() | mask } else { r.bits() & !mask };
        unsafe { w.bits(bits) }
    });

    let dier = regs.dier().read();
    let any_enabled = dier.uie().bit_is_set()
        || dier.cc0ie().bit_is_set()
        || dier.cc1ie().bit_is_set()
        || dier.cc2ie().bit_is_set()
        || dier.cc3ie().bit_is_set()
        || dier.tie().bit_is_set();
    if any_enabled {
        info.interrupt.unpend();
        unsafe { info.interrupt.enable() };
    } else {
        info.interrupt.disable();
    }
}

fn clear_interrupt(info: &'static Info, flags: InterruptFlags) {
    let mask = flags.bits();
    clear_sr(info, mask);
    PENDING[info.index].fetch_and(!mask, Ordering::AcqRel);
}

async fn wait_for_flags(info: &'static Info, flags: InterruptFlags) -> InterruptFlags {
    let mask = flags.bits();
    poll_fn(|cx| {
        WAKERS[info.index].register(cx.waker());

        let latched = PENDING[info.index].load(Ordering::Acquire) & mask;
        if latched != 0 {
            PENDING[info.index].fetch_and(!latched, Ordering::AcqRel);
            return Poll::Ready(InterruptFlags(latched));
        }

        let live = info.regs().sr().read().bits() & mask;
        if live != 0 {
            clear_interrupt(info, InterruptFlags(live));
            return Poll::Ready(InterruptFlags(live));
        }

        Poll::Pending
    })
    .await
}

fn compute_psc_arr(pclk: u32, frequency: Hertz) -> Result<(u16, u16), ConfigError> {
    let frequency_hz = frequency.hz();
    if frequency_hz == 0 || pclk == 0 {
        return Err(ConfigError::InvalidFrequency);
    }

    let ticks = pclk / frequency_hz;
    if ticks == 0 {
        return Err(ConfigError::InvalidFrequency);
    }

    // Prefer a large ARR for duty-cycle resolution: ticks = (PSC+1)*(ARR+1).
    if ticks <= 0x1_0000 {
        return Ok((0, (ticks - 1) as u16));
    }

    for psc in 1u32..=0xffff {
        let arr_plus = ticks / (psc + 1);
        if arr_plus == 0 {
            break;
        }
        if arr_plus <= 0x1_0000 && (psc + 1) * arr_plus == ticks {
            return Ok((psc as u16, (arr_plus - 1) as u16));
        }
        if arr_plus <= 0x1_0000 {
            // Closest representable period with this PSC.
            return Ok((psc as u16, (arr_plus - 1) as u16));
        }
    }

    Err(ConfigError::InvalidFrequency)
}

// PAC field accessors are used directly in each method below.
fn set_channel_enable(info: &'static Info, channel: Channel, enabled: bool) {
    let regs = info.regs();
    match channel {
        Channel::Ch0 => {
            regs.ccer().modify(|_, w| {
                w.cc0e().bit(enabled);
                w
            });
        }
        Channel::Ch1 => {
            regs.ccer().modify(|_, w| {
                w.cc1e().bit(enabled);
                w
            });
        }
        Channel::Ch2 => {
            regs.ccer().modify(|_, w| {
                w.cc2e().bit(enabled);
                w
            });
        }
        Channel::Ch3 => {
            regs.ccer().modify(|_, w| {
                w.cc3e().bit(enabled);
                w
            });
        }
    }
}

fn channel_enabled(info: &'static Info, channel: Channel) -> bool {
    let regs = info.regs();
    match channel {
        Channel::Ch0 => regs.ccer().read().cc0e().bit_is_set(),
        Channel::Ch1 => regs.ccer().read().cc1e().bit_is_set(),
        Channel::Ch2 => regs.ccer().read().cc2e().bit_is_set(),
        Channel::Ch3 => regs.ccer().read().cc3e().bit_is_set(),
    }
}

fn set_compare(info: &'static Info, channel: Channel, value: u16) {
    let regs = info.regs();
    unsafe {
        match channel {
            Channel::Ch0 => regs.ccr0().write_with_zero(|w| w.ccr0().bits(value)),
            Channel::Ch1 => regs.ccr1().write_with_zero(|w| w.ccr1().bits(value)),
            Channel::Ch2 => regs.ccr2().write_with_zero(|w| w.ccr2().bits(value)),
            Channel::Ch3 => regs.ccr3().write_with_zero(|w| w.ccr3().bits(value)),
        };
    }
}

fn get_compare(info: &'static Info, channel: Channel) -> u16 {
    let regs = info.regs();
    match channel {
        Channel::Ch0 => regs.ccr0().read().ccr0().bits(),
        Channel::Ch1 => regs.ccr1().read().ccr1().bits(),
        Channel::Ch2 => regs.ccr2().read().ccr2().bits(),
        Channel::Ch3 => regs.ccr3().read().ccr3().bits(),
    }
}

fn set_pwm_mode(regs: &pac::gptim0::RegisterBlock, channel: Channel, mode: PwmMode) {
    match channel {
        Channel::Ch0 => regs.ccmr1().modify(|_, w| {
            w.cc0s().output();
            match mode {
                PwmMode::Mode1 => w.oc0m().pwm1(),
                PwmMode::Mode2 => w.oc0m().pwm2(),
            }
        }),
        Channel::Ch1 => regs.ccmr1().modify(|_, w| {
            w.cc1s().output();
            match mode {
                PwmMode::Mode1 => w.oc1m().pwm1(),
                PwmMode::Mode2 => w.oc1m().pwm2(),
            }
        }),
        Channel::Ch2 => regs.ccmr2().modify(|_, w| {
            w.cc2s().output();
            match mode {
                PwmMode::Mode1 => w.oc2m().pwm1(),
                PwmMode::Mode2 => w.oc2m().pwm2(),
            }
        }),
        Channel::Ch3 => regs.ccmr2().modify(|_, w| {
            w.cc3s().output();
            match mode {
                PwmMode::Mode1 => w.oc3m().pwm1(),
                PwmMode::Mode2 => w.oc3m().pwm2(),
            }
        }),
    };
}

fn set_output_preload(regs: &pac::gptim0::RegisterBlock, channel: Channel) {
    match channel {
        Channel::Ch0 => regs.ccmr1().modify(|_, w| w.oc0pe().set_bit()),
        Channel::Ch1 => regs.ccmr1().modify(|_, w| w.oc1pe().set_bit()),
        Channel::Ch2 => regs.ccmr2().modify(|_, w| w.oc2pe().set_bit()),
        Channel::Ch3 => regs.ccmr2().modify(|_, w| w.oc3pe().set_bit()),
    };
}

fn configure_pwm_channel(info: &'static Info, channel: Channel, pwm: PwmConfig, duty: u16) {
    let regs = info.regs();

    // Disable channel while reconfiguring (vendor `timer_config_oc`).
    set_channel_enable(info, channel, false);
    set_pwm_mode(regs, channel, pwm.mode);
    set_output_preload(regs, channel);

    set_compare(info, channel, duty);

    match channel {
        Channel::Ch0 => {
            regs.ccer().modify(|_, w| {
                w.cc0e().set_bit();
                w.cc0p().bit(pwm.polarity == OutputPolarity::ActiveLow);
                w.cc0np().clear_bit();
                w
            });
        }
        Channel::Ch1 => {
            regs.ccer().modify(|_, w| {
                w.cc1e().set_bit();
                w.cc1p().bit(pwm.polarity == OutputPolarity::ActiveLow);
                w.cc1np().clear_bit();
                w
            });
        }
        Channel::Ch2 => {
            regs.ccer().modify(|_, w| {
                w.cc2e().set_bit();
                w.cc2p().bit(pwm.polarity == OutputPolarity::ActiveLow);
                w.cc2np().clear_bit();
                w
            });
        }
        Channel::Ch3 => {
            regs.ccer().modify(|_, w| {
                w.cc3e().set_bit();
                w.cc3p().bit(pwm.polarity == OutputPolarity::ActiveLow);
                w.cc3np().clear_bit();
                w
            });
        }
    }
}

fn configure_capture_channel(info: &'static Info, channel: Channel, polarity: CapturePolarity) {
    let regs = info.regs();

    // Disable channel while reconfiguring.
    set_channel_enable(info, channel, false);

    // Select input mapping and clear the input filter; the filter and
    // output-mode fields share the same bits, selected by `CCxS`.
    // (`input_ti1` is the value-1 mapping the vendor example selects.)
    match channel {
        Channel::Ch0 => regs.ccmr1().modify(|_, w| unsafe {
            w.cc0s().input_ti1();
            w.oc0m().frozen();
            w.ic0f().bits(0)
        }),
        Channel::Ch1 => regs.ccmr1().modify(|_, w| unsafe {
            w.cc1s().input_ti1();
            w.oc1m().frozen();
            w.ic1f().bits(0)
        }),
        Channel::Ch2 => regs.ccmr2().modify(|_, w| unsafe {
            w.cc2s().input_ti1();
            w.oc2m().frozen();
            w.ic2f().bits(0)
        }),
        Channel::Ch3 => regs.ccmr2().modify(|_, w| unsafe {
            w.cc3s().input_ti1();
            w.oc3m().frozen();
            w.ic3f().bits(0)
        }),
    };

    let regs = info.regs();
    match channel {
        Channel::Ch0 => {
            regs.ccer().modify(|_, w| {
                w.cc0e().set_bit();
                if polarity == CapturePolarity::Falling {
                    w.cc0p().set_bit();
                    w.cc0np().clear_bit();
                } else if polarity == CapturePolarity::Both {
                    w.cc0p().set_bit();
                    w.cc0np().set_bit();
                } else {
                    w.cc0p().clear_bit();
                    w.cc0np().clear_bit();
                }
                w
            });
        }
        Channel::Ch1 => {
            regs.ccer().modify(|_, w| {
                w.cc1e().set_bit();
                if polarity == CapturePolarity::Falling {
                    w.cc1p().set_bit();
                    w.cc1np().clear_bit();
                } else if polarity == CapturePolarity::Both {
                    w.cc1p().set_bit();
                    w.cc1np().set_bit();
                } else {
                    w.cc1p().clear_bit();
                    w.cc1np().clear_bit();
                }
                w
            });
        }
        Channel::Ch2 => {
            regs.ccer().modify(|_, w| {
                w.cc2e().set_bit();
                if polarity == CapturePolarity::Falling {
                    w.cc2p().set_bit();
                    w.cc2np().clear_bit();
                } else if polarity == CapturePolarity::Both {
                    w.cc2p().set_bit();
                    w.cc2np().set_bit();
                } else {
                    w.cc2p().clear_bit();
                    w.cc2np().clear_bit();
                }
                w
            });
        }
        Channel::Ch3 => {
            regs.ccer().modify(|_, w| {
                w.cc3e().set_bit();
                if polarity == CapturePolarity::Falling {
                    w.cc3p().set_bit();
                    w.cc3np().clear_bit();
                } else if polarity == CapturePolarity::Both {
                    w.cc3p().set_bit();
                    w.cc3np().set_bit();
                } else {
                    w.cc3p().clear_bit();
                    w.cc3np().clear_bit();
                }
                w
            });
        }
    }
}

fn configure_output_af<'d>(pin: Peri<'d, impl GpioPin>, af: AlternateFunction) -> Flex<'d> {
    let mut flex = Flex::new(pin);
    flex.set_as_output();
    flex.set_pull(Pull::None);
    flex.set_alternate_function(af);
    flex
}

fn configure_input_af<'d>(pin: Peri<'d, impl GpioPin>, af: AlternateFunction) -> Flex<'d> {
    let mut flex = Flex::new(pin);
    flex.set_as_input();
    flex.set_pull(Pull::None);
    flex.set_alternate_function(af);
    flex
}

/// Owned general-purpose timer in basic up-counting mode.
pub struct Timer<'d> {
    _peri: Peri<'d, AnyTimer>,
    info: &'static Info,
}

impl<'d> Timer<'d> {
    /// Enable clocks, reset the block, apply `config`, and leave the counter stopped.
    ///
    /// Does not bind the update interrupt; use [`Self::new`] for
    /// interrupt-driven use.
    pub fn new_blocking<T: Instance>(peri: Peri<'d, T>, config: Config) -> Result<Self, ConfigError> {
        Self::create(peri, config)
    }

    /// Enable clocks, reset the block, apply `config`, and leave the counter stopped.
    ///
    /// `irq` proves the timer update vector is bound; interrupt-driven
    /// waits arm it on demand.
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::create(peri, config)
    }

    fn create<T: Instance>(peri: Peri<'d, T>, config: Config) -> Result<Self, ConfigError> {
        let info = T::info();
        enable_instance(info)?;
        let this = Self {
            _peri: peri.into(),
            info,
        };
        apply_basic_config(info, config);
        Ok(this)
    }

    /// Re-apply basic timer configuration while the counter is stopped.
    pub fn configure(&mut self, config: Config) {
        set_enabled(self.info, false);
        apply_basic_config(self.info, config);
    }

    /// Start or stop the counter (`CR1.CEN`).
    pub fn set_enabled(&mut self, enabled: bool) {
        set_enabled(self.info, enabled);
    }

    /// Whether the counter is enabled.
    pub fn is_enabled(&self) -> bool {
        self.info.regs().cr1().read().cen().bit_is_set()
    }

    /// Current counter value.
    pub fn counter(&self) -> u16 {
        self.info.regs().cnt().read().cnt().bits()
    }

    /// Auto-reload period (`ARR`).
    pub fn period(&self) -> u16 {
        self.info.regs().arr().read().arr().bits()
    }

    /// Set the auto-reload period.
    pub fn set_period(&mut self, period: u16) {
        unsafe {
            self.info.regs().arr().write_with_zero(|w| w.arr().bits(period));
        }
    }

    /// Prescaler (`PSC`).
    pub fn prescaler(&self) -> u16 {
        self.info.regs().psc().read().psc().bits()
    }

    /// Kernel clock frequency in hertz.
    pub fn clock_hz(&self) -> Result<u32, ConfigError> {
        self.info
            .kernel_clock_hz()
            .filter(|&hz| hz != 0)
            .ok_or(ConfigError::ClockUnavailable)
    }

    /// Tick frequency after the prescaler: `f_kernel / (PSC + 1)`.
    pub fn tick_hz(&self) -> Result<u32, ConfigError> {
        let pclk = self.clock_hz()?;
        Ok(pclk / (u32::from(self.prescaler()) + 1))
    }

    /// Enable or disable interrupt sources and unmask the NVIC when any are set.
    pub fn set_interrupt_enabled(&mut self, flags: InterruptFlags, enabled: bool) {
        set_interrupt_enabled(self.info, flags, enabled);
    }

    /// Clear status flags (vendor `timer_clear_status`).
    pub fn clear_interrupt(&mut self, flags: InterruptFlags) {
        clear_interrupt(self.info, flags);
    }

    /// Raw pending `SR` bits masked to the interrupt set.
    pub fn interrupt_status(&self) -> InterruptFlags {
        InterruptFlags(self.info.regs().sr().read().bits() & SR_IT_MASK)
    }

    /// Wait until any of `flags` is reported by [`InterruptHandler`].
    pub async fn wait_for(&mut self, flags: InterruptFlags) -> InterruptFlags {
        if flags.bits() == 0 {
            return InterruptFlags::empty();
        }
        self.set_interrupt_enabled(flags, true);
        wait_for_flags(self.info, flags).await
    }
}

impl<'d> embassy_embedded_hal::SetConfig for Timer<'d> {
    type Config = Config;
    type ConfigError = core::convert::Infallible;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.configure(*config);
        Ok(())
    }
}

impl<'d> Drop for Timer<'d> {
    fn drop(&mut self) {
        self.info.interrupt.disable();
        set_enabled(self.info, false);
        let _ = rcc::disable_peripheral(self.info.peripheral);
    }
}

/// PWM pin wrapper that owns the GPIO in the correct alternate function.
pub struct PwmPin<'d, T: Instance, C: TimerChannel> {
    pin: Flex<'d>,
    _phantom: PhantomData<(T, C)>,
}

impl<'d, T: Instance, C: TimerChannel> PwmPin<'d, T, C> {
    /// Mux a datasheet-documented timer channel pin as a PWM output.
    pub fn new<P: TimerPin<T, C>>(pin: Peri<'d, P>) -> Self {
        Self {
            pin: configure_output_af(pin, P::AF),
            _phantom: PhantomData,
        }
    }

    /// Mux `pin` with an explicit alternate function as a PWM output.
    pub fn new_af(pin: Peri<'d, impl GpioPin>, af: AlternateFunction) -> Self {
        Self {
            pin: configure_output_af(pin, af),
            _phantom: PhantomData,
        }
    }
}

/// Input-capture pin wrapper.
pub struct CapturePin<'d, T: Instance, C: TimerChannel> {
    pin: Flex<'d>,
    _phantom: PhantomData<(T, C)>,
}

impl<'d, T: Instance, C: TimerChannel> CapturePin<'d, T, C> {
    /// Mux a datasheet-documented timer channel pin as a capture input.
    pub fn new<P: TimerPin<T, C>>(pin: Peri<'d, P>) -> Self {
        Self {
            pin: configure_input_af(pin, P::AF),
            _phantom: PhantomData,
        }
    }

    /// Mux `pin` with an explicit alternate function as a capture input.
    pub fn new_af(pin: Peri<'d, impl GpioPin>, af: AlternateFunction) -> Self {
        Self {
            pin: configure_input_af(pin, af),
            _phantom: PhantomData,
        }
    }
}

/// One PWM channel obtained from [`SimplePwm::channel`].
pub struct SimplePwmChannel<'a> {
    info: &'static Info,
    channel: Channel,
    max_duty: u16,
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a> SimplePwmChannel<'a> {
    /// Enable the channel output.
    pub fn enable(&mut self) {
        set_channel_enable(self.info, self.channel, true);
    }

    /// Disable the channel output.
    pub fn disable(&mut self) {
        set_channel_enable(self.info, self.channel, false);
    }

    /// Whether the channel output is enabled.
    pub fn is_enabled(&self) -> bool {
        channel_enabled(self.info, self.channel)
    }

    /// Maximum duty cycle (`ARR + 1`).
    pub fn max_duty_cycle(&self) -> u16 {
        self.max_duty
    }

    /// Set the compare value / duty cycle (`0..=max_duty_cycle`).
    pub fn set_duty_cycle(&mut self, duty: u16) {
        // Saturate into range: `CCR` is 16 bits while `max_duty` (ARR + 1)
        // can be `0x10000`, so a full 100% duty is not representable and
        // clamps to one tick below maximum.
        let value = if self.max_duty == 0 {
            0
        } else if duty >= self.max_duty {
            self.max_duty - 1
        } else {
            duty
        };
        set_compare(self.info, self.channel, value);
    }

    /// Current compare value.
    pub fn get_duty_cycle(&self) -> u16 {
        get_compare(self.info, self.channel)
    }
}

impl embedded_hal::pwm::ErrorType for SimplePwmChannel<'_> {
    type Error = core::convert::Infallible;
}

impl embedded_hal::pwm::SetDutyCycle for SimplePwmChannel<'_> {
    fn max_duty_cycle(&self) -> u16 {
        SimplePwmChannel::max_duty_cycle(self)
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        SimplePwmChannel::set_duty_cycle(self, duty);
        Ok(())
    }
}

/// Simple up-counting PWM driver for one GPTimer.
pub struct SimplePwm<'d> {
    _peri: Peri<'d, AnyTimer>,
    info: &'static Info,
    _ch0: Option<Flex<'d>>,
    _ch1: Option<Flex<'d>>,
    _ch2: Option<Flex<'d>>,
    _ch3: Option<Flex<'d>>,
}

impl<'d> SimplePwm<'d> {
    /// Create a PWM timer at `frequency`, optionally attaching channel pins.
    #[allow(clippy::too_many_arguments)]
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        ch0: Option<PwmPin<'d, T, Ch0>>,
        ch1: Option<PwmPin<'d, T, Ch1>>,
        ch2: Option<PwmPin<'d, T, Ch2>>,
        ch3: Option<PwmPin<'d, T, Ch3>>,
        frequency: Hertz,
        pwm: PwmConfig,
    ) -> Result<Self, ConfigError> {
        let info = T::info();
        enable_instance(info)?;

        let pclk = info
            .kernel_clock_hz()
            .filter(|&hz| hz != 0)
            .ok_or(ConfigError::ClockUnavailable)?;
        let (prescaler, period) = compute_psc_arr(pclk, frequency)?;

        apply_basic_config(
            info,
            Config {
                prescaler,
                period,
                clock_division: ClockDivision::Div1,
                autoreload_preload: pwm.autoreload_preload,
            },
        );

        let this = Self {
            _peri: peri.into(),
            info,
            _ch0: ch0.map(|p| p.pin),
            _ch1: ch1.map(|p| p.pin),
            _ch2: ch2.map(|p| p.pin),
            _ch3: ch3.map(|p| p.pin),
        };

        if this._ch0.is_some() {
            configure_pwm_channel(info, Channel::Ch0, pwm, 0);
        }
        if this._ch1.is_some() {
            configure_pwm_channel(info, Channel::Ch1, pwm, 0);
        }
        if this._ch2.is_some() {
            configure_pwm_channel(info, Channel::Ch2, pwm, 0);
        }
        if this._ch3.is_some() {
            configure_pwm_channel(info, Channel::Ch3, pwm, 0);
        }

        set_enabled(info, true);
        Ok(this)
    }

    /// Maximum duty cycle (`ARR + 1`).
    pub fn max_duty_cycle(&self) -> u16 {
        self.info.regs().arr().read().arr().bits().saturating_add(1)
    }

    /// Output frequency programmed into PSC/ARR.
    pub fn frequency_hz(&self) -> Result<u32, ConfigError> {
        let pclk = self
            .info
            .kernel_clock_hz()
            .filter(|&hz| hz != 0)
            .ok_or(ConfigError::ClockUnavailable)?;
        let psc = u32::from(self.info.regs().psc().read().psc().bits()) + 1;
        let arr = u32::from(self.info.regs().arr().read().arr().bits()) + 1;
        Ok(pclk / psc / arr)
    }

    /// Borrow a channel for duty-cycle control.
    pub fn channel(&mut self, channel: Channel) -> SimplePwmChannel<'_> {
        SimplePwmChannel {
            info: self.info,
            channel,
            max_duty: self.max_duty_cycle(),
            _phantom: PhantomData,
        }
    }

    /// Borrow channel 0.
    pub fn ch0(&mut self) -> SimplePwmChannel<'_> {
        self.channel(Channel::Ch0)
    }

    /// Borrow channel 1.
    pub fn ch1(&mut self) -> SimplePwmChannel<'_> {
        self.channel(Channel::Ch1)
    }

    /// Borrow channel 2.
    pub fn ch2(&mut self) -> SimplePwmChannel<'_> {
        self.channel(Channel::Ch2)
    }

    /// Borrow channel 3.
    pub fn ch3(&mut self) -> SimplePwmChannel<'_> {
        self.channel(Channel::Ch3)
    }

    /// Enable or disable interrupt sources.
    pub fn set_interrupt_enabled(&mut self, flags: InterruptFlags, enabled: bool) {
        set_interrupt_enabled(self.info, flags, enabled);
    }

    /// Clear status flags.
    pub fn clear_interrupt(&mut self, flags: InterruptFlags) {
        clear_interrupt(self.info, flags);
    }

    /// Wait until any of `flags` is reported by [`InterruptHandler`].
    pub async fn wait_for(&mut self, flags: InterruptFlags) -> InterruptFlags {
        if flags.bits() == 0 {
            return InterruptFlags::empty();
        }
        self.set_interrupt_enabled(flags, true);
        wait_for_flags(self.info, flags).await
    }

    /// Start or stop the counter.
    pub fn set_enabled(&mut self, enabled: bool) {
        set_enabled(self.info, enabled);
    }
}

impl<'d> Drop for SimplePwm<'d> {
    fn drop(&mut self) {
        self.info.interrupt.disable();
        set_enabled(self.info, false);
        let _ = rcc::disable_peripheral(self.info.peripheral);
    }
}

/// Input-capture driver for a single channel.
pub struct InputCapture<'d> {
    _peri: Peri<'d, AnyTimer>,
    info: &'static Info,
    _pin: Flex<'d>,
    channel: Channel,
}

impl<'d> InputCapture<'d> {
    /// Configure up-counting capture on one channel.
    pub fn new<T: Instance, C: TimerChannel>(
        peri: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        pin: CapturePin<'d, T, C>,
        config: Config,
        polarity: CapturePolarity,
    ) -> Result<Self, ConfigError> {
        let info = T::info();
        enable_instance(info)?;
        apply_basic_config(info, config);
        configure_capture_channel(info, C::CHANNEL, polarity);
        set_enabled(info, true);

        Ok(Self {
            _peri: peri.into(),
            info,
            _pin: pin.pin,
            channel: C::CHANNEL,
        })
    }

    /// Capture channel.
    pub fn channel(&self) -> Channel {
        self.channel
    }

    /// Latest captured counter value (`CCR`).
    pub fn capture(&self) -> u16 {
        get_compare(self.info, self.channel)
    }

    /// Current free-running counter.
    pub fn counter(&self) -> u16 {
        self.info.regs().cnt().read().cnt().bits()
    }

    /// Enable or disable the capture interrupt for this channel.
    pub fn set_interrupt_enabled(&mut self, enabled: bool) {
        set_interrupt_enabled(self.info, InterruptFlags::cc(self.channel), enabled);
    }

    /// Clear the capture flag for this channel.
    pub fn clear_interrupt(&mut self) {
        clear_interrupt(self.info, InterruptFlags::cc(self.channel));
    }

    /// Wait for the next capture event on this channel.
    pub async fn wait_for_capture(&mut self) -> u16 {
        self.set_interrupt_enabled(true);
        let _ = wait_for_flags(self.info, InterruptFlags::cc(self.channel)).await;
        self.capture()
    }

    /// Wait until any of `flags` is reported by [`InterruptHandler`].
    pub async fn wait_for(&mut self, flags: InterruptFlags) -> InterruptFlags {
        if flags.bits() == 0 {
            return InterruptFlags::empty();
        }
        set_interrupt_enabled(self.info, flags, true);
        wait_for_flags(self.info, flags).await
    }

    /// Start or stop the counter.
    pub fn set_enabled(&mut self, enabled: bool) {
        set_enabled(self.info, enabled);
    }
}

impl<'d> Drop for InputCapture<'d> {
    fn drop(&mut self) {
        self.info.interrupt.disable();
        set_enabled(self.info, false);
        let _ = rcc::disable_peripheral(self.info.peripheral);
    }
}

/// Datasheet-documented timer channel pin for an instance.
pub trait TimerPin<T: Instance, C: TimerChannel>: GpioPin {
    /// Alternate-function number for this pin/signal.
    const AF: AlternateFunction;
}

/// Datasheet-documented external trigger pin for an instance.
pub trait EtrPin<T: Instance>: GpioPin {
    /// Alternate-function number for this pin/signal.
    const AF: AlternateFunction;
}

macro_rules! impl_timer_pin {
    ($pin:ident, $instance:ident, $ch:ident, $af:ident) => {
        impl TimerPin<peripherals::$instance, $ch> for peripherals::$pin {
            const AF: AlternateFunction = AlternateFunction::$af;
        }
    };
}

macro_rules! impl_etr_pin {
    ($pin:ident, $instance:ident, $af:ident) => {
        impl EtrPin<peripherals::$instance> for peripherals::$pin {
            const AF: AlternateFunction = AlternateFunction::$af;
        }
    };
}

// Datasheet Table 4-4 (Fun=4..7). GPIO00=PA0 … GPIO15=PA15, GPIO16=PB0, …
impl_timer_pin!(PA0, TIMER0, Ch0, Function6);
impl_timer_pin!(PA5, TIMER0, Ch0, Function6);
impl_timer_pin!(PA10, TIMER0, Ch0, Function6);
impl_timer_pin!(PA1, TIMER0, Ch1, Function6);
impl_timer_pin!(PA14, TIMER0, Ch1, Function6);
impl_timer_pin!(PA2, TIMER0, Ch2, Function6);
impl_timer_pin!(PA3, TIMER0, Ch3, Function6);
impl_etr_pin!(PA0, TIMER0, Function7);
impl_etr_pin!(PA5, TIMER0, Function7);
impl_etr_pin!(PA10, TIMER0, Function7);

impl_timer_pin!(PA8, TIMER1, Ch0, Function6);
impl_timer_pin!(PA15, TIMER1, Ch0, Function6);
impl_timer_pin!(PB12, TIMER1, Ch0, Function6);
impl_timer_pin!(PC13, TIMER1, Ch0, Function6);
impl_timer_pin!(PA9, TIMER1, Ch1, Function6);
impl_timer_pin!(PB0, TIMER1, Ch1, Function6);
impl_timer_pin!(PB13, TIMER1, Ch1, Function6);
impl_timer_pin!(PA11, TIMER1, Ch2, Function6);
impl_timer_pin!(PB14, TIMER1, Ch2, Function6);
impl_timer_pin!(PC15, TIMER1, Ch2, Function6);
impl_timer_pin!(PA12, TIMER1, Ch3, Function6);
impl_timer_pin!(PB15, TIMER1, Ch3, Function6);
impl_etr_pin!(PC8, TIMER1, Function7);
impl_etr_pin!(PC12, TIMER1, Function6);

impl_timer_pin!(PA2, TIMER2, Ch0, Function7);
impl_timer_pin!(PC15, TIMER2, Ch0, Function7);
impl_timer_pin!(PA3, TIMER2, Ch1, Function7);
impl_timer_pin!(PC9, TIMER2, Ch1, Function7);
impl_etr_pin!(PA1, TIMER2, Function7);
impl_etr_pin!(PB15, TIMER2, Function7);

impl_timer_pin!(PA8, TIMER3, Ch0, Function7);
impl_timer_pin!(PA15, TIMER3, Ch0, Function7);
impl_timer_pin!(PB12, TIMER3, Ch0, Function7);
impl_timer_pin!(PC13, TIMER3, Ch0, Function7);
impl_timer_pin!(PA9, TIMER3, Ch1, Function7);
impl_timer_pin!(PB0, TIMER3, Ch1, Function7);
impl_timer_pin!(PB13, TIMER3, Ch1, Function7);
impl_etr_pin!(PA4, TIMER3, Function7);
impl_etr_pin!(PB14, TIMER3, Function7);

/// Mux a documented ETR pin and return the owned GPIO wrap.
pub fn take_etr_pin<'d, T: Instance>(pin: Peri<'d, impl EtrPin<T>>) -> Flex<'d> {
    // AF is read through a local helper trait object-free path.
    take_etr_pin_inner(pin)
}

fn take_etr_pin_inner<'d, T: Instance, P: EtrPin<T>>(pin: Peri<'d, P>) -> Flex<'d> {
    configure_input_af(pin, P::AF)
}

/// Alternate-function numbers used by the SDK TIMER0 PWM example.
pub mod af {
    use crate::gpio::AlternateFunction;

    /// Channel pins on TIMER0 (SDK `gpio_set_iomux(..., 6)`).
    pub const TIMER_CH: AlternateFunction = AlternateFunction::Function6;
    /// ETR pin on TIMER0 (SDK `gpio_set_iomux(..., 7)`).
    pub const TIMER0_ETR: AlternateFunction = AlternateFunction::Function7;
}
