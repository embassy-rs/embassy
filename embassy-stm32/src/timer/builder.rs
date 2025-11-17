//! Custom timer driver.
//!
//! This is flexible timer driver for all STM32 timers. Tries to give quite unhindered access to
//! most timer features while offering some type level protection for incorrect configuration.
//!
//! The available functionality depends on the timer type.
//!
use core::u16;

use embassy_hal_internal::Peri;
use stm32_metapac::timer::vals::{Etp, Etps, FilterValue, Ts};

use crate::{
    time::Hertz,
    timer::{
        self, Ch1, CoreInstance, ExternalTriggerPin, GeneralInstance4Channel, TimerPin,
        low_level::{CountingMode, InputCaptureMode, InputTISelection, OutputCompareMode, SlaveMode, Timer},
        simple_pwm::PwmPin,
    },
};

enum ChannelMode {
    InternalOutput {
        duty: u16,
    },
    // TODO: Look into the capture mode configuration Timer::set_input_capture_mode
    Input {
        filter: FilterValue,
        mode: InputCaptureMode,
        ti_selection: InputTISelection,
        prescaler_factor: u8,
    },
    Output {
        mode: OutputCompareMode,
        duty: u16,
    },
}

impl ChannelMode {
    fn init<T: GeneralInstance4Channel>(self, channel: timer::Channel, tim: &mut Timer<'_, T>) {
        match self {
            ChannelMode::InternalOutput { duty } => {
                tim.set_output_compare_mode(channel, OutputCompareMode::Frozen);
                tim.set_compare_value(channel, duty as u32)
            }
            ChannelMode::Input {
                filter,
                mode,
                ti_selection,
                prescaler_factor,
            } => {
                tim.set_input_capture_filter(channel, filter);
                tim.set_input_capture_mode(channel, mode);
                tim.set_input_capture_prescaler(channel, prescaler_factor);
                tim.set_input_ti_selection(channel, ti_selection);
            }
            ChannelMode::Output { mode, duty } => {
                tim.set_output_compare_mode(channel, mode);
                tim.set_compare_value(channel, duty as u32)
            }
        }
    }
}

enum ExternalTrigger {
    Unused,
    Etr {
        filter: FilterValue,
        polarity: Etp,
        trigger_prescaler: Etps,
    },
}

impl ExternalTrigger {
    fn init<T: GeneralInstance4Channel>(self, tim: &mut Timer<'_, T>) {
        match self {
            ExternalTrigger::Unused => (),
            ExternalTrigger::Etr {
                filter,
                polarity,
                trigger_prescaler,
            } => {
                tim.regs_gp16().af1().modify(|w| w.set_etrsel(0)); // 0: ETR input
                tim.regs_gp16().smcr().modify(|w| {
                    w.set_etf(filter);
                    w.set_ece(false); // <--- TODO: I really need to look into how to set this and the SMS bits
                    w.set_etp(polarity);
                    w.set_etps(trigger_prescaler);
                });
            }
        }
    }
}

enum Speed {
    Hertz(Hertz),
    Manual { arr: u32, psc: u16 },
}

/// Used to construct a [Timer]
pub struct TimerBuilder<'d, T: CoreInstance> {
    tim: Peri<'d, T>,
    ch1: ChannelMode,
    ch2: ChannelMode,
    ch3: ChannelMode,
    ch4: ChannelMode,
    etr: ExternalTrigger,
    trig_source: Ts,
    slave_mode: SlaveMode,

    one_pulse_mode: bool,
    counting_mode: CountingMode,
    speed: Speed,
}

impl<'d, T: CoreInstance> TimerBuilder<'d, T> {
    /// Construct a [CustomPwmBuilder] which can be used to construct a [CustomPwm]
    pub fn new(tim: Peri<'d, T>) -> Self {
        Self {
            tim,
            ch1: ChannelMode::InternalOutput { duty: 0 },
            ch2: ChannelMode::InternalOutput { duty: 0 },
            ch3: ChannelMode::InternalOutput { duty: 0 },
            ch4: ChannelMode::InternalOutput { duty: 0 },
            etr: ExternalTrigger::Unused,
            one_pulse_mode: false,
            counting_mode: CountingMode::EdgeAlignedUp,
            speed: Speed::Manual { arr: u32::MAX, psc: 0 },
            trig_source: Ts::ITR0,
            slave_mode: SlaveMode::DISABLED,
        }
    }
}

impl<'d, T: CoreInstance> TimerBuilder<'d, T> {
    /// Set manually frequency by specifying prescaler and period
    pub fn prescaler_and_period(mut self, prescaler: u16, period_ticks: u32) -> Self {
        self.speed = Speed::Manual {
            arr: period_ticks,
            psc: prescaler,
        };
        self
    }

    /// Set frequency
    pub fn frequency(mut self, hz: Hertz) -> Self {
        self.speed = Speed::Hertz(hz);
        self
    }

    /// Set one pulse mode
    pub fn one_pulse_mode(mut self) -> Self {
        self.one_pulse_mode = true;
        self
    }
}

impl<'d, T: GeneralInstance4Channel> TimerBuilder<'d, T> {
    /// Setup channel 1 as output
    pub fn etr<#[cfg(afio)] A>(
        mut self,
        _pin: Peri<'d, if_afio!(impl ExternalTriggerPin<T, A>)>,
        filter: FilterValue,
        polarity: Etp,
        trigger_prescaler: Etps,
    ) -> Self {
        self.etr = ExternalTrigger::Etr {
            filter,
            polarity,
            trigger_prescaler,
        };
        self
    }

    /// Setup timer to be triggered from ch1 compare match event
    pub fn etr_as_trigger<#[cfg(afio)] A>(
        self,
        pin: Peri<'d, if_afio!(impl ExternalTriggerPin<T, A>)>,
        filter: FilterValue,
        polarity: Etp,
        trigger_prescaler: Etps,
        mode: TriggerMode,
    ) -> Self {
        let mut s = self.etr(pin, filter, polarity, trigger_prescaler);
        s.trig_source = Ts::TI1F_ED;
        s.slave_mode = match mode {
            TriggerMode::ResetMode => SlaveMode::RESET_MODE,
            TriggerMode::GatedMode => SlaveMode::GATED_MODE,
            TriggerMode::TriggerMode => SlaveMode::TRIGGER_MODE,
            TriggerMode::ExternalClockMode => SlaveMode::EXT_CLOCK_MODE,
        };
        s
    }
}

impl<'d, T: GeneralInstance4Channel> TimerBuilder<'d, T> {
    /// Set ch1 to be used as internal output, can be used as time base etc
    pub fn ch1_internal(mut self, duty: u16) -> Self {
        self.ch1 = ChannelMode::InternalOutput { duty };
        self
    }

    /// Setup channel 1 as output
    pub fn ch1<#[cfg(afio)] A>(
        mut self,
        _pin: if_afio!(PwmPin<'d, T, Ch1, A>),
        mode: OutputCompareMode,
        duty: u16,
    ) -> Self {
        self.ch1 = ChannelMode::Output { mode, duty };
        self
    }

    /// Setup channel 1 as input
    pub fn ch1_input<#[cfg(afio)] A>(
        self,
        _pin: Peri<'d, if_afio!(impl TimerPin<T, Ch1, A>)>,
        filter: FilterValue,
        mode: InputCaptureMode,
        ti_selection: InputTISelection,
        prescaler_factor: u8,
    ) -> Self {
        let mut s = self;
        s.ch1 = ChannelMode::Input {
            filter,
            mode,
            ti_selection,
            prescaler_factor,
        };
        s
    }

    /// Setup channel 1 as input and set this as this timers trigger source
    pub fn ch1_input_as_trigger<#[cfg(afio)] A>(
        self,
        pin: Peri<'d, if_afio!(impl TimerPin<T, Ch1, A>)>,
        filter: FilterValue,
        capture_mode: InputCaptureMode,
        ti_selection: InputTISelection,
        prescaler_factor: u8,
        mode: TriggerMode,
        source: TriggerSource,
    ) -> Self {
        let mut s = self.ch1_input(pin, filter, capture_mode, ti_selection, prescaler_factor);
        s.trig_source = match source {
            TriggerSource::EdgeDetector => Ts::TI1F_ED,
            TriggerSource::Filtered => Ts::TI1FP1,
        };
        s.slave_mode = match mode {
            TriggerMode::ResetMode => SlaveMode::RESET_MODE,
            TriggerMode::GatedMode => SlaveMode::GATED_MODE,
            TriggerMode::TriggerMode => SlaveMode::TRIGGER_MODE,
            TriggerMode::ExternalClockMode => SlaveMode::EXT_CLOCK_MODE,
        };

        s
    }
}

/// Trigger mode
pub enum TriggerMode {
    /// Reset Mode - Rising edge of the selected trigger input (TRGI) reinitializes the counter and generates an update of the registers.
    ResetMode,
    /// Gated Mode - The counter clock is enabled when the trigger input (TRGI) is high. The counter stops (but is not reset) as soon as the trigger becomes low. Both start and stop of the counter are controlled.
    GatedMode,
    /// Trigger Mode - The counter starts at a rising edge of the trigger TRGI (but it is not reset). Only the start of the counter is controlled.
    TriggerMode,
    /// External Clock Mode 1 - Rising edges of the selected trigger (TRGI) clock the counter.
    ExternalClockMode,
}

/// Trigger source be connected to the channels filter output or direcly at the edge detector
pub enum TriggerSource {
    /// Connecto directly at edge detector
    EdgeDetector,

    /// Connec to to channels filtered output
    Filtered,
}

impl<'d, T: GeneralInstance4Channel> TimerBuilder<'d, T> {
    /// Finalize configuration and create the [CustomPwm]
    pub fn finalize(self) -> Timer<'d, T> {
        let mut timer = Timer::new(self.tim);

        self.ch1.init(super::Channel::Ch1, &mut timer);
        self.ch2.init(super::Channel::Ch2, &mut timer);
        self.ch3.init(super::Channel::Ch3, &mut timer);
        self.ch4.init(super::Channel::Ch4, &mut timer);
        self.etr.init(&mut timer);

        timer.set_trigger_source(self.trig_source);
        timer.set_slave_mode(self.slave_mode);

        timer.set_counting_mode(self.counting_mode);
        match self.speed {
            Speed::Hertz(hz) => timer.set_frequency(hz),
            Speed::Manual { arr, psc } => {
                timer.set_max_compare_value(arr);
                timer.set_prescaler(psc);
            }
        }
        timer.generate_update_event();

        timer.regs_core().cr1().modify(|r| r.set_opm(self.one_pulse_mode));
        timer.enable_outputs();
        timer.start();

        timer
    }
}
