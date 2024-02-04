//! Timers, PWM, quadrature decoder.

//! Timer inheritance

// sealed:
//
// Core -------------------------> 1CH -------------------------> 1CH_CMP
//   |                              |                              ^   |
//   +--> Basic_NoCr2 --> Basic     +--> 2CH --> GP16 --> GP32     |   +--> 2CH_CMP --> ADV
//            |             |             |      ^  |              |           ^         ^
//            |             |             +------|--|--------------|-----------+         |
//            |             +--------------------+  +--------------|-----------|---------+
//            |             |                                      |           |
//            |             +--------------------------------------|-----------+
//            +----------------------------------------------------+

//! BasicInstance --> CaptureCompare16bitInstance --+--> ComplementaryCaptureCompare16bitInstance  
//!                                                 |  
//!                                                 +--> CaptureCompare32bitInstance  
//!
//! mapping:
//!
//! BasicInstance --> Basic Timer  
//! CaptureCompare16bitInstance --> 1-channel Timer, 2-channel Timer, General Purpose 16-bit Timer  
//! CaptureCompare32bitInstance --> General Purpose 32-bit Timer  
//! ComplementaryCaptureCompare16bitInstance --> 1-channel with one complentary Timer, 2-channel with one complentary Timer, Advance Control Timer  

#[cfg(not(stm32l0))]
pub mod complementary_pwm;
pub mod qei;
pub mod simple_pwm;

use stm32_metapac::timer::vals;

use crate::interrupt;
use crate::rcc::RccPeripheral;
use crate::time::Hertz;

/// Low-level timer access.
#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

pub(crate) mod sealed {
    use super::*;

    /// Virtual Core 16-bit timer instance.  
    pub trait CoreInstance: RccPeripheral {
        /// Interrupt for this timer.
        type Interrupt: interrupt::typelevel::Interrupt;

        /// Get access to the virutal core 16bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs_core() -> crate::pac::timer::TimCore;

        /// Start the timer.
        fn start(&mut self) {
            Self::regs_core().cr1().modify(|r| r.set_cen(true));
        }

        /// Stop the timer.
        fn stop(&mut self) {
            Self::regs_core().cr1().modify(|r| r.set_cen(false));
        }

        /// Reset the counter value to 0
        fn reset(&mut self) {
            Self::regs_core().cnt().write(|r| r.set_cnt(0));
        }

        /// Set the frequency of how many times per second the timer counts up to the max value or down to 0.
        ///
        /// This means that in the default edge-aligned mode,
        /// the timer counter will wrap around at the same frequency as is being set.
        /// In center-aligned mode (which not all timers support), the wrap-around frequency is effectively halved
        /// because it needs to count up and down.
        fn set_frequency(&mut self, frequency: Hertz) {
            let f = frequency.0;
            let timer_f = Self::frequency().0;
            assert!(f > 0);
            let pclk_ticks_per_timer_period = timer_f / f;
            let psc: u16 = unwrap!(((pclk_ticks_per_timer_period - 1) / (1 << 16)).try_into());
            let divide_by = pclk_ticks_per_timer_period / (u32::from(psc) + 1);

            // the timer counts `0..=arr`, we want it to count `0..divide_by`
            let arr = unwrap!(u16::try_from(divide_by - 1));

            let regs = Self::regs_core();
            regs.psc().write(|r| r.set_psc(psc));
            regs.arr().write(|r| r.set_arr(arr));

            regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTERONLY));
            regs.egr().write(|r| r.set_ug(true));
            regs.cr1().modify(|r| r.set_urs(vals::Urs::ANYEVENT));
        }

        /// Clear update interrupt.
        ///
        /// Returns whether the update interrupt flag was set.
        fn clear_update_interrupt(&mut self) -> bool {
            let regs = Self::regs_core();
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
        fn enable_update_interrupt(&mut self, enable: bool) {
            Self::regs_core().dier().modify(|r| r.set_uie(enable));
        }

        /// Enable/disable autoreload preload.
        fn set_autoreload_preload(&mut self, enable: bool) {
            Self::regs_core().cr1().modify(|r| r.set_arpe(enable));
        }

        /// Get the timer frequency.
        fn get_frequency(&self) -> Hertz {
            let timer_f = Self::frequency();

            let regs = Self::regs_core();
            let arr = regs.arr().read().arr();
            let psc = regs.psc().read().psc();

            timer_f / arr / (psc + 1)
        }
    }

    /// Virtual Basic without CR2 16-bit timer instance.
    pub trait BasicNoCr2Instance: CoreInstance {
        /// Get access to the Baisc 16bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs_basic_no_cr2() -> crate::pac::timer::TimBasicNoCr2;

        /// Enable/disable the update dma.
        fn enable_update_dma(&mut self, enable: bool) {
            Self::regs_basic_no_cr2().dier().modify(|r| r.set_ude(enable));
        }

        /// Get the update dma enable/disable state.
        fn get_update_dma_state(&self) -> bool {
            Self::regs_basic_no_cr2().dier().read().ude()
        }
    }

    /// Basic 16-bit timer instance.
    pub trait BasicInstance: BasicNoCr2Instance {
        /// Get access to the Baisc 16bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs_basic() -> crate::pac::timer::TimBasic;
    }

    /// Gneral-purpose 1 channel 16-bit timer instance.
    pub trait GeneralPurpose1ChannelInstance: CoreInstance {
        /// Get access to the general purpose 1 channel 16bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs_1ch() -> crate::pac::timer::Tim1ch;

        /// Set clock divider.
        fn set_clock_division(&mut self, ckd: vals::Ckd) {
            Self::regs_1ch().cr1().modify(|r| r.set_ckd(ckd));
        }

        /// Get max compare value. This depends on the timer frequency and the clock frequency from RCC.
        fn get_max_compare_value(&self) -> u16 {
            Self::regs_1ch().arr().read().arr()
        }
    }

    /// Gneral-purpose 1 channel 16-bit  timer instance.
    pub trait GeneralPurpose2ChannelInstance: GeneralPurpose1ChannelInstance {
        /// Get access to the general purpose 2 channel 16bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs_2ch() -> crate::pac::timer::Tim2ch;
    }

    /// Gneral-purpose 16-bit timer instance.
    pub trait GeneralPurpose16bitInstance: BasicInstance + GeneralPurpose2ChannelInstance {
        /// Get access to the general purpose 16bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs_gp16() -> crate::pac::timer::TimGp16;

        /// Set counting mode.
        fn set_counting_mode(&mut self, mode: CountingMode) {
            let (cms, dir) = mode.into();

            let timer_enabled = Self::regs_core().cr1().read().cen();
            // Changing from edge aligned to center aligned (and vice versa) is not allowed while the timer is running.
            // Changing direction is discouraged while the timer is running.
            assert!(!timer_enabled);

            Self::regs_gp16().cr1().modify(|r| r.set_dir(dir));
            Self::regs_gp16().cr1().modify(|r| r.set_cms(cms))
        }

        /// Get counting mode.
        fn get_counting_mode(&self) -> CountingMode {
            let cr1 = Self::regs_gp16().cr1().read();
            (cr1.cms(), cr1.dir()).into()
        }

        /// Set input capture filter.
        fn set_input_capture_filter(&mut self, channel: Channel, icf: vals::FilterValue) {
            let raw_channel = channel.index();
            Self::regs_gp16()
                .ccmr_input(raw_channel / 2)
                .modify(|r| r.set_icf(raw_channel % 2, icf));
        }

        /// Clear input interrupt.
        fn clear_input_interrupt(&mut self, channel: Channel) {
            Self::regs_gp16().sr().modify(|r| r.set_ccif(channel.index(), false));
        }

        /// Enable input interrupt.
        fn enable_input_interrupt(&mut self, channel: Channel, enable: bool) {
            Self::regs_gp16().dier().modify(|r| r.set_ccie(channel.index(), enable));
        }

        /// Set input capture prescaler.
        fn set_input_capture_prescaler(&mut self, channel: Channel, factor: u8) {
            let raw_channel = channel.index();
            Self::regs_gp16()
                .ccmr_input(raw_channel / 2)
                .modify(|r| r.set_icpsc(raw_channel % 2, factor));
        }

        /// Set input TI selection.
        fn set_input_ti_selection(&mut self, channel: Channel, tisel: InputTISelection) {
            let raw_channel = channel.index();
            Self::regs_gp16()
                .ccmr_input(raw_channel / 2)
                .modify(|r| r.set_ccs(raw_channel % 2, tisel.into()));
        }

        /// Set input capture mode.
        fn set_input_capture_mode(&mut self, channel: Channel, mode: InputCaptureMode) {
            Self::regs_gp16().ccer().modify(|r| match mode {
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
        fn set_output_compare_mode(&mut self, channel: Channel, mode: OutputCompareMode) {
            let raw_channel: usize = channel.index();
            Self::regs_gp16()
                .ccmr_output(raw_channel / 2)
                .modify(|w| w.set_ocm(raw_channel % 2, mode.into()));
        }

        /// Set output polarity.
        fn set_output_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
            Self::regs_gp16()
                .ccer()
                .modify(|w| w.set_ccp(channel.index(), polarity.into()));
        }

        /// Enable/disable a channel.
        fn enable_channel(&mut self, channel: Channel, enable: bool) {
            Self::regs_gp16().ccer().modify(|w| w.set_cce(channel.index(), enable));
        }

        /// Get enable/disable state of a channel
        fn get_channel_enable_state(&self, channel: Channel) -> bool {
            Self::regs_gp16().ccer().read().cce(channel.index())
        }

        /// Set compare value for a channel.
        fn set_compare_value(&mut self, channel: Channel, value: u16) {
            Self::regs_gp16().ccr(channel.index()).modify(|w| w.set_ccr(value));
        }

        /// Get capture value for a channel.
        fn get_capture_value(&mut self, channel: Channel) -> u16 {
            Self::regs_gp16().ccr(channel.index()).read().ccr()
        }

        /// Get compare value for a channel.
        fn get_compare_value(&self, channel: Channel) -> u16 {
            Self::regs_gp16().ccr(channel.index()).read().ccr()
        }

        /// Set output compare preload.
        fn set_output_compare_preload(&mut self, channel: Channel, preload: bool) {
            let channel_index = channel.index();
            Self::regs_gp16()
                .ccmr_output(channel_index / 2)
                .modify(|w| w.set_ocpe(channel_index % 2, preload));
        }

        /// Get capture compare DMA selection
        fn get_cc_dma_selection(&self) -> super::vals::Ccds {
            Self::regs_gp16().cr2().read().ccds()
        }

        /// Set capture compare DMA selection
        fn set_cc_dma_selection(&mut self, ccds: super::vals::Ccds) {
            Self::regs_gp16().cr2().modify(|w| w.set_ccds(ccds))
        }

        /// Get capture compare DMA enable state
        fn get_cc_dma_enable_state(&self, channel: Channel) -> bool {
            Self::regs_gp16().dier().read().ccde(channel.index())
        }

        /// Set capture compare DMA enable state
        fn set_cc_dma_enable_state(&mut self, channel: Channel, ccde: bool) {
            Self::regs_gp16().dier().modify(|w| w.set_ccde(channel.index(), ccde))
        }
    }

    #[cfg(not(stm32l0))]
    /// Gneral-purpose 32-bit timer instance.
    pub trait GeneralPurpose32bitInstance: GeneralPurpose16bitInstance {
        /// Get access to the general purpose 32bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs_gp32() -> crate::pac::timer::TimGp32;

        /// Set timer frequency.
        fn set_frequency(&mut self, frequency: Hertz) {
            let f = frequency.0;
            assert!(f > 0);
            let timer_f = Self::frequency().0;
            let pclk_ticks_per_timer_period = (timer_f / f) as u64;
            let psc: u16 = unwrap!(((pclk_ticks_per_timer_period - 1) / (1 << 32)).try_into());
            let arr: u32 = unwrap!((pclk_ticks_per_timer_period / (psc as u64 + 1)).try_into());

            let regs = Self::regs_gp32();
            regs.psc().write(|r| r.set_psc(psc));
            regs.arr().write(|r| r.set_arr(arr));

            regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTERONLY));
            regs.egr().write(|r| r.set_ug(true));
            regs.cr1().modify(|r| r.set_urs(vals::Urs::ANYEVENT));
        }

        /// Get timer frequency.
        fn get_frequency(&self) -> Hertz {
            let timer_f = Self::frequency();

            let regs = Self::regs_gp32();
            let arr = regs.arr().read().arr();
            let psc = regs.psc().read().psc();

            timer_f / arr / (psc + 1)
        }

        /// Set comapre value for a channel.
        fn set_compare_value(&mut self, channel: Channel, value: u32) {
            Self::regs_gp32().ccr(channel.index()).modify(|w| w.set_ccr(value));
        }

        /// Get capture value for a channel.
        fn get_capture_value(&mut self, channel: Channel) -> u32 {
            Self::regs_gp32().ccr(channel.index()).read().ccr()
        }

        /// Get max compare value. This depends on the timer frequency and the clock frequency from RCC.
        fn get_max_compare_value(&self) -> u32 {
            Self::regs_gp32().arr().read().arr()
        }

        /// Get compare value for a channel.
        fn get_compare_value(&self, channel: Channel) -> u32 {
            Self::regs_gp32().ccr(channel.index()).read().ccr()
        }
    }

    #[cfg(not(stm32l0))]
    /// Gneral-purpose 1 channel with one complementary 16-bit timer instance.
    pub trait GeneralPurpose1ChannelComplementaryInstance: BasicNoCr2Instance + GeneralPurpose1ChannelInstance {
        /// Get access to the general purpose 1 channel with one complementary 16bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs_1ch_cmp() -> crate::pac::timer::Tim1chCmp;

        /// Set clock divider for the dead time.
        fn set_dead_time_clock_division(&mut self, value: vals::Ckd) {
            Self::regs_1ch_cmp().cr1().modify(|w| w.set_ckd(value));
        }

        /// Set dead time, as a fraction of the max duty value.
        fn set_dead_time_value(&mut self, value: u8) {
            Self::regs_1ch_cmp().bdtr().modify(|w| w.set_dtg(value));
        }

        /// Enable timer outputs.
        fn enable_outputs(&mut self) {
            Self::regs_1ch_cmp().bdtr().modify(|w| w.set_moe(true));
        }
    }

    #[cfg(not(stm32l0))]
    /// Gneral-purpose 2 channel with one complementary 16-bit timer instance.
    pub trait GeneralPurpose2ChannelComplementaryInstance:
        BasicInstance + GeneralPurpose2ChannelInstance + GeneralPurpose1ChannelComplementaryInstance
    {
        /// Get access to the general purpose 2 channel with one complementary 16bit timer registers.
        ///
        /// Note: This works even if the timer is more capable, because registers
        /// for the less capable timers are a subset. This allows writing a driver
        /// for a given set of capabilities, and having it transparently work with
        /// more capable timers.
        fn regs_2ch_cmp() -> crate::pac::timer::Tim2chCmp;
    }

    #[cfg(not(stm32l0))]
    /// Advanced control timer instance.
    pub trait AdvancedControlInstance:
        GeneralPurpose2ChannelComplementaryInstance + GeneralPurpose16bitInstance
    {
        /// Get access to the advanced timer registers.
        fn regs_advanced() -> crate::pac::timer::TimAdv;

        /// Set complementary output polarity.
        fn set_complementary_output_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
            Self::regs_advanced()
                .ccer()
                .modify(|w| w.set_ccnp(channel.index(), polarity.into()));
        }

        /// Enable/disable a complementary channel.
        fn enable_complementary_channel(&mut self, channel: Channel, enable: bool) {
            Self::regs_advanced()
                .ccer()
                .modify(|w| w.set_ccne(channel.index(), enable));
        }
    }
}

/// Timer channel.
#[derive(Clone, Copy)]
pub enum Channel {
    /// Channel 1.
    Ch1,
    /// Channel 2.
    Ch2,
    /// Channel 3.
    Ch3,
    /// Channel 4.
    Ch4,
}

impl Channel {
    /// Get the channel index (0..3)
    pub fn index(&self) -> usize {
        match self {
            Channel::Ch1 => 0,
            Channel::Ch2 => 1,
            Channel::Ch3 => 2,
            Channel::Ch4 => 3,
        }
    }
}

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

/// Basic 16-bit timer instance.
pub trait BasicInstance: sealed::BasicInstance + sealed::BasicNoCr2Instance + sealed::CoreInstance + 'static {}

// It's just a General-purpose 16-bit timer instance.
/// Capture Compare timer instance.
pub trait CaptureCompare16bitInstance:
    BasicInstance
    + sealed::GeneralPurpose2ChannelInstance
    + sealed::GeneralPurpose1ChannelInstance
    + sealed::GeneralPurpose16bitInstance
    + 'static
{
}

#[cfg(not(stm32l0))]
// It's just a General-purpose 32-bit timer instance.
/// Capture Compare 32-bit timer instance.
pub trait CaptureCompare32bitInstance:
    CaptureCompare16bitInstance + sealed::GeneralPurpose32bitInstance + 'static
{
}

#[cfg(not(stm32l0))]
// It's just a Advanced Control timer instance.
/// Complementary Capture Compare 32-bit timer instance.
pub trait ComplementaryCaptureCompare16bitInstance:
    CaptureCompare16bitInstance
    + sealed::GeneralPurpose1ChannelComplementaryInstance
    + sealed::GeneralPurpose2ChannelComplementaryInstance
    + sealed::AdvancedControlInstance
    + 'static
{
}

pin_trait!(Channel1Pin, CaptureCompare16bitInstance);
pin_trait!(Channel2Pin, CaptureCompare16bitInstance);
pin_trait!(Channel3Pin, CaptureCompare16bitInstance);
pin_trait!(Channel4Pin, CaptureCompare16bitInstance);
pin_trait!(ExternalTriggerPin, CaptureCompare16bitInstance);

cfg_if::cfg_if! {
    if #[cfg(not(stm32l0))] {
        pin_trait!(Channel1ComplementaryPin, ComplementaryCaptureCompare16bitInstance);
        pin_trait!(Channel2ComplementaryPin, ComplementaryCaptureCompare16bitInstance);
        pin_trait!(Channel3ComplementaryPin, ComplementaryCaptureCompare16bitInstance);
        pin_trait!(Channel4ComplementaryPin, ComplementaryCaptureCompare16bitInstance);

        pin_trait!(BreakInputPin, ComplementaryCaptureCompare16bitInstance);
        pin_trait!(BreakInput2Pin, ComplementaryCaptureCompare16bitInstance);

        pin_trait!(BreakInputComparator1Pin, ComplementaryCaptureCompare16bitInstance);
        pin_trait!(BreakInputComparator2Pin, ComplementaryCaptureCompare16bitInstance);

        pin_trait!(BreakInput2Comparator1Pin, ComplementaryCaptureCompare16bitInstance);
        pin_trait!(BreakInput2Comparator2Pin, ComplementaryCaptureCompare16bitInstance);
    }
}

#[allow(unused)]
macro_rules! impl_core_timer {
    ($inst:ident, $irq:ident) => {
        impl sealed::CoreInstance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;

            fn regs_core() -> crate::pac::timer::TimCore {
                unsafe { crate::pac::timer::TimCore::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_basic_no_cr2_timer {
    ($inst:ident) => {
        impl sealed::BasicNoCr2Instance for crate::peripherals::$inst {
            fn regs_basic_no_cr2() -> crate::pac::timer::TimBasicNoCr2 {
                unsafe { crate::pac::timer::TimBasicNoCr2::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_basic_timer {
    ($inst:ident) => {
        impl sealed::BasicInstance for crate::peripherals::$inst {
            fn regs_basic() -> crate::pac::timer::TimBasic {
                unsafe { crate::pac::timer::TimBasic::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_1ch_timer {
    ($inst:ident) => {
        impl sealed::GeneralPurpose1ChannelInstance for crate::peripherals::$inst {
            fn regs_1ch() -> crate::pac::timer::Tim1ch {
                unsafe { crate::pac::timer::Tim1ch::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_2ch_timer {
    ($inst:ident) => {
        impl sealed::GeneralPurpose2ChannelInstance for crate::peripherals::$inst {
            fn regs_2ch() -> crate::pac::timer::Tim2ch {
                unsafe { crate::pac::timer::Tim2ch::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_gp16_timer {
    ($inst:ident) => {
        impl sealed::GeneralPurpose16bitInstance for crate::peripherals::$inst {
            fn regs_gp16() -> crate::pac::timer::TimGp16 {
                unsafe { crate::pac::timer::TimGp16::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_gp32_timer {
    ($inst:ident) => {
        impl sealed::GeneralPurpose32bitInstance for crate::peripherals::$inst {
            fn regs_gp32() -> crate::pac::timer::TimGp32 {
                crate::pac::$inst
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_1ch_cmp_timer {
    ($inst:ident) => {
        impl sealed::GeneralPurpose1ChannelComplementaryInstance for crate::peripherals::$inst {
            fn regs_1ch_cmp() -> crate::pac::timer::Tim1chCmp {
                unsafe { crate::pac::timer::Tim1chCmp::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_2ch_cmp_timer {
    ($inst:ident) => {
        impl sealed::GeneralPurpose2ChannelComplementaryInstance for crate::peripherals::$inst {
            fn regs_2ch_cmp() -> crate::pac::timer::Tim2chCmp {
                unsafe { crate::pac::timer::Tim2chCmp::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_adv_timer {
    ($inst:ident) => {
        impl sealed::AdvancedControlInstance for crate::peripherals::$inst {
            fn regs_advanced() -> crate::pac::timer::TimAdv {
                unsafe { crate::pac::timer::TimAdv::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }
    };
}

foreach_interrupt! {

    ($inst:ident, timer, TIM_BASIC, UP, $irq:ident) => {
        impl_core_timer!($inst, $irq);
        impl_basic_no_cr2_timer!($inst);
        impl_basic_timer!($inst);
        impl BasicInstance for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_1CH, UP, $irq:ident) => {
        impl_core_timer!($inst, $irq);
        impl_basic_no_cr2_timer!($inst);
        impl_basic_timer!($inst);
        impl_1ch_timer!($inst);
        impl_2ch_timer!($inst);
        impl_gp16_timer!($inst);
        impl BasicInstance for crate::peripherals::$inst {}
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {}
    };


    ($inst:ident, timer, TIM_2CH, UP, $irq:ident) => {
        impl_core_timer!($inst, $irq);
        impl_basic_no_cr2_timer!($inst);
        impl_basic_timer!($inst);
        impl_1ch_timer!($inst);
        impl_2ch_timer!($inst);
        impl_gp16_timer!($inst);
        impl BasicInstance for crate::peripherals::$inst {}
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_GP16, UP, $irq:ident) => {
        impl_core_timer!($inst, $irq);
        impl_basic_no_cr2_timer!($inst);
        impl_basic_timer!($inst);
        impl_1ch_timer!($inst);
        impl_2ch_timer!($inst);
        impl_gp16_timer!($inst);
        impl BasicInstance for crate::peripherals::$inst {}
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_GP32, UP, $irq:ident) => {
        impl_core_timer!($inst, $irq);
        impl_basic_no_cr2_timer!($inst);
        impl_basic_timer!($inst);
        impl_1ch_timer!($inst);
        impl_2ch_timer!($inst);
        impl_gp16_timer!($inst);
        impl_gp32_timer!($inst);
        impl BasicInstance for crate::peripherals::$inst {}
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {}
        impl CaptureCompare32bitInstance for crate::peripherals::$inst {}
    };

    ($inst:ident, timer, TIM_1CH_CMP, UP, $irq:ident) => {
        impl_core_timer!($inst, $irq);
        impl_basic_no_cr2_timer!($inst);
        impl_basic_timer!($inst);
        impl_1ch_timer!($inst);
        impl_2ch_timer!($inst);
        impl_gp16_timer!($inst);
        impl_1ch_cmp_timer!($inst);
        impl_2ch_cmp_timer!($inst);
        impl_adv_timer!($inst);
        impl BasicInstance for crate::peripherals::$inst {}
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {}
        impl ComplementaryCaptureCompare16bitInstance for crate::peripherals::$inst {}
    };


    ($inst:ident, timer, TIM_2CH_CMP, UP, $irq:ident) => {
        impl_core_timer!($inst, $irq);
        impl_basic_no_cr2_timer!($inst);
        impl_basic_timer!($inst);
        impl_1ch_timer!($inst);
        impl_2ch_timer!($inst);
        impl_gp16_timer!($inst);
        impl_1ch_cmp_timer!($inst);
        impl_2ch_cmp_timer!($inst);
        impl_adv_timer!($inst);
        impl BasicInstance for crate::peripherals::$inst {}
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {}
        impl ComplementaryCaptureCompare16bitInstance for crate::peripherals::$inst {}
    };


    ($inst:ident, timer, TIM_ADV, UP, $irq:ident) => {
        impl_core_timer!($inst, $irq);
        impl_basic_no_cr2_timer!($inst);
        impl_basic_timer!($inst);
        impl_1ch_timer!($inst);
        impl_2ch_timer!($inst);
        impl_gp16_timer!($inst);
        impl_1ch_cmp_timer!($inst);
        impl_2ch_cmp_timer!($inst);
        impl_adv_timer!($inst);
        impl BasicInstance for crate::peripherals::$inst {}
        impl CaptureCompare16bitInstance for crate::peripherals::$inst {}
        impl ComplementaryCaptureCompare16bitInstance for crate::peripherals::$inst {}
    };
}

// Update Event trigger DMA for every timer
dma_trait!(UpDma, BasicInstance);

dma_trait!(Ch1Dma, CaptureCompare16bitInstance);
dma_trait!(Ch2Dma, CaptureCompare16bitInstance);
dma_trait!(Ch3Dma, CaptureCompare16bitInstance);
dma_trait!(Ch4Dma, CaptureCompare16bitInstance);
