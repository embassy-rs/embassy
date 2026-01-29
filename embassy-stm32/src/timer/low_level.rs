//! Low-level timer driver.
//!
//! This is an unopinionated, very low-level driver for all STM32 timers. It allows direct register
//! manipulation with the `regs_*()` methods, and has utility functions that are thin wrappers
//! over the registers.
//!
//! The available functionality depends on the timer type.

use core::mem::ManuallyDrop;

use embassy_hal_internal::Peri;
// Re-export useful enums
pub use stm32_metapac::timer::vals::{FilterValue, Mms as MasterMode, Sms as SlaveMode, Ts as TriggerSource};

use super::*;
use crate::dma::{self, Transfer, WritableRingBuffer};
use crate::pac::timer::vals;
use crate::rcc;
use crate::time::Hertz;

/// Input capture mode.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InputCaptureMode {
    /// Rising edge only.
    Rising,
    /// Falling edge only.
    Falling,
    /// Both rising or falling edges.
    BothEdges,
}

/// Input TI selection.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InputTISelection {
    /// Normal
    Normal,
    /// Alternate
    Alternate,
    /// TRC
    TRC,
}

impl From<InputTISelection> for stm32_metapac::timer::vals::CcmrInputCcs {
    fn from(tisel: InputTISelection) -> Self {
        match tisel {
            InputTISelection::Normal => stm32_metapac::timer::vals::CcmrInputCcs::TI4,
            InputTISelection::Alternate => stm32_metapac::timer::vals::CcmrInputCcs::TI3,
            InputTISelection::TRC => stm32_metapac::timer::vals::CcmrInputCcs::TRC,
        }
    }
}

/// Timer counting mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CountingMode {
    #[default]
    /// The timer counts up to the reload value and then resets back to 0.
    EdgeAlignedUp,
    /// The timer counts down to 0 and then resets back to the reload value.
    EdgeAlignedDown,
    /// The timer counts up to the reload value and then counts back to 0.
    ///
    /// The output compare interrupt flags of channels configured in output are
    /// set when the counter is counting down.
    CenterAlignedDownInterrupts,
    /// The timer counts up to the reload value and then counts back to 0.
    ///
    /// The output compare interrupt flags of channels configured in output are
    /// set when the counter is counting up.
    CenterAlignedUpInterrupts,
    /// The timer counts up to the reload value and then counts back to 0.
    ///
    /// The output compare interrupt flags of channels configured in output are
    /// set when the counter is counting both up or down.
    CenterAlignedBothInterrupts,
}

impl CountingMode {
    /// Return whether this mode is edge-aligned (up or down).
    pub fn is_edge_aligned(&self) -> bool {
        matches!(self, CountingMode::EdgeAlignedUp | CountingMode::EdgeAlignedDown)
    }

    /// Return whether this mode is center-aligned.
    pub fn is_center_aligned(&self) -> bool {
        matches!(
            self,
            CountingMode::CenterAlignedDownInterrupts
                | CountingMode::CenterAlignedUpInterrupts
                | CountingMode::CenterAlignedBothInterrupts
        )
    }
}

impl From<CountingMode> for (vals::Cms, vals::Dir) {
    fn from(value: CountingMode) -> Self {
        match value {
            CountingMode::EdgeAlignedUp => (vals::Cms::EDGE_ALIGNED, vals::Dir::UP),
            CountingMode::EdgeAlignedDown => (vals::Cms::EDGE_ALIGNED, vals::Dir::DOWN),
            CountingMode::CenterAlignedDownInterrupts => (vals::Cms::CENTER_ALIGNED1, vals::Dir::UP),
            CountingMode::CenterAlignedUpInterrupts => (vals::Cms::CENTER_ALIGNED2, vals::Dir::UP),
            CountingMode::CenterAlignedBothInterrupts => (vals::Cms::CENTER_ALIGNED3, vals::Dir::UP),
        }
    }
}

impl From<(vals::Cms, vals::Dir)> for CountingMode {
    fn from(value: (vals::Cms, vals::Dir)) -> Self {
        match value {
            (vals::Cms::EDGE_ALIGNED, vals::Dir::UP) => CountingMode::EdgeAlignedUp,
            (vals::Cms::EDGE_ALIGNED, vals::Dir::DOWN) => CountingMode::EdgeAlignedDown,
            (vals::Cms::CENTER_ALIGNED1, _) => CountingMode::CenterAlignedDownInterrupts,
            (vals::Cms::CENTER_ALIGNED2, _) => CountingMode::CenterAlignedUpInterrupts,
            (vals::Cms::CENTER_ALIGNED3, _) => CountingMode::CenterAlignedBothInterrupts,
        }
    }
}

/// Output compare mode.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputCompareMode {
    /// The comparison between the output compare register TIMx_CCRx and
    /// the counter TIMx_CNT has no effect on the outputs.
    /// (this mode is used to generate a timing base).
    Frozen,
    /// Set channel to active level on match. OCxREF signal is forced high when the
    /// counter TIMx_CNT matches the capture/compare register x (TIMx_CCRx).
    ActiveOnMatch,
    /// Set channel to inactive level on match. OCxREF signal is forced low when the
    /// counter TIMx_CNT matches the capture/compare register x (TIMx_CCRx).
    InactiveOnMatch,
    /// Toggle - OCxREF toggles when TIMx_CNT=TIMx_CCRx.
    Toggle,
    /// Force inactive level - OCxREF is forced low.
    ForceInactive,
    /// Force active level - OCxREF is forced high.
    ForceActive,
    /// PWM mode 1 - In upcounting, channel is active as long as TIMx_CNT<TIMx_CCRx
    /// else inactive. In downcounting, channel is inactive (OCxREF=0) as long as
    /// TIMx_CNT>TIMx_CCRx else active (OCxREF=1).
    PwmMode1,
    /// PWM mode 2 - In upcounting, channel is inactive as long as
    /// TIMx_CNT<TIMx_CCRx else active. In downcounting, channel is active as long as
    /// TIMx_CNT>TIMx_CCRx else inactive.
    PwmMode2,

    #[cfg(timer_v2)]
    /// In up-counting mode, the channel is active until a trigger
    /// event is detected (on tim_trgi signal). Then, a comparison is performed as in PWM
    /// mode 1 and the channels becomes active again at the next update. In down-counting
    /// mode, the channel is inactive until a trigger event is detected (on tim_trgi signal).
    /// Then, a comparison is performed as in PWM mode 1 and the channels becomes
    /// inactive again at the next update.
    OnePulseMode1,

    #[cfg(timer_v2)]
    /// In up-counting mode, the channel is inactive until a
    /// trigger event is detected (on tim_trgi signal). Then, a comparison is performed as in
    /// PWM mode 2 and the channels becomes inactive again at the next update. In down
    /// counting mode, the channel is active until a trigger event is detected (on tim_trgi
    /// signal). Then, a comparison is performed as in PWM mode 1 and the channels
    /// becomes active again at the next update.
    OnePulseMode2,

    #[cfg(timer_v2)]
    /// Combined PWM mode 1 - tim_oc1ref has the same behavior as in PWM mode 1.
    /// tim_oc1refc is the logical OR between tim_oc1ref and tim_oc2ref.
    CombinedPwmMode1,

    #[cfg(timer_v2)]
    /// Combined PWM mode 2 - tim_oc1ref has the same behavior as in PWM mode 2.
    /// tim_oc1refc is the logical AND between tim_oc1ref and tim_oc2ref.
    CombinedPwmMode2,

    #[cfg(timer_v2)]
    /// tim_oc1ref has the same behavior as in PWM mode 1. tim_oc1refc outputs tim_oc1ref
    /// when the counter is counting up, tim_oc2ref when it is counting down.
    AsymmetricPwmMode1,

    #[cfg(timer_v2)]
    /// tim_oc1ref has the same behavior as in PWM mode 2. tim_oc1refc outputs tim_oc1ref
    /// when the counter is counting up, tim_oc2ref when it is counting down.
    AsymmetricPwmMode2,
}

impl From<OutputCompareMode> for crate::pac::timer::vals::Ocm {
    fn from(mode: OutputCompareMode) -> Self {
        match mode {
            OutputCompareMode::Frozen => crate::pac::timer::vals::Ocm::FROZEN,
            OutputCompareMode::ActiveOnMatch => crate::pac::timer::vals::Ocm::ACTIVE_ON_MATCH,
            OutputCompareMode::InactiveOnMatch => crate::pac::timer::vals::Ocm::INACTIVE_ON_MATCH,
            OutputCompareMode::Toggle => crate::pac::timer::vals::Ocm::TOGGLE,
            OutputCompareMode::ForceInactive => crate::pac::timer::vals::Ocm::FORCE_INACTIVE,
            OutputCompareMode::ForceActive => crate::pac::timer::vals::Ocm::FORCE_ACTIVE,
            OutputCompareMode::PwmMode1 => crate::pac::timer::vals::Ocm::PWM_MODE1,
            OutputCompareMode::PwmMode2 => crate::pac::timer::vals::Ocm::PWM_MODE2,
            #[cfg(timer_v2)]
            OutputCompareMode::OnePulseMode1 => crate::pac::timer::vals::Ocm::RETRIGERRABLE_OPM_MODE_1,
            #[cfg(timer_v2)]
            OutputCompareMode::OnePulseMode2 => crate::pac::timer::vals::Ocm::RETRIGERRABLE_OPM_MODE_2,
            #[cfg(timer_v2)]
            OutputCompareMode::CombinedPwmMode1 => crate::pac::timer::vals::Ocm::COMBINED_PWM_MODE_1,
            #[cfg(timer_v2)]
            OutputCompareMode::CombinedPwmMode2 => crate::pac::timer::vals::Ocm::COMBINED_PWM_MODE_2,
            #[cfg(timer_v2)]
            OutputCompareMode::AsymmetricPwmMode1 => crate::pac::timer::vals::Ocm::ASYMMETRIC_PWM_MODE_1,
            #[cfg(timer_v2)]
            OutputCompareMode::AsymmetricPwmMode2 => crate::pac::timer::vals::Ocm::ASYMMETRIC_PWM_MODE_2,
        }
    }
}

/// Timer output pin polarity.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputPolarity {
    /// Active high (higher duty value makes the pin spend more time high).
    ActiveHigh,
    /// Active low (higher duty value makes the pin spend more time low).
    ActiveLow,
}

impl From<OutputPolarity> for bool {
    fn from(mode: OutputPolarity) -> Self {
        match mode {
            OutputPolarity::ActiveHigh => false,
            OutputPolarity::ActiveLow => true,
        }
    }
}

/// Rounding mode for timer period/frequency configuration.
///
/// When configuring a timer, the exact requested period may not be achievable
/// due to hardware limitations (prescaler and counter are integers). This enum
/// controls how the driver rounds the configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RoundTo {
    /// Round towards a slower timer (higher period, lower frequency).
    ///
    /// The actual period will be >= the requested period.
    Slower,
    /// Round towards a faster timer (lower period, higher frequency).
    ///
    /// The actual period will be <= the requested period.
    Faster,
}

/// Result of PSC/ARR calculation for timer configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct PscArrConfig {
    /// Prescaler value (0-65535). The timer clock is divided by `psc + 1`.
    psc: u16,
    /// Auto-reload value. The timer counts from 0 to `arr`, then wraps.
    arr: u64,
    /// The actual period in clock cycles that will be achieved: `(psc + 1) * (arr + 1)`.
    actual_period_clocks: u64,
}

/// Error returned when the requested timer period is out of range.
///
/// This occurs when:
/// - For `RoundTo::Faster`: The requested period is less than 2 (minimum achievable is 2, since ARR >= 1).
/// - For `RoundTo::Slower`: The required prescaler exceeds 16 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OutOfRangeError;

/// Calculate prescaler (PSC) and auto-reload (ARR) values for a desired timer period.
///
/// # Arguments
/// * `period_clocks` - The desired period in timer clock cycles
/// * `round` - How to round when exact period is not achievable
/// * `max_arr_bits` - Maximum bits for ARR register (16 or 32)
///
/// # Returns
/// A [`PscArrConfig`] containing the calculated values, or an [`OutOfRangeError`] if the
/// requested period cannot be achieved with the given rounding mode.
///
/// # Errors
/// Returns `OutOfRangeError` when:
/// - `RoundTo::Faster` and `period_clocks < 2`: Cannot achieve period <= 1 (minimum is 2 since ARR >= 1).
/// - `RoundTo::Slower` and the required prescaler exceeds 16 bits.
fn calculate_psc_arr(period_clocks: u64, round: RoundTo, max_arr_bits: usize) -> Result<PscArrConfig, OutOfRangeError> {
    let max_arr: u64 = (1 << max_arr_bits) - 1;

    // Minimum achievable period is 2 (psc=0, arr=1), since ARR=0 is not valid.
    const MIN_PERIOD: u64 = 2;

    // For Faster, we need actual_period_clocks <= period_clocks
    // If period_clocks < MIN_PERIOD, we can't achieve this
    if round == RoundTo::Faster && period_clocks < MIN_PERIOD {
        return Err(OutOfRangeError);
    }

    // We need: period_clocks = (psc + 1) * (arr + 1)
    // Calculate minimum prescaler needed: psc >= period_clocks / (max_arr + 1) - 1
    let psc_min = period_clocks.saturating_sub(1) / (max_arr + 1);
    let psc: u16 = match psc_min.try_into() {
        Ok(v) => v,
        Err(_) => {
            // Prescaler would overflow
            match round {
                RoundTo::Slower => return Err(OutOfRangeError), // Can't achieve actual >= requested
                RoundTo::Faster => u16::MAX,                    // Use max psc; we only need actual <= requested
            }
        }
    };

    // Calculate arr for this prescaler
    let psc_plus_1 = u64::from(psc) + 1;

    // actual_clocks = (psc + 1) * (arr + 1), so arr = actual_clocks / (psc + 1) - 1
    // We want actual_clocks as close to period_clocks as possible, respecting rounding mode
    let arr = match round {
        RoundTo::Faster => {
            // Round down: actual_clocks <= period_clocks
            // arr + 1 <= period_clocks / (psc + 1)
            // arr <= period_clocks / (psc + 1) - 1
            (period_clocks / psc_plus_1).saturating_sub(1)
        }
        RoundTo::Slower => {
            // Round up: actual_clocks >= period_clocks
            // arr + 1 >= ceil(period_clocks / (psc + 1))
            // arr >= ceil(period_clocks / (psc + 1)) - 1
            period_clocks.div_ceil(psc_plus_1).saturating_sub(1)
        }
    };

    // Clamp arr to valid range (min is 1, not 0)
    let arr = arr.clamp(1, max_arr);
    let actual_period_clocks = psc_plus_1 * (arr + 1);

    Ok(PscArrConfig {
        psc,
        arr,
        actual_period_clocks,
    })
}

/// Helper to round a division according to the rounding mode.
fn div_round(numerator: u64, denominator: u64, round: RoundTo) -> u64 {
    match round {
        RoundTo::Faster => numerator / denominator,
        RoundTo::Slower => numerator.div_ceil(denominator),
    }
}

/// Low-level timer driver.
pub struct Timer<'d, T: CoreInstance> {
    tim: Peri<'d, T>,
}

impl<'d, T: CoreInstance> Drop for Timer<'d, T> {
    fn drop(&mut self) {
        rcc::disable::<T>();
    }
}

impl<'d, T: CoreInstance> Timer<'d, T> {
    /// Create a new timer driver.
    pub fn new(tim: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();

        Self { tim }
    }

    pub(crate) unsafe fn clone_unchecked(&self) -> ManuallyDrop<Self> {
        let tim = unsafe { self.tim.clone_unchecked() };
        ManuallyDrop::new(Self { tim })
    }

    /// Get access to the virutal core 16bit timer registers.
    ///
    /// Note: This works even if the timer is more capable, because registers
    /// for the less capable timers are a subset. This allows writing a driver
    /// for a given set of capabilities, and having it transparently work with
    /// more capable timers.
    pub fn regs_core(&self) -> crate::pac::timer::TimCore {
        unsafe { crate::pac::timer::TimCore::from_ptr(T::regs()) }
    }

    #[cfg(not(stm32l0))]
    fn regs_gp32_unchecked(&self) -> crate::pac::timer::TimGp32 {
        unsafe { crate::pac::timer::TimGp32::from_ptr(T::regs()) }
    }

    #[cfg(stm32l0)]
    fn regs_gp32_unchecked(&self) -> crate::pac::timer::TimGp16 {
        unsafe { crate::pac::timer::TimGp16::from_ptr(T::regs()) }
    }

    /// Start the timer.
    pub fn start(&self) {
        self.regs_core().cr1().modify(|r| r.set_cen(true));
    }

    /// Generate timer update event from software.
    ///
    /// Set URS to avoid generating interrupt or DMA request. This update event is only
    /// used to load value from pre-load registers. If called when the timer is running,
    /// it may disrupt the output waveform.
    pub fn generate_update_event(&self) {
        self.regs_core().cr1().modify(|r| r.set_urs(vals::Urs::COUNTER_ONLY));
        self.regs_core().egr().write(|r| r.set_ug(true));
        self.regs_core().cr1().modify(|r| r.set_urs(vals::Urs::ANY_EVENT));
    }

    /// Stop the timer.
    pub fn stop(&self) {
        self.regs_core().cr1().modify(|r| r.set_cen(false));
    }

    /// Reset the counter value to 0
    pub fn reset(&self) {
        self.regs_core().cnt().write(|r| r.set_cnt(0));
    }

    /// get the capability of the timer
    pub fn bits(&self) -> TimerBits {
        match T::Word::bits() {
            16 => TimerBits::Bits16,
            #[cfg(not(stm32l0))]
            32 => TimerBits::Bits32,
            _ => unreachable!(),
        }
    }

    /// Set the timer period in timer clock cycles.
    ///
    /// The timer will count for `clocks` clock cycles before wrapping.
    /// The actual period may differ from the requested value due to hardware
    /// limitations; the `round` parameter controls how rounding is performed.
    pub fn set_period_clocks(&self, clocks: u64, round: RoundTo) {
        self.set_period_clocks_internal(clocks, round, T::Word::bits());
    }

    pub(crate) fn set_period_clocks_internal(&self, clocks: u64, round: RoundTo, max_arr_bits: usize) {
        // TODO: we might want to propagate errors to the user instead of panicking.
        let config = unwrap!(calculate_psc_arr(clocks, round, max_arr_bits));
        let arr: T::Word = unwrap!(T::Word::try_from(config.arr));

        let regs = self.regs_gp32_unchecked();
        regs.psc().write_value(config.psc);
        #[cfg(stm32l0)]
        regs.arr().write(|r| r.set_arr(unwrap!(arr.try_into())));
        #[cfg(not(stm32l0))]
        regs.arr().write_value(arr.into());
    }

    /// Set the frequency of how many times per second the timer counts up to the max value or down to 0.
    ///
    /// This means that in the default edge-aligned mode,
    /// the timer counter will wrap around at the same frequency as is being set.
    /// In center-aligned mode (which not all timers support), the wrap-around frequency is effectively halved
    /// because it needs to count up and down.
    ///
    /// The actual frequency may differ from the requested value due to hardware
    /// limitations; the `round` parameter controls how rounding is performed.
    pub fn set_frequency(&self, frequency: Hertz, round: RoundTo) {
        let f = frequency.0;
        assert!(f > 0);
        let timer_f = T::frequency().0 as u64;
        let clocks = div_round(timer_f, f as u64, round);
        self.set_period_clocks(clocks, round);
    }

    /// Set the timer period in milliseconds.
    ///
    /// The actual period may differ from the requested value due to hardware
    /// limitations; the `round` parameter controls how rounding is performed.
    pub fn set_period_ms(&self, ms: u32, round: RoundTo) {
        let timer_f = T::frequency().0 as u64;
        let clocks = div_round(timer_f * ms as u64, 1_000, round);
        self.set_period_clocks(clocks, round);
    }

    /// Set the timer period in microseconds.
    ///
    /// The actual period may differ from the requested value due to hardware
    /// limitations; the `round` parameter controls how rounding is performed.
    pub fn set_period_us(&self, us: u32, round: RoundTo) {
        let timer_f = T::frequency().0 as u64;
        let clocks = div_round(timer_f * us as u64, 1_000_000, round);
        self.set_period_clocks(clocks, round);
    }

    /// Set the timer period in seconds.
    ///
    /// The actual period may differ from the requested value due to hardware
    /// limitations; the `round` parameter controls how rounding is performed.
    pub fn set_period_secs(&self, secs: u32, round: RoundTo) {
        let timer_f = T::frequency().0 as u64;
        let clocks = timer_f * secs as u64;
        self.set_period_clocks(clocks, round);
    }

    /// Set the timer period using an `embassy_time::Duration`.
    ///
    /// The actual period may differ from the requested value due to hardware
    /// limitations; the `round` parameter controls how rounding is performed.
    #[cfg(feature = "time")]
    pub fn set_period(&self, period: embassy_time::Duration, round: RoundTo) {
        let timer_f = T::frequency().0 as u64;
        let clocks = div_round(timer_f * period.as_ticks(), embassy_time::TICK_HZ, round);
        self.set_period_clocks(clocks, round);
    }

    /// Set tick frequency.
    pub fn set_tick_freq(&mut self, freq: Hertz) {
        let f = freq;
        assert!(f.0 > 0);
        let timer_f = self.get_clock_frequency();

        let pclk_ticks_per_timer_period = timer_f / f;
        let psc: u16 = unwrap!((pclk_ticks_per_timer_period - 1).try_into());

        let regs = self.regs_core();
        regs.psc().write_value(psc);

        // Generate an Update Request
        regs.egr().write(|r| r.set_ug(true));
    }

    /// Clear update interrupt.
    ///
    /// Returns whether the update interrupt flag was set.
    pub fn clear_update_interrupt(&self) -> bool {
        let regs = self.regs_core();
        let sr = regs.sr().read();
        if sr.uif() {
            regs.sr().modify(|r| {
                r.set_uif(false);
            });
            true
        } else {
            false
        }
    }

    /// Enable/disable the update interrupt.
    pub fn enable_update_interrupt(&self, enable: bool) {
        self.regs_core().dier().modify(|r| r.set_uie(enable));
    }

    /// Enable/disable autoreload preload.
    pub fn set_autoreload_preload(&self, enable: bool) {
        self.regs_core().cr1().modify(|r| r.set_arpe(enable));
    }

    /// Get the timer frequency.
    pub fn get_frequency(&self) -> Hertz {
        let timer_f = T::frequency();

        let regs = self.regs_gp32_unchecked();
        #[cfg(not(stm32l0))]
        let arr = regs.arr().read();
        #[cfg(stm32l0)]
        let arr = regs.arr().read().arr();
        let psc = regs.psc().read();

        timer_f / arr / (psc + 1)
    }

    /// Get the clock frequency of the timer (before prescaler is applied).
    pub fn get_clock_frequency(&self) -> Hertz {
        T::frequency()
    }
}

impl<'d, T: BasicNoCr2Instance> Timer<'d, T> {
    /// Get access to the Baisc 16bit timer registers.
    ///
    /// Note: This works even if the timer is more capable, because registers
    /// for the less capable timers are a subset. This allows writing a driver
    /// for a given set of capabilities, and having it transparently work with
    /// more capable timers.
    pub fn regs_basic_no_cr2(&self) -> crate::pac::timer::TimBasicNoCr2 {
        unsafe { crate::pac::timer::TimBasicNoCr2::from_ptr(T::regs()) }
    }

    /// Enable/disable the update dma.
    pub fn enable_update_dma(&self, enable: bool) {
        self.regs_basic_no_cr2().dier().modify(|r| r.set_ude(enable));
    }

    /// Get the update dma enable/disable state.
    pub fn get_update_dma_state(&self) -> bool {
        self.regs_basic_no_cr2().dier().read().ude()
    }
}

impl<'d, T: BasicInstance> Timer<'d, T> {
    /// Get access to the Baisc 16bit timer registers.
    ///
    /// Note: This works even if the timer is more capable, because registers
    /// for the less capable timers are a subset. This allows writing a driver
    /// for a given set of capabilities, and having it transparently work with
    /// more capable timers.
    pub fn regs_basic(&self) -> crate::pac::timer::TimBasic {
        unsafe { crate::pac::timer::TimBasic::from_ptr(T::regs()) }
    }
}

impl<'d, T: GeneralInstance1Channel> Timer<'d, T> {
    /// Get access to the general purpose 1 channel 16bit timer registers.
    ///
    /// Note: This works even if the timer is more capable, because registers
    /// for the less capable timers are a subset. This allows writing a driver
    /// for a given set of capabilities, and having it transparently work with
    /// more capable timers.
    pub fn regs_1ch(&self) -> crate::pac::timer::Tim1ch {
        unsafe { crate::pac::timer::Tim1ch::from_ptr(T::regs()) }
    }

    /// Set clock divider.
    pub fn set_clock_division(&self, ckd: vals::Ckd) {
        self.regs_1ch().cr1().modify(|r| r.set_ckd(ckd));
    }

    /// Get max compare value. This depends on the timer frequency and the clock frequency from RCC.
    pub fn get_max_compare_value(&self) -> T::Word {
        #[cfg(not(stm32l0))]
        return unwrap!(self.regs_gp32_unchecked().arr().read().try_into());
        #[cfg(stm32l0)]
        return unwrap!(self.regs_gp32_unchecked().arr().read().arr().try_into());
    }

    /// Set the max compare value.
    ///
    /// An update event is generated to load the new value. The update event is
    /// generated such that it will not cause an interrupt or DMA request.
    pub fn set_max_compare_value(&self, ticks: T::Word) {
        let arr = ticks;

        let regs = self.regs_gp32_unchecked();
        #[cfg(not(stm32l0))]
        regs.arr().write_value(arr.into());
        #[cfg(stm32l0)]
        regs.arr().write(|r| r.set_arr(unwrap!(arr.try_into())));

        regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTER_ONLY));
        regs.egr().write(|r| r.set_ug(true));
        regs.cr1().modify(|r| r.set_urs(vals::Urs::ANY_EVENT));
    }
}

impl<'d, T: GeneralInstance2Channel> Timer<'d, T> {
    /// Get access to the general purpose 2 channel 16bit timer registers.
    ///
    /// Note: This works even if the timer is more capable, because registers
    /// for the less capable timers are a subset. This allows writing a driver
    /// for a given set of capabilities, and having it transparently work with
    /// more capable timers.
    pub fn regs_2ch(&self) -> crate::pac::timer::Tim2ch {
        unsafe { crate::pac::timer::Tim2ch::from_ptr(T::regs()) }
    }
}

impl<'d, T: GeneralInstance4Channel> Timer<'d, T> {
    /// Get access to the general purpose 16bit timer registers.
    ///
    /// Note: This works even if the timer is more capable, because registers
    /// for the less capable timers are a subset. This allows writing a driver
    /// for a given set of capabilities, and having it transparently work with
    /// more capable timers.
    pub fn regs_gp16(&self) -> crate::pac::timer::TimGp16 {
        unsafe { crate::pac::timer::TimGp16::from_ptr(T::regs()) }
    }

    /// Enable timer outputs.
    pub fn enable_outputs(&self) {
        self.tim.enable_outputs()
    }

    /// Set counting mode.
    pub fn set_counting_mode(&self, mode: CountingMode) {
        let (cms, dir) = mode.into();

        let timer_enabled = self.regs_core().cr1().read().cen();
        // Changing from edge aligned to center aligned (and vice versa) is not allowed while the timer is running.
        // Changing direction is discouraged while the timer is running.
        assert!(!timer_enabled);

        self.regs_gp16().cr1().modify(|r| r.set_dir(dir));
        self.regs_gp16().cr1().modify(|r| r.set_cms(cms))
    }

    /// Get counting mode.
    pub fn get_counting_mode(&self) -> CountingMode {
        let cr1 = self.regs_gp16().cr1().read();
        (cr1.cms(), cr1.dir()).into()
    }

    /// Set input capture filter.
    pub fn set_input_capture_filter(&self, channel: Channel, icf: vals::FilterValue) {
        let raw_channel = channel.index();
        self.regs_gp16()
            .ccmr_input(raw_channel / 2)
            .modify(|r| r.set_icf(raw_channel % 2, icf));
    }

    /// Clear input interrupt.
    pub fn clear_input_interrupt(&self, channel: Channel) {
        self.regs_gp16().sr().modify(|r| r.set_ccif(channel.index(), false));
    }

    /// Get input interrupt.
    pub fn get_input_interrupt(&self, channel: Channel) -> bool {
        self.regs_gp16().sr().read().ccif(channel.index())
    }

    /// Enable input interrupt.
    pub fn enable_input_interrupt(&self, channel: Channel, enable: bool) {
        self.regs_gp16().dier().modify(|r| r.set_ccie(channel.index(), enable));
    }

    /// Set input capture prescaler.
    pub fn set_input_capture_prescaler(&self, channel: Channel, factor: u8) {
        let raw_channel = channel.index();
        self.regs_gp16()
            .ccmr_input(raw_channel / 2)
            .modify(|r| r.set_icpsc(raw_channel % 2, factor));
    }

    /// Set input TI selection.
    pub fn set_input_ti_selection(&self, channel: Channel, tisel: InputTISelection) {
        let raw_channel = channel.index();
        self.regs_gp16()
            .ccmr_input(raw_channel / 2)
            .modify(|r| r.set_ccs(raw_channel % 2, tisel.into()));
    }

    /// Set input capture mode.
    pub fn set_input_capture_mode(&self, channel: Channel, mode: InputCaptureMode) {
        self.regs_gp16().ccer().modify(|r| match mode {
            InputCaptureMode::Rising => {
                r.set_ccnp(channel.index(), false);
                r.set_ccp(channel.index(), false);
            }
            InputCaptureMode::Falling => {
                r.set_ccnp(channel.index(), false);
                r.set_ccp(channel.index(), true);
            }
            InputCaptureMode::BothEdges => {
                r.set_ccnp(channel.index(), true);
                r.set_ccp(channel.index(), true);
            }
        });
    }

    /// Set output compare mode.
    pub fn set_output_compare_mode(&self, channel: Channel, mode: OutputCompareMode) {
        let raw_channel: usize = channel.index();
        self.regs_gp16()
            .ccmr_output(raw_channel / 2)
            .modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
    }

    /// Set output polarity.
    pub fn set_output_polarity(&self, channel: Channel, polarity: OutputPolarity) {
        self.regs_gp16()
            .ccer()
            .modify(|w| w.set_ccp(channel.index(), polarity.into()));
    }

    /// Enable/disable a channel.
    pub fn enable_channel(&self, channel: Channel, enable: bool) {
        self.regs_gp16().ccer().modify(|w| w.set_cce(channel.index(), enable));
    }

    /// Get enable/disable state of a channel
    pub fn get_channel_enable_state(&self, channel: Channel) -> bool {
        self.regs_gp16().ccer().read().cce(channel.index())
    }

    /// Set compare value for a channel.
    pub fn set_compare_value(&self, channel: Channel, value: T::Word) {
        #[cfg(not(stm32l0))]
        self.regs_gp32_unchecked()
            .ccr(channel.index())
            .write_value(value.into());
        #[cfg(stm32l0)]
        self.regs_gp16()
            .ccr(channel.index())
            .modify(|w| w.set_ccr(unwrap!(value.try_into())));
    }

    /// Get compare value for a channel.
    pub fn get_compare_value(&self, channel: Channel) -> T::Word {
        #[cfg(not(stm32l0))]
        return unwrap!(self.regs_gp32_unchecked().ccr(channel.index()).read().try_into());
        #[cfg(stm32l0)]
        return unwrap!(self.regs_gp32_unchecked().ccr(channel.index()).read().ccr().try_into());
    }

    pub(crate) fn clamp_compare_value<W: Word>(&mut self, channel: Channel) {
        self.set_compare_value(
            channel,
            unwrap!(
                self.get_compare_value(channel)
                    .into()
                    .clamp(0, W::max() as u32)
                    .try_into()
            ),
        );
    }

    /// Setup a ring buffer for the channel
    pub fn setup_ring_buffer<'a, W: Word + Into<T::Word>, D: super::UpDma<T>>(
        &mut self,
        dma: Peri<'a, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
        channel: Channel,
        dma_buf: &'a mut [W],
    ) -> WritableRingBuffer<'a, W> {
        #[allow(clippy::let_unit_value)] // eg. stm32f334
        let req = dma.request();

        unsafe {
            use crate::dma::TransferOptions;
            #[cfg(not(any(bdma, gpdma)))]
            use crate::dma::{Burst, FifoThreshold};

            let dma_transfer_option = TransferOptions {
                #[cfg(not(any(bdma, gpdma)))]
                fifo_threshold: Some(FifoThreshold::Full),
                #[cfg(not(any(bdma, gpdma)))]
                mburst: Burst::Incr8,
                ..Default::default()
            };

            WritableRingBuffer::new(
                dma::Channel::new(dma, irq),
                req,
                self.regs_1ch().ccr(channel.index()).as_ptr() as *mut W,
                dma_buf,
                dma_transfer_option,
            )
        }
    }

    /// Generate a sequence of PWM waveform
    ///
    /// Note:
    /// you will need to provide corresponding TIMx_UP DMA channel to use this method.
    pub fn setup_update_dma<'a, W: Word + Into<T::Word>, D: super::UpDma<T>>(
        &mut self,
        dma: Peri<'a, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
        channel: Channel,
        duty: &'a [W],
    ) -> Transfer<'a> {
        self.setup_update_dma_inner(dma.request(), dma, irq, channel, duty)
    }

    /// Generate a sequence of PWM waveform
    ///
    /// Note:
    /// The DMA channel provided does not need to correspond to the requested channel.
    pub fn setup_channel_update_dma<'a, C: TimerChannel, W: Word + Into<T::Word>, D: super::Dma<T, C>>(
        &mut self,
        dma: Peri<'a, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
        channel: Channel,
        duty: &'a [W],
    ) -> Transfer<'a> {
        self.setup_update_dma_inner(dma.request(), dma, irq, channel, duty)
    }

    fn setup_update_dma_inner<'a, W: Word + Into<T::Word>, D: dma::ChannelInstance>(
        &mut self,
        request: dma::Request,
        dma: Peri<'a, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
        channel: Channel,
        duty: &'a [W],
    ) -> Transfer<'a> {
        unsafe {
            use crate::dma::TransferOptions;
            #[cfg(not(any(bdma, gpdma)))]
            use crate::dma::{Burst, FifoThreshold};

            let dma_transfer_option = TransferOptions {
                #[cfg(not(any(bdma, gpdma)))]
                fifo_threshold: Some(FifoThreshold::Full),
                #[cfg(not(any(bdma, gpdma)))]
                mburst: Burst::Incr8,
                ..Default::default()
            };

            let mut dma_channel = dma::Channel::new(dma, irq);
            dma_channel
                .write(
                    request,
                    duty,
                    self.regs_gp16().ccr(channel.index()).as_ptr() as *mut W,
                    dma_transfer_option,
                )
                .unchecked_extend_lifetime()
        }
    }

    /// Generate a multichannel sequence of PWM waveforms using DMA triggered by timer update events.
    ///
    /// This method utilizes the timer's DMA burst transfer capability to update multiple CCRx registers
    /// in sequence on each update event (UEV). The data is written via the DMAR register using the
    /// DMA base address (DBA) and burst length (DBL) configured in the DCR register.
    ///
    /// The `duty` buffer must be structured as a flattened 2D array in row-major order, where each row
    /// represents a single update event and each column corresponds to a specific timer channel (starting
    /// from `starting_channel` up to and including `ending_channel`).
    ///
    /// For example, if using channels 1 through 4, a buffer of 4 update steps might look like:
    ///
    /// ```rust,ignore
    /// let dma_buf: [u16; 16] = [
    ///     ch1_duty_1, ch2_duty_1, ch3_duty_1, ch4_duty_1, // update 1
    ///     ch1_duty_2, ch2_duty_2, ch3_duty_2, ch4_duty_2, // update 2
    ///     ch1_duty_3, ch2_duty_3, ch3_duty_3, ch4_duty_3, // update 3
    ///     ch1_duty_4, ch2_duty_4, ch3_duty_4, ch4_duty_4, // update 4
    /// ];
    /// ```
    ///
    /// Each group of `N` values (where `N` is number of channels) is transferred on one update event,
    /// updating the duty cycles of all selected channels simultaneously.
    ///
    /// Note:
    /// You will need to provide corresponding `TIMx_UP` DMA channel to use this method.
    /// Also be aware that embassy timers use one of timers internally. It is possible to
    /// switch this timer by using `time-driver-timX` feature.
    ///
    pub fn setup_update_dma_burst<'a, W: Word + Into<T::Word>, D: super::UpDma<T>>(
        &mut self,
        dma: Peri<'a, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
        starting_channel: Channel,
        ending_channel: Channel,
        duty: &'a [W],
    ) -> Transfer<'a> {
        let cr1_addr = self.regs_gp16().cr1().as_ptr() as u32;
        let start_ch_index = starting_channel.index();
        let end_ch_index = ending_channel.index();

        assert!(start_ch_index <= end_ch_index);

        let ccrx_addr = self.regs_gp16().ccr(start_ch_index).as_ptr() as u32;
        self.regs_gp16()
            .dcr()
            .modify(|w| w.set_dba(((ccrx_addr - cr1_addr) / 4) as u8));
        self.regs_gp16()
            .dcr()
            .modify(|w| w.set_dbl((end_ch_index - start_ch_index) as u8));

        #[allow(clippy::let_unit_value)] // eg. stm32f334
        let req = dma.request();

        unsafe {
            use crate::dma::TransferOptions;
            #[cfg(not(any(bdma, gpdma)))]
            use crate::dma::{Burst, FifoThreshold};

            let dma_transfer_option = TransferOptions {
                #[cfg(not(any(bdma, gpdma)))]
                fifo_threshold: Some(FifoThreshold::Full),
                #[cfg(not(any(bdma, gpdma)))]
                mburst: Burst::Incr4,
                ..Default::default()
            };

            let mut dma_channel = dma::Channel::new(dma, irq);
            dma_channel
                .write(
                    req,
                    duty,
                    self.regs_gp16().dmar().as_ptr() as *mut W,
                    dma_transfer_option,
                )
                .unchecked_extend_lifetime()
        }
    }

    /// Get capture value for a channel.
    pub fn get_capture_value(&self, channel: Channel) -> T::Word {
        self.get_compare_value(channel)
    }

    /// Set output compare preload.
    pub fn set_output_compare_preload(&self, channel: Channel, preload: bool) {
        let channel_index = channel.index();
        self.regs_gp16()
            .ccmr_output(channel_index / 2)
            .modify(|w| w.set_ocpe(channel_index % 2, preload));
    }

    /// Get capture compare DMA selection
    pub fn get_cc_dma_selection(&self) -> vals::Ccds {
        self.regs_gp16().cr2().read().ccds()
    }

    /// Set capture compare DMA selection
    pub fn set_cc_dma_selection(&self, ccds: vals::Ccds) {
        self.regs_gp16().cr2().modify(|w| w.set_ccds(ccds))
    }

    /// Get capture compare DMA enable state
    pub fn get_cc_dma_enable_state(&self, channel: Channel) -> bool {
        self.regs_gp16().dier().read().ccde(channel.index())
    }

    /// Set capture compare DMA enable state
    pub fn set_cc_dma_enable_state(&self, channel: Channel, ccde: bool) {
        self.regs_gp16().dier().modify(|w| w.set_ccde(channel.index(), ccde))
    }

    /// Set Timer Master Mode
    pub fn set_master_mode(&self, mms: MasterMode) {
        self.regs_gp16().cr2().modify(|w| w.set_mms(mms));
    }

    /// Set Timer Slave Mode
    pub fn set_slave_mode(&self, sms: SlaveMode) {
        self.regs_gp16().smcr().modify(|r| r.set_sms(sms));
    }

    /// Set Timer Trigger Source
    pub fn set_trigger_source(&self, ts: TriggerSource) {
        self.regs_gp16().smcr().modify(|r| r.set_ts(ts));
    }
}

#[cfg(not(stm32l0))]
impl<'d, T: GeneralInstance32bit4Channel> Timer<'d, T> {
    /// Get access to the general purpose 32bit timer registers.
    ///
    /// Note: This works even if the timer is more capable, because registers
    /// for the less capable timers are a subset. This allows writing a driver
    /// for a given set of capabilities, and having it transparently work with
    /// more capable timers.
    pub fn regs_gp32(&self) -> crate::pac::timer::TimGp32 {
        unsafe { crate::pac::timer::TimGp32::from_ptr(T::regs()) }
    }
}

#[cfg(not(stm32l0))]
impl<'d, T: AdvancedInstance1Channel> Timer<'d, T> {
    /// Get access to the general purpose 1 channel with one complementary 16bit timer registers.
    ///
    /// Note: This works even if the timer is more capable, because registers
    /// for the less capable timers are a subset. This allows writing a driver
    /// for a given set of capabilities, and having it transparently work with
    /// more capable timers.
    pub fn regs_1ch_cmp(&self) -> crate::pac::timer::Tim1chCmp {
        unsafe { crate::pac::timer::Tim1chCmp::from_ptr(T::regs()) }
    }

    /// Set clock divider for the dead time.
    pub fn set_dead_time_clock_division(&self, value: vals::Ckd) {
        self.regs_1ch_cmp().cr1().modify(|w| w.set_ckd(value));
    }

    /// Set dead time, as a fraction of the max duty value.
    pub fn set_dead_time_value(&self, value: u8) {
        self.regs_1ch_cmp().bdtr().modify(|w| w.set_dtg(value));
    }

    /// Set state of OSSI-bit in BDTR register
    pub fn set_ossi(&self, val: vals::Ossi) {
        self.regs_1ch_cmp().bdtr().modify(|w| w.set_ossi(val));
    }

    /// Get state of OSSI-bit in BDTR register
    pub fn get_ossi(&self) -> vals::Ossi {
        self.regs_1ch_cmp().bdtr().read().ossi()
    }

    /// Set state of OSSR-bit in BDTR register
    pub fn set_ossr(&self, val: vals::Ossr) {
        self.regs_1ch_cmp().bdtr().modify(|w| w.set_ossr(val));
    }

    /// Get state of OSSR-bit in BDTR register
    pub fn get_ossr(&self) -> vals::Ossr {
        self.regs_1ch_cmp().bdtr().read().ossr()
    }

    /// Set state of MOE-bit in BDTR register to en-/disable output
    pub fn set_moe(&self, enable: bool) {
        self.regs_1ch_cmp().bdtr().modify(|w| w.set_moe(enable));
    }

    /// Get state of MOE-bit in BDTR register
    pub fn get_moe(&self) -> bool {
        self.regs_1ch_cmp().bdtr().read().moe()
    }
}

#[cfg(not(stm32l0))]
impl<'d, T: AdvancedInstance2Channel> Timer<'d, T> {
    /// Get access to the general purpose 2 channel with one complementary 16bit timer registers.
    ///
    /// Note: This works even if the timer is more capable, because registers
    /// for the less capable timers are a subset. This allows writing a driver
    /// for a given set of capabilities, and having it transparently work with
    /// more capable timers.
    pub fn regs_2ch_cmp(&self) -> crate::pac::timer::Tim2chCmp {
        unsafe { crate::pac::timer::Tim2chCmp::from_ptr(T::regs()) }
    }
}

#[cfg(not(stm32l0))]
impl<'d, T: AdvancedInstance4Channel> Timer<'d, T> {
    /// Get access to the advanced timer registers.
    pub fn regs_advanced(&self) -> crate::pac::timer::TimAdv {
        unsafe { crate::pac::timer::TimAdv::from_ptr(T::regs()) }
    }

    /// Set complementary output polarity.
    pub fn set_complementary_output_polarity(&self, channel: Channel, polarity: OutputPolarity) {
        self.regs_advanced()
            .ccer()
            .modify(|w| w.set_ccnp(channel.index(), polarity.into()));
    }

    /// Enable/disable a complementary channel.
    pub fn enable_complementary_channel(&self, channel: Channel, enable: bool) {
        self.regs_advanced()
            .ccer()
            .modify(|w| w.set_ccne(channel.index(), enable));
    }

    /// Set Output Idle State
    pub fn set_ois(&self, channel: Channel, val: bool) {
        self.regs_advanced().cr2().modify(|w| w.set_ois(channel.index(), val));
    }
    /// Set Output Idle State Complementary Channel
    pub fn set_oisn(&self, channel: Channel, val: bool) {
        self.regs_advanced().cr2().modify(|w| w.set_oisn(channel.index(), val));
    }

    /// Set master mode selection 2
    pub fn set_mms2_selection(&self, mms2: vals::Mms2) {
        self.regs_advanced().cr2().modify(|w| w.set_mms2(mms2));
    }

    /// Set repetition counter
    pub fn set_repetition_counter(&self, val: u16) {
        self.regs_advanced().rcr().modify(|w| w.set_rep(val));
    }

    /// Trigger software break 1 or 2
    /// Setting this bit generates a break event. This bit is automatically cleared by the hardware.
    pub fn trigger_software_break(&self, n: usize) {
        self.regs_advanced().egr().write(|r| r.set_bg(n, true));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test cases: (period_clocks, max_arr_bits, expect_fail_slower, expect_fail_faster)
    const TEST_CASES: &[(u64, usize, bool, bool)] = &[
        // Small periods (no prescaler needed for 16-bit)
        // period=0,1 fail for Faster because min achievable is 2 (arr=1)
        (0, 16, false, true),
        (1, 16, false, true),
        (2, 16, false, false), // Minimum achievable period
        (100, 16, false, false),
        (1000, 16, false, false),
        (65535, 16, false, false),
        (65536, 16, false, false),
        // Periods requiring prescaler for 16-bit
        (65537, 16, false, false),
        (100_000, 16, false, false),
        (1_000_000, 16, false, false),
        (10_000_000, 16, false, false),
        // Edge cases around boundaries
        (131070, 16, false, false), // 2 * 65535
        (131072, 16, false, false), // 2 * 65536
        (196605, 16, false, false), // 3 * 65535
        // 32-bit timer cases
        (0, 32, false, true),
        (1, 32, false, true),
        (2, 32, false, false),
        (100_000, 32, false, false),
        (1_000_000_000, 32, false, false),
        (4_294_967_295, 32, false, false), // u32::MAX
        (4_294_967_296, 32, false, false), // u32::MAX + 1
        // Very large periods that would overflow 16-bit prescaler for Slower
        // max_arr for 16-bit is 65535, so max period with psc=65535 is 65536*65536 = 4_294_967_296
        // Anything larger than that fails for Slower (need actual >= requested, impossible)
        // For Faster, it still works (need actual <= requested, can always use max period)
        (4_294_967_297, 16, true, false), // Just over 16-bit max, fails Slower only
    ];

    fn actual_clocks(psc: u16, arr: u64) -> u64 {
        (psc as u64 + 1) * (arr + 1)
    }

    #[test]
    fn test_calculate_psc_arr() {
        for &(period_clocks, max_arr_bits, expect_fail_slower, expect_fail_faster) in TEST_CASES {
            let max_arr: u64 = (1 << max_arr_bits) - 1;

            for round in [RoundTo::Slower, RoundTo::Faster] {
                let expect_fail = match round {
                    RoundTo::Slower => expect_fail_slower,
                    RoundTo::Faster => expect_fail_faster,
                };

                let result = calculate_psc_arr(period_clocks, round, max_arr_bits);

                if expect_fail {
                    assert!(
                        result.is_err(),
                        "Expected failure for period_clocks={}, round={:?}, max_arr_bits={}, but got {:?}",
                        period_clocks,
                        round,
                        max_arr_bits,
                        result
                    );
                    continue;
                }

                let config = result.unwrap_or_else(|_| {
                    panic!(
                        "Unexpected failure for period_clocks={}, round={:?}, max_arr_bits={}",
                        period_clocks, round, max_arr_bits
                    )
                });

                // Verify actual_period_clocks matches (psc + 1) * (arr + 1)
                let computed_actual = actual_clocks(config.psc, config.arr);
                assert_eq!(
                    config.actual_period_clocks, computed_actual,
                    "actual_period_clocks mismatch for period_clocks={}, round={:?}",
                    period_clocks, round
                );

                // Verify arr is within bounds (min is 1)
                assert!(
                    config.arr >= 1 && config.arr <= max_arr,
                    "arr {} out of bounds [1, {}] for period_clocks={}, round={:?}",
                    config.arr,
                    max_arr,
                    period_clocks,
                    round
                );

                // Check rounding constraint
                match round {
                    RoundTo::Slower => {
                        assert!(
                            config.actual_period_clocks >= period_clocks,
                            "Slower: actual {} < requested {} for period_clocks={}, max_arr_bits={}",
                            config.actual_period_clocks,
                            period_clocks,
                            period_clocks,
                            max_arr_bits
                        );
                    }
                    RoundTo::Faster => {
                        assert!(
                            config.actual_period_clocks <= period_clocks,
                            "Faster: actual {} > requested {} for period_clocks={}, max_arr_bits={}",
                            config.actual_period_clocks,
                            period_clocks,
                            period_clocks,
                            max_arr_bits
                        );
                    }
                }

                // Test mutations: verify the solution is not obviously suboptimal.
                // Try all combinations of psc +/- 1 and arr +/- 1
                // This doesn't guarantee optimality. but it's enough to catch dumb off-by-one bugs.
                // Guaranteeing optimality would require searching all divisors of `period_clocks` which is obviously too expensive.
                let mutations: [(i32, i64); 8] = [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)];

                for (psc_delta, arr_delta) in mutations {
                    let new_psc = config.psc as i32 + psc_delta;
                    let new_arr = config.arr as i64 + arr_delta;

                    // Skip invalid mutations
                    if new_psc < 0 || new_psc > u16::MAX as i32 {
                        continue;
                    }
                    if new_arr < 1 || new_arr > max_arr as i64 {
                        continue;
                    }

                    let new_psc = new_psc as u16;
                    let new_arr = new_arr as u64;
                    let new_actual = actual_clocks(new_psc, new_arr);

                    // Check if mutation satisfies the rounding constraint
                    let satisfies_constraint = match round {
                        RoundTo::Slower => new_actual >= period_clocks,
                        RoundTo::Faster => new_actual <= period_clocks,
                    };

                    if satisfies_constraint {
                        // If it satisfies the constraint, it should not be better (closer) than our solution
                        let our_distance = (config.actual_period_clocks as i64 - period_clocks as i64).abs();
                        let new_distance = (new_actual as i64 - period_clocks as i64).abs();

                        assert!(
                            new_distance >= our_distance,
                            "Found better solution via mutation for period_clocks={}, round={:?}, max_arr_bits={}: \
                             original (psc={}, arr={}, actual={}, dist={}) vs \
                             mutated (psc={}, arr={}, actual={}, dist={})",
                            period_clocks,
                            round,
                            max_arr_bits,
                            config.psc,
                            config.arr,
                            config.actual_period_clocks,
                            our_distance,
                            new_psc,
                            new_arr,
                            new_actual,
                            new_distance
                        );
                    }
                    // If mutation doesn't satisfy constraint, that's fine - our solution is better
                }
            }
        }
    }

    #[test]
    fn test_div_round() {
        // Faster (round down)
        assert_eq!(div_round(10, 3, RoundTo::Faster), 3);
        assert_eq!(div_round(9, 3, RoundTo::Faster), 3);
        assert_eq!(div_round(11, 3, RoundTo::Faster), 3);
        assert_eq!(div_round(12, 3, RoundTo::Faster), 4);

        // Slower (round up)
        assert_eq!(div_round(10, 3, RoundTo::Slower), 4);
        assert_eq!(div_round(9, 3, RoundTo::Slower), 3);
        assert_eq!(div_round(11, 3, RoundTo::Slower), 4);
        assert_eq!(div_round(12, 3, RoundTo::Slower), 4);
    }
}
