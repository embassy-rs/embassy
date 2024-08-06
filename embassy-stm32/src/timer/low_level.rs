//! Low-level timer driver.
//!
//! This is an unopinionated, low-level driver for all STM32 timers. It wraps [`RawTimer`] and adds
//! utility functions that are thin wrappers over the registers.
//!
//! The available functionality depends on the timer type.

// Re-export useful enums
pub use stm32_metapac::timer::vals::{FilterValue, Sms as SlaveMode, Ts as TriggerSource};

use super::raw::RawTimer;
use super::{Channel, IsCcDmaTim, IsCoreTim, IsGeneral1ChTim, IsGeneral2ChTim, IsGeneral4ChTim, IsUpDmaTim};
#[cfg(not(timer_l0))]
use super::{IsAdvanced1ChTim, IsAdvanced4ChTim};
use crate::pac::timer::vals;
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
///
/// The low-level driver is just a wrapper around [`RawTimer`] that provides some convenience
/// methods which abstract away raw register access.
pub struct Timer<'d, Tim> {
    /// The raw timer driver that is wrapped by this driver.
    pub raw: RawTimer<'d, Tim>,
}

impl<'d, Tim: IsCoreTim> Timer<'d, Tim> {
    /// Create a new timer driver from a raw timer driver.
    pub fn new(raw: RawTimer<'d, Tim>) -> Self {
        Self { raw }
    }

    /// Start the timer.
    pub fn start(&self) {
        self.raw.cr1_core().modify(|r| r.set_cen(true));
    }

    /// Stop the timer.
    pub fn stop(&self) {
        self.raw.cr1_core().modify(|r| r.set_cen(false));
    }

    /// Enable timer outputs.
    ///
    /// Calling this is necessary to enable outputs on advanced timers. See
    /// [`RawTimer::enable_outputs()`] for details.
    pub fn enable_outputs(&self) {
        self.raw.enable_outputs()
    }

    /// Reset the counter value to 0
    pub fn reset(&self) {
        self.raw.cnt().write(|r| r.set_cnt(0));
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
        let timer_f = self.raw.clock_frequency().0;

        #[cfg(not(timer_l0))]
        if let Some(regs_32) = self.raw.try_get_32bit_regs() {
            let pclk_ticks_per_timer_period = (timer_f / f) as u64;
            let psc: u16 = unwrap!(((pclk_ticks_per_timer_period - 1) / (1 << 32)).try_into());
            let divide_by = pclk_ticks_per_timer_period / (u64::from(psc) + 1);

            // the timer counts `0..=arr`, we want it to count `0..divide_by`
            let arr: u32 = unwrap!(u32::try_from(divide_by - 1));

            regs_32.psc().write_value(psc);
            regs_32.arr().write_value(arr);

            regs_32.cr1().modify(|r| r.set_urs(vals::Urs::COUNTERONLY));
            regs_32.egr().write(|r| r.set_ug(true));
            regs_32.cr1().modify(|r| r.set_urs(vals::Urs::ANYEVENT));
            return;
        }

        let pclk_ticks_per_timer_period = timer_f / f;
        let psc: u16 = unwrap!(((pclk_ticks_per_timer_period - 1) / (1 << 16)).try_into());
        let divide_by = pclk_ticks_per_timer_period / (u32::from(psc) + 1);

        // the timer counts `0..=arr`, we want it to count `0..divide_by`
        let arr = unwrap!(u16::try_from(divide_by - 1));

        self.raw.psc().write_value(psc);
        self.raw.arr().write(|r| r.set_arr(arr));

        self.raw.cr1_core().modify(|r| r.set_urs(vals::Urs::COUNTERONLY));
        self.raw.egr_core().write(|r| r.set_ug(true));
        self.raw.cr1_core().modify(|r| r.set_urs(vals::Urs::ANYEVENT));
    }

    /// Get max compare value. This depends on the timer frequency and the clock frequency from RCC.
    pub fn max_compare_value(&self) -> u32 {
        #[cfg(not(timer_l0))]
        if let Some(regs_32) = self.raw.try_get_32bit_regs() {
            return regs_32.arr().read();
        }
        self.raw.arr().read().arr() as u32
    }

    /// Set tick frequency.
    pub fn set_tick_frequency(&self, freq: Hertz) {
        let f = freq;
        assert!(f.0 > 0);
        let timer_f = self.raw.clock_frequency();

        let pclk_ticks_per_timer_period = timer_f / f;
        let psc: u16 = unwrap!((pclk_ticks_per_timer_period - 1).try_into());
        self.raw.psc().write_value(psc);

        // Generate an Update Request
        self.raw.egr_core().write(|r| r.set_ug(true));
    }

    /// Clear update interrupt.
    ///
    /// Returns whether the update interrupt flag was set.
    pub fn clear_update_interrupt(&self) -> bool {
        let sr = self.raw.sr_core().read();
        if sr.uif() {
            self.raw.sr_core().modify(|r| {
                r.set_uif(false);
            });
            true
        } else {
            false
        }
    }

    /// Enable/disable the update interrupt.
    pub fn enable_update_interrupt(&self, enable: bool) {
        self.raw.dier_core().modify(|r| r.set_uie(enable));
    }

    /// Enable/disable autoreload preload.
    pub fn set_autoreload_preload(&self, enable: bool) {
        self.raw.cr1_core().modify(|r| r.set_arpe(enable));
    }

    /// Get the timer frequency.
    pub fn frequency(&self) -> Hertz {
        let timer_f = self.raw.clock_frequency();

        #[cfg(not(timer_l0))]
        if let Some(regs_32) = self.raw.try_get_32bit_regs() {
            let arr = regs_32.arr().read();
            let psc = regs_32.psc().read();
            return timer_f / arr / (psc + 1);
        }

        let arr = self.raw.arr().read().arr();
        let psc = self.raw.psc().read();
        timer_f / arr / (psc + 1)
    }

    /// Get the internal clock frequency of the timer (before prescaler is applied).
    pub fn clock_frequency(&self) -> Hertz {
        self.raw.clock_frequency()
    }
}

impl<'d, Tim: IsUpDmaTim> Timer<'d, Tim> {
    /// Enable/disable the update dma.
    pub fn enable_update_dma(&self, enable: bool) {
        self.raw.dier_updma().modify(|r| r.set_ude(enable));
    }

    /// Get the update dma enable/disable state.
    pub fn update_dma_state(&self) -> bool {
        self.raw.dier_updma().read().ude()
    }
}

impl<'d, Tim: IsGeneral1ChTim> Timer<'d, Tim> {
    /// Convert a [`Channel`] enum to index, checking that it corresponds to a valid channel for
    /// this timer.
    fn to_index(&self, channel: Channel) -> usize {
        let i = channel.index();
        assert!(i < self.raw.channel_count().to_usize());
        i
    }

    /// Set clock divider.
    pub fn set_clock_division(&self, ckd: vals::Ckd) {
        self.raw.cr1_1ch().modify(|r| r.set_ckd(ckd));
    }

    /// Set input capture filter.
    pub fn set_input_capture_filter(&self, channel: Channel, icf: vals::FilterValue) {
        let i = self.to_index(channel);
        self.raw.ccmr_input_1ch(i / 2).modify(|r| r.set_icf(i % 2, icf));
    }

    /// Clear input interrupt.
    pub fn clear_input_interrupt(&self, channel: Channel) {
        let i = self.to_index(channel);
        self.raw.sr_1ch().modify(|r| r.set_ccif(i, false));
    }

    /// Get input interrupt.
    pub fn get_input_interrupt(&self, channel: Channel) -> bool {
        let i = self.to_index(channel);
        self.raw.sr_1ch().read().ccif(i)
    }

    /// Enable input interrupt.
    pub fn enable_input_interrupt(&self, channel: Channel, enable: bool) {
        let i = self.to_index(channel);
        self.raw.dier_1ch().modify(|r| r.set_ccie(i, enable));
    }

    /// Set input capture prescaler.
    pub fn set_input_capture_prescaler(&self, channel: Channel, factor: u8) {
        let i = self.to_index(channel);
        self.raw.ccmr_input_1ch(i / 2).modify(|r| r.set_icpsc(i % 2, factor));
    }

    /// Set input TI selection.
    pub fn set_input_ti_selection(&self, channel: Channel, tisel: InputTISelection) {
        let i = self.to_index(channel);
        self.raw
            .ccmr_input_1ch(i / 2)
            .modify(|r| r.set_ccs(i % 2, tisel.into()));
    }

    /// Set input capture mode.
    pub fn set_input_capture_mode(&self, channel: Channel, mode: InputCaptureMode) {
        let i = self.to_index(channel);
        self.raw.ccer_1ch().modify(|r| match mode {
            InputCaptureMode::Rising => {
                r.set_ccnp(i, false);
                r.set_ccp(i, false);
            }
            InputCaptureMode::Falling => {
                r.set_ccnp(i, false);
                r.set_ccp(i, true);
            }
            InputCaptureMode::BothEdges => {
                r.set_ccnp(i, true);
                r.set_ccp(i, true);
            }
        });
    }

    /// Set output compare mode.
    pub fn set_output_compare_mode(&self, channel: Channel, mode: OutputCompareMode) {
        let i = self.to_index(channel);
        self.raw
            .ccmr_output_1ch(i / 2)
            .modify(|w| w.set_ocm(i % 2, mode.into()));
    }

    /// Set output polarity.
    pub fn set_output_polarity(&self, channel: Channel, polarity: OutputPolarity) {
        let i = self.to_index(channel);
        self.raw.ccer_1ch().modify(|w| w.set_ccp(i, polarity.into()));
    }

    /// Enable/disable a channel.
    pub fn enable_channel(&self, channel: Channel, enable: bool) {
        let i = self.to_index(channel);
        self.raw.ccer_1ch().modify(|w| w.set_cce(i, enable));
    }

    /// Get enable/disable state of a channel
    pub fn channel_enable_state(&self, channel: Channel) -> bool {
        let i = self.to_index(channel);
        self.raw.ccer_1ch().read().cce(i)
    }

    /// Set compare value for a channel.
    pub fn set_compare_value(&self, channel: Channel, value: u32) {
        let i = self.to_index(channel);

        #[cfg(not(timer_l0))]
        if let Some(regs_32) = self.raw.try_get_32bit_regs() {
            regs_32.ccr(i).write_value(value);
            return;
        }

        let value = unwrap!(u16::try_from(value));
        self.raw.ccr(i).modify(|w| w.set_ccr(value));
    }

    /// Get compare value for a channel.
    pub fn compare_value(&self, channel: Channel) -> u32 {
        let i = self.to_index(channel);

        #[cfg(not(timer_l0))]
        if let Some(regs_32) = self.raw.try_get_32bit_regs() {
            return regs_32.ccr(i).read();
        }

        self.raw.ccr(i).read().ccr() as u32
    }

    /// Get capture value for a channel.
    pub fn capture_value(&self, channel: Channel) -> u32 {
        self.compare_value(channel)
    }

    /// Set output compare preload.
    pub fn set_output_compare_preload(&self, channel: Channel, preload: bool) {
        let i = self.to_index(channel);
        self.raw.ccmr_output_1ch(i / 2).modify(|w| w.set_ocpe(i % 2, preload));
    }

    /// Set divider for the dead time and sampling clock.
    pub fn set_dead_time_clock_division(&self, value: vals::Ckd) {
        self.raw.cr1_1ch().modify(|w| w.set_ckd(value));
    }
}

impl<'d, Tim: IsGeneral2ChTim> Timer<'d, Tim> {
    /// Set Timer Slave Mode
    pub fn set_slave_mode(&self, sms: SlaveMode) {
        self.raw.smcr_2ch().modify(|r| r.set_sms(sms));
    }

    /// Set Timer Trigger Source
    pub fn set_trigger_source(&self, ts: TriggerSource) {
        self.raw.smcr_2ch().modify(|r| r.set_ts(ts));
    }
}

impl<'d, Tim: IsCcDmaTim> Timer<'d, Tim> {
    /// Get capture compare DMA selection
    pub fn cc_dma_selection(&self) -> vals::Ccds {
        self.raw.cr2_ccdma().read().ccds()
    }

    /// Set capture compare DMA selection
    pub fn set_cc_dma_selection(&self, ccds: vals::Ccds) {
        self.raw.cr2_ccdma().modify(|w| w.set_ccds(ccds))
    }

    /// Get capture compare DMA enable state
    pub fn cc_dma_enable_state(&self, channel: Channel) -> bool {
        let i = self.to_index(channel);
        self.raw.dier_ccdma().read().ccde(i)
    }

    /// Set capture compare DMA enable state
    pub fn set_cc_dma_enable_state(&self, channel: Channel, ccde: bool) {
        let i = self.to_index(channel);
        self.raw.dier_ccdma().modify(|w| w.set_ccde(i, ccde))
    }
}

// TODO: on `timer_l0`, these methods are available also for `IsGeneral2ChTim`
impl<'d, Tim: IsGeneral4ChTim> Timer<'d, Tim> {
    /// Set counting mode.
    ///
    /// You can only call this method if the timer is not enabled, otherwise we panic.
    pub fn set_counting_mode(&self, mode: CountingMode) {
        let (cms, dir) = mode.into();

        let timer_enabled = self.raw.cr1_core().read().cen();
        // Changing from edge aligned to center aligned (and vice versa) is not allowed while the timer is running.
        // Changing direction is discouraged while the timer is running.
        assert!(!timer_enabled);

        self.raw.cr1_4ch().modify(|r| r.set_dir(dir));
        self.raw.cr1_4ch().modify(|r| r.set_cms(cms))
    }

    /// Get counting mode.
    pub fn counting_mode(&self) -> CountingMode {
        let cr1 = self.raw.cr1_4ch().read();
        (cr1.cms(), cr1.dir()).into()
    }
}

#[cfg(not(timer_l0))]
impl<'d, Tim: IsAdvanced1ChTim> Timer<'d, Tim> {
    /// Set dead time, as a fraction of the max duty value.
    pub fn set_dead_time_value(&self, value: u8) {
        self.raw.bdtr().modify(|w| w.set_dtg(value));
    }

    /// Set state of MOE-bit in BDTR register to en-/disable output
    pub fn set_moe(&self, enable: bool) {
        self.raw.bdtr().modify(|w| w.set_moe(enable));
    }
}

#[cfg(not(timer_l0))]
impl<'d, Tim: IsAdvanced4ChTim> Timer<'d, Tim> {
    /// Set complementary output polarity.
    pub fn set_complementary_output_polarity(&self, channel: Channel, polarity: OutputPolarity) {
        let i = self.to_index(channel);
        self.raw.ccer_adv1ch().modify(|w| w.set_ccnp(i, polarity.into()));
    }

    /// Enable/disable a complementary channel.
    pub fn enable_complementary_channel(&self, channel: Channel, enable: bool) {
        let i = self.to_index(channel);
        self.raw.ccer_adv1ch().modify(|w| w.set_ccne(i, enable));
    }
}
