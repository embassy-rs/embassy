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
use crate::pac::timer::vals;
use crate::time::Hertz;
use crate::{dma, rcc};

/// Input capture mode.
#[derive(Clone, Copy)]
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
        T::BITS
    }

    /// Set the frequency of how many times per second the timer counts up to the max value or down to 0.
    ///
    /// This means that in the default edge-aligned mode,
    /// the timer counter will wrap around at the same frequency as is being set.
    /// In center-aligned mode (which not all timers support), the wrap-around frequency is effectively halved
    /// because it needs to count up and down.
    pub fn set_frequency(&self, frequency: Hertz) {
        match T::BITS {
            TimerBits::Bits16 => {
                self.set_frequency_internal(frequency, 16);
            }
            #[cfg(not(stm32l0))]
            TimerBits::Bits32 => {
                self.set_frequency_internal(frequency, 32);
            }
        }
    }

    pub(crate) fn set_frequency_internal(&self, frequency: Hertz, max_divide_by_bits: u8) {
        let f = frequency.0;
        assert!(f > 0);
        let timer_f = T::frequency().0;

        let pclk_ticks_per_timer_period = (timer_f / f) as u64;
        let psc: u16 = unwrap!(((pclk_ticks_per_timer_period - 1) / (1 << max_divide_by_bits)).try_into());
        let divide_by = pclk_ticks_per_timer_period / (u64::from(psc) + 1);

        match T::BITS {
            TimerBits::Bits16 => {
                // the timer counts `0..=arr`, we want it to count `0..divide_by`
                let arr = unwrap!(u16::try_from(divide_by - 1));

                let regs = self.regs_core();
                regs.psc().write_value(psc);
                regs.arr().write(|r| r.set_arr(arr));
            }
            #[cfg(not(stm32l0))]
            TimerBits::Bits32 => {
                // the timer counts `0..=arr`, we want it to count `0..divide_by`
                let arr: u32 = unwrap!(u32::try_from(divide_by - 1));

                let regs = self.regs_gp32_unchecked();
                regs.psc().write_value(psc);
                regs.arr().write_value(arr);
            }
        }
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

        match T::BITS {
            TimerBits::Bits16 => {
                let regs = self.regs_core();
                let arr = regs.arr().read().arr();
                let psc = regs.psc().read();

                timer_f / arr / (psc + 1)
            }
            #[cfg(not(stm32l0))]
            TimerBits::Bits32 => {
                let regs = self.regs_gp32_unchecked();
                let arr = regs.arr().read();
                let psc = regs.psc().read();

                timer_f / arr / (psc + 1)
            }
        }
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
    pub fn get_max_compare_value(&self) -> u32 {
        match T::BITS {
            TimerBits::Bits16 => self.regs_1ch().arr().read().arr() as u32,
            #[cfg(not(stm32l0))]
            TimerBits::Bits32 => self.regs_gp32_unchecked().arr().read(),
        }
    }

    /// Set the max compare value.
    ///
    /// An update event is generated to load the new value. The update event is
    /// generated such that it will not cause an interrupt or DMA request.
    pub fn set_max_compare_value(&self, ticks: u32) {
        match T::BITS {
            TimerBits::Bits16 => {
                let arr = unwrap!(u16::try_from(ticks));

                let regs = self.regs_1ch();
                regs.arr().write(|r| r.set_arr(arr));

                regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTER_ONLY));
                regs.egr().write(|r| r.set_ug(true));
                regs.cr1().modify(|r| r.set_urs(vals::Urs::ANY_EVENT));
            }
            #[cfg(not(stm32l0))]
            TimerBits::Bits32 => {
                let arr = ticks;

                let regs = self.regs_gp32_unchecked();
                regs.arr().write_value(arr);

                regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTER_ONLY));
                regs.egr().write(|r| r.set_ug(true));
                regs.cr1().modify(|r| r.set_urs(vals::Urs::ANY_EVENT));
            }
        }
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
    pub fn set_compare_value(&self, channel: Channel, value: u32) {
        match T::BITS {
            TimerBits::Bits16 => {
                let value = unwrap!(u16::try_from(value));
                self.regs_gp16().ccr(channel.index()).modify(|w| w.set_ccr(value));
            }
            #[cfg(not(stm32l0))]
            TimerBits::Bits32 => {
                self.regs_gp32_unchecked().ccr(channel.index()).write_value(value);
            }
        }
    }

    /// Get compare value for a channel.
    pub fn get_compare_value(&self, channel: Channel) -> u32 {
        match T::BITS {
            TimerBits::Bits16 => self.regs_gp16().ccr(channel.index()).read().ccr() as u32,
            #[cfg(not(stm32l0))]
            TimerBits::Bits32 => self.regs_gp32_unchecked().ccr(channel.index()).read(),
        }
    }

    /// Generate a sequence of PWM waveform
    ///
    /// Note:
    /// you will need to provide corresponding TIMx_UP DMA channel to use this method.
    pub async fn waveform_up(&mut self, dma: Peri<'_, impl super::UpDma<T>>, channel: Channel, duty: &[u16]) {
        #[allow(clippy::let_unit_value)] // eg. stm32f334
        let req = dma.request();

        let original_update_dma_state = self.get_update_dma_state();

        if !original_update_dma_state {
            self.enable_update_dma(true);
        }

        self.waveform_helper(dma, req, channel, duty).await;

        // Since DMA is closed before timer update event trigger DMA is turn off,
        // this can almost always trigger a DMA FIFO error.
        //
        // optional TODO:
        // clean FEIF after disable UDE
        if !original_update_dma_state {
            self.enable_update_dma(false);
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
    pub async fn waveform_up_multi_channel(
        &mut self,
        dma: Peri<'_, impl super::UpDma<T>>,
        starting_channel: Channel,
        ending_channel: Channel,
        duty: &[u16],
    ) {
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

        let original_update_dma_state = self.get_update_dma_state();
        if !original_update_dma_state {
            self.enable_update_dma(true);
        }

        unsafe {
            #[cfg(not(any(bdma, gpdma)))]
            use crate::dma::{Burst, FifoThreshold};
            use crate::dma::{Transfer, TransferOptions};

            let dma_transfer_option = TransferOptions {
                #[cfg(not(any(bdma, gpdma)))]
                fifo_threshold: Some(FifoThreshold::Full),
                #[cfg(not(any(bdma, gpdma)))]
                mburst: Burst::Incr4,
                ..Default::default()
            };

            Transfer::new_write(
                dma,
                req,
                duty,
                self.regs_gp16().dmar().as_ptr() as *mut u16,
                dma_transfer_option,
            )
            .await
        };

        if !original_update_dma_state {
            self.enable_update_dma(false);
        }
    }

    /// Generate a sequence of PWM waveform
    pub async fn waveform<C: TimerChannel>(&mut self, dma: Peri<'_, impl super::Dma<T, C>>, duty: &[u16]) {
        use crate::pac::timer::vals::Ccds;

        #[allow(clippy::let_unit_value)] // eg. stm32f334
        let req = dma.request();

        let cc_channel = C::CHANNEL;

        let original_cc_dma_on_update = self.get_cc_dma_selection() == Ccds::ON_UPDATE;
        let original_cc_dma_enabled = self.get_cc_dma_enable_state(cc_channel);

        // redirect CC DMA request onto Update Event
        if !original_cc_dma_on_update {
            self.set_cc_dma_selection(Ccds::ON_UPDATE)
        }

        if !original_cc_dma_enabled {
            self.set_cc_dma_enable_state(cc_channel, true);
        }

        self.waveform_helper(dma, req, cc_channel, duty).await;

        // Since DMA is closed before timer Capture Compare Event trigger DMA is turn off,
        // this can almost always trigger a DMA FIFO error.
        //
        // optional TODO:
        // clean FEIF after disable UDE
        if !original_cc_dma_enabled {
            self.set_cc_dma_enable_state(cc_channel, false);
        }

        if !original_cc_dma_on_update {
            self.set_cc_dma_selection(Ccds::ON_COMPARE)
        }
    }

    async fn waveform_helper(
        &mut self,
        dma: Peri<'_, impl dma::Channel>,
        req: dma::Request,
        channel: Channel,
        duty: &[u16],
    ) {
        let original_duty_state = self.get_compare_value(channel);
        let original_enable_state = self.get_channel_enable_state(channel);

        if !original_enable_state {
            self.enable_channel(channel, true);
        }

        unsafe {
            #[cfg(not(any(bdma, gpdma)))]
            use crate::dma::{Burst, FifoThreshold};
            use crate::dma::{Transfer, TransferOptions};

            let dma_transfer_option = TransferOptions {
                #[cfg(not(any(bdma, gpdma)))]
                fifo_threshold: Some(FifoThreshold::Full),
                #[cfg(not(any(bdma, gpdma)))]
                mburst: Burst::Incr8,
                ..Default::default()
            };

            match self.bits() {
                TimerBits::Bits16 => {
                    Transfer::new_write(
                        dma,
                        req,
                        duty,
                        self.regs_1ch().ccr(channel.index()).as_ptr() as *mut u16,
                        dma_transfer_option,
                    )
                    .await
                }
                #[cfg(not(any(stm32l0)))]
                TimerBits::Bits32 => {
                    #[cfg(not(any(bdma, gpdma)))]
                    panic!("unsupported timer bits");

                    #[cfg(any(bdma, gpdma))]
                    Transfer::new_write(
                        dma,
                        req,
                        duty,
                        self.regs_1ch().ccr(channel.index()).as_ptr() as *mut u32,
                        dma_transfer_option,
                    )
                    .await
                }
            };
        };

        // restore output compare state
        if !original_enable_state {
            self.enable_channel(channel, false);
        }

        self.set_compare_value(channel, original_duty_state);
    }

    /// Get capture value for a channel.
    pub fn get_capture_value(&self, channel: Channel) -> u32 {
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
