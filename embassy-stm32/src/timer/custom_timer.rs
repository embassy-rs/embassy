//! Custom timer driver.
//!
//! This is flexible timer driver for all STM32 timers. Tries to give quite unhindered access to
//! most timer features while offering some type level protection for incorrect configuration.
//!
//! The available functionality depends on the timer type.
//!
use core::{marker::PhantomData, u16};

use embassy_hal_internal::Peri;
use stm32_metapac::timer::vals::{Etp, Etps, FilterValue, Ts};

use crate::{
    time::Hertz,
    timer::{
        Ch1, Ch2, Ch3, Ch4, CoreInstance, ExternalTriggerPin, GeneralInstance4Channel, TimerPin,
        input_capture::InputCaptureFuture,
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
        let CustomPwmBuilder {
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
        CustomPwmBuilder {
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

/// Used to construct a [CustomPwm]
pub struct CustomPwmBuilder<
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
    if_afio!(
        CustomPwmBuilder<
            'd,
            T,
            ch_mode::InternalOutput,
            ch_mode::InternalOutput,
            ch_mode::InternalOutput,
            ch_mode::InternalOutput,
            external_trigger::Unused,
            trigger_source::Internal,
        >
    )
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
    CustomPwmBuilder<'d, T, CH1, CH2, CH3, CH4, ETR, TS>
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
    CustomPwmBuilder<'d, T, CH1, CH2, CH3, CH4, external_trigger::Unused, TS>
{
    /// Setup channel 1 as output
    pub fn etr<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(impl ExternalTriggerPin<T, A>),
        filter: FilterValue,
        polarity: Etp,
        trigger_prescaler: Etps,
    ) -> CustomPwmBuilder<'d, T, CH1, CH2, CH3, CH4, external_trigger::Etr, TS> {
        set_field!(self, etr: external_trigger::Etr{filter, polarity, trigger_prescaler })
    }
}

impl<'d, T: GeneralInstance4Channel, CH2: ch_mode::Mode, CH3: ch_mode::Mode, CH4: ch_mode::Mode, ETR, TS>
    CustomPwmBuilder<'d, T, ch_mode::InternalOutput, CH2, CH3, CH4, ETR, TS>
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
    ) -> CustomPwmBuilder<'d, T, ch_mode::Output, CH2, CH3, CH4, ETR, TS> {
        set_field!(self, ch1: ch_mode::Output { mode, duty })
    }

    /// Setup channel 1 as input
    pub fn ch1_input<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(impl TimerPin<T, Ch1, A>),
        filter: FilterValue,
        mode: InputCaptureMode,
        ti_selection: InputTISelection,
        prescaler_factor: u8,
    ) -> CustomPwmBuilder<'d, T, ch_mode::Input, CH2, CH3, CH4, ETR, TS> {
        set_field!(self, ch1: ch_mode::Input { filter, mode, ti_selection, prescaler_factor })
    }
}

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode, CH3: ch_mode::Mode, CH4: ch_mode::Mode, ETR, TS>
    CustomPwmBuilder<'d, T, CH1, ch_mode::InternalOutput, CH3, CH4, ETR, TS>
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
    ) -> CustomPwmBuilder<'d, T, CH1, ch_mode::Output, CH3, CH4, ETR, TS> {
        set_field!(self, ch2: ch_mode::Output { mode, duty })
    }

    /// Setup channel 2 as input
    pub fn ch2_input<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(impl TimerPin<T, Ch2, A>),
        filter: FilterValue,
        mode: InputCaptureMode,
        ti_selection: InputTISelection,
        prescaler_factor: u8,
    ) -> CustomPwmBuilder<'d, T, CH1, ch_mode::Input, CH3, CH4, ETR, TS> {
        set_field!(self, ch2: ch_mode::Input { filter, mode, ti_selection, prescaler_factor })
    }
}

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode, CH2: ch_mode::Mode, CH4: ch_mode::Mode, ETR, TS>
    CustomPwmBuilder<'d, T, CH1, CH2, ch_mode::InternalOutput, CH4, ETR, TS>
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
    ) -> CustomPwmBuilder<'d, T, CH1, CH2, ch_mode::Output, CH4, ETR, TS> {
        set_field!(self, ch3: ch_mode::Output { mode, duty })
    }

    /// Setup channel 3 as input
    pub fn ch3_input<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(impl TimerPin<T, Ch3, A>),
        filter: FilterValue,
        mode: InputCaptureMode,
        ti_selection: InputTISelection,
        prescaler_factor: u8,
    ) -> CustomPwmBuilder<'d, T, CH1, CH2, ch_mode::Input, CH4, ETR, TS> {
        set_field!(self, ch3: ch_mode::Input { filter, mode, ti_selection, prescaler_factor })
    }
}

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode, CH2: ch_mode::Mode, CH3: ch_mode::Mode, ETR, TS>
    CustomPwmBuilder<'d, T, CH1, CH2, CH3, ch_mode::InternalOutput, ETR, TS>
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
    ) -> CustomPwmBuilder<'d, T, CH1, CH2, CH3, ch_mode::Output, ETR, TS> {
        set_field!(self, ch4: ch_mode::Output { mode, duty })
    }

    /// Setup channel 3 as input
    pub fn ch4_input<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(impl TimerPin<T, Ch3, A>),
        filter: FilterValue,
        mode: InputCaptureMode,
        ti_selection: InputTISelection,
        prescaler_factor: u8,
    ) -> CustomPwmBuilder<'d, T, CH1, CH2, CH3, ch_mode::Input, ETR, TS> {
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
    CustomPwmBuilder<'d, T, CH1, CH2, CH3, CH4, external_trigger::Etr, trigger_source::Internal>
{
    /// Setup timer to be triggered from ch1 compare match event
    pub fn trigger_from_etr(
        self,
        mode: TriggerMode,
    ) -> CustomPwmBuilder<'d, T, CH1, CH2, CH3, CH4, external_trigger::Etr, trigger_source::Etr> {
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
    CustomPwmBuilder<'d, T, ch_mode::Input, CH2, CH3, CH4, ETR, trigger_source::Internal>
{
    /// Setup timer to be triggered from ch1 compare match event
    pub fn trigger_from_ch1(
        self,
        mode: TriggerMode,
        source: TriggerSource,
    ) -> CustomPwmBuilder<'d, T, ch_mode::Input, CH2, CH3, CH4, ETR, trigger_source::Ch1> {
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

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode, CH3: ch_mode::Mode, CH4: ch_mode::Mode, ETR, #[cfg(afio)] A>
    CustomPwmBuilder<'d, T, CH1, ch_mode::Input, CH3, CH4, ETR, trigger_source::Internal>
{
    /// Setup timer to be triggered from ch1 compare match event
    pub fn trigger_from_ch2(
        self,
        mode: TriggerMode,
    ) -> CustomPwmBuilder<'d, T, CH1, ch_mode::Input, CH3, CH4, ETR, trigger_source::Ch2> {
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
    A,
> CustomPwmBuilder<'d, T, CH1, CH2, CH3, ch_mode::InternalOutput, ETR, TS>
{
    /// Finalize configuration and create the [CustomPwm]
    pub fn finalize(self) -> CustomPwm<'d, T> {
        use ch_mode::Mode;
        let mut inner = Timer::new(self.tim);

        self.ch1.init(super::Channel::Ch1, &mut inner);
        self.ch2.init(super::Channel::Ch2, &mut inner);
        self.ch3.init(super::Channel::Ch3, &mut inner);
        self.ch4.init(super::Channel::Ch4, &mut inner);
        self.etr.init(&mut inner);

        inner.set_trigger_source(self.trig_source);
        inner.set_slave_mode(self.slave_mode);

        inner.set_counting_mode(self.counting_mode);
        match self.speed {
            Speed::Hertz(hz) => inner.set_frequency(hz),
            Speed::Manual { arr, psc } => {
                inner.set_max_compare_value(arr);
                inner.set_prescaler(psc);
            }
        }
        inner.generate_update_event();

        inner.regs_core().cr1().modify(|r| r.set_opm(self.one_pulse_mode));
        inner.enable_outputs();
        inner.start();

        CustomPwm { inner }
    }
}

/// Use [CustomPwmBuilder::new] to create a new timer
pub struct CustomPwm<'d, T: CoreInstance> {
    pub(crate) inner: Timer<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> CustomPwm<'d, T> {
    /// Set compare value
    pub fn set_compare_value(&mut self, duty: u16, channel: super::Channel) {
        self.inner.set_compare_value(channel, duty as u32);
    }

    /// Generate a sequence of PWM waveform
    ///
    /// Note:
    /// you will need to provide corresponding TIMx_UP DMA channel to use this method.
    pub async fn waveform_up(&mut self, dma: Peri<'_, impl super::UpDma<T>>, channel: super::Channel, duty: &[u16]) {
        self.inner.waveform_up(dma, channel, duty).await;
    }

    /// Get capture value for a channel.
    pub fn get_capture_value(&self, channel: super::Channel) -> u32 {
        self.inner.get_capture_value(channel)
    }

    fn new_future(&self, channel: super::Channel) -> InputCaptureFuture<T> {
        self.inner.enable_channel(channel, true);
        self.inner.enable_input_interrupt(channel, true);

        InputCaptureFuture {
            channel,
            phantom: PhantomData,
        }
    }

    /// Asynchronously wait until the pin sees an edge as configured on timer init.
    pub async fn wait_for_configured_edge(&mut self, channel: super::Channel) -> u32 {
        self.new_future(channel).await
    }

    /// Asynchronously wait until the period event
    pub async fn wait_for_period(&mut self) -> u32 {
        todo!()
    }
}

async fn _example(
    tim: Peri<'_, crate::peripherals::TIM1>,
    dma: Peri<'_, impl super::UpDma<crate::peripherals::TIM1>>,
    trigger_pin: crate::peripherals::PA8,
    out_pin: Peri<'_, crate::peripherals::PA9>,
    capture_pin: crate::peripherals::PA10,
) {
    let out_pin: PwmPin<'_, _, _, crate::gpio::AfioRemap<0>> = PwmPin::new(out_pin, crate::gpio::OutputType::PushPull);

    let mut tim = CustomPwmBuilder::new(tim)
        //.frequency(Hertz(123))
        .prescaler_and_period(0, 1337)
        .ch1_input(
            trigger_pin,
            FilterValue::FDTS_DIV32_N8,
            InputCaptureMode::BothEdges,
            InputTISelection::Normal,
            1,
        )
        .trigger_from_ch1(TriggerMode::TriggerMode, TriggerSource::Filtered)
        .ch2(out_pin, OutputCompareMode::PwmMode2, 800)
        .ch3_input(
            capture_pin,
            FilterValue::FCK_INT_N2,
            InputCaptureMode::Rising,
            InputTISelection::Normal,
            0,
        )
        .one_pulse_mode()
        .finalize();

    tim.set_compare_value(150, super::Channel::Ch2);
    tim.waveform_up(dma, super::Channel::Ch1, &[100, 400, 800, 1100, 1200])
        .await;
    let _capture = tim.wait_for_configured_edge(super::Channel::Ch3).await;
}

async fn _example2(tim: Peri<'_, crate::peripherals::TIM1>, trigger_pin: crate::peripherals::PA12) {
    let mut tim = CustomPwmBuilder::<_, _, _, _, _, _, _, crate::gpio::AfioRemap<0>>::new(tim)
        //.frequency(Hertz(123))
        .prescaler_and_period(0, 1337)
        .etr(trigger_pin, FilterValue::FDTS_DIV32_N8, Etp::NOT_INVERTED, Etps::DIV1)
        .trigger_from_etr(TriggerMode::TriggerMode)
        .ch1_internal(1234)
        .one_pulse_mode()
        .finalize();

    // Should trigger 1234 ticks after PA12 goes high
    let _capture = tim.wait_for_configured_edge(super::Channel::Ch1).await;
}
