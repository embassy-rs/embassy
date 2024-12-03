//! Low-level timer driver.
//!
//! This is an unopinionated, very low-level driver for all STM32 timers. It allows direct register
//! manipulation with the `regs_*()` methods, and has utility functions that are thin wrappers
//! over the registers.
//!
//! The available functionality depends on the timer type.

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
// Re-export useful enums
pub use stm32_metapac::timer::vals::{FilterValue, Sms as SlaveMode, Ts as TriggerSource};

use super::*;
use crate::pac::timer::vals;
use crate::rcc;
use crate::time::Hertz;

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
            CountingMode::EdgeAlignedUp => (vals::Cms::EDGEALIGNED, vals::Dir::UP),
            CountingMode::EdgeAlignedDown => (vals::Cms::EDGEALIGNED, vals::Dir::DOWN),
            CountingMode::CenterAlignedDownInterrupts => (vals::Cms::CENTERALIGNED1, vals::Dir::UP),
            CountingMode::CenterAlignedUpInterrupts => (vals::Cms::CENTERALIGNED2, vals::Dir::UP),
            CountingMode::CenterAlignedBothInterrupts => (vals::Cms::CENTERALIGNED3, vals::Dir::UP),
        }
    }
}

impl From<(vals::Cms, vals::Dir)> for CountingMode {
    fn from(value: (vals::Cms, vals::Dir)) -> Self {
        match value {
            (vals::Cms::EDGEALIGNED, vals::Dir::UP) => CountingMode::EdgeAlignedUp,
            (vals::Cms::EDGEALIGNED, vals::Dir::DOWN) => CountingMode::EdgeAlignedDown,
            (vals::Cms::CENTERALIGNED1, _) => CountingMode::CenterAlignedDownInterrupts,
            (vals::Cms::CENTERALIGNED2, _) => CountingMode::CenterAlignedUpInterrupts,
            (vals::Cms::CENTERALIGNED3, _) => CountingMode::CenterAlignedBothInterrupts,
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
    // TODO: there's more modes here depending on the chip family.
}

impl From<OutputCompareMode> for stm32_metapac::timer::vals::Ocm {
    fn from(mode: OutputCompareMode) -> Self {
        match mode {
            OutputCompareMode::Frozen => stm32_metapac::timer::vals::Ocm::FROZEN,
            OutputCompareMode::ActiveOnMatch => stm32_metapac::timer::vals::Ocm::ACTIVEONMATCH,
            OutputCompareMode::InactiveOnMatch => stm32_metapac::timer::vals::Ocm::INACTIVEONMATCH,
            OutputCompareMode::Toggle => stm32_metapac::timer::vals::Ocm::TOGGLE,
            OutputCompareMode::ForceInactive => stm32_metapac::timer::vals::Ocm::FORCEINACTIVE,
            OutputCompareMode::ForceActive => stm32_metapac::timer::vals::Ocm::FORCEACTIVE,
            OutputCompareMode::PwmMode1 => stm32_metapac::timer::vals::Ocm::PWMMODE1,
            OutputCompareMode::PwmMode2 => stm32_metapac::timer::vals::Ocm::PWMMODE2,
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
    tim: PeripheralRef<'d, T>,
}

impl<'d, T: CoreInstance> Drop for Timer<'d, T> {
    fn drop(&mut self) {
        rcc::disable::<T>();
    }
}

impl<'d, T: CoreInstance> Timer<'d, T> {
    /// Create a new timer driver.
    pub fn new(tim: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(tim);

        rcc::enable_and_reset::<T>();

        Self { tim }
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

    /// Stop the timer.
    pub fn stop(&self) {
        self.regs_core().cr1().modify(|r| r.set_cen(false));
    }

    /// Reset the counter value to 0
    pub fn reset(&self) {
        self.regs_core().cnt().write(|r| r.set_cnt(0));
    }

    /// Set the frequency of how many times per second the timer counts up to the max value or down to 0.
    ///
    /// This means that in the default edge-aligned mode,
    /// the timer counter will wrap around at the same frequency as is being set.
    /// In center-aligned mode (which not all timers support), the wrap-around frequency is effectively halved
    /// because it needs to count up and down.
    pub fn set_frequency(&self, frequency: Hertz) {
        let f = frequency.0;
        assert!(f > 0);
        let timer_f = T::frequency().0;

        match T::BITS {
            TimerBits::Bits16 => {
                let pclk_ticks_per_timer_period = timer_f / f;
                let psc: u16 = unwrap!(((pclk_ticks_per_timer_period - 1) / (1 << 16)).try_into());
                let divide_by = pclk_ticks_per_timer_period / (u32::from(psc) + 1);

                // the timer counts `0..=arr`, we want it to count `0..divide_by`
                let arr = unwrap!(u16::try_from(divide_by - 1));

                let regs = self.regs_core();
                regs.psc().write_value(psc);
                regs.arr().write(|r| r.set_arr(arr));

                regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTERONLY));
                regs.egr().write(|r| r.set_ug(true));
                regs.cr1().modify(|r| r.set_urs(vals::Urs::ANYEVENT));
            }
            #[cfg(not(stm32l0))]
            TimerBits::Bits32 => {
                let pclk_ticks_per_timer_period = (timer_f / f) as u64;
                let psc: u16 = unwrap!(((pclk_ticks_per_timer_period - 1) / (1 << 32)).try_into());
                let divide_by = pclk_ticks_per_timer_period / (u64::from(psc) + 1);

                // the timer counts `0..=arr`, we want it to count `0..divide_by`
                let arr: u32 = unwrap!(u32::try_from(divide_by - 1));

                let regs = self.regs_gp32_unchecked();
                regs.psc().write_value(psc);
                regs.arr().write_value(arr);

                regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTERONLY));
                regs.egr().write(|r| r.set_ug(true));
                regs.cr1().modify(|r| r.set_urs(vals::Urs::ANYEVENT));
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

    /// Set state of MOE-bit in BDTR register to en-/disable output
    pub fn set_moe(&self, enable: bool) {
        self.regs_1ch_cmp().bdtr().modify(|w| w.set_moe(enable));
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
}
