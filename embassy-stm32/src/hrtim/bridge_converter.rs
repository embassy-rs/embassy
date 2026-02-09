//! Fixed-frequency bridge converter driver.
//!
//! Our implementation of the bridge converter uses a single channel and three compare registers,
//! allowing implementation of a synchronous buck or boost converter in continuous or discontinuous
//! conduction mode.
//!
//! It is important to remember that in synchronous topologies, energy can flow in reverse during
//! light loading conditions, and that the low-side switch must be active for a short time to drive
//! a bootstrapped high-side switch.
use stm32_hrtim::compare_register::HrCompareRegister;
use stm32_hrtim::control::{HrPwmControl, HrPwmCtrl};
use stm32_hrtim::output::{HrOutput, Output1Pin, Output2Pin};
use stm32_hrtim::timer::{HrTim, HrTimer};
use stm32_hrtim::{HrParts, HrPwmAdvExt, HrPwmBuilder, HrtimPrescaler, PreloadSource, capture};

use crate::hrtim::HrPwmBuilderExt;
use crate::peripherals::HRTIM1;
use crate::rcc::SealedRccPeripheral;
use crate::time::Hertz;

/// Fixed-frequency bridge converter driver.
pub struct BridgeConverter<TIM, PSCL> {
    timer: HrParts<TIM, PSCL>,
    dead_time: u16,
    primary_duty: u16,
    min_secondary_duty: u16,
    max_secondary_duty: u16,
}

impl<TIM: HrPwmAdvExt, PSCL: HrtimPrescaler> BridgeConverter<TIM, PSCL>
where
    TIM: stm32_hrtim::timer::InstanceX + HrPwmAdvExt<PreloadSource = PreloadSource>,
    HrTim<TIM, PSCL, capture::NoDma, capture::NoDma>: HrTimer,
{
    /// Create a new HRTIM bridge converter driver.
    pub fn new<P1, P2>(
        timer: TIM,
        pin1: P1,
        pin2: P2,
        frequency: Hertz,
        prescaler: PSCL,
        hr_control: &mut HrPwmControl,
    ) -> Self
    where
        P1: Output1Pin<TIM>,
        P2: Output2Pin<TIM>,
        HrPwmBuilder<TIM, PSCL, PreloadSource, P1, P2>: HrPwmBuilderExt<TIM, PSCL, P1, P2>,
    {
        let f = frequency.0;

        let timer_f = HRTIM1::frequency().0;

        let psc_min = (timer_f / f) / (u16::MAX as u32 / 32);
        let psc = PSCL::VALUE as u32;
        assert!(
            psc >= psc_min,
            "Prescaler set too low to be able to reach target frequency"
        );

        let timer_f = 32 * (timer_f / psc);
        let per: u16 = (timer_f / f) as u16;

        let mut timer = timer
            .pwm_advanced(pin1, pin2)
            .prescaler(prescaler)
            .period(per)
            .preload(PreloadSource::OnRepetitionUpdate)
            .finalize(hr_control);

        // The dead-time generation unit cannot be used because it forces the other output
        // to be completely complementary to the first output, which restricts certain waveforms
        // Therefore, software-implemented dead time must be used when setting the duty cycles

        // Set output 1 to active on a period event
        timer.out1.enable_set_event(&timer.timer);

        // Set output 1 to inactive on a compare 1 event
        timer.out1.enable_rst_event(&timer.cr1);

        // Set output 2 to active on a compare 2 event
        timer.out2.enable_set_event(&timer.cr2);

        // Set output 2 to inactive on a compare 3 event
        timer.out2.enable_rst_event(&timer.cr3);

        Self {
            timer,
            dead_time: 0,
            primary_duty: 0,
            min_secondary_duty: 0,
            max_secondary_duty: 0,
        }
    }

    /// Start HRTIM.
    pub fn start(&mut self, hr_control: &mut HrPwmCtrl) {
        self.timer.timer.start(hr_control);
    }

    /// Stop HRTIM.
    pub fn stop(&mut self, hr_control: &mut HrPwmCtrl) {
        self.timer.timer.stop(hr_control);
    }

    /*
    /// Enable burst mode.
    pub fn enable_burst_mode(&mut self) {
        T::regs().tim(C::raw()).outr().modify(|w| {
            // Enable Burst Mode
            w.set_idlem(0, true);
            w.set_idlem(1, true);

            // Set output to active during the burst
            w.set_idles(0, true);
            w.set_idles(1, true);
        })
    }

    /// Disable burst mode.
    pub fn disable_burst_mode(&mut self) {
        T::regs().tim(C::raw()).outr().modify(|w| {
            // Disable Burst Mode
            w.set_idlem(0, false);
            w.set_idlem(1, false);
        })
    }*/

    fn update_primary_duty_or_dead_time(&mut self) {
        self.min_secondary_duty = self.primary_duty + self.dead_time;
        self.timer.cr1.set_duty(self.primary_duty);
        self.timer.cr2.set_duty(self.min_secondary_duty);
    }

    /// Set the dead time as a proportion of the maximum compare value
    pub fn set_dead_time(&mut self, dead_time: u16) {
        self.dead_time = dead_time;
        self.max_secondary_duty = self.get_max_compare_value() - dead_time;
        self.update_primary_duty_or_dead_time();
    }

    /// Get the maximum compare value of a duty cycle
    pub fn get_max_compare_value(&mut self) -> u16 {
        self.timer.timer.get_period()
    }

    /// The primary duty is the period in which the primary switch is active
    ///
    /// In the case of a buck converter, this is the high-side switch
    /// In the case of a boost converter, this is the low-side switch
    pub fn set_primary_duty(&mut self, primary_duty: u16) {
        self.primary_duty = primary_duty;
        self.update_primary_duty_or_dead_time();
    }

    /// The secondary duty is the period in any switch is active
    ///
    /// If less than or equal to the primary duty, the secondary switch will be active for one tick
    /// If a fully complementary output is desired, the secondary duty can be set to the max compare
    pub fn set_secondary_duty(&mut self, secondary_duty: u16) {
        let secondary_duty = if secondary_duty > self.max_secondary_duty {
            self.max_secondary_duty
        } else if secondary_duty <= self.min_secondary_duty {
            self.min_secondary_duty + 1
        } else {
            secondary_duty
        };

        self.timer.cr3.set_duty(secondary_duty);
    }
}
