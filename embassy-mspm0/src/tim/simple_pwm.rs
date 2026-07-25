//! Simple pulse width modulation (PWM) driver.

use core::marker::PhantomData;

pub use super::{Ch0, Ch1, Ch2, Ch3};
use super::{Instance, TimerChannel, TimerPin};
use crate::Peri;
use crate::gpio::{AnyPin, PfType, Pull, SealedPin};
use crate::pac::tim::Tim;
use crate::pac::tim::vals::{Act, Ccpiv, Ccpo, Cm, Coc, Cvae, CxC, PwrenKey, Repeat, ResetKey};

/// The clock source for the timer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockSel {
    /// Use the low frequency clock.
    ///
    /// The LFCLK runs at 32.768 kHz.
    LfClk,

    /// Use the middle frequency clock.
    ///
    /// The MFCLK runs at 4 MHz.
    MfClk,
    // BusClk depends on the timer's power domain and cannot be used for now.
    // BusClk,
}

impl ClockSel {
    fn frequency(&self) -> u32 {
        match self {
            ClockSel::LfClk => 32_768,
            ClockSel::MfClk => 4_000_000,
        }
    }
}

/// The way the counter generates the PWM period.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CountingMode {
    /// Edge aligned PWM.
    ///
    /// The counter counts up and every output is asserted at the start of the period. This is
    /// `DL_TIMER_PWM_MODE_EDGE_ALIGN_UP` in the TI SDK.
    EdgeAligned,

    /// Center aligned PWM.
    ///
    /// The counter counts up and down, meaning every output is centered inside the period. This is
    /// `DL_TIMER_PWM_MODE_CENTER_ALIGN` in the TI SDK.
    CenterAligned,
}

/// A capture and compare channel of a timer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Channel {
    /// Channel 0, `CCP0`.
    Ch0,
    /// Channel 1, `CCP1`.
    Ch1,
    /// Channel 2, `CCP2`.
    Ch2,
    /// Channel 3, `CCP3`.
    Ch3,
}

impl Channel {
    /// The index of this channel.
    pub const fn index(&self) -> usize {
        match self {
            Channel::Ch0 => 0,
            Channel::Ch1 => 1,
            Channel::Ch2 => 2,
            Channel::Ch3 => 3,
        }
    }
}

/// The polarity of a PWM output.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputPolarity {
    /// The output is high while the channel is active.
    ActiveHigh,
    /// The output is low while the channel is active.
    ActiveLow,
}

/// PWM configuration error.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// The frequency cannot be reached with the selected clock source.
    InvalidFrequency,
}

/// PWM config.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Config {
    /// Timer clock source.
    pub clock_source: ClockSel,

    /// The frequency of the generated PWM signal in Hz.
    pub frequency: u32,

    /// How the counter generates the PWM period.
    pub counting_mode: CountingMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clock_source: ClockSel::MfClk,
            frequency: 1_000,
            counting_mode: CountingMode::EdgeAligned,
        }
    }
}

/// PWM pin wrapper.
///
/// This wraps a pin to make it usable with PWM.
pub struct PwmPin<'d, T: Instance, C: TimerChannel> {
    pin: Peri<'d, AnyPin>,
    phantom: PhantomData<(T, C)>,
}

impl<'d, T: Instance, C: TimerChannel> PwmPin<'d, T, C> {
    /// Create a new PWM pin instance.
    pub fn new(pin: Peri<'d, impl TimerPin<T, C>>, invert: bool) -> Self {
        pin.set_as_pf(pin.pf_num(), PfType::output(Pull::None, invert));

        Self {
            pin: pin.into(),
            phantom: PhantomData,
        }
    }
}

/// A single channel of a PWM driver, obtained from [`SimplePwm::channel`], [`SimplePwm::ch0`], etc.
///
/// It is not possible to change the PWM frequency here because the frequency is shared with all
/// channels of the timer.
pub struct SimplePwmChannel<'d, T: Instance> {
    channel: Channel,
    load: u32,
    counting_mode: CountingMode,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> SimplePwmChannel<'d, T> {
    /// Enable the channel.
    pub fn enable(&mut self) {
        regs::<T>().commonregs(0).odis().modify(|w| {
            // `false` drives the pin from OCTL; `true` forces it low.
            w.set_c0ccp(self.channel.index(), false);
        });
    }

    /// Disable the channel.
    ///
    /// The output of the channel is forced low while the channel is disabled.
    pub fn disable(&mut self) {
        regs::<T>().commonregs(0).odis().modify(|w| {
            w.set_c0ccp(self.channel.index(), true);
        });
    }

    /// Check whether the channel is enabled.
    pub fn is_enabled(&self) -> bool {
        !regs::<T>().commonregs(0).odis().read().c0ccp(self.channel.index())
    }

    /// Get the max duty value.
    ///
    /// This value depends on the configured frequency and the timer's clock source.
    pub fn max_duty_cycle(&self) -> u16 {
        max_duty_cycle(self.load, self.counting_mode)
    }

    /// Set the duty cycle of the channel.
    ///
    /// The value ranges from 0 for 0% duty, to [`max_duty_cycle`](Self::max_duty_cycle) for 100%
    /// duty, both included.
    pub fn set_duty_cycle(&mut self, duty: u16) {
        let max = self.max_duty_cycle();
        assert!(duty <= max);

        // While counting up, the output is set at the zero event and cleared once the counter
        // reaches the compare value. While counting up and down, the output is set while counting
        // up and cleared while counting down, meaning a larger compare value is a shorter pulse.
        let compare = match self.counting_mode {
            CountingMode::EdgeAligned => duty as u32,
            CountingMode::CenterAligned => (max - duty) as u32,
        };

        set_compare_value::<T>(self.channel, compare);
    }

    /// Set the duty cycle to 0%, or always inactive.
    pub fn set_duty_cycle_fully_off(&mut self) {
        self.set_duty_cycle(0);
    }

    /// Set the duty cycle to 100%, or always active.
    pub fn set_duty_cycle_fully_on(&mut self) {
        self.set_duty_cycle(self.max_duty_cycle());
    }

    /// Set the duty cycle to `num / denom`.
    ///
    /// The caller is responsible for ensuring that `num` is less than or equal to `denom`, and that
    /// `denom` is not zero.
    pub fn set_duty_cycle_fraction(&mut self, num: u16, denom: u16) {
        assert!(denom != 0);
        assert!(num <= denom);
        let duty = u32::from(num) * u32::from(self.max_duty_cycle()) / u32::from(denom);

        // This is safe because we know that `num <= denom`, so `duty <= self.max_duty_cycle()` (u16)
        self.set_duty_cycle(duty as u16);
    }

    /// Set the duty cycle to `percent / 100`.
    ///
    /// The caller is responsible for ensuring that `percent` is less than or equal to 100.
    pub fn set_duty_cycle_percent(&mut self, percent: u8) {
        self.set_duty_cycle_fraction(u16::from(percent), 100)
    }

    /// Get the duty cycle of the channel.
    ///
    /// The value ranges from 0 for 0% duty, to [`max_duty_cycle`](Self::max_duty_cycle) for 100%
    /// duty, both included.
    pub fn current_duty_cycle(&self) -> u16 {
        let compare = compare_value::<T>(self.channel) as u16;

        match self.counting_mode {
            CountingMode::EdgeAligned => compare,
            CountingMode::CenterAligned => self.max_duty_cycle() - compare,
        }
    }

    /// Set the output polarity of the channel.
    pub fn set_polarity(&mut self, polarity: OutputPolarity) {
        regs::<T>().counterregs(0).octl(self.channel.index()).modify(|w| {
            w.set_ccpoinv(matches!(polarity, OutputPolarity::ActiveLow));
        });
    }
}

impl<'d, T: Instance> embedded_hal::pwm::ErrorType for SimplePwmChannel<'d, T> {
    type Error = core::convert::Infallible;
}

impl<'d, T: Instance> embedded_hal::pwm::SetDutyCycle for SimplePwmChannel<'d, T> {
    fn max_duty_cycle(&self) -> u16 {
        self.max_duty_cycle()
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        self.set_duty_cycle(duty);
        Ok(())
    }
}

/// Simple PWM driver.
pub struct SimplePwm<'d, T: Instance> {
    _timer: Peri<'d, T>,
    pins: [Option<Peri<'d, AnyPin>>; 4],
    counting_mode: CountingMode,
    load: u32,
}

impl<'d, T: Instance> SimplePwm<'d, T> {
    /// Create a new simple PWM driver.
    ///
    /// Channels which are not routed to a pin may still be created, however only the channels with
    /// a [`PwmPin`] will drive an output.
    pub fn new(
        timer: Peri<'d, T>,
        ch0: Option<PwmPin<'d, T, Ch0>>,
        ch1: Option<PwmPin<'d, T, Ch1>>,
        ch2: Option<PwmPin<'d, T, Ch2>>,
        ch3: Option<PwmPin<'d, T, Ch3>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let mut this = Self {
            _timer: timer,
            pins: [
                ch0.map(|pin| pin.pin),
                ch1.map(|pin| pin.pin),
                ch2.map(|pin| pin.pin),
                ch3.map(|pin| pin.pin),
            ],
            counting_mode: config.counting_mode,
            load: 0,
        };

        this.init(&config)?;

        Ok(this)
    }

    fn init(&mut self, config: &Config) -> Result<(), ConfigError> {
        let r = regs::<T>();

        // Reset timer
        r.gprcm(0).rstctl().write(|w| {
            w.set_resetassert(true);
            w.set_key(ResetKey::KEY);
            w.set_resetstkyclr(true);
        });

        // Power up timer
        r.gprcm(0).pwren().write(|w| {
            w.set_enable(true);
            w.set_key(PwrenKey::KEY);
        });

        // Following the instructions according to SLAU847D 23.2.1: TIMCLK Configuration

        // 1. Select the TIMCLK source.
        r.clksel().write(|w| match config.clock_source {
            ClockSel::LfClk => {
                w.set_lfclk_sel(true);
                w.set_mfclk_sel(false);
                w.set_busclk_sel(false);
            }
            ClockSel::MfClk => {
                w.set_mfclk_sel(true);
                w.set_lfclk_sel(false);
                w.set_busclk_sel(false);
            }
        });

        r.pdbgctl().modify(|w| {
            // Halt the PWM outputs when the debugger halts the core.
            w.set_free(false);
        });

        // 2. and 3. are the clock divider and the prescaler, both are set up by the frequency.
        let timing = self.set_frequency_inner(config)?;

        // 4. Enable the TIMCLK.
        r.commonregs(0).cclkctl().modify(|w| {
            w.set_clken(true);
        });

        for channel in [Channel::Ch0, Channel::Ch1, Channel::Ch2, Channel::Ch3]
            .into_iter()
            .take(T::info().channels as usize)
        {
            let index = channel.index();

            // The output starts low and is generated by the signal generator.
            r.counterregs(0).octl(index).write(|w| {
                w.set_ccpo(Ccpo::FUNCVAL);
                w.set_ccpoinv(false);
                w.set_ccpiv(Ccpiv::LOW);
            });

            r.counterregs(0).ccctl(index).write(|w| {
                w.set_coc(Coc::COMPARE);
            });

            r.counterregs(0).ccact(index).write(|w| match config.counting_mode {
                CountingMode::EdgeAligned => {
                    w.set_zact(Act::CCP_HIGH);
                    w.set_cuact(Act::CCP_LOW);
                }
                CountingMode::CenterAligned => {
                    w.set_cuact(Act::CCP_HIGH);
                    w.set_cdact(Act::CCP_LOW);
                }
            });

            // Start with the output inactive.
            set_compare_value::<T>(
                channel,
                match config.counting_mode {
                    CountingMode::EdgeAligned => 0,
                    CountingMode::CenterAligned => timing.load,
                },
            );

            // Drive the pin instead of capturing it.
            r.commonregs(0).ccpd().modify(|w| {
                w.set_c0ccp(index, true);
            });
        }

        r.counterregs(0).ctrctl().modify(|w| {
            w.set_repeat(Repeat::REPEAT_1);
            w.set_cvae(Cvae::ZEROVAL);
            w.set_cm(match config.counting_mode {
                CountingMode::EdgeAligned => Cm::UP,
                CountingMode::CenterAligned => Cm::UP_DOWN,
            });

            // Must explicitly set CZC, CAC and CLC to 0 in order for all the timers to count.
            //
            // The reset value of these registers is 0x07, which is a reserved value.
            w.set_czc(CxC::CCTL0);
            w.set_cac(CxC::CCTL0);
            w.set_clc(CxC::CCTL0);

            w.set_en(true);
        });

        Ok(())
    }

    /// Get a single channel.
    pub fn channel(&mut self, channel: Channel) -> SimplePwmChannel<'_, T> {
        assert!(
            (channel.index() as u8) < T::info().channels,
            "the timer does not have this capture and compare channel"
        );

        SimplePwmChannel {
            channel,
            load: self.load,
            counting_mode: self.counting_mode,
            phantom: PhantomData,
        }
    }

    /// Channel 0.
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    pub fn ch0(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch0)
    }

    /// Channel 1.
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    pub fn ch1(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch1)
    }

    /// Channel 2.
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    pub fn ch2(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch2)
    }

    /// Channel 3.
    ///
    /// This is just a convenience wrapper around [`Self::channel`].
    pub fn ch3(&mut self) -> SimplePwmChannel<'_, T> {
        self.channel(Channel::Ch3)
    }

    /// Start the counter.
    pub fn start(&mut self) {
        regs::<T>().counterregs(0).ctrctl().modify(|w| {
            w.set_en(true);
        });
    }

    /// Stop the counter.
    ///
    /// Every output keeps the value it had when the counter was stopped.
    pub fn stop(&mut self) {
        regs::<T>().counterregs(0).ctrctl().modify(|w| {
            w.set_en(false);
        });
    }

    /// Set the PWM frequency.
    ///
    /// Note: when you call this, the max duty value changes, so you will have to call
    /// [`SimplePwmChannel::set_duty_cycle`] on all channels with the duty calculated based on the
    /// new max duty.
    pub fn set_frequency(&mut self, clock_source: ClockSel, frequency: u32) -> Result<(), ConfigError> {
        self.set_frequency_inner(&Config {
            clock_source,
            frequency,
            counting_mode: self.counting_mode,
        })?;

        Ok(())
    }

    /// Get the max duty value.
    ///
    /// This value depends on the configured frequency and the timer's clock source.
    pub fn max_duty_cycle(&self) -> u16 {
        max_duty_cycle(self.load, self.counting_mode)
    }

    fn set_frequency_inner(&mut self, config: &Config) -> Result<Timing, ConfigError> {
        let timing = Timing::compute::<T>(config)?;
        let r = regs::<T>();

        r.clkdiv().modify(|w| {
            // The RATIO field encodes "divide by N" as `N - 1` (0 => /1 .. 7 => /8).
            w.set_ratio((timing.divider - 1) as u8);
        });

        r.commonregs(0).cps().modify(|w| {
            w.set_pcnt((timing.prescaler - 1) as u8);
        });

        set_load_value::<T>(timing.load);
        self.load = timing.load;

        Ok(timing)
    }
}

impl<'d, T: Instance> Drop for SimplePwm<'d, T> {
    fn drop(&mut self) {
        self.stop();

        for pin in self.pins.iter().flatten() {
            pin.set_as_disconnected();
        }
    }
}

/// The counter setup needed to reach a requested frequency.
struct Timing {
    /// The TIMCLK divider, `1` to `8`.
    divider: u8,
    /// The counter clock prescaler, `1` to `256`.
    prescaler: u16,
    /// The counter load value.
    load: u32,
}

impl Timing {
    fn compute<T: Instance>(config: &Config) -> Result<Self, ConfigError> {
        if config.frequency == 0 {
            return Err(ConfigError::InvalidFrequency);
        }

        // The duty cycle is expressed as a `u16`, so a period longer than `u16::MAX` ticks would
        // only add counter resolution which cannot be reached through the duty cycle. This is why
        // the period of the 32-bit timers is limited the same way as the one of the 16-bit timers.
        let max_load = u16::MAX as u32 - 1;

        // While counting up the period is `load + 1` ticks long, while counting up and down it is
        // `2 * load` ticks long.
        let max_ticks = match config.counting_mode {
            CountingMode::EdgeAligned => max_load + 1,
            CountingMode::CenterAligned => max_load * 2,
        };

        // Amount of clock source ticks in one PWM period.
        let ticks = config.clock_source.frequency() / config.frequency;

        if ticks < 2 {
            // The frequency is too high for the clock source.
            return Err(ConfigError::InvalidFrequency);
        }

        let max_prescaler = if T::info().prescaler { 256 } else { 1 };

        // The clock source has to be divided by this much for the period to fit the counter.
        let division = ticks.div_ceil(max_ticks).max(1);

        if division > 8 * max_prescaler {
            // The frequency is too low for the clock source.
            return Err(ConfigError::InvalidFrequency);
        }

        let divider = division.div_ceil(max_prescaler).clamp(1, 8);
        let prescaler = division.div_ceil(divider);

        let ticks = ticks / (divider * prescaler);
        let load = match config.counting_mode {
            CountingMode::EdgeAligned => ticks - 1,
            CountingMode::CenterAligned => ticks / 2,
        };

        Ok(Self {
            divider: divider as u8,
            prescaler: prescaler as u16,
            load,
        })
    }
}

fn max_duty_cycle(load: u32, counting_mode: CountingMode) -> u16 {
    // The load value is limited by `Timing::compute`, so this always fits a `u16`.
    match counting_mode {
        CountingMode::EdgeAligned => (load + 1) as u16,
        CountingMode::CenterAligned => load as u16,
    }
}

fn regs<T: Instance>() -> Tim {
    T::info().regs
}

fn set_load_value<T: Instance>(value: u32) {
    // The PAC exposes LOAD/CCVAL as `u32`; the timer's bit width masks the write in hardware.
    regs::<T>().counterregs(0).load().write_value(value);
}

fn set_compare_value<T: Instance>(channel: Channel, value: u32) {
    regs::<T>().counterregs(0).cc(channel.index()).write_value(value);
}

fn compare_value<T: Instance>(channel: Channel) -> u32 {
    regs::<T>().counterregs(0).cc(channel.index()).read()
}
