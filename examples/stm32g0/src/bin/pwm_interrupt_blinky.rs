#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::peripherals::PA5;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::simple_pwm::{InterruptHandler, PwmPin, SimplePwm};
use embassy_stm32::timer::{self, Channel};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// This test is meant for the target nucleo G070 RB
// On arduino pin d4 (pb5) a pwm signal of about 0.3 hz can me measured.
// Attach a led and a resistor of 330 ohm in series to watch the pwm
// The user led arduino pin d13 (pa5) will flash with exactly 1 hrz.

bind_interrupts!(
    struct Irqs {
        TIM3 => InterruptHandler<peripherals::TIM3>;
    }
);

#[embassy_executor::task]
async fn pwm_task(mut pwm_test: PwmTest) {
    pwm_test.task().await;
}

pub struct PwmTest {
    pwm3: SimplePwm<'static, peripherals::TIM3>,
    led: Output<'static, PA5>,
    max3: u16,
    duty: u16,
    counter: usize,
}

impl PwmTest {
    fn new(mut pwm3: SimplePwm<'static, peripherals::TIM3>, led: Output<'static, PA5>) -> Self {
        let max3 = pwm3.get_max_duty();
        pwm3.enable(timer::Channel::Ch2);
        pwm3.enable_update_interrupt(true);
        PwmTest {
            pwm3,
            max3,
            duty: 0,
            counter: 0,
            led,
        }
    }
    async fn task(&mut self) {
        loop {
            self.duty = (self.duty + 200) % self.max3;
            self.pwm3.set_duty(Channel::Ch2, self.duty);
            // note that the update interrupt will be call exact 100 times per second!
            self.pwm3.wait_update_interrupt().await;
            self.counter = (self.counter + 1) % 100;
            match self.counter {
                10 => self.led.set_high(),
                30 => self.led.set_low(),
                _ => (),
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");
    let d4_pb5 = PwmPin::new_ch2(p.PB5, OutputType::PushPull);
    let pwm3 = SimplePwm::<'static>::new(
        p.TIM3,
        None,
        Some(d4_pb5),
        None,
        None,
        Hertz(100),
        embassy_stm32::timer::CountingMode::EdgeAlignedUp,
    );
    let led_g = Output::new(p.PA5, Level::High, Speed::Low);
    let pwm_test = PwmTest::new(pwm3, led_g);
    // note that at the end the pwmTest task is the owner of pwmTest.
    // PwmTest is the owner of the pwm and the led.
    spawner.spawn(pwm_task(pwm_test)).unwrap();

    loop {
        info!("high");
        Timer::after_millis(300).await;

        info!("low");
        Timer::after_millis(300).await;
    }
}
