#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AfType, Flex, OutputType, Speed};
use embassy_stm32::time::{khz, Hertz};
use embassy_stm32::timer::low_level::{OutputCompareMode, Timer as LLTimer};
use embassy_stm32::timer::{Channel, Channel1Pin, Channel2Pin, Channel3Pin, Channel4Pin, GeneralInstance32bit4Channel};
use embassy_stm32::{Config, Peri};
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
    let max = pwm.get_max_duty();
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
pub struct SimplePwm32<'d, T: GeneralInstance32bit4Channel> {
    tim: LLTimer<'d, T>,
    _ch1: Flex<'d>,
    _ch2: Flex<'d>,
    _ch3: Flex<'d>,
    _ch4: Flex<'d>,
}

impl<'d, T: GeneralInstance32bit4Channel> SimplePwm32<'d, T> {
    pub fn new(
        tim: Peri<'d, T>,
        ch1: Peri<'d, impl Channel1Pin<T>>,
        ch2: Peri<'d, impl Channel2Pin<T>>,
        ch3: Peri<'d, impl Channel3Pin<T>>,
        ch4: Peri<'d, impl Channel4Pin<T>>,
        freq: Hertz,
    ) -> Self {
        let af1 = ch1.af_num();
        let af2 = ch2.af_num();
        let af3 = ch3.af_num();
        let af4 = ch4.af_num();
        let mut ch1 = Flex::new(ch1);
        let mut ch2 = Flex::new(ch2);
        let mut ch3 = Flex::new(ch3);
        let mut ch4 = Flex::new(ch4);
        ch1.set_as_af_unchecked(af1, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        ch2.set_as_af_unchecked(af2, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        ch3.set_as_af_unchecked(af3, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        ch4.set_as_af_unchecked(af4, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        let mut this = Self {
            tim: LLTimer::new(tim),
            _ch1: ch1,
            _ch2: ch2,
            _ch3: ch3,
            _ch4: ch4,
        };

        this.set_frequency(freq);
        this.tim.start();

        let r = this.tim.regs_gp32();
        r.ccmr_output(0)
            .modify(|w| w.set_ocm(0, OutputCompareMode::PwmMode1.into()));
        r.ccmr_output(0)
            .modify(|w| w.set_ocm(1, OutputCompareMode::PwmMode1.into()));
        r.ccmr_output(1)
            .modify(|w| w.set_ocm(0, OutputCompareMode::PwmMode1.into()));
        r.ccmr_output(1)
            .modify(|w| w.set_ocm(1, OutputCompareMode::PwmMode1.into()));

        this
    }

    pub fn enable(&mut self, channel: Channel) {
        self.tim.regs_gp32().ccer().modify(|w| w.set_cce(channel.index(), true));
    }

    pub fn disable(&mut self, channel: Channel) {
        self.tim
            .regs_gp32()
            .ccer()
            .modify(|w| w.set_cce(channel.index(), false));
    }

    pub fn set_frequency(&mut self, freq: Hertz) {
        self.tim.set_frequency(freq);
    }

    pub fn get_max_duty(&self) -> u32 {
        self.tim.regs_gp32().arr().read()
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u32) {
        defmt::assert!(duty < self.get_max_duty());
        self.tim.regs_gp32().ccr(channel.index()).write_value(duty)
    }
}
