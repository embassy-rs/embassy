//! Simple PWM driver.

use core::convert::Infallible;
use core::marker::PhantomData;

use embassy_hal_internal::Peri;

use crate::gpio::{AnyPin, PfType, Pull, SealedPin};
use crate::pac::tim::{Tim as Regs, vals};
use crate::tim::{
    self, Ch0, Ch1, Ch2, Ch3, ClockSel, CounterWidth, CountingMode, General2ChannelInstance, General4ChannelInstance,
    Info, Instance, TimerBits, TimerPin,
};

/// Marker type for a PWM instance with 2 channels.
pub enum TwoChannels {}

/// Marker type for a PWM instance with 4 channels.
pub enum FourChannels {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The desired frequency can not be reached with the selected clock source and timer.
    InvalidFrequency,
}

/// Simple PWM configuration.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// Clock source for timer.
    pub clock_source: ClockSel,

    /// Frequency at which to count the timer.
    pub frequency: u32,

    /// The counting mode of the timer.
    pub counting_mode: CountingMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clock_source: ClockSel::LfClk,
            frequency: 1000,
            counting_mode: CountingMode::EdgeAlignedUp,
        }
    }
}

/// Simple PWM driver.
///
/// This type has 2 generics:
/// - Channels: the number of channels the simple PWM instance supports
/// - Bits: the size of the counting registers in bits.
///
/// It is not possible to change the pwm frequency because the frequency configuration is shared with all channels.
#[must_use = "Dropping a simple pwm will disable the timer and all channels"]
pub struct SimplePwm<'d, Channels, Bits> {
    info: &'static Info,
    ch0: Option<Peri<'d, AnyPin>>,
    ch1: Option<Peri<'d, AnyPin>>,
    ch2: Option<Peri<'d, AnyPin>>,
    ch3: Option<Peri<'d, AnyPin>>,
    _marker: PhantomData<(Channels, Bits)>,
}

impl<'d, Channels, Bits: TimerBits> SimplePwm<'d, Channels, Bits> {
    /// Set PWM frequency.
    ///
    /// The actual frequency may differ from the requested value due to hardware limitations. The timer will round
    /// towards a slower (longer) period.
    pub fn set_frequency(&mut self, mode: CountingMode, freq: u32) -> Result<(), Error> {
        if freq == 0 {
            return Err(Error::InvalidFrequency);
        }

        let regs = self.info.regs;

        // Stop counting while we change frequency.
        regs.counterregs(0).ctrctl().modify(|w| {
            w.set_en(false);
        });

        let src_freq = {
            let clksel = regs.clksel().read();

            if clksel.lfclk_sel() {
                3_2768_u32
            } else if clksel.mfclk_sel() {
                4_000_000_u32
            } else if clksel.busclk_sel() {
                todo!("BusClk support: depends on power domain and sysctl clock config")
            } else {
                unreachable!()
            }
        };

        // Some timers do not have a prescaler. Prescaler is always fixed to 1.
        let has_prescale = self.info.prescaler;
        let max_load = match self.info.width {
            CounterWidth::Bits16 => u16::MAX as u32,
            CounterWidth::Bits32 => u32::MAX,
        };

        // Equation:
        //     fPWM = fSRC / (prescaler * divideRatio * period)
        //
        // rearrange for period:
        //     period = fSRC / (prescaler * divideRatio)
        //
        // for edge aligned counting:
        //     period = load - 1
        //     load = 1 + (fSRC / (prescaler * divideRatio * fPWM))
        //
        // for center aligned counting:
        //    period = load / 2
        //    load = fSRC / (prescaler * divideRatio * fPWM * 2)
        const DIVIDERS: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8];

        // ratio, prescaler, load
        let mut result = None::<(u8, u8, u32)>;

        'outer: for ratio in DIVIDERS {
            for prescaler in 0..u8::MAX {
                // If a timer has no prescaler then assume it is fixed.
                if !has_prescale && prescaler > 0 {
                    continue 'outer;
                }

                trace!("Trying ratio: {} prescale: {}", ratio, prescaler);

                let ratio = *ratio as u64;
                let prescaler = (prescaler as u64) + 1;
                // ratio * prescaler will never overflow. frequency could overflow during multiply, so do u64 multiplications to prevent runtime overflow.
                let Some(mut denom) = (ratio * prescaler).checked_mul(freq as u64) else {
                    trace!("Denominator overflow trying to calculate load for {} Hz", freq);
                    continue;
                };

                if mode == CountingMode::CenterAligned {
                    // Effectively double the frequency of the timer when using up down counting.
                    let Some(denom_updown) = denom.checked_mul(2) else {
                        trace!("Denominator overflow trying to calculate load for {} Hz", freq);
                        continue;
                    };

                    denom = denom_updown;
                }

                let mut load = src_freq as u64 / denom;

                if mode != CountingMode::CenterAligned {
                    load = load.saturating_sub(1);
                }

                // A load value of 0 will not count.
                //
                // If the timer can't represent the required load value then we need more division.
                if load == 0 || load > max_load as u64 {
                    trace!("Invalid load value for {} Hz", freq);
                    continue;
                }

                result = Some((ratio as u8, (prescaler - 1) as u8, load as u32));
                break 'outer;
            }
        }

        let (ratio, prescaler, load) = result.ok_or(Error::InvalidFrequency)?;
        trace!(
            "Selected: ratio (enum): {}, prescaler: {}, load: {}",
            ratio as u8, prescaler, load
        );

        regs.counterregs(0).load().write_value(load);

        regs.clkdiv().write(|w| {
            // Register is ratio + 1
            w.set_ratio(ratio - 1);
        });

        regs.commonregs(0).cps().write(|w| {
            w.set_pcnt(prescaler);
        });

        // Adjust the actions for new counting mode.
        //
        // While this applies to every channel the channel pin needs to be initialize for a change to be visible.
        for ch in 0..self.info.channels.min(4) {
            let (zero_action, load_action, up_action, down_action) = match mode {
                CountingMode::EdgeAlignedUp => (
                    vals::Act::CcpHigh,
                    vals::Act::Disabled,
                    vals::Act::CcpLow,
                    vals::Act::Disabled,
                ),
                CountingMode::EdgeAlignedDown => (
                    vals::Act::Disabled,
                    vals::Act::CcpHigh,
                    vals::Act::Disabled,
                    vals::Act::CcpLow,
                ),
                CountingMode::CenterAligned => (
                    vals::Act::Disabled,
                    vals::Act::Disabled,
                    vals::Act::CcpToggle,
                    vals::Act::CcpToggle,
                ),
            };

            let ccact = regs.counterregs(0).ccact(ch as usize);
            ccact.write(|w| {
                w.set_zact(zero_action);
                w.set_lact(load_action);
                w.set_cuact(up_action);
                w.set_cdact(down_action);
            });
        }

        // Restart counting
        regs.counterregs(0).ctrctl().modify(|w| {
            w.set_en(true);
            w.set_cm(match mode {
                CountingMode::EdgeAlignedUp => vals::Cm::Up,
                CountingMode::EdgeAlignedDown => vals::Cm::Down,
                CountingMode::CenterAligned => vals::Cm::UpDown,
            });
        });

        Ok(())
    }

    /// Get the frequency of the PWM output.
    pub fn get_frequency(&self) -> u32 {
        get_frequency_inner(self.info.regs)
    }

    /// Get the max duty cycle that each channel can provide.
    pub fn max_duty_cycle(&self) -> u32 {
        max_duty_cycle_inner(self.info.regs)
    }

    /// Channel 0
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch0(&mut self) -> SimplePwmChannel<'d, Bits> {
        SimplePwmChannel {
            info: self.info,
            index: 0,
            _marker: PhantomData,
        }
    }

    /// Channel 1
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch1(&mut self) -> SimplePwmChannel<'d, Bits> {
        SimplePwmChannel {
            info: self.info,
            index: 1,
            _marker: PhantomData,
        }
    }
}

impl<'d, Bits: TimerBits> SimplePwm<'d, TwoChannels, Bits> {
    /// Splits a [`SimplePwm`] into two pwm channels.
    ///
    /// This returns all two channels, including channels that
    /// aren't configured with a [`PwmPin`].
    pub fn split(&mut self) -> SimplePwm2Channels<'d, Bits> {
        SimplePwm2Channels {
            ch0: SimplePwmChannel {
                info: self.info,
                index: 0,
                _marker: PhantomData,
            },
            ch1: SimplePwmChannel {
                info: self.info,
                index: 1,
                _marker: PhantomData,
            },
        }
    }
}

impl<'d, Bits: TimerBits> SimplePwm<'d, FourChannels, Bits> {
    /// Channel 2
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch2(&mut self) -> SimplePwmChannel<'d, Bits> {
        SimplePwmChannel {
            info: self.info,
            index: 2,
            _marker: PhantomData,
        }
    }

    /// Channel 3
    ///
    /// If you need to use multiple channels, use [`Self::split`].
    pub fn ch3(&mut self) -> SimplePwmChannel<'d, Bits> {
        SimplePwmChannel {
            info: self.info,
            index: 3,
            _marker: PhantomData,
        }
    }

    /// Splits a [`SimplePwm`] into four pwm channels.
    ///
    /// This returns all four channels, including channels that
    /// aren't configured with a [`PwmPin`].
    pub fn split(&mut self) -> SimplePwm4Channels<'d, Bits> {
        SimplePwm4Channels {
            ch0: SimplePwmChannel {
                info: self.info,
                index: 0,
                _marker: PhantomData,
            },
            ch1: SimplePwmChannel {
                info: self.info,
                index: 1,
                _marker: PhantomData,
            },
            ch2: SimplePwmChannel {
                info: self.info,
                index: 2,
                _marker: PhantomData,
            },
            ch3: SimplePwmChannel {
                info: self.info,
                index: 3,
                _marker: PhantomData,
            },
        }
    }
}

impl<'d> SimplePwm<'d, TwoChannels, u16> {
    /// Create a new simple PWM instance with 2 channels with a 16-bit counter.
    ///
    /// This can be used with any timer that supports PWM outputs. A 32-bit timer or a timer with 4 output channels can
    /// always be used in this function.
    pub fn new_2ch_16bit<T: General2ChannelInstance>(
        _tim: Peri<'d, T>,
        ch0: Option<PwmPin<'d, T, Ch0>>,
        ch1: Option<PwmPin<'d, T, Ch1>>,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_inner(
            T::info(),
            config,
            ch0.and_then(|mut ch| ch.pin.take()),
            ch1.and_then(|mut ch| ch.pin.take()),
            None,
            None,
        )
    }
}

impl<'d> SimplePwm<'d, FourChannels, u16> {
    /// Create a new simple PWM instance with 4 channels with a 16-bit counter.
    ///
    /// This will only work for timers which have 4 PWM channels.
    pub fn new_4ch_16bit<T: General4ChannelInstance>(
        _tim: Peri<'d, T>,
        ch0: Option<PwmPin<'d, T, Ch0>>,
        ch1: Option<PwmPin<'d, T, Ch1>>,
        ch2: Option<PwmPin<'d, T, Ch2>>,
        ch3: Option<PwmPin<'d, T, Ch3>>,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_inner(
            T::info(),
            config,
            ch0.and_then(|mut ch| ch.pin.take()),
            ch1.and_then(|mut ch| ch.pin.take()),
            ch2.and_then(|mut ch| ch.pin.take()),
            ch3.and_then(|mut ch| ch.pin.take()),
        )
    }
}

impl<'d, Channels, Bits> Drop for SimplePwm<'d, Channels, Bits> {
    fn drop(&mut self) {
        if let Some(ref mut ch0) = self.ch0 {
            ch0.set_as_disconnected();
        }

        if let Some(ref mut ch1) = self.ch1 {
            ch1.set_as_disconnected();
        }

        if let Some(ref mut ch2) = self.ch2 {
            ch2.set_as_disconnected();
        }

        if let Some(ref mut ch3) = self.ch3 {
            ch3.set_as_disconnected();
        }

        // Normally a write to CCLKCTL.CLKEN would stop the timer.
        //
        // However due to errata TIMER_ERR_06 instead stop the timer by writing CTRCTL.EN.
        self.info.regs.counterregs(0).ctrctl().modify(|w| {
            w.set_en(false);
        });

        // Power gate the timer to reduce power consumption (even if the TI SDK claims it does not save much).
        self.info.regs.gprcm(0).pwren().write(|w| {
            w.set_enable(false);
            w.set_key(vals::PwrenKey::Key);
        });
    }
}

/// A single channel of a pwm, obtained from [`SimplePwm::split`], [`SimplePwm::ch1`], etc.
///
/// It is not possible to change the pwm frequency because the frequency configuration is shared with all channels.
#[must_use = "Dropping a channel will disable it"]
pub struct SimplePwmChannel<'d, Bits: TimerBits> {
    info: &'static Info,
    index: u8,
    _marker: PhantomData<(&'d (), Bits)>,
}

impl<'d, Bits: TimerBits> SimplePwmChannel<'d, Bits> {
    /// Enable the channel.
    pub fn enable(&mut self) {
        // Because channels can exist on multiple threads and all output disables exist in the same register,
        // we must guard the enable with a critical section.
        critical_section::with(|_| {
            self.info.regs.commonregs(0).odis().modify(|w| {
                w.set_c0ccp(self.index as usize, false);
            });
        });
    }

    /// Disable the channel.
    pub fn disable(&mut self) {
        // Because channels can exist on multiple threads and all output disables exist in the same register,
        // we must guard the enable with a critical section.
        critical_section::with(|_| {
            self.info.regs.commonregs(0).odis().modify(|w| {
                w.set_c0ccp(self.index as usize, true);
            });
        });
    }

    /// Is the channel enabled?
    pub fn is_enabled(&self) -> bool {
        !self.info.regs.commonregs(0).odis().read().c0ccp(self.index as usize)
    }

    /// Get the frequency of the PWM peripheral providing this channel.
    pub fn get_frequency(&self) -> u32 {
        get_frequency_inner(self.info.regs)
    }

    /// Set the duty cycle of this channel to be fully off.
    pub fn set_duty_cycle_fully_off(&mut self) {
        set_duty_cycle_inner(self.info.regs, self.index, 0);
    }

    /// Set the duty cycle of this channel to be fully on.
    pub fn set_duty_cycle_fully_on(&mut self) {
        let duty = max_duty_cycle_inner(self.info.regs);
        set_duty_cycle_inner(self.info.regs, self.index, duty);
    }

    // TODO: fractional duty cycle

    // TODO: Output polarity

    // TODO: Output compare mode?
}

impl<'d> SimplePwmChannel<'d, u16> {
    /// Get the max duty cycle of the channel.
    ///
    /// This value is equivalent to a 100% duty cycle.
    pub fn max_duty_cycle(&self) -> u16 {
        // This will never return a value greater than u16::MAX becauses Bits = u16
        max_duty_cycle_inner(self.info.regs) as u16
    }

    /// Set the duty cycle of this channel
    ///
    /// This must be a value equal to or less than [`SimplePwmChannel::max_duty_cycle`].
    pub fn set_duty_cycle(&mut self, duty: u16) {
        set_duty_cycle_inner(self.info.regs, self.index, duty as u32)
    }

    /// Get the current duty cycle of this channel.
    pub fn current_duty_cycle(&self) -> u16 {
        // This will never return a value greater than u16::MAX becauses Bits = u16
        current_duty_cycle_inner(self.info.regs, self.index) as u16
    }
}

impl<'d> SimplePwmChannel<'d, u32> {
    /// Get the max duty cycle of the channel.
    ///
    /// This value is equivalent to a 100% duty cycle.
    pub fn max_duty_cycle(&mut self) -> u32 {
        max_duty_cycle_inner(self.info.regs)
    }

    /// Set the duty cycle of this channel
    ///
    /// This must be a value equal to or less than [`SimplePwmChannel::max_duty_cycle`].
    pub fn set_duty_cycle(&mut self, duty: u32) {
        set_duty_cycle_inner(self.info.regs, self.index, duty)
    }

    /// Get the current duty cycle of this channel.
    pub fn current_duty_cycle(&self) -> u32 {
        current_duty_cycle_inner(self.info.regs, self.index)
    }
}

impl<'d, Bits: TimerBits> embedded_hal::pwm::ErrorType for SimplePwmChannel<'d, Bits> {
    // FIXME: Add error type since duty could be too high.
    type Error = Infallible;
}

impl<'d, Bits: TimerBits> embedded_hal::pwm::SetDutyCycle for SimplePwmChannel<'d, Bits> {
    fn max_duty_cycle(&self) -> u16 {
        // TODO: The hardware could actually have u32::MAX here, which would be a lie when clamped to u16.
        // maybe this means we can only implement this for Bits = u16?
        max_duty_cycle_inner(self.info.regs) as u16
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        set_duty_cycle_inner(self.info.regs, self.index, duty as u32);
        Ok(())
    }
}

impl<'d, Bits: TimerBits> Drop for SimplePwmChannel<'d, Bits> {
    /// Dropping a PWM channel will disable the channel.
    fn drop(&mut self) {
        self.disable();
    }
}

/// A group of two [`SimplePwmChannel`] from [`SimplePwm::split`]
///
/// This type is only available for a [`TwoChannels`] PWM.
pub struct SimplePwm2Channels<'d, Bits: TimerBits> {
    pub ch0: SimplePwmChannel<'d, Bits>,
    pub ch1: SimplePwmChannel<'d, Bits>,
}

/// A group of four [`SimplePwmChannel`] from [`SimplePwm::split`]
///
/// This type is only available for a [`FourChannels`] PWM.
pub struct SimplePwm4Channels<'d, Bits: TimerBits> {
    pub ch0: SimplePwmChannel<'d, Bits>,
    pub ch1: SimplePwmChannel<'d, Bits>,
    pub ch2: SimplePwmChannel<'d, Bits>,
    pub ch3: SimplePwmChannel<'d, Bits>,
}

/// PWM output pin wrapper.
///
/// This takes a pin and configures it to act as a PWM output.
///
/// # Drop
///
/// Dropping a pin will disconnect the PWM output.
pub struct PwmPin<'d, T: Instance, Channel: tim::TimerChannel> {
    /// Pin is wrapped by an [`Option`]. This is because [`SimplePwm`] when created will [`Option::take`] the value.
    /// This prevents the pins from being disconnected when moved into the pwm.
    pin: Option<Peri<'d, AnyPin>>,
    _marker: PhantomData<(T, Channel)>,
}

impl<'d, T: Instance, Channel: tim::TimerChannel> PwmPin<'d, T, Channel> {
    /// Create and configure the pin to act as a PWM output pin.
    pub fn new(pin: Peri<'d, impl TimerPin<T, Channel>>, pull: Pull, invert: bool) -> Self {
        // TODO: PfType config.
        let pin = new_pin!(pin, PfType::output(pull, invert));

        Self {
            pin,
            _marker: PhantomData,
        }
    }
}

impl<'d, T: Instance, Channel: tim::TimerChannel> Drop for PwmPin<'d, T, Channel> {
    fn drop(&mut self) {
        if let Some(ref mut pin) = self.pin {
            pin.set_as_disconnected();
        }
    }
}

impl<'d, Channels, Bits: TimerBits> SimplePwm<'d, Channels, Bits> {
    fn new_inner(
        info: &'static Info,
        config: Config,
        ch0: Option<Peri<'d, AnyPin>>,
        ch1: Option<Peri<'d, AnyPin>>,
        ch2: Option<Peri<'d, AnyPin>>,
        ch3: Option<Peri<'d, AnyPin>>,
    ) -> Result<Self, Error> {
        let regs = info.regs;

        // Reset
        regs.gprcm(0).rstctl().write(|w| {
            w.set_resetassert(true);
            w.set_resetstkyclr(true);
            w.set_key(vals::ResetKey::Key);
        });

        regs.gprcm(0).pwren().write(|w| {
            w.set_enable(true);
            w.set_key(vals::PwrenKey::Key);
        });

        // Wait a short period for peripheral init
        cortex_m::asm::delay(16);

        regs.clksel().write(|w| {
            w.set_lfclk_sel(config.clock_source == ClockSel::LfClk);
            w.set_mfclk_sel(config.clock_source == ClockSel::MfClk);
            // TODO: Bus clock when enabled for timers
            w.set_busclk_sel(false);
        });

        // Ungate the clock for the timer.
        regs.commonregs(0).cclkctl().write(|w| {
            w.set_clken(true);
        });

        regs.counterregs(0).ctrctl().write(|w| {
            // Do not enable timer yet.
            w.set_en(false);
            w.set_repeat(vals::Repeat::Repeat1);
            // CM is set later
            w.set_cvae(vals::Cvae::Ldval);

            // Because some timers have QEI capability and reset value is not zero.
            // These must be set or some timers will not count.
            //
            // We must also pick the lowest CC instance in use so that the timer will advance.
            if let Some((lowest_enabled, _)) = [ch0.is_some(), ch1.is_some(), ch2.is_some(), ch3.is_some()]
                .iter()
                .enumerate()
                .filter(|(_, present)| **present)
                .next()
            {
                let cxc = match lowest_enabled {
                    0 => vals::CxC::Cctl0,
                    1 => vals::CxC::Cctl1,
                    2 => vals::CxC::Cctl2,
                    3 => vals::CxC::Cctl3,
                    _ => unreachable!(),
                };

                w.set_czc(cxc);
                w.set_cac(cxc);
                w.set_clc(cxc);
            }
        });

        for (ch, _) in [ch0.is_some(), ch1.is_some(), ch2.is_some(), ch3.is_some()]
            .iter()
            .filter(|v| **v)
            .enumerate()
        {
            regs.counterregs(0).octl(ch).modify(|w| {
                w.set_ccpo(vals::Ccpo::Funcval);
                w.set_ccpiv(vals::Ccpiv::Low); // Low output when disabled (can be inverted using PwmPin)
                w.set_ccpoinv(false); // CCACT controls polarity
            });

            regs.counterregs(0).ccctl(ch).modify(|w| {
                w.set_coc(vals::Coc::Compare);
                w.set_acond(vals::Acond::Timclk);
                // TODO: LCOND, ZCOND reset value is reserved?
                w.set_lcond(vals::Lcond::CcTrigEdge);
                w.set_zcond(vals::Zcond::CcTrigEdge);
                w.set_ccond(vals::Ccond::CcTrigEdge);
                w.set_ccupd(vals::Ccupd::Immediately); // TI SDK sets this
            });
        }

        regs.commonregs(0).ccpd().modify(|w| {
            w.set_c0ccp(0, ch0.is_some());
            w.set_c0ccp(1, ch1.is_some());
            w.set_c0ccp(2, ch2.is_some());
            w.set_c0ccp(3, ch3.is_some());
        });

        let mut pwm = Self {
            info,
            ch0,
            ch1,
            ch2,
            ch3,
            _marker: PhantomData,
        };
        pwm.set_frequency(config.counting_mode, config.frequency)?;

        Ok(pwm)
    }
}

fn get_frequency_inner(r: Regs) -> u32 {
    let clksel = r.clksel().read();

    let src = if clksel.lfclk_sel() {
        32_768
    } else if clksel.mfclk_sel() {
        4_000_000
    } else if clksel.busclk_sel() {
        todo!("BusClk support")
    } else {
        unreachable!()
    };

    let clkdiv = r.clkdiv().read().ratio() + 1;
    // PCNT is u8, max prescale could overflow
    let prescale = (r.commonregs(0).cps().read().pcnt() as u16) + 1;
    let load = r.counterregs(0).load().read();

    // Equation:
    //     fPWM = fSRC / (prescaler * divideRatio * period)
    //
    // rearrange for period:
    //     period = fSRC / (prescaler * divideRatio)
    //
    // for edge aligned counting:
    //     period = load - 1
    //
    // for center aligned counting:
    //    period = load / 2

    let period = match r.counterregs(0).ctrctl().read().cm() {
        vals::Cm::Down | vals::Cm::Up => load as u64 + 1,
        vals::Cm::UpDown => load as u64 * 2,
        _ => unreachable!(),
    };

    let denom = (prescale as u64) * (clkdiv as u64) * period;

    (src as u64 / denom) as u32
}

fn max_duty_cycle_inner(r: Regs) -> u32 {
    r.counterregs(0).load().read().saturating_add(1)
}

fn set_duty_cycle_inner(r: Regs, index: u8, duty: u32) {
    assert!(duty <= max_duty_cycle_inner(r));
    r.counterregs(0).cc(index as usize).write_value(duty);
}

fn current_duty_cycle_inner(r: Regs, index: u8) -> u32 {
    r.counterregs(0).cc(index as usize).read()
}

#[cfg(test)]
mod tests {
    use crate::tim::simple_pwm::{FourChannels, SimplePwm, SimplePwmChannel, TwoChannels};

    /// Assert that a type is [`Send`] and [`Sync`].
    fn assert_send_sync<T: Send + Sync>() {}

    /// Check that notable types are [`Send`] and [`Sync`].
    #[test]
    fn is_send_sync() {
        assert_send_sync::<SimplePwm<'static, TwoChannels, u16>>();
        assert_send_sync::<SimplePwm<'static, FourChannels, u16>>();
        assert_send_sync::<SimplePwmChannel<'static, u16>>();

        assert_send_sync::<SimplePwm<'static, TwoChannels, u32>>();
        assert_send_sync::<SimplePwm<'static, FourChannels, u32>>();
        assert_send_sync::<SimplePwmChannel<'static, u32>>();
    }
}
