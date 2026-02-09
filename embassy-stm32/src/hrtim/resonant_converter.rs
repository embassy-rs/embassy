//! Variable-frequency resonant converter driver.
//!
//! This implementation of a resonsant converter is appropriate for a half or full bridge,
//! but does not include secondary rectification, which is appropriate for applications
//! with a low-voltage on the secondary side.
use stm32_hrtim::control::HrPwmControl;
pub use stm32_hrtim::deadtime::DeadtimeConfig;
use stm32_hrtim::output::{HrOutput, Output1Pin, Output2Pin};
use stm32_hrtim::timer::{HrTim, HrTimer};
use stm32_hrtim::{HrParts, HrPwmAdvExt, HrPwmBuilder, HrtimPrescaler, InterleavedMode, PreloadSource, capture};

use crate::hrtim::HrPwmBuilderExt;
use crate::peripherals::HRTIM1;
use crate::rcc::SealedRccPeripheral;
use crate::time::Hertz;

/// Variable-frequency resonant converter driver.
pub struct ResonantConverter<TIM, PSCL> {
    timer: HrParts<TIM, PSCL>,
    min_period: u16,
    max_period: u16,
}

impl<TIM: HrPwmAdvExt, PSCL: HrtimPrescaler> ResonantConverter<TIM, PSCL>
where
    TIM: stm32_hrtim::timer::InstanceX + HrPwmAdvExt<PreloadSource = PreloadSource>,
    HrTim<TIM, PSCL, capture::NoDma, capture::NoDma>: HrTimer,
{
    /// Create a new variable-frequency resonant converter driver.
    pub fn new<P1, P2>(
        timer: TIM,
        pin1: P1,
        pin2: P2,
        prescaler: PSCL,
        min_frequency: Hertz,
        max_frequency: Hertz,
        hr_control: &mut HrPwmControl,
        deadtime_cfg: DeadtimeConfig,
    ) -> Self
    where
        P1: Output1Pin<TIM>,
        P2: Output2Pin<TIM>,
        HrPwmBuilder<TIM, PSCL, PreloadSource, P1, P2>: HrPwmBuilderExt<TIM, PSCL, P1, P2>,
    {
        let f_min = min_frequency.0;

        let timer_f = HRTIM1::frequency().0;

        let psc_min = (timer_f / f_min) / (u16::MAX as u32 / 32);
        let psc = PSCL::VALUE as u32;
        assert!(
            psc >= psc_min,
            "Prescaler set too low to be able to reach target frequency"
        );

        let timer_f = 32 * (timer_f / psc);
        let max_period: u16 = (timer_f / f_min) as u16;

        let mut timer = timer
            .pwm_advanced(pin1, pin2)
            .prescaler(prescaler)
            .period(max_period)
            .preload(PreloadSource::OnRepetitionUpdate)
            .interleaved_mode(InterleavedMode::Dual)
            // Dead-time generator can be used in this case because the primary fets
            // of a resonant converter are always complementary
            .deadtime(deadtime_cfg)
            .finalize(hr_control);

        // Set output 1 to active on a period event
        timer.out1.enable_set_event(&timer.timer);

        // Set output 1 to inactive on a compare 1 event
        timer.out1.enable_rst_event(&timer.cr1);

        timer.out1.enable();
        timer.out2.enable();

        let min_period = max_period * (min_frequency.0 / max_frequency.0) as u16;

        Self {
            timer,
            min_period: min_period,
            max_period: max_period,
        }
    }

    /// Set the timer period.
    pub fn set_period(&mut self, period: u16) {
        assert!(period < self.max_period);
        assert!(period > self.min_period);

        self.timer.timer.set_period(period);
    }

    /// Get the minimum compare value of a duty cycle
    pub fn get_min_period(&mut self) -> u16 {
        self.min_period
    }

    /// Get the maximum compare value of a duty cycle
    pub fn get_max_period(&mut self) -> u16 {
        self.max_period
    }
}
