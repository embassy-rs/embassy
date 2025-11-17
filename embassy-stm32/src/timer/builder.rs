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
        Ch1, Ch2, Ch3, Ch4, CoreInstance, ExternalTriggerPin, GeneralInstance4Channel, TimerPin,
        low_level::{CountingMode, InputCaptureMode, InputTISelection, OutputCompareMode, SlaveMode, Timer},
        simple_pwm::PwmPin,
    },
};

mod ch_mode {
    use crate::timer::{
        Channel,
        low_level::{InputCaptureMode, InputTISelection},
    };

    use super::*;

    pub trait Mode {
        fn init<T: GeneralInstance4Channel>(self, channel: Channel, tim: &mut Timer<'_, T>);
    }

    pub struct InternalOutput {
        pub(crate) duty: u16,
    }

    // TODO: Look into the capture mode configuration Timer::set_input_capture_mode
    pub struct Input {
        pub(crate) filter: FilterValue,
        pub(crate) mode: InputCaptureMode,
        pub(crate) ti_selection: InputTISelection,
        pub(crate) prescaler_factor: u8,
    }
    pub struct Output {
        pub(crate) mode: OutputCompareMode,
        pub(crate) duty: u16,
    }

    impl Mode for InternalOutput {
        fn init<T: GeneralInstance4Channel>(self, channel: Channel, tim: &mut Timer<'_, T>) {
            tim.set_output_compare_mode(channel, OutputCompareMode::Frozen);
            tim.set_compare_value(channel, self.duty as u32)
        }
    }

    impl Mode for Input {
        fn init<T: GeneralInstance4Channel>(self, channel: Channel, tim: &mut Timer<'_, T>) {
            tim.set_input_capture_filter(channel, self.filter);
            tim.set_input_capture_mode(channel, self.mode);
            tim.set_input_capture_prescaler(channel, self.prescaler_factor);
            tim.set_input_ti_selection(channel, self.ti_selection);
        }
    }

    impl Mode for Output {
        fn init<T: GeneralInstance4Channel>(self, channel: Channel, tim: &mut Timer<'_, T>) {
            tim.set_output_compare_mode(channel, self.mode);
            tim.set_compare_value(channel, self.duty as u32)
        }
    }
}

mod trigger_source {
    pub struct Internal;
    pub struct Ch1;
    pub struct Ch2;
    pub struct Etr;
}

mod external_trigger {
    use stm32_metapac::timer::vals::{Etp, Etps};

    use super::*;

    pub trait Trigger {
        fn init<T: GeneralInstance4Channel>(self, tim: &mut Timer<'_, T>);
    }
    pub struct Unused;
    pub struct Etr {
        pub(crate) filter: FilterValue,
        pub(crate) polarity: Etp,
        pub(crate) trigger_prescaler: Etps,
    }

    impl Trigger for Unused {
        fn init<T: GeneralInstance4Channel>(self, _tim: &mut Timer<'_, T>) {}
    }

    impl Trigger for Etr {
        fn init<T: GeneralInstance4Channel>(self, tim: &mut Timer<'_, T>) {
            tim.regs_gp16().af1().modify(|w| w.set_etrsel(0)); // 0: ETR input
            tim.regs_gp16().smcr().modify(|w| {
                w.set_etf(self.filter);
                w.set_ece(false); // <--- TODO: I really need to look into how to set this and the SMS bits
                w.set_etp(self.polarity);
                w.set_etps(self.trigger_prescaler);
            });
        }
    }
}

enum Speed {
    Hertz(Hertz),
    Manual { arr: u32, psc: u16 },
}

macro_rules! set_field {
    (
        $this:expr
        $(, tim: $tim:expr)*
        $(, ch1: $ch1:expr)*
        $(, ch2: $ch2:expr)*
        $(, ch3: $ch3:expr)*
        $(, ch4: $ch4:expr)*
        $(, etr: $etr:expr)*
        $(, one_pulse_mode: $one_pulse_mode:expr)*
        $(, counting_mode: $counting_mode:expr)*
        $(, speed: $speed:expr)*
        $(, trigger_source: $trigger_source:expr)*
        $(, trig_source: $trig_source:expr)*
        $(, slave_mode: $slave_mode:expr)*

    ) => {{
        #[allow(unused_variables)]
        let TimerBuilder {
            tim,
            ch1,
            ch2,
            ch3,
            ch4,
            etr,
            one_pulse_mode,
            counting_mode,
            speed,
            trigger_source,
            trig_source,
            slave_mode,
        } = $this;
        TimerBuilder {
            tim $(: $tim)*,
            ch1 $(: $ch1)*,
            ch2 $(: $ch2)*,
            ch3 $(: $ch3)*,
            ch4 $(: $ch4)*,
            etr $(: $etr)*,
            one_pulse_mode $(: $one_pulse_mode)*,
            counting_mode $(: $counting_mode)*,
            speed $(: $speed)*,
            trigger_source $(: $trigger_source)*,
            trig_source $(: $trig_source)*,
            slave_mode $(: $slave_mode)*,
        }}
    };
}

/// Used to construct a [Timer]
pub struct TimerBuilder<
    'd,
    T: CoreInstance,
    CH1: ch_mode::Mode,
    CH2: ch_mode::Mode,
    CH3: ch_mode::Mode,
    CH4: ch_mode::Mode,
    ETR,
    TS,
> {
    tim: Peri<'d, T>,
    ch1: CH1,
    ch2: CH2,
    ch3: CH3,
    ch4: CH4,
    etr: ETR,

    trigger_source: TS,
    trig_source: Ts,
    slave_mode: SlaveMode,

    one_pulse_mode: bool,
    counting_mode: CountingMode,
    speed: Speed,
}

impl<'d, T: CoreInstance>
    TimerBuilder<
        'd,
        T,
        ch_mode::InternalOutput,
        ch_mode::InternalOutput,
        ch_mode::InternalOutput,
        ch_mode::InternalOutput,
        external_trigger::Unused,
        trigger_source::Internal,
    >
{
    /// Construct a [CustomPwmBuilder] which can be used to construct a [CustomPwm]
    pub fn new(tim: Peri<'d, T>) -> Self {
        Self {
            tim,
            ch1: ch_mode::InternalOutput { duty: 0 },
            ch2: ch_mode::InternalOutput { duty: 0 },
            ch3: ch_mode::InternalOutput { duty: 0 },
            ch4: ch_mode::InternalOutput { duty: 0 },
            etr: external_trigger::Unused,
            one_pulse_mode: false,
            counting_mode: CountingMode::EdgeAlignedUp,
            speed: Speed::Manual { arr: u32::MAX, psc: 0 },
            trigger_source: trigger_source::Internal,
            trig_source: Ts::ITR0,
            slave_mode: SlaveMode::DISABLED,
        }
    }
}

impl<'d, T: CoreInstance, CH1: ch_mode::Mode, CH2: ch_mode::Mode, CH3: ch_mode::Mode, CH4: ch_mode::Mode, ETR, TS>
    TimerBuilder<'d, T, CH1, CH2, CH3, CH4, ETR, TS>
{
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

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode, CH2: ch_mode::Mode, CH3: ch_mode::Mode, CH4: ch_mode::Mode, TS>
    TimerBuilder<'d, T, CH1, CH2, CH3, CH4, external_trigger::Unused, TS>
{
    /// Setup channel 1 as output
    pub fn etr<#[cfg(afio)] A>(
        self,
        _pin: Peri<'d, if_afio!(impl ExternalTriggerPin<T, A>)>,
        filter: FilterValue,
        polarity: Etp,
        trigger_prescaler: Etps,
    ) -> TimerBuilder<'d, T, CH1, CH2, CH3, CH4, external_trigger::Etr, TS> {
        set_field!(self, etr: external_trigger::Etr{filter, polarity, trigger_prescaler })
    }
}

impl<'d, T: GeneralInstance4Channel, CH2: ch_mode::Mode, CH3: ch_mode::Mode, CH4: ch_mode::Mode, ETR, TS>
    TimerBuilder<'d, T, ch_mode::InternalOutput, CH2, CH3, CH4, ETR, TS>
{
    /// Set ch1 to be used as internal output, can be used as time base etc
    pub fn ch1_internal(self, duty: u16) -> Self {
        set_field!(self, ch1: ch_mode::InternalOutput { duty })
    }

    /// Setup channel 1 as output
    pub fn ch1<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(PwmPin<'d, T, Ch1, A>),
        mode: OutputCompareMode,
        duty: u16,
    ) -> TimerBuilder<'d, T, ch_mode::Output, CH2, CH3, CH4, ETR, TS> {
        set_field!(self, ch1: ch_mode::Output { mode, duty })
    }

    /// Setup channel 1 as input
    pub fn ch1_input<#[cfg(afio)] A>(
        self,
        _pin: Peri<'d, if_afio!(impl TimerPin<T, Ch1, A>)>,
        filter: FilterValue,
        mode: InputCaptureMode,
        ti_selection: InputTISelection,
        prescaler_factor: u8,
    ) -> TimerBuilder<'d, T, ch_mode::Input, CH2, CH3, CH4, ETR, TS> {
        set_field!(self, ch1: ch_mode::Input { filter, mode, ti_selection, prescaler_factor })
    }
}

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode, CH3: ch_mode::Mode, CH4: ch_mode::Mode, ETR, TS>
    TimerBuilder<'d, T, CH1, ch_mode::InternalOutput, CH3, CH4, ETR, TS>
{
    /// Set ch2 to be used as internal output, can be used as time base etc
    pub fn ch2_internal(self, duty: u16) -> Self {
        set_field!(self, ch2: ch_mode::InternalOutput { duty })
    }

    /// Setup channel 2 as output
    pub fn ch2<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(PwmPin<'d, T, Ch2, A>),
        mode: OutputCompareMode,
        duty: u16,
    ) -> TimerBuilder<'d, T, CH1, ch_mode::Output, CH3, CH4, ETR, TS> {
        set_field!(self, ch2: ch_mode::Output { mode, duty })
    }

    /// Setup channel 2 as input
    pub fn ch2_input<#[cfg(afio)] A>(
        self,
        _pin: Peri<'d, if_afio!(impl TimerPin<T, Ch2, A>)>,
        filter: FilterValue,
        mode: InputCaptureMode,
        ti_selection: InputTISelection,
        prescaler_factor: u8,
    ) -> TimerBuilder<'d, T, CH1, ch_mode::Input, CH3, CH4, ETR, TS> {
        set_field!(self, ch2: ch_mode::Input { filter, mode, ti_selection, prescaler_factor })
    }
}

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode, CH2: ch_mode::Mode, CH4: ch_mode::Mode, ETR, TS>
    TimerBuilder<'d, T, CH1, CH2, ch_mode::InternalOutput, CH4, ETR, TS>
{
    /// Set ch3 to be used as internal output, can be used as time base etc
    pub fn ch3_internal(self, duty: u16) -> Self {
        set_field!(self, ch3: ch_mode::InternalOutput { duty })
    }

    /// Setup channel 3 as output
    pub fn ch3<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(PwmPin<'d, T, Ch3, A>),
        mode: OutputCompareMode,
        duty: u16,
    ) -> TimerBuilder<'d, T, CH1, CH2, ch_mode::Output, CH4, ETR, TS> {
        set_field!(self, ch3: ch_mode::Output { mode, duty })
    }

    /// Setup channel 3 as input
    pub fn ch3_input<#[cfg(afio)] A>(
        self,
        _pin: Peri<'d, if_afio!(impl TimerPin<T, Ch3, A>)>,
        filter: FilterValue,
        mode: InputCaptureMode,
        ti_selection: InputTISelection,
        prescaler_factor: u8,
    ) -> TimerBuilder<'d, T, CH1, CH2, ch_mode::Input, CH4, ETR, TS> {
        set_field!(self, ch3: ch_mode::Input { filter, mode, ti_selection, prescaler_factor })
    }
}

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode, CH2: ch_mode::Mode, CH3: ch_mode::Mode, ETR, TS>
    TimerBuilder<'d, T, CH1, CH2, CH3, ch_mode::InternalOutput, ETR, TS>
{
    /// Set ch4 to be used as internal output, can be used as time base etc
    pub fn ch4_internal(self, duty: u16) -> Self {
        set_field!(self, ch4: ch_mode::InternalOutput { duty })
    }

    /// Setup channel 4 as output
    pub fn ch4<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(PwmPin<'d, T, Ch4, A>),
        mode: OutputCompareMode,
        duty: u16,
    ) -> TimerBuilder<'d, T, CH1, CH2, CH3, ch_mode::Output, ETR, TS> {
        set_field!(self, ch4: ch_mode::Output { mode, duty })
    }

    /// Setup channel 3 as input
    pub fn ch4_input<#[cfg(afio)] A>(
        self,
        _pin: Peri<'d, if_afio!(impl TimerPin<T, Ch3, A>)>,
        filter: FilterValue,
        mode: InputCaptureMode,
        ti_selection: InputTISelection,
        prescaler_factor: u8,
    ) -> TimerBuilder<'d, T, CH1, CH2, CH3, ch_mode::Input, ETR, TS> {
        set_field!(self, ch4: ch_mode::Input { filter, mode, ti_selection, prescaler_factor })
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

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode, CH2: ch_mode::Mode, CH3: ch_mode::Mode, CH4: ch_mode::Mode>
    TimerBuilder<'d, T, CH1, CH2, CH3, CH4, external_trigger::Etr, trigger_source::Internal>
{
    /// Setup timer to be triggered from ch1 compare match event
    pub fn trigger_from_etr(
        self,
        mode: TriggerMode,
    ) -> TimerBuilder<'d, T, CH1, CH2, CH3, CH4, external_trigger::Etr, trigger_source::Etr> {
        set_field!(self, trigger_source: trigger_source::Etr,
            trig_source: Ts::TI1F_ED,
            slave_mode: match mode {
                TriggerMode::ResetMode => SlaveMode::RESET_MODE,
                TriggerMode::GatedMode => SlaveMode::GATED_MODE,
                TriggerMode::TriggerMode => SlaveMode::TRIGGER_MODE,
                TriggerMode::ExternalClockMode => SlaveMode::EXT_CLOCK_MODE,
            }
        )
    }
}

impl<'d, T: GeneralInstance4Channel, CH2: ch_mode::Mode, CH3: ch_mode::Mode, CH4: ch_mode::Mode, ETR>
    TimerBuilder<'d, T, ch_mode::Input, CH2, CH3, CH4, ETR, trigger_source::Internal>
{
    /// Setup timer to be triggered from ch1 compare match event
    pub fn trigger_from_ch1(
        self,
        mode: TriggerMode,
        source: TriggerSource,
    ) -> TimerBuilder<'d, T, ch_mode::Input, CH2, CH3, CH4, ETR, trigger_source::Ch1> {
        set_field!(self, trigger_source: trigger_source::Ch1,
            trig_source: match source {
                TriggerSource::EdgeDetector => Ts::TI1F_ED,
                TriggerSource::Filtered => Ts::TI1FP1,
            },
            slave_mode: match mode {
                TriggerMode::ResetMode => SlaveMode::RESET_MODE,
                TriggerMode::GatedMode => SlaveMode::GATED_MODE,
                TriggerMode::TriggerMode => SlaveMode::TRIGGER_MODE,
                TriggerMode::ExternalClockMode => SlaveMode::EXT_CLOCK_MODE,
            }
        )
    }
}

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode, CH3: ch_mode::Mode, CH4: ch_mode::Mode, ETR>
    TimerBuilder<'d, T, CH1, ch_mode::Input, CH3, CH4, ETR, trigger_source::Internal>
{
    /// Setup timer to be triggered from ch1 compare match event
    pub fn trigger_from_ch2(
        self,
        mode: TriggerMode,
    ) -> TimerBuilder<'d, T, CH1, ch_mode::Input, CH3, CH4, ETR, trigger_source::Ch2> {
        set_field!(self, trigger_source: trigger_source::Ch2, trig_source: Ts::TI2FP2, slave_mode: match mode {
            TriggerMode::ResetMode => SlaveMode::RESET_MODE,
            TriggerMode::GatedMode => SlaveMode::GATED_MODE,
            TriggerMode::TriggerMode => SlaveMode::TRIGGER_MODE,
            TriggerMode::ExternalClockMode => SlaveMode::EXT_CLOCK_MODE,
        })
    }
}

impl<
    'd,
    T: GeneralInstance4Channel,
    CH1: ch_mode::Mode,
    CH2: ch_mode::Mode,
    CH3: ch_mode::Mode,
    ETR: external_trigger::Trigger,
    TS,
> TimerBuilder<'d, T, CH1, CH2, CH3, ch_mode::InternalOutput, ETR, TS>
{
    /// Finalize configuration and create the [CustomPwm]
    pub fn finalize(self) -> Timer<'d, T> {
        use ch_mode::Mode;
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
