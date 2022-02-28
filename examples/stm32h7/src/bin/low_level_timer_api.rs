#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use core::marker::PhantomData;

use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use embassy_stm32::gpio::low_level::AFType;
use embassy_stm32::gpio::Speed;
use embassy_stm32::pwm::*;
use embassy_stm32::time::{Hertz, U32Ext};
use embassy_stm32::{Config, Peripherals};
use example_common::*;

pub fn config() -> Config {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(400.mhz().into());
    config.rcc.hclk = Some(400.mhz().into());
    config.rcc.pll1.q_ck = Some(100.mhz().into());
    config.rcc.pclk1 = Some(100.mhz().into());
    config.rcc.pclk2 = Some(100.mhz().into());
    config.rcc.pclk3 = Some(100.mhz().into());
    config.rcc.pclk4 = Some(100.mhz().into());
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut pwm = SimplePwm32::new(p.TIM5, p.PA0, p.PA1, p.PA2, p.PA3, 10000.hz());
    let max = pwm.get_max_duty();
    pwm.enable(Channel::Ch1);

    info!("PWM initialized");
    info!("PWM max duty {}", max);

    loop {
        pwm.set_duty(Channel::Ch1, 0);
        Timer::after(Duration::from_millis(300)).await;
        pwm.set_duty(Channel::Ch1, max / 4);
        Timer::after(Duration::from_millis(300)).await;
        pwm.set_duty(Channel::Ch1, max / 2);
        Timer::after(Duration::from_millis(300)).await;
        pwm.set_duty(Channel::Ch1, max - 1);
        Timer::after(Duration::from_millis(300)).await;
    }
}
pub struct SimplePwm32<'d, T: CaptureCompare32bitInstance> {
    phantom: PhantomData<&'d mut T>,
    inner: T,
}

impl<'d, T: CaptureCompare32bitInstance> SimplePwm32<'d, T> {
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
        <T as embassy_stm32::rcc::low_level::RccPeripheral>::reset();

        unsafe {
            ch1.set_speed(Speed::VeryHigh);
            ch1.set_as_af(ch1.af_num(), AFType::OutputPushPull);
            ch2.set_speed(Speed::VeryHigh);
            ch2.set_as_af(ch1.af_num(), AFType::OutputPushPull);
            ch3.set_speed(Speed::VeryHigh);
            ch3.set_as_af(ch1.af_num(), AFType::OutputPushPull);
            ch4.set_speed(Speed::VeryHigh);
            ch4.set_as_af(ch1.af_num(), AFType::OutputPushPull);
        }

        let mut this = Self {
            inner: tim,
            phantom: PhantomData,
        };

        this.set_freq(freq);
        this.inner.start();

        unsafe {
            T::regs_gp32()
                .ccmr_output(0)
                .modify(|w| w.set_ocm(0, OutputCompareMode::PwmMode1.into()));
            T::regs_gp32()
                .ccmr_output(0)
                .modify(|w| w.set_ocm(1, OutputCompareMode::PwmMode1.into()));
            T::regs_gp32()
                .ccmr_output(1)
                .modify(|w| w.set_ocm(0, OutputCompareMode::PwmMode1.into()));
            T::regs_gp32()
                .ccmr_output(1)
                .modify(|w| w.set_ocm(1, OutputCompareMode::PwmMode1.into()));
        }
        this
    }

    pub fn enable(&mut self, channel: Channel) {
        unsafe {
            T::regs_gp32()
                .ccer()
                .modify(|w| w.set_cce(channel.raw(), true));
        }
    }

    pub fn disable(&mut self, channel: Channel) {
        unsafe {
            T::regs_gp32()
                .ccer()
                .modify(|w| w.set_cce(channel.raw(), false));
        }
    }

    pub fn set_freq<F: Into<Hertz>>(&mut self, freq: F) {
        <T as embassy_stm32::timer::low_level::GeneralPurpose32bitInstance>::set_frequency(
            &mut self.inner,
            freq,
        );
    }

    pub fn get_max_duty(&self) -> u32 {
        unsafe { T::regs_gp32().arr().read().arr() }
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u32) {
        defmt::assert!(duty < self.get_max_duty());
        unsafe {
            T::regs_gp32()
                .ccr(channel.raw())
                .modify(|w| w.set_ccr(duty))
        }
    }
}
