use crate::{
    pwm::{pins::*, CaptureCompareCapable16bitInstance, Channel, OutputCompareMode},
    time::Hertz,
};
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

pub struct SimplePwm<'d, T> {
    phantom: PhantomData<&'d mut T>,
    inner: T,
}

impl<'d, T: CaptureCompareCapable16bitInstance> SimplePwm<'d, T> {
    pub fn new<F: Into<Hertz>>(
        tim: impl Unborrow<Target = T> + 'd,
        ch1: impl Unborrow<Target = impl Channel1Pin<T>> + 'd,
        ch2: impl Unborrow<Target = impl Channel2Pin<T>> + 'd,
        ch3: impl Unborrow<Target = impl Channel3Pin<T>> + 'd,
        ch4: impl Unborrow<Target = impl Channel4Pin<T>> + 'd,
        freq: F,
    ) -> Self {
        unborrow!(tim, ch1, ch2, ch3, ch4);

        T::enable();
        <T as crate::rcc::sealed::RccPeripheral>::reset();

        unsafe {
            ch1.configure();
            ch2.configure();
            ch3.configure();
            ch4.configure();
        }

        let mut this = Self {
            inner: tim,
            phantom: PhantomData,
        };

        this.inner.set_frequency(freq);
        this.inner.start();

        unsafe {
            this.inner
                .set_output_compare_mode(Channel::Ch1, OutputCompareMode::PwmMode1);
            this.inner
                .set_output_compare_mode(Channel::Ch2, OutputCompareMode::PwmMode1);
            this.inner
                .set_output_compare_mode(Channel::Ch3, OutputCompareMode::PwmMode1);
            this.inner
                .set_output_compare_mode(Channel::Ch4, OutputCompareMode::PwmMode1);
        }
        this
    }

    pub fn enable(&mut self, channel: Channel) {
        unsafe {
            self.inner.enable_channel(channel, true);
        }
    }

    pub fn disable(&mut self, channel: Channel) {
        unsafe {
            self.inner.enable_channel(channel, false);
        }
    }

    pub fn set_freq<F: Into<Hertz>>(&mut self, freq: F) {
        self.inner.set_frequency(freq);
    }

    pub fn get_max_duty(&self) -> u16 {
        unsafe { self.inner.get_max_compare_value() }
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        assert!(duty < self.get_max_duty());
        unsafe { self.inner.set_compare_value(channel, duty) }
    }
}
