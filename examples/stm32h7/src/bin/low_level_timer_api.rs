#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::low_level::AFType;
use embassy_stm32::gpio::Speed;
use embassy_stm32::time::{khz, Hertz};
use embassy_stm32::timer::*;
use embassy_stm32::{into_ref, Config, Peripheral, PeripheralRef};
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
pub struct SimplePwm32<'d, T: CaptureCompare32bitInstance> {
    inner: PeripheralRef<'d, T>,
}

impl<'d, T: CaptureCompare32bitInstance> SimplePwm32<'d, T> {
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        ch1: impl Peripheral<P = impl Channel1Pin<T>> + 'd,
        ch2: impl Peripheral<P = impl Channel2Pin<T>> + 'd,
        ch3: impl Peripheral<P = impl Channel3Pin<T>> + 'd,
        ch4: impl Peripheral<P = impl Channel4Pin<T>> + 'd,
        freq: Hertz,
    ) -> Self {
        into_ref!(tim, ch1, ch2, ch3, ch4);

        T::enable_and_reset();

        ch1.set_speed(Speed::VeryHigh);
        ch1.set_as_af(ch1.af_num(), AFType::OutputPushPull);
        ch2.set_speed(Speed::VeryHigh);
        ch2.set_as_af(ch1.af_num(), AFType::OutputPushPull);
        ch3.set_speed(Speed::VeryHigh);
        ch3.set_as_af(ch1.af_num(), AFType::OutputPushPull);
        ch4.set_speed(Speed::VeryHigh);
        ch4.set_as_af(ch1.af_num(), AFType::OutputPushPull);

        let mut this = Self { inner: tim };

        this.set_frequency(freq);
        this.inner.start();

        let r = T::regs_gp32();
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
        T::regs_gp32().ccer().modify(|w| w.set_cce(channel.index(), true));
    }

    pub fn disable(&mut self, channel: Channel) {
        T::regs_gp32().ccer().modify(|w| w.set_cce(channel.index(), false));
    }

    pub fn set_frequency(&mut self, freq: Hertz) {
        <T as embassy_stm32::timer::low_level::GeneralPurpose32bitInstance>::set_frequency(&mut self.inner, freq);
    }

    pub fn get_max_duty(&self) -> u32 {
        T::regs_gp32().arr().read().arr()
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u32) {
        defmt::assert!(duty < self.get_max_duty());
        T::regs_gp32().ccr(channel.index()).modify(|w| w.set_ccr(duty))
    }
}
