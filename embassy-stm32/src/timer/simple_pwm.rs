use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

use super::*;
#[allow(unused_imports)]
use crate::gpio::sealed::{AFType, Pin};
use crate::gpio::{AnyPin, OutputType};
use crate::time::Hertz;
use crate::Peripheral;
use crate::_generated::interrupt::typelevel::Interrupt;

// Declare a signal to awake user code for signaling the update interrupt id happen
static SIGNAL_UPDATE: Signal<CriticalSectionRawMutex, usize> = Signal::new();

pub struct InterruptHandler<T: CaptureCompare16bitInstance> {
    _phantom: PhantomData<T>,
}

impl<T: CaptureCompare16bitInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        let sr = regs.sr().read();
        if sr.uif() {
            SIGNAL_UPDATE.signal(0);
            // clear the flag
            critical_section::with(|_| {
                regs.sr().modify(|w| w.set_uif(false));
            })
        }
    }
}

pub struct Ch1;
pub struct Ch2;
pub struct Ch3;
pub struct Ch4;

pub struct PwmPin<'d, Perip, Channel> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(Perip, Channel)>,
}

macro_rules! channel_impl {
    ($new_chx:ident, $channel:ident, $pin_trait:ident) => {
        impl<'d, Perip: CaptureCompare16bitInstance> PwmPin<'d, Perip, $channel> {
            pub fn $new_chx(pin: impl Peripheral<P = impl $pin_trait<Perip>> + 'd, output_type: OutputType) -> Self {
                into_ref!(pin);
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(pin.af_num(), output_type.into());
                    #[cfg(gpio_v2)]
                    pin.set_speed(crate::gpio::Speed::VeryHigh);
                });
                PwmPin {
                    _pin: pin.map_into(),
                    phantom: PhantomData,
                }
            }
        }
    };
}

channel_impl!(new_ch1, Ch1, Channel1Pin);
channel_impl!(new_ch2, Ch2, Channel2Pin);
channel_impl!(new_ch3, Ch3, Channel3Pin);
channel_impl!(new_ch4, Ch4, Channel4Pin);

pub struct SimplePwm<'d, T> {
    inner: PeripheralRef<'d, T>,
}

impl<'d, T: CaptureCompare16bitInstance> SimplePwm<'d, T> {
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        _ch1: Option<PwmPin<'d, T, Ch1>>,
        _ch2: Option<PwmPin<'d, T, Ch2>>,
        _ch3: Option<PwmPin<'d, T, Ch3>>,
        _ch4: Option<PwmPin<'d, T, Ch4>>,
        freq: Hertz,
        counting_mode: CountingMode,
    ) -> Self {
        Self::new_inner(tim, freq, counting_mode)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd, freq: Hertz, counting_mode: CountingMode) -> Self {
        into_ref!(tim);

        T::enable_and_reset();

        let mut this = Self { inner: tim };

        this.inner.set_counting_mode(counting_mode);
        this.set_freq(freq);
        this.inner.start();

        this.inner.enable_outputs();

        this.inner
            .set_output_compare_mode(Channel::Ch1, OutputCompareMode::PwmMode1);
        this.inner
            .set_output_compare_mode(Channel::Ch2, OutputCompareMode::PwmMode1);
        this.inner
            .set_output_compare_mode(Channel::Ch3, OutputCompareMode::PwmMode1);
        this.inner
            .set_output_compare_mode(Channel::Ch4, OutputCompareMode::PwmMode1);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        this
    }

    pub fn enable(&mut self, channel: Channel) {
        self.inner.enable_channel(channel, true);
    }

    pub fn disable(&mut self, channel: Channel) {
        self.inner.enable_channel(channel, false);
    }

    pub fn set_freq(&mut self, freq: Hertz) {
        let multiplier = if self.inner.get_counting_mode().is_center_aligned() {
            2u8
        } else {
            1u8
        };
        self.inner.set_frequency(freq * multiplier);
    }

    pub fn get_max_duty(&self) -> u16 {
        self.inner.get_max_compare_value() + 1
    }

    pub fn enable_update_interrupt(&mut self, enable: bool) {
        self.inner.enable_update_interrupt(enable);
    }

    pub async fn wait_update_interrupt(&self) {
        _ = SIGNAL_UPDATE.wait().await;
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        assert!(duty <= self.get_max_duty());
        self.inner.set_compare_value(channel, duty)
    }

    pub fn set_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
        self.inner.set_output_polarity(channel, polarity);
    }
}

impl<'d, T: CaptureCompare16bitInstance> embedded_hal_02::Pwm for SimplePwm<'d, T> {
    type Channel = Channel;
    type Time = Hertz;
    type Duty = u16;

    fn disable(&mut self, channel: Self::Channel) {
        self.inner.enable_channel(channel, false);
    }

    fn enable(&mut self, channel: Self::Channel) {
        self.inner.enable_channel(channel, true);
    }

    fn get_period(&self) -> Self::Time {
        self.inner.get_frequency().into()
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        self.inner.get_compare_value(channel)
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.inner.get_max_compare_value() + 1
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        assert!(duty <= self.get_max_duty());
        self.inner.set_compare_value(channel, duty)
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        self.inner.set_frequency(period.into());
    }
}
