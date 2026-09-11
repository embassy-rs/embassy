//! General-purpose timers (TIMER0–TIMER3 / GPTimer).
//!
//! Register programming follows the vendor `tremo_timer` driver and the
//! ASR6601 SDK `gptimer` examples (`simple_timer`, `pwm`, `input_capture`).
//!
//! This is an Embassy-style subset: up-counting, PWM output, input capture,
//! update/CC interrupts, and typed alternate-function pins from the datasheet
//! mux tables. Encoder, slave-mode, DMA burst, and OR remaps are omitted.
//!
//! Several multi-bit CR1/CCMR/SMCR/OR fields and the data registers lack PAC
//! field accessors; those are programmed with the bit masks from
//! `tremo_timer.h`.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Poll;

use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AlternateFunction, Flex, Pin as GpioPin, Pull};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt as TypelevelInterrupt};
use crate::rcc::{self, Peripheral as RccPeripheral};
use crate::{Peri, PeripheralType, interrupt, pac, peripherals};

const CR1_CEN: u32 = 1 << 0;
const CR1_DIR: u32 = 1 << 4;
const CR1_CMS_MASK: u32 = 0x60;
const CR1_ARPE: u32 = 1 << 7;
const CR1_CKD_MASK: u32 = 0x300;

const CCMR_OC_MODE_MASK_EVEN: u32 = 0x70;
const CCMR_OC_MODE_MASK_ODD: u32 = 0x7000;
const CCMR_CC_SEL_MASK_EVEN: u32 = 0x3;
const CCMR_CC_SEL_MASK_ODD: u32 = 0x300;
const CCMR_IC_FILTER_MASK_EVEN: u32 = 0xf0;
const CCMR_IC_FILTER_MASK_ODD: u32 = 0xf000;
const CCMR_OCPE_EVEN: u32 = 1 << 3;
const CCMR_OCPE_ODD: u32 = 1 << 11;

const OC_PWM1_EVEN: u32 = 0x60;
const OC_PWM1_ODD: u32 = 0x6000;
const OC_PWM2_EVEN: u32 = 0x70;
const OC_PWM2_ODD: u32 = 0x7000;

const CC_SEL_INPUT_SAME_EVEN: u32 = 0x1;
const CC_SEL_INPUT_SAME_ODD: u32 = 0x100;

const CCER_CC0E: u32 = 1 << 0;
const CCER_CC0P: u32 = 1 << 1;
const CCER_CC0NP: u32 = 1 << 3;
const CCER_CC1E: u32 = 1 << 4;
const CCER_CC1P: u32 = 1 << 5;
const CCER_CC1NP: u32 = 1 << 7;
const CCER_CC2E: u32 = 1 << 8;
const CCER_CC2P: u32 = 1 << 9;
const CCER_CC2NP: u32 = 1 << 11;
const CCER_CC3E: u32 = 1 << 12;
const CCER_CC3P: u32 = 1 << 13;
const CCER_CC3NP: u32 = 1 << 15;

const SR_IT_MASK: u32 = 0x5f;
const DIER_IT_MASK: u32 = 0x5f;

static WAKERS: [AtomicWaker; 4] = [const { AtomicWaker::new() }; 4];
static PENDING: [AtomicU32; 4] = [const { AtomicU32::new(0) }; 4];

/// TIMER driver error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Requested frequency cannot be represented with 16-bit PSC/ARR.
    InvalidFrequency,
    /// Kernel clock is unknown (RCC not initialized) or zero.
    ClockUnavailable,
    /// RCC rejected the clock or reset request.
    Rcc(rcc::Error),
}

impl From<rcc::Error> for Error {
    fn from(value: rcc::Error) -> Self {
        Self::Rcc(value)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFrequency => write!(f, "TIMER frequency cannot be represented"),
            Self::ClockUnavailable => write!(f, "TIMER kernel clock unavailable"),
            Self::Rcc(err) => write!(f, "TIMER RCC error: {err:?}"),
        }
    }
}

impl core::error::Error for Error {}

/// Capture/compare channel (vendor 0-based indexing).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Channel {
    Ch0 = 0,
    Ch1 = 1,
    Ch2 = 2,
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

mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> &'static pac::timer0::RegisterBlock;
        fn rcc_peripheral() -> RccPeripheral;
        fn nv_interrupt() -> pac::Interrupt;
        const INDEX: usize;
        fn kernel_clock_hz() -> Option<u32>;
    }

    pub trait TimerChannel {}
}

/// Compile-time timer channel.
#[allow(private_bounds)]
pub trait TimerChannel: sealed::TimerChannel {
    /// Runtime channel value.
    const CHANNEL: Channel;
}

impl sealed::TimerChannel for Ch0 {}
impl sealed::TimerChannel for Ch1 {}
impl sealed::TimerChannel for Ch2 {}
impl sealed::TimerChannel for Ch3 {}

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
pub enum ClockDivision {
    Div1 = 0x0,
    Div2 = 0x100,
    Div4 = 0x200,
}

/// PWM output compare mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    Rising,
    Falling,
    Both,
}

/// Interrupt / status flags (`DIER` / `SR`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterruptFlags(u32);

impl InterruptFlags {
    pub const UPDATE: Self = Self(1 << 0);
    pub const CC0: Self = Self(1 << 1);
    pub const CC1: Self = Self(1 << 2);
    pub const CC2: Self = Self(1 << 3);
    pub const CC3: Self = Self(1 << 4);
    pub const TRIGGER: Self = Self(1 << 6);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn bits(self) -> u32 {
        self.0 & SR_IT_MASK
    }

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
pub trait Instance: sealed::Instance + PeripheralType + 'static {
    /// NVIC vector for this instance.
    type Interrupt: TypelevelInterrupt;
}

macro_rules! impl_timer {
    ($name:ident, $pac:ident, $rcc:ident, $irq:ident, $index:expr, $clock:ident) => {
        impl sealed::Instance for peripherals::$name {
            #[inline]
            fn regs() -> &'static pac::timer0::RegisterBlock {
                unsafe { &*pac::$pac::ptr() }
            }

            #[inline]
            fn rcc_peripheral() -> RccPeripheral {
                RccPeripheral::$rcc
            }

            #[inline]
            fn nv_interrupt() -> pac::Interrupt {
                pac::Interrupt::$irq
            }

            const INDEX: usize = $index;

            fn kernel_clock_hz() -> Option<u32> {
                rcc::clocks().map(|c| c.$clock)
            }
        }

        impl Instance for peripherals::$name {
            type Interrupt = interrupt::typelevel::$irq;
        }
    };
}

// TIMER0/2 sit on the 0x4000_xxxx bus (PCLK0); TIMER1/3 on 0x4001_xxxx (PCLK1).
impl_timer!(TIMER0, Timer0, Timer0, TIMER0, 0, pclk0_hz);
impl_timer!(TIMER1, Timer1, Timer1, TIMER1, 1, pclk1_hz);
impl_timer!(TIMER2, Timer2, Timer2, TIMER2, 2, pclk0_hz);
impl_timer!(TIMER3, Timer3, Timer3, TIMER3, 3, pclk1_hz);

/// Interrupt handler for one GPTimer instance.
///
/// Clears pending `SR` flags that are enabled in `DIER` and wakes tasks waiting
/// on [`Timer::wait_for`], [`SimplePwm::wait_for`], or [`InputCapture::wait_for`].
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        let pending = regs.sr().read().bits() & regs.dier().read().bits() & SR_IT_MASK;
        if pending == 0 {
            return;
        }

        // Vendor `timer_clear_status`: write zeros to the flags being cleared.
        unsafe {
            regs.sr().write_with_zero(|w| w.bits(!pending));
        }

        PENDING[T::INDEX].fetch_or(pending, Ordering::Release);
        WAKERS[T::INDEX].wake();
    }
}

fn enable_instance<T: Instance>() -> Result<(), Error> {
    rcc::enable_peripheral(T::rcc_peripheral())?;
    rcc::reset_peripheral(T::rcc_peripheral())?;
    T::nv_interrupt().disable();
    T::nv_interrupt().unpend();
    PENDING[T::INDEX].store(0, Ordering::Release);
    Ok(())
}

fn apply_basic_config<T: Instance>(config: Config) {
    let regs = T::regs();
    regs.cr1().modify(|r, w| unsafe {
        let mut bits = r.bits();
        bits &= !(CR1_DIR | CR1_CMS_MASK | CR1_CKD_MASK | CR1_ARPE | CR1_CEN);
        bits |= config.clock_division as u32;
        if config.autoreload_preload {
            bits |= CR1_ARPE;
        }
        w.bits(bits)
    });

    unsafe {
        regs.arr().write_with_zero(|w| w.bits(u32::from(config.period)));
        regs.psc().write_with_zero(|w| w.bits(u32::from(config.prescaler)));
        regs.egr().write_with_zero(|w| w.ug().set_bit());
        regs.sr().write_with_zero(|w| w.bits(!InterruptFlags::UPDATE.bits()));
    }
}

fn set_enabled<T: Instance>(enabled: bool) {
    T::regs().cr1().modify(|r, w| unsafe {
        let bits = if enabled {
            r.bits() | CR1_CEN
        } else {
            r.bits() & !CR1_CEN
        };
        w.bits(bits)
    });
}

fn set_interrupt_enabled<T: Instance>(flags: InterruptFlags, enabled: bool) {
    let mask = flags.bits() & DIER_IT_MASK;
    T::regs().dier().modify(|r, w| unsafe {
        let bits = if enabled { r.bits() | mask } else { r.bits() & !mask };
        w.bits(bits)
    });

    if T::regs().dier().read().bits() & DIER_IT_MASK != 0 {
        T::nv_interrupt().unpend();
        unsafe { T::nv_interrupt().enable() };
    } else {
        T::nv_interrupt().disable();
    }
}

fn clear_interrupt<T: Instance>(flags: InterruptFlags) {
    let mask = flags.bits();
    unsafe {
        T::regs().sr().write_with_zero(|w| w.bits(!mask));
    }
    PENDING[T::INDEX].fetch_and(!mask, Ordering::AcqRel);
}

async fn wait_for_flags<T: Instance>(flags: InterruptFlags) -> InterruptFlags {
    let mask = flags.bits();
    poll_fn(|cx| {
        WAKERS[T::INDEX].register(cx.waker());

        let latched = PENDING[T::INDEX].load(Ordering::Acquire) & mask;
        if latched != 0 {
            PENDING[T::INDEX].fetch_and(!latched, Ordering::AcqRel);
            return Poll::Ready(InterruptFlags(latched));
        }

        let live = T::regs().sr().read().bits() & mask;
        if live != 0 {
            clear_interrupt::<T>(InterruptFlags(live));
            return Poll::Ready(InterruptFlags(live));
        }

        Poll::Pending
    })
    .await
}

fn compute_psc_arr(pclk: u32, frequency_hz: u32) -> Result<(u16, u16), Error> {
    if frequency_hz == 0 || pclk == 0 {
        return Err(Error::InvalidFrequency);
    }

    let ticks = pclk / frequency_hz;
    if ticks == 0 {
        return Err(Error::InvalidFrequency);
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

    Err(Error::InvalidFrequency)
}

fn channel_enable_mask(channel: Channel) -> u32 {
    match channel {
        Channel::Ch0 => CCER_CC0E,
        Channel::Ch1 => CCER_CC1E,
        Channel::Ch2 => CCER_CC2E,
        Channel::Ch3 => CCER_CC3E,
    }
}

fn channel_polarity_mask(channel: Channel) -> u32 {
    match channel {
        Channel::Ch0 => CCER_CC0P,
        Channel::Ch1 => CCER_CC1P,
        Channel::Ch2 => CCER_CC2P,
        Channel::Ch3 => CCER_CC3P,
    }
}

fn channel_np_mask(channel: Channel) -> u32 {
    match channel {
        Channel::Ch0 => CCER_CC0NP,
        Channel::Ch1 => CCER_CC1NP,
        Channel::Ch2 => CCER_CC2NP,
        Channel::Ch3 => CCER_CC3NP,
    }
}

fn channel_both_edge_mask(channel: Channel) -> u32 {
    channel_polarity_mask(channel) | channel_np_mask(channel)
}

fn set_channel_enable<T: Instance>(channel: Channel, enabled: bool) {
    let mask = channel_enable_mask(channel);
    T::regs().ccer().modify(|r, w| unsafe {
        let bits = if enabled { r.bits() | mask } else { r.bits() & !mask };
        w.bits(bits)
    });
}

fn channel_enabled<T: Instance>(channel: Channel) -> bool {
    T::regs().ccer().read().bits() & channel_enable_mask(channel) != 0
}

fn set_compare<T: Instance>(channel: Channel, value: u16) {
    let regs = T::regs();
    let bits = u32::from(value);
    unsafe {
        match channel {
            Channel::Ch0 => {
                regs.ccr0().write_with_zero(|w| w.bits(bits));
            }
            Channel::Ch1 => {
                regs.ccr1().write_with_zero(|w| w.bits(bits));
            }
            Channel::Ch2 => {
                regs.ccr2().write_with_zero(|w| w.bits(bits));
            }
            Channel::Ch3 => {
                regs.ccr3().write_with_zero(|w| w.bits(bits));
            }
        }
    }
}

fn get_compare<T: Instance>(channel: Channel) -> u16 {
    let regs = T::regs();
    (match channel {
        Channel::Ch0 => regs.ccr0().read().bits(),
        Channel::Ch1 => regs.ccr1().read().bits(),
        Channel::Ch2 => regs.ccr2().read().bits(),
        Channel::Ch3 => regs.ccr3().read().bits(),
    }) as u16
}

fn configure_pwm_channel<T: Instance>(channel: Channel, pwm: PwmConfig, duty: u16) {
    let regs = T::regs();
    let enable = channel_enable_mask(channel);
    let polarity = channel_polarity_mask(channel);
    let np = channel_np_mask(channel);

    // Disable channel while reconfiguring (vendor `timer_config_oc`).
    regs.ccer().modify(|r, w| unsafe { w.bits(r.bits() & !enable) });

    let (mode_mask, mode_bits, sel_mask, ocpe) = if channel.index() % 2 == 0 {
        let mode = match pwm.mode {
            PwmMode::Mode1 => OC_PWM1_EVEN,
            PwmMode::Mode2 => OC_PWM2_EVEN,
        };
        (CCMR_OC_MODE_MASK_EVEN, mode, CCMR_CC_SEL_MASK_EVEN, CCMR_OCPE_EVEN)
    } else {
        let mode = match pwm.mode {
            PwmMode::Mode1 => OC_PWM1_ODD,
            PwmMode::Mode2 => OC_PWM2_ODD,
        };
        (CCMR_OC_MODE_MASK_ODD, mode, CCMR_CC_SEL_MASK_ODD, CCMR_OCPE_ODD)
    };

    match channel {
        Channel::Ch0 | Channel::Ch1 => {
            regs.ccmr1()
                .modify(|r, w| unsafe { w.bits((r.bits() & !(mode_mask | sel_mask)) | mode_bits | ocpe) });
        }
        Channel::Ch2 | Channel::Ch3 => {
            regs.ccmr2()
                .modify(|r, w| unsafe { w.bits((r.bits() & !(mode_mask | sel_mask)) | mode_bits | ocpe) });
        }
    }

    set_compare::<T>(channel, duty);

    regs.ccer().modify(|r, w| unsafe {
        let mut bits = r.bits() & !(polarity | np);
        if pwm.polarity == OutputPolarity::ActiveLow {
            bits |= polarity;
        }
        bits |= enable;
        w.bits(bits)
    });
}

fn configure_capture_channel<T: Instance>(channel: Channel, polarity: CapturePolarity) {
    let regs = T::regs();
    let enable = channel_enable_mask(channel);
    let both = channel_both_edge_mask(channel);

    regs.ccer().modify(|r, w| unsafe { w.bits(r.bits() & !enable) });

    let (sel_mask, sel_bits, filter_mask, oc_mask) = if channel.index() % 2 == 0 {
        (
            CCMR_CC_SEL_MASK_EVEN,
            CC_SEL_INPUT_SAME_EVEN,
            CCMR_IC_FILTER_MASK_EVEN,
            CCMR_OC_MODE_MASK_EVEN,
        )
    } else {
        (
            CCMR_CC_SEL_MASK_ODD,
            CC_SEL_INPUT_SAME_ODD,
            CCMR_IC_FILTER_MASK_ODD,
            CCMR_OC_MODE_MASK_ODD,
        )
    };

    match channel {
        Channel::Ch0 | Channel::Ch1 => {
            regs.ccmr1()
                .modify(|r, w| unsafe { w.bits((r.bits() & !(sel_mask | filter_mask | oc_mask)) | sel_bits) });
        }
        Channel::Ch2 | Channel::Ch3 => {
            regs.ccmr2()
                .modify(|r, w| unsafe { w.bits((r.bits() & !(sel_mask | filter_mask | oc_mask)) | sel_bits) });
        }
    }

    let polarity_bits = match polarity {
        CapturePolarity::Rising => 0,
        CapturePolarity::Falling => channel_polarity_mask(channel),
        CapturePolarity::Both => both,
    };

    regs.ccer()
        .modify(|r, w| unsafe { w.bits((r.bits() & !both) | polarity_bits | enable) });
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
pub struct Timer<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Timer<'d, T> {
    /// Enable clocks, reset the block, apply `config`, and leave the counter stopped.
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        enable_instance::<T>()?;
        let this = Self { _peri: peri };
        apply_basic_config::<T>(config);
        Ok(this)
    }

    /// Re-apply basic timer configuration while the counter is stopped.
    pub fn configure(&self, config: Config) {
        set_enabled::<T>(false);
        apply_basic_config::<T>(config);
    }

    /// Start or stop the counter (`CR1.CEN`).
    pub fn set_enabled(&self, enabled: bool) {
        set_enabled::<T>(enabled);
    }

    /// Whether the counter is enabled.
    pub fn is_enabled(&self) -> bool {
        T::regs().cr1().read().bits() & CR1_CEN != 0
    }

    /// Current counter value.
    pub fn counter(&self) -> u16 {
        T::regs().cnt().read().bits() as u16
    }

    /// Auto-reload period (`ARR`).
    pub fn period(&self) -> u16 {
        T::regs().arr().read().bits() as u16
    }

    /// Set the auto-reload period.
    pub fn set_period(&self, period: u16) {
        unsafe {
            T::regs().arr().write_with_zero(|w| w.bits(u32::from(period)));
        }
    }

    /// Prescaler (`PSC`).
    pub fn prescaler(&self) -> u16 {
        T::regs().psc().read().bits() as u16
    }

    /// Kernel clock frequency in hertz.
    pub fn clock_hz(&self) -> Result<u32, Error> {
        T::kernel_clock_hz()
            .filter(|&hz| hz != 0)
            .ok_or(Error::ClockUnavailable)
    }

    /// Tick frequency after the prescaler: `f_kernel / (PSC + 1)`.
    pub fn tick_hz(&self) -> Result<u32, Error> {
        let pclk = self.clock_hz()?;
        Ok(pclk / (u32::from(self.prescaler()) + 1))
    }

    /// Enable or disable interrupt sources and unmask the NVIC when any are set.
    pub fn set_interrupt_enabled(&self, flags: InterruptFlags, enabled: bool) {
        set_interrupt_enabled::<T>(flags, enabled);
    }

    /// Clear status flags (vendor `timer_clear_status`).
    pub fn clear_interrupt(&self, flags: InterruptFlags) {
        clear_interrupt::<T>(flags);
    }

    /// Raw pending `SR` bits masked to the interrupt set.
    pub fn interrupt_status(&self) -> InterruptFlags {
        InterruptFlags(T::regs().sr().read().bits() & SR_IT_MASK)
    }

    /// Wait until any of `flags` is reported by [`InterruptHandler`].
    pub async fn wait_for(&self, flags: InterruptFlags) -> InterruptFlags {
        wait_for_flags::<T>(flags).await
    }
}

impl<'d, T: Instance> Drop for Timer<'d, T> {
    fn drop(&mut self) {
        T::nv_interrupt().disable();
        set_enabled::<T>(false);
        let _ = rcc::disable_peripheral(T::rcc_peripheral());
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
pub struct SimplePwmChannel<'a, T: Instance> {
    channel: Channel,
    max_duty: u16,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T: Instance> SimplePwmChannel<'a, T> {
    /// Enable the channel output.
    pub fn enable(&mut self) {
        set_channel_enable::<T>(self.channel, true);
    }

    /// Disable the channel output.
    pub fn disable(&mut self) {
        set_channel_enable::<T>(self.channel, false);
    }

    /// Whether the channel output is enabled.
    pub fn is_enabled(&self) -> bool {
        channel_enabled::<T>(self.channel)
    }

    /// Maximum duty cycle (`ARR + 1`).
    pub fn max_duty_cycle(&self) -> u16 {
        self.max_duty
    }

    /// Set the compare value / duty cycle (`0..=max_duty_cycle`).
    pub fn set_duty_cycle(&mut self, duty: u16) {
        let value = if self.max_duty == 0 {
            0
        } else if duty >= self.max_duty {
            self.max_duty - 1
        } else {
            duty
        };
        set_compare::<T>(self.channel, value);
    }

    /// Current compare value.
    pub fn get_duty_cycle(&self) -> u16 {
        get_compare::<T>(self.channel)
    }
}

impl<'a, T: Instance> embedded_hal::pwm::ErrorType for SimplePwmChannel<'a, T> {
    type Error = core::convert::Infallible;
}

impl<'a, T: Instance> embedded_hal::pwm::SetDutyCycle for SimplePwmChannel<'a, T> {
    fn max_duty_cycle(&self) -> u16 {
        SimplePwmChannel::max_duty_cycle(self)
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        SimplePwmChannel::set_duty_cycle(self, duty);
        Ok(())
    }
}

/// Simple up-counting PWM driver for one GPTimer.
pub struct SimplePwm<'d, T: Instance> {
    _peri: Peri<'d, T>,
    _ch0: Option<Flex<'d>>,
    _ch1: Option<Flex<'d>>,
    _ch2: Option<Flex<'d>>,
    _ch3: Option<Flex<'d>>,
}

impl<'d, T: Instance> SimplePwm<'d, T> {
    /// Create a PWM timer at `frequency_hz`, optionally attaching channel pins.
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        ch0: Option<PwmPin<'d, T, Ch0>>,
        ch1: Option<PwmPin<'d, T, Ch1>>,
        ch2: Option<PwmPin<'d, T, Ch2>>,
        ch3: Option<PwmPin<'d, T, Ch3>>,
        frequency_hz: u32,
        pwm: PwmConfig,
    ) -> Result<Self, Error> {
        enable_instance::<T>()?;

        let pclk = T::kernel_clock_hz()
            .filter(|&hz| hz != 0)
            .ok_or(Error::ClockUnavailable)?;
        let (prescaler, period) = compute_psc_arr(pclk, frequency_hz)?;

        apply_basic_config::<T>(Config {
            prescaler,
            period,
            clock_division: ClockDivision::Div1,
            autoreload_preload: pwm.autoreload_preload,
        });

        let this = Self {
            _peri: peri,
            _ch0: ch0.map(|p| p.pin),
            _ch1: ch1.map(|p| p.pin),
            _ch2: ch2.map(|p| p.pin),
            _ch3: ch3.map(|p| p.pin),
        };

        if this._ch0.is_some() {
            configure_pwm_channel::<T>(Channel::Ch0, pwm, 0);
        }
        if this._ch1.is_some() {
            configure_pwm_channel::<T>(Channel::Ch1, pwm, 0);
        }
        if this._ch2.is_some() {
            configure_pwm_channel::<T>(Channel::Ch2, pwm, 0);
        }
        if this._ch3.is_some() {
            configure_pwm_channel::<T>(Channel::Ch3, pwm, 0);
        }

        set_enabled::<T>(true);
        Ok(this)
    }

    /// Maximum duty cycle (`ARR + 1`).
    pub fn max_duty_cycle(&self) -> u16 {
        (T::regs().arr().read().bits() as u16).saturating_add(1)
    }

    /// Output frequency programmed into PSC/ARR.
    pub fn frequency_hz(&self) -> Result<u32, Error> {
        let pclk = T::kernel_clock_hz()
            .filter(|&hz| hz != 0)
            .ok_or(Error::ClockUnavailable)?;
        let psc = T::regs().psc().read().bits() + 1;
        let arr = T::regs().arr().read().bits() + 1;
        Ok(pclk / psc / arr)
    }

    /// Borrow a channel for duty-cycle control.
    pub fn channel(&mut self, channel: Channel) -> SimplePwmChannel<'_, T> {
        SimplePwmChannel {
            channel,
            max_duty: self.max_duty_cycle(),
            _phantom: PhantomData,
        }
    }

    /// Borrow channel 0.
    pub fn ch0(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch0)
    }

    /// Borrow channel 1.
    pub fn ch1(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch1)
    }

    /// Borrow channel 2.
    pub fn ch2(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch2)
    }

    /// Borrow channel 3.
    pub fn ch3(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch3)
    }

    /// Enable or disable interrupt sources.
    pub fn set_interrupt_enabled(&self, flags: InterruptFlags, enabled: bool) {
        set_interrupt_enabled::<T>(flags, enabled);
    }

    /// Clear status flags.
    pub fn clear_interrupt(&self, flags: InterruptFlags) {
        clear_interrupt::<T>(flags);
    }

    /// Wait until any of `flags` is reported by [`InterruptHandler`].
    pub async fn wait_for(&self, flags: InterruptFlags) -> InterruptFlags {
        wait_for_flags::<T>(flags).await
    }

    /// Start or stop the counter.
    pub fn set_enabled(&self, enabled: bool) {
        set_enabled::<T>(enabled);
    }
}

impl<'d, T: Instance> Drop for SimplePwm<'d, T> {
    fn drop(&mut self) {
        T::nv_interrupt().disable();
        set_enabled::<T>(false);
        let _ = rcc::disable_peripheral(T::rcc_peripheral());
    }
}

/// Input-capture driver for a single channel.
pub struct InputCapture<'d, T: Instance> {
    _peri: Peri<'d, T>,
    _pin: Flex<'d>,
    channel: Channel,
}

impl<'d, T: Instance> InputCapture<'d, T> {
    /// Configure up-counting capture on one channel.
    pub fn new<C: TimerChannel>(
        peri: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        pin: CapturePin<'d, T, C>,
        config: Config,
        polarity: CapturePolarity,
    ) -> Result<Self, Error> {
        enable_instance::<T>()?;
        apply_basic_config::<T>(config);
        configure_capture_channel::<T>(C::CHANNEL, polarity);
        set_enabled::<T>(true);

        Ok(Self {
            _peri: peri,
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
        get_compare::<T>(self.channel)
    }

    /// Current free-running counter.
    pub fn counter(&self) -> u16 {
        T::regs().cnt().read().bits() as u16
    }

    /// Enable or disable the capture interrupt for this channel.
    pub fn set_interrupt_enabled(&self, enabled: bool) {
        set_interrupt_enabled::<T>(InterruptFlags::cc(self.channel), enabled);
    }

    /// Clear the capture flag for this channel.
    pub fn clear_interrupt(&self) {
        clear_interrupt::<T>(InterruptFlags::cc(self.channel));
    }

    /// Wait for the next capture event on this channel.
    pub async fn wait_for_capture(&self) -> u16 {
        let _ = wait_for_flags::<T>(InterruptFlags::cc(self.channel)).await;
        self.capture()
    }

    /// Wait until any of `flags` is reported by [`InterruptHandler`].
    pub async fn wait_for(&self, flags: InterruptFlags) -> InterruptFlags {
        wait_for_flags::<T>(flags).await
    }

    /// Start or stop the counter.
    pub fn set_enabled(&self, enabled: bool) {
        set_enabled::<T>(enabled);
    }
}

impl<'d, T: Instance> Drop for InputCapture<'d, T> {
    fn drop(&mut self) {
        T::nv_interrupt().disable();
        set_enabled::<T>(false);
        let _ = rcc::disable_peripheral(T::rcc_peripheral());
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
