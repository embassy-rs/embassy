#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AfType, OutputType, Speed};
use embassy_stm32::time::{khz, Hertz};
use embassy_stm32::timer::low_level::{OutputCompareMode, Timer as LLTimer};
use embassy_stm32::timer::raw::{RawTimer, RawTimerPin};
use embassy_stm32::timer::{Ch1, Ch2, Ch3, Ch4, Channel, General32BitInstance, General32BitTim, TimerPin};
use embassy_stm32::{Config, Peripheral};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // 100mhz
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut pwm = SimplePwm32::new(p.TIM5, p.PA0, p.PA1, p.PA2, p.PA3, khz(10));
    let max = pwm.max_duty();
    pwm.enable(Channel::Ch1);

    info!("PWM initialized");
    info!("PWM max duty {}", max);

    loop {
        pwm.set_duty(Channel::Ch1, 0);
        Timer::after_millis(300).await;
        pwm.set_duty(Channel::Ch1, max / 4);
        Timer::after_millis(300).await;
        pwm.set_duty(Channel::Ch1, max / 2);
        Timer::after_millis(300).await;
        pwm.set_duty(Channel::Ch1, max - 1);
        Timer::after_millis(300).await;
    }
}
pub struct SimplePwm32<'d> {
    tim: LLTimer<'d, General32BitTim>,
    _pins: [RawTimerPin<'d>; 4],
}

impl<'d> SimplePwm32<'d> {
    pub fn new<T: General32BitInstance>(
        tim: impl Peripheral<P = T> + 'd,
        ch1: impl Peripheral<P = impl TimerPin<T, Ch1>> + 'd,
        ch2: impl Peripheral<P = impl TimerPin<T, Ch2>> + 'd,
        ch3: impl Peripheral<P = impl TimerPin<T, Ch3>> + 'd,
        ch4: impl Peripheral<P = impl TimerPin<T, Ch4>> + 'd,
        freq: Hertz,
    ) -> Self {
        let tim = RawTimer::new_general_32bit(tim);
        let ch1 = RawTimerPin::new(ch1, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        let ch2 = RawTimerPin::new(ch2, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        let ch3 = RawTimerPin::new(ch3, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        let ch4 = RawTimerPin::new(ch4, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        let mut this = Self {
            tim: LLTimer::new(tim),
            _pins: [ch1, ch2, ch3, ch4],
        };

        this.set_frequency(freq);
        this.tim.start();
        for n in 0..4 {
            this.tim
                .raw
                .ccmr_output_1ch(n / 2)
                .modify(|w| w.set_ocm(n % 2, OutputCompareMode::PwmMode1.into()));
        }

        this
    }

    pub fn enable(&mut self, channel: Channel) {
        self.tim.raw.ccer_1ch().modify(|w| w.set_cce(channel.index(), true));
    }

    pub fn disable(&mut self, channel: Channel) {
        self.tim.raw.ccer_1ch().modify(|w| w.set_cce(channel.index(), false));
    }

    pub fn set_frequency(&mut self, freq: Hertz) {
        self.tim.set_frequency(freq);
    }

    pub fn max_duty(&self) -> u32 {
        self.tim.raw.arr_32bit().read()
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u32) {
        defmt::assert!(duty < self.max_duty());
        self.tim.raw.ccr_32bit(channel.index()).write_value(duty)
    }
}
