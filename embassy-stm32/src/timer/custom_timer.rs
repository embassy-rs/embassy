use core::u16;

use embassy_hal_internal::Peri;
use stm32_metapac::timer::vals::FilterValue;

use crate::{
    dac::Ch2,
    timer::{
        Ch1, Ch3, Ch4, CoreInstance, GeneralInstance1Channel, GeneralInstance2Channel, GeneralInstance4Channel,
        TimerPin,
        low_level::{CountingMode, OutputCompareMode, Timer},
        simple_pwm::PwmPin,
    },
};

mod ch_mode {
    use crate::timer::{Channel, low_level::InputTISelection};

    use super::*;

    pub trait Mode<T: CoreInstance> {
        fn init(self, channel: Channel, tim: &mut Timer<'_, T>);
    }

    pub struct Unused;
    pub struct Input(pub(crate) FilterValue);
    pub struct Output(pub(crate) OutputCompareMode);

    impl<T: CoreInstance> Mode<T> for Unused {
        fn init(self, _channel: Channel, _tim: &mut Timer<'_, T>) {}
    }

    impl<T: GeneralInstance4Channel> Mode<T> for Input {
        fn init(self, channel: Channel, tim: &mut Timer<'_, T>) {
            tim.set_input_ti_selection(channel, InputTISelection::Normal);
            tim.set_input_capture_filter(channel, self.0);
        }
    }

    impl<T: GeneralInstance1Channel> Mode<T> for Output {
        fn init(self, channel: Channel, tim: &mut Timer<'_, T>) {
            tim.set_output_compare_mode(channel, self.0);
        }
    }
}

/// Used to construct a [CustomPwm]
pub struct CustomPwmBuilder<
    'd,
    T: CoreInstance,
    CH1: ch_mode::Mode<T>,
    CH2: ch_mode::Mode<T>,
    CH3: ch_mode::Mode<T>,
    CH4: ch_mode::Mode<T>,
> {
    tim: Peri<'d, T>,
    ch1: CH1,
    ch2: CH2,
    ch3: CH3,
    ch4: CH4,

    counting_mode: CountingMode,
    arr: u32,
    psc: u16,
}

impl<'d, T: CoreInstance> CustomPwmBuilder<'d, T, ch_mode::Unused, ch_mode::Unused, ch_mode::Unused, ch_mode::Unused> {
    /// Construct a [CustomPwmBuilder] which can be used to construct a [CustomPwm]
    pub fn new(tim: Peri<'d, T>) -> Self {
        Self {
            tim,
            ch1: ch_mode::Unused,
            ch2: ch_mode::Unused,
            ch3: ch_mode::Unused,
            ch4: ch_mode::Unused,

            counting_mode: CountingMode::EdgeAlignedUp,
            arr: u32::MAX,
            psc: 0,
        }
    }

    // TODO allow u32 too?
    pub fn period(mut self, arr: u16) -> Self {
        self.arr = arr as u32;
        self
    }

    pub fn prescaler(mut self, psc: u16) -> Self {
        self.psc = psc;
        self
    }
}

impl<'d, T: GeneralInstance1Channel, CH2: ch_mode::Mode<T>, CH3: ch_mode::Mode<T>, CH4: ch_mode::Mode<T>>
    CustomPwmBuilder<'d, T, ch_mode::Unused, CH2, CH3, CH4>
{
    pub fn ch1<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(PwmPin<'d, T, Ch1, A>),
        mode: OutputCompareMode,
    ) -> CustomPwmBuilder<'d, T, ch_mode::Output, CH2, CH3, CH4> {
        let CustomPwmBuilder {
            tim,
            ch1: _,
            ch2,
            ch3,
            ch4,
            counting_mode,
            arr,
            psc,
        } = self;
        CustomPwmBuilder {
            tim,
            ch1: ch_mode::Output(mode),
            ch2,
            ch3,
            ch4,
            counting_mode,
            arr,
            psc,
        }
    }
}

impl<'d, T: GeneralInstance2Channel, CH1: ch_mode::Mode<T>, CH3: ch_mode::Mode<T>, CH4: ch_mode::Mode<T>>
    CustomPwmBuilder<'d, T, CH1, ch_mode::Unused, CH3, CH4>
{
    pub fn ch2<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(PwmPin<'d, T, Ch2, A>),
        mode: OutputCompareMode,
    ) -> CustomPwmBuilder<'d, T, CH1, ch_mode::Output, CH3, CH4> {
        let CustomPwmBuilder {
            tim,
            ch1,
            ch2: _,
            ch3,
            ch4,
            counting_mode,
            arr,
            psc,
        } = self;
        CustomPwmBuilder {
            tim,
            ch1,
            ch2: ch_mode::Output(mode),
            ch3,
            ch4,
            counting_mode,
            arr,
            psc,
        }
    }
}

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode<T>, CH2: ch_mode::Mode<T>, CH4: ch_mode::Mode<T>>
    CustomPwmBuilder<'d, T, CH1, CH2, ch_mode::Unused, CH4>
{
    pub fn ch3<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(PwmPin<'d, T, Ch3, A>),
        mode: OutputCompareMode,
    ) -> CustomPwmBuilder<'d, T, CH1, CH2, ch_mode::Output, CH4> {
        let CustomPwmBuilder {
            tim,
            ch1,
            ch2,
            ch3: _,
            ch4,
            counting_mode,
            arr,
            psc,
        } = self;
        CustomPwmBuilder {
            tim,
            ch1,
            ch2,
            ch3: ch_mode::Output(mode),
            ch4,
            counting_mode,
            arr,
            psc,
        }
    }
}

impl<'d, T: GeneralInstance4Channel, CH1: ch_mode::Mode<T>, CH2: ch_mode::Mode<T>, CH3: ch_mode::Mode<T>>
    CustomPwmBuilder<'d, T, CH1, CH2, CH3, ch_mode::Unused>
{
    pub fn ch4<#[cfg(afio)] A>(
        self,
        _pin: if_afio!(PwmPin<'d, T, Ch4, A>),
        mode: OutputCompareMode,
    ) -> CustomPwmBuilder<'d, T, CH1, CH2, CH3, ch_mode::Output> {
        let CustomPwmBuilder {
            tim,
            ch1,
            ch2,
            ch3,
            ch4: _,
            counting_mode,
            arr,
            psc,
        } = self;
        CustomPwmBuilder {
            tim,
            ch1,
            ch2,
            ch3,
            ch4: ch_mode::Output(mode),
            counting_mode,
            arr,
            psc,
        }
    }

    pub fn ch4_input<C, #[cfg(afio)] A>(
        self,
        _pin: if_afio!(impl TimerPin<T, Ch4, A>),
        filter: FilterValue,
    ) -> CustomPwmBuilder<'d, T, CH1, CH2, CH3, ch_mode::Input> {
        let CustomPwmBuilder {
            tim,
            ch1,
            ch2,
            ch3,
            ch4: _,
            counting_mode,
            arr,
            psc,
        } = self;
        CustomPwmBuilder {
            tim,
            ch1,
            ch2,
            ch3,
            ch4: ch_mode::Input(filter),
            counting_mode,
            arr,
            psc,
        }
    }
}

impl<'d, T: GeneralInstance1Channel, CH1: ch_mode::Mode<T>, CH2: ch_mode::Mode<T>, CH3: ch_mode::Mode<T>>
    CustomPwmBuilder<'d, T, CH1, CH2, CH3, ch_mode::Unused>
{
    pub fn finalize(self) -> CustomPwm<'d, T> {
        use ch_mode::Mode;
        let mut inner = Timer::new(self.tim);

        self.ch1.init(super::Channel::Ch1, &mut inner);
        self.ch2.init(super::Channel::Ch2, &mut inner);
        self.ch3.init(super::Channel::Ch3, &mut inner);
        self.ch4.init(super::Channel::Ch4, &mut inner);

        //inner.set_counting_mode(self.counting_mode);
        inner.set_max_compare_value(self.arr);
        inner.set_prescaler(self.psc);

        CustomPwm { inner }
    }
}

pub struct CustomPwm<'d, T: CoreInstance> {
    inner: Timer<'d, T>,
}
