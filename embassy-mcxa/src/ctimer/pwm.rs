//! CTimer-based PWM driver.
//!
//! Every output owns its timer activity requirement, including after splitting
//! a multi-output PWM. Suspending or dropping one output does not stop sibling
//! PWM/capture channels. Each guard is released only after its own output stops.

use embassy_hal_internal::Peri;
pub use embedded_hal_1::pwm::SetDutyCycle;
use embedded_hal_1::pwm::{Error, ErrorKind, ErrorType};

use super::{AnyChannel, CTimer, CTimerChannel, Channel, Info, Instance, OutputPin};
use crate::clocks::{PoweredClock, WakeGuard};
use crate::gpio::{AnyPin, Level, SealedPin};
use crate::pac::ctimer::Pwmen;
use crate::pac::port::Mux;

/// PWM error.
#[derive(Debug)]
pub enum PwmError {
    /// Resume the output before changing its duty cycle.
    Suspended,
    /// Invalid Duty Cycle.
    InvalidDutyCycle,
    /// Invalid Channel Number.
    InvalidChannel,
    /// Channel mismatch
    ChannelMismatch,
}

impl Error for PwmError {
    fn kind(&self) -> ErrorKind {
        match self {
            PwmError::Suspended => ErrorKind::Other,
            PwmError::InvalidDutyCycle => ErrorKind::Other,
            PwmError::InvalidChannel => ErrorKind::Other,
            PwmError::ChannelMismatch => ErrorKind::Other,
        }
    }
}

/// Pwm Configuration
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct Config {
    /// The point at which the counter wraps around.
    ///
    /// This value represents the maximum possible period.
    pub freq: u16,

    /// Duty cycle.
    pub duty_cycle: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            freq: 20_000,
            duty_cycle: 0,
        }
    }
}

/// Representation of a single PWM channel.
///
/// This PWM representation can change duty cycle, but not frequency
/// of the PWM.
pub struct Pwm<'d> {
    info: &'static Info,
    duty_ch: Peri<'d, AnyChannel>,
    pin: Peri<'d, AnyPin>,
    source_freq: u32,
    pwm_freq: u16,
    max_period: u16,
    power: PoweredClock,
    wake_guard: Option<WakeGuard>,
    suspended: bool,
    saved_mux: Option<Mux>,
}

impl<'d> Pwm<'d> {
    fn new_inner(
        info: &'static Info,
        ctimer: &CTimer<'d>,
        duty_ch: Peri<'d, AnyChannel>,
        pin: Peri<'d, AnyPin>,
        pwm_freq: u16,
    ) -> Self {
        Self {
            info,
            duty_ch,
            pin,
            source_freq: ctimer._freq,
            pwm_freq,
            max_period: 0,
            power: ctimer.power,
            wake_guard: ctimer._wg.clone(),
            suspended: false,
            saved_mux: None,
        }
    }

    /// Stop this output without stopping other timer users.
    ///
    /// This does not turn off the attached device or guarantee a particular
    /// electrical output level. The product must make that safe before calling.
    pub fn suspend(&mut self) {
        if !self.suspended {
            critical_section::with(|_| {
                self.set_pwm_mode(false);
                self.info.stop_if_unused();
                self.suspended = true;
                self.wake_guard = None;
            });
        }
    }

    /// Stop PWM and hold its pad at a product-selected GPIO level.
    ///
    /// The GPIO output latch and direction are set before changing the mux.
    /// The pad keeps that level without needing the timer clock. Resume
    /// restores the original peripheral mux after restarting the timer.
    /// `level` is a pad voltage, not a PWM percentage or match value.
    pub fn suspend_with_level(&mut self, level: Level) {
        critical_section::with(|_| {
            let pin = &self.pin;
            let number = usize::from(pin.pin());
            match level {
                Level::High => pin.gpio().psor().write(|w| w.0 = 1 << number),
                Level::Low => pin.gpio().pcor().write(|w| w.0 = 1 << number),
            }
            pin.gpio()
                .pddr()
                .modify(|w| w.set_pdd(number, crate::pac::gpio::Pdd::Pdd1));
            if self.saved_mux.is_none() {
                self.saved_mux = Some(pin.pcr_reg().read().mux());
            }
            pin.set_function(Mux::Mux0);
            self.suspend();
        });
    }

    /// Reacquire this output's requirement before restarting it.
    pub fn resume(&mut self) {
        if self.suspended {
            critical_section::with(|_| {
                self.wake_guard = WakeGuard::for_power(&self.power);
                self.set_pwm_mode(true);
                self.enable();
                if let Some(mux) = self.saved_mux.take() {
                    self.pin.set_function(mux);
                }
                self.suspended = false;
            });
        }
    }

    /// Whether this output is explicitly suspended.
    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    fn enable(&mut self) {
        self.info.regs().tcr().modify(|w| w.set_cen(true));
    }

    fn disable(&mut self) {
        self.info.regs().tcr().modify(|w| w.set_cen(false));
    }

    fn set_pwm_mode(&self, enabled: bool) {
        let mode = if enabled { Pwmen::Pwm } else { Pwmen::Match };
        self.info.regs().pwmc().modify(|w| match self.duty_ch.number() {
            Channel::Zero => {
                w.set_pwmen0(mode);
            }
            Channel::One => {
                w.set_pwmen1(mode);
            }
            Channel::Two => {
                w.set_pwmen2(mode);
            }
            Channel::Three => {
                w.set_pwmen3(mode);
            }
        });
    }

    fn clear_status(&self) {
        self.info.regs().mcr().modify(|w| {
            // Clear stop, reset, and interrupt bits for the PWM channel
            match self.duty_ch.number() {
                Channel::Zero => {
                    w.set_mr0i(false);
                    w.set_mr0r(false);
                    w.set_mr0s(false);
                }
                Channel::One => {
                    w.set_mr1i(false);
                    w.set_mr1r(false);
                    w.set_mr1s(false);
                }
                Channel::Two => {
                    w.set_mr2i(false);
                    w.set_mr2r(false);
                    w.set_mr2s(false);
                }
                Channel::Three => {
                    w.set_mr3i(false);
                    w.set_mr3r(false);
                    w.set_mr3s(false);
                }
            }

            match self.duty_ch.number() {
                Channel::Zero => {
                    w.set_mr0rl(true);
                }
                Channel::One => {
                    w.set_mr1rl(true);
                }
                Channel::Two => {
                    w.set_mr2rl(true);
                }
                Channel::Three => {
                    w.set_mr3rl(true);
                }
            }
        });
    }

    fn configure_duty_cycle(&self, duty_cycle: u32) {
        self.info
            .regs()
            .mr(self.duty_ch.number().into())
            .write(|w| w.set_match_(duty_cycle));
        self.info
            .regs()
            .msr(self.duty_ch.number().into())
            .write(|w| w.set_match_shadow(duty_cycle));
    }
}

/// Single channel PWM driver
///
/// A single channel is used for Duty Cycle and a single channel is
/// used for PWM period match.
pub struct SinglePwm<'d> {
    pwm: Pwm<'d>,
    period_ch: Peri<'d, AnyChannel>,
}

impl<'d> SinglePwm<'d> {
    /// Create Pwm driver with a single pin as output.
    ///
    /// Upon `Drop`, the external `pin` will be placed into `Disabled`
    /// state.
    pub fn new<T: Instance, DUTY: CTimerChannel<T>, PIN: OutputPin<T>>(
        ctimer: CTimer<'d>,
        duty_ch: Peri<'d, DUTY>,
        period_ch: Peri<'d, impl CTimerChannel<T>>,
        pin: Peri<'d, PIN>,
        config: Config,
    ) -> Result<Self, PwmError>
    where
        (T, DUTY, PIN): ValidMatchConfig,
    {
        if !core::ptr::eq(ctimer.info, T::info()) {
            return Err(PwmError::ChannelMismatch);
        }
        pin.mux();

        let mut inst = Self {
            pwm: Pwm::new_inner(T::info(), &ctimer, duty_ch.into(), pin.into(), config.freq),
            period_ch: period_ch.into(),
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    /// Degrade `self` into the underlying PWM representation.
    ///
    /// Upon calling this method, changing frequency will be disallowed.
    pub fn degrade(self) -> Pwm<'d> {
        self.pwm
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), PwmError> {
        self.pwm.disable();
        self.pwm.set_pwm_mode(true);
        self.pwm.clear_status();

        self.pwm.info.regs().mcr().modify(|w| match self.period_ch.number() {
            Channel::Zero => {
                w.set_mr0r(true);
            }
            Channel::One => {
                w.set_mr1r(true);
            }
            Channel::Two => {
                w.set_mr2r(true);
            }
            Channel::Three => {
                w.set_mr3r(true);
            }
        });

        // Configure PWM period
        // MR uses period_ticks - 1 because the counter includes zero. The duty
        // range includes period_ticks so 0% can use an unreachable match value.
        let period_ticks = self.pwm.source_freq / u32::from(self.pwm.pwm_freq);
        let max_duty = u16::try_from(period_ticks).map_err(|_| PwmError::InvalidDutyCycle)?;
        let period_match = period_ticks - 1;
        self.pwm.max_period = max_duty;
        self.pwm
            .info
            .regs()
            .mr(self.period_ch.number().into())
            .write(|w| w.set_match_(period_match));

        // Configure PWM duty cycle
        let duty_cycle = ((period_ticks) * (100 - u32::from(config.duty_cycle))) / 100;
        self.pwm.configure_duty_cycle(duty_cycle);

        // Start CTimer
        self.pwm.enable();

        Ok(())
    }
}

/// Dual channel PWM driver.
///
/// A single period match channel is shared for two independent PWM
/// outputs. That is, both PWM output channels run on the same
/// frequency, with optionally different duty cycles.
pub struct DualPwm<'d> {
    pub pwm0: Pwm<'d>,
    pub pwm1: Pwm<'d>,
    period_ch: Peri<'d, AnyChannel>,
}

impl<'d> DualPwm<'d> {
    /// Create Pwm driver with a two pins for two PWM outputs.
    ///
    /// Upon `Drop`, all external pins will be placed into `Disabled`
    /// state.
    pub fn new<T: Instance, DUTY0: CTimerChannel<T>, DUTY1: CTimerChannel<T>, PIN0: OutputPin<T>, PIN1: OutputPin<T>>(
        ctimer: CTimer<'d>,
        duty_ch0: Peri<'d, DUTY0>,
        duty_ch1: Peri<'d, DUTY1>,
        period_ch: Peri<'d, impl CTimerChannel<T>>,
        pin0: Peri<'d, PIN0>,
        pin1: Peri<'d, PIN1>,
        config: Config,
    ) -> Result<Self, PwmError>
    where
        (T, DUTY0, PIN0): ValidMatchConfig,
        (T, DUTY1, PIN1): ValidMatchConfig,
    {
        if !core::ptr::eq(ctimer.info, T::info()) {
            return Err(PwmError::ChannelMismatch);
        }
        pin0.mux();
        pin1.mux();

        let mut inst = Self {
            pwm0: Pwm::new_inner(T::info(), &ctimer, duty_ch0.into(), pin0.into(), config.freq),
            pwm1: Pwm::new_inner(T::info(), &ctimer, duty_ch1.into(), pin1.into(), config.freq),
            period_ch: period_ch.into(),
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    /// Split `self` into its underlying channels.
    ///
    /// Upon calling this method, changing PWM frequency will be
    /// disallowed. Only duty cycles can be changed.
    pub fn split(self) -> (Pwm<'d>, Pwm<'d>) {
        (self.pwm0, self.pwm1)
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), PwmError> {
        self.pwm0.disable();

        self.pwm0.set_pwm_mode(true);
        self.pwm1.set_pwm_mode(true);

        self.pwm0.clear_status();
        self.pwm1.clear_status();

        self.pwm0.info.regs().mcr().modify(|w| match self.period_ch.number() {
            Channel::Zero => {
                w.set_mr0r(true);
            }
            Channel::One => {
                w.set_mr1r(true);
            }
            Channel::Two => {
                w.set_mr2r(true);
            }
            Channel::Three => {
                w.set_mr3r(true);
            }
        });

        // Configure PWM period
        let period = self.pwm0.source_freq / u32::from(self.pwm0.pwm_freq) - 1;

        self.pwm0.max_period = period as u16;
        self.pwm1.max_period = period as u16;

        self.pwm0
            .info
            .regs()
            .mr(self.period_ch.number().into())
            .write(|w| w.set_match_(period));

        // Configure PWM duty cycle
        let duty_cycle = ((period + 1) * (100 - u32::from(config.duty_cycle))) / 100;
        self.pwm0.configure_duty_cycle(duty_cycle);
        self.pwm1.configure_duty_cycle(duty_cycle);

        // Start CTimer
        self.pwm0.enable();

        Ok(())
    }
}

/// Triple channel PWM driver.
///
/// A single period match channel is shared for three independent PWM
/// outputs. That is, all three PWM output channels run on the same
/// frequency, with optionally different duty cycles.
pub struct TriplePwm<'d> {
    pub pwm0: Pwm<'d>,
    pub pwm1: Pwm<'d>,
    pub pwm2: Pwm<'d>,
    period_ch: Peri<'d, AnyChannel>,
}

impl<'d> TriplePwm<'d> {
    /// Create Pwm driver using three pins for three PWM outputs.
    ///
    /// Upon `Drop`, all external pins will be placed into `Disabled`
    /// state.
    pub fn new<
        T: Instance,
        DUTY0: CTimerChannel<T>,
        DUTY1: CTimerChannel<T>,
        DUTY2: CTimerChannel<T>,
        PIN0: OutputPin<T>,
        PIN1: OutputPin<T>,
        PIN2: OutputPin<T>,
    >(
        ctimer: CTimer<'d>,
        duty_ch0: Peri<'d, DUTY0>,
        duty_ch1: Peri<'d, DUTY1>,
        duty_ch2: Peri<'d, DUTY2>,
        period_ch: Peri<'d, impl CTimerChannel<T>>,
        pin0: Peri<'d, PIN0>,
        pin1: Peri<'d, PIN1>,
        pin2: Peri<'d, PIN2>,
        config: Config,
    ) -> Result<Self, PwmError>
    where
        (T, DUTY0, PIN0): ValidMatchConfig,
        (T, DUTY1, PIN1): ValidMatchConfig,
        (T, DUTY2, PIN2): ValidMatchConfig,
    {
        if !core::ptr::eq(ctimer.info, T::info()) {
            return Err(PwmError::ChannelMismatch);
        }
        pin0.mux();
        pin1.mux();
        pin2.mux();

        let mut inst = Self {
            pwm0: Pwm::new_inner(T::info(), &ctimer, duty_ch0.into(), pin0.into(), config.freq),
            pwm1: Pwm::new_inner(T::info(), &ctimer, duty_ch1.into(), pin1.into(), config.freq),
            pwm2: Pwm::new_inner(T::info(), &ctimer, duty_ch2.into(), pin2.into(), config.freq),
            period_ch: period_ch.into(),
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    /// Split `self` into its underlying channels.
    ///
    /// Upon calling this method, changing PWM frequency will be
    /// disallowed. Only duty cycles can be changed.
    pub fn split(self) -> (Pwm<'d>, Pwm<'d>, Pwm<'d>) {
        (self.pwm0, self.pwm1, self.pwm2)
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), PwmError> {
        self.pwm0.disable();

        self.pwm0.set_pwm_mode(true);
        self.pwm1.set_pwm_mode(true);
        self.pwm2.set_pwm_mode(true);

        self.pwm0.clear_status();
        self.pwm1.clear_status();
        self.pwm2.clear_status();

        self.pwm0.info.regs().mcr().modify(|w| match self.period_ch.number() {
            Channel::Zero => {
                w.set_mr0r(true);
            }
            Channel::One => {
                w.set_mr1r(true);
            }
            Channel::Two => {
                w.set_mr2r(true);
            }
            Channel::Three => {
                w.set_mr3r(true);
            }
        });

        // Configure PWM period
        let period = self.pwm0.source_freq / u32::from(self.pwm0.pwm_freq) - 1;

        self.pwm0.max_period = period as u16;
        self.pwm1.max_period = period as u16;
        self.pwm2.max_period = period as u16;

        self.pwm0
            .info
            .regs()
            .mr(self.period_ch.number().into())
            .write(|w| w.set_match_(period));

        // Configure PWM duty cycle
        let duty_cycle = ((period + 1) * (100 - u32::from(config.duty_cycle))) / 100;
        self.pwm0.configure_duty_cycle(duty_cycle);
        self.pwm1.configure_duty_cycle(duty_cycle);
        self.pwm2.configure_duty_cycle(duty_cycle);

        // Start CTimer
        self.pwm0.enable();

        Ok(())
    }
}

impl<'d> Drop for Pwm<'d> {
    fn drop(&mut self) {
        self.suspend();
        self.pin.set_as_disabled();
    }
}

impl<'d> ErrorType for SinglePwm<'d> {
    type Error = PwmError;
}

impl<'d> SetDutyCycle for SinglePwm<'d> {
    fn max_duty_cycle(&self) -> u16 {
        self.pwm.max_period
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        self.pwm.set_duty_cycle(duty)
    }
}

impl<'d> ErrorType for Pwm<'d> {
    type Error = PwmError;
}

impl<'d> SetDutyCycle for Pwm<'d> {
    fn max_duty_cycle(&self) -> u16 {
        self.max_period
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        if self.suspended {
            return Err(PwmError::Suspended);
        }
        let max_duty = self.max_duty_cycle();

        if duty > max_duty {
            return Err(PwmError::InvalidDutyCycle);
        }

        self.info
            .regs()
            .msr(self.duty_ch.number().into())
            .write(|w| w.set_match_shadow(u32::from(duty)));

        Ok(())
    }
}

pub(crate) trait SealedValidMatchConfig {}

/// Valid match channel + pin configuration marker trait
#[allow(private_bounds)]
pub trait ValidMatchConfig: SealedValidMatchConfig {}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_ctimer_match {
    ($peri:ident, $ch:ident, $pin:ident) => {
        impl $crate::ctimer::pwm::SealedValidMatchConfig
            for (
                $crate::peripherals::$peri,
                $crate::peripherals::$ch,
                $crate::peripherals::$pin,
            )
        {
        }

        impl $crate::ctimer::pwm::ValidMatchConfig
            for (
                $crate::peripherals::$peri,
                $crate::peripherals::$ch,
                $crate::peripherals::$pin,
            )
        {
        }
    };
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use core::marker::PhantomData;
    use core::sync::atomic::AtomicU8;
    use std::boxed::Box;

    fn test_info() -> &'static Info {
        let memory = Box::leak(Box::new([0u32; 64]));
        // SAFETY: isolated aligned RAM covers the entire CTIMER register block.
        let regs = unsafe { crate::pac::ctimer::Ctimer::from_ptr(memory.as_mut_ptr().cast()) };
        Box::leak(Box::new(Info {
            regs,
            wait_cell: maitake_sync::WaitCell::new(),
            irq_flags: AtomicU8::new(0),
        }))
    }

    fn channel(number: Channel) -> Peri<'static, AnyChannel> {
        // SAFETY: tests use isolated registers, not a hardware channel singleton.
        unsafe { Peri::new_unchecked(AnyChannel::new(number)) }
    }

    fn test_core(info: &'static Info) -> CTimer<'static> {
        CTimer {
            info,
            _freq: 12_000_000,
            _wg: Some(WakeGuard::new()),
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            _phantom: PhantomData,
        }
    }

    fn test_pwm(ctimer: &CTimer<'static>, number: Channel) -> Pwm<'static> {
        let mut pwm = Pwm::new_inner(ctimer.info, ctimer, channel(number), crate::test_support::pin(), 25_000);
        pwm.max_period = 480;
        pwm.set_pwm_mode(true);
        pwm.enable();
        pwm
    }

    #[test]
    fn degraded_single_pwm_keeps_the_consumed_timer_guard() {
        critical_section::with(|cs| {
            let info = test_info();
            let core = test_core(info);
            let single = SinglePwm {
                pwm: test_pwm(&core, Channel::Zero),
                period_ch: channel(Channel::Three),
            };
            drop(core);
            let mut pwm = single.degrade();
            assert!(crate::clocks::active_wake_guards(&cs));
            pwm.suspend();
            assert!(pwm.is_suspended());
            assert!(!info.regs().tcr().read().cen());
            assert!(!crate::clocks::active_wake_guards(&cs));
            assert!(matches!(pwm.set_duty_cycle(1), Err(PwmError::Suspended)));
            pwm.resume();
            assert!(info.regs().tcr().read().cen());
            assert!(crate::clocks::active_wake_guards(&cs));
            drop(pwm);
            assert!(!info.regs().tcr().read().cen());
            assert!(!crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn dropping_one_split_dual_channel_does_not_stop_its_sibling() {
        critical_section::with(|cs| {
            let info = test_info();
            let core = test_core(info);
            let dual = DualPwm {
                pwm0: test_pwm(&core, Channel::Zero),
                pwm1: test_pwm(&core, Channel::One),
                period_ch: channel(Channel::Three),
            };
            drop(core);
            let (first, mut second) = dual.split();
            drop(first);
            assert!(info.regs().tcr().read().cen());
            assert_eq!(info.regs().pwmc().read().pwmen1(), Pwmen::Pwm);
            assert!(crate::clocks::active_wake_guards(&cs));
            second.set_duty_cycle(240).unwrap();
            drop(second);
            assert!(!info.regs().tcr().read().cen());
            assert!(!crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn triple_channels_release_requirements_independently() {
        critical_section::with(|cs| {
            let info = test_info();
            let core = test_core(info);
            let triple = TriplePwm {
                pwm0: test_pwm(&core, Channel::Zero),
                pwm1: test_pwm(&core, Channel::One),
                pwm2: test_pwm(&core, Channel::Two),
                period_ch: channel(Channel::Three),
            };
            drop(core);
            let (mut first, mut second, mut third) = triple.split();
            first.suspend();
            second.suspend();
            assert!(info.regs().tcr().read().cen());
            assert!(crate::clocks::active_wake_guards(&cs));
            third.suspend();
            assert!(!info.regs().tcr().read().cen());
            assert!(!crate::clocks::active_wake_guards(&cs));
            first.resume();
            assert!(info.regs().tcr().read().cen());
            assert!(crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn stopping_pwm_keeps_a_shared_capture_timer_running() {
        let info = test_info();
        let core = test_core(info);
        let pwm = test_pwm(&core, Channel::Zero);
        info.regs().ccr().modify(|w| w.set_cap1re(true));
        drop(pwm);
        assert!(info.regs().tcr().read().cen());
        assert!(info.regs().ccr().read().cap1re());
    }

    #[test]
    fn gpio_level_suspension_restores_the_peripheral_mux_on_resume() {
        critical_section::with(|cs| {
            let core = test_core(test_info());
            let mut pwm = test_pwm(&core, Channel::Zero);
            drop(core);
            pwm.pin.set_function(Mux::from_bits(4));
            pwm.suspend_with_level(Level::High);
            assert_eq!(pwm.pin.pcr_reg().read().mux(), Mux::Mux0);
            assert_eq!(
                pwm.pin.gpio().pddr().read().pdd(usize::from(pwm.pin.pin())),
                crate::pac::gpio::Pdd::Pdd1
            );
            assert!(!crate::clocks::active_wake_guards(&cs));
            pwm.resume();
            assert_eq!(pwm.pin.pcr_reg().read().mux(), Mux::from_bits(4));
            assert!(crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn gpio_low_off_round_trips_on_a_nonzero_port_and_pin() {
        critical_section::with(|cs| {
            let info = test_info();
            let core = test_core(info);
            let mut pwm = Pwm::new_inner(
                info,
                &core,
                channel(Channel::Two),
                crate::test_support::pin_at(2, 18),
                1_000,
            );
            pwm.max_period = 11_999;
            pwm.pin.set_function(Mux::from_bits(4));
            pwm.set_pwm_mode(true);
            pwm.enable();
            drop(core);

            for _ in 0..2 {
                pwm.suspend_with_level(Level::Low);
                assert_eq!(pwm.pin.gpio().pcor().read().0, 1 << 18);
                assert_eq!(pwm.pin.gpio().psor().read().0, 0);
                assert_eq!(pwm.pin.gpio().pddr().read().pdd(18), crate::pac::gpio::Pdd::Pdd1);
                assert_eq!(pwm.pin.pcr_reg().read().mux(), Mux::Mux0);
                assert_eq!(info.regs().pwmc().read().pwmen2(), Pwmen::Match);
                assert!(!info.regs().tcr().read().cen());
                assert!(pwm.is_suspended());
                assert!(!crate::clocks::active_wake_guards(&cs));

                pwm.resume();
                assert_eq!(pwm.pin.pcr_reg().read().mux(), Mux::from_bits(4));
                assert_eq!(info.regs().pwmc().read().pwmen2(), Pwmen::Pwm);
                assert!(info.regs().tcr().read().cen());
                assert!(!pwm.is_suspended());
                assert!(crate::clocks::active_wake_guards(&cs));
            }
        });
    }
}
