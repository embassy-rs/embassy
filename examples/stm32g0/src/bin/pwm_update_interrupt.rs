#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::{self, Channel};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use crate::peripherals::TIM3;

// This test is meant for the target nucleo G070 RB
// On arduino pin d4 (pb5) a pwm signal of  1.0 hz can me measured, with increasing duty cycle
// On each update interrupt arduino pin d5 (pb4) will be made high for 1 ms.
// With a logic scope one can measure these signals
// The user led arduino pin d13 (pa5) will flash with exactly 1 hrz, with duty cycle 20%

bind_interrupts!(
    struct Irqs {
      // for stm32g070cb change TIM3_TIM4 to TIM3
      TIM3_TIM4 => timer::UpdateInterruptHandler<TIM3>;
    }
);

#[embassy_executor::task]
async fn pwm_task(mut pwm_test: PwmTest) {
    pwm_test.task().await;
}

pub struct PwmTest {
    pwm3: SimplePwm<'static, peripherals::TIM3>,
    led: Output<'static>,
    d5_pb4: Output<'static>,
    max3: u32,
    duty: u32,
    counter: usize,
}

impl PwmTest {
    fn new(mut pwm3: SimplePwm<'static, peripherals::TIM3>, led: Output<'static>, d5_pb4: Output<'static>) -> Self {
        let max3 = pwm3.get_max_duty();
        pwm3.reset();
        pwm3.enable_update_interrupt(true);
        pwm3.enable(Channel::Ch2);
        PwmTest {
            pwm3,
            max3,
            duty: 4000,
            counter: 0,
            led,
            d5_pb4,
        }
    }
    async fn task(&mut self) {
        loop {
            self.duty = (self.duty + 200) % self.max3;
            self.pwm3.set_duty(Channel::Ch2, self.duty);
            // note that the update interrupt will be call exact 100 times per second!
            self.pwm3.get_update_future().await;
            self.counter = (self.counter + 1) % 100;
            self.d5_pb4.set_high();
            match self.counter {
                1 => info!("Update interrupt"),
                10 => self.led.set_high(),
                30 => self.led.set_low(),

                _ => (),
            }
            Timer::after_millis(1).await;
            self.d5_pb4.set_low();
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Testing the update interupt for a simple_pwm instance");
    let d4_pb5 = PwmPin::new_ch2(p.PB5, OutputType::PushPull);
    let pwm3 = SimplePwm::<'static>::new(
        p.TIM3,
        None,
        Some(d4_pb5),
        None,
        None,
        Hertz(100),
        CountingMode::EdgeAlignedUp,
    );
    let led_g = Output::new(p.PA5, Level::High, Speed::Low);
    let d5_pb4 = Output::new(p.PB4, Level::High, Speed::Low);
    let pwm_test = PwmTest::new(pwm3, led_g, d5_pb4);
    // note that the pwm_task is the owner of pwm_test.
    // pwm_test is the owner of the pwm and the led.
    spawner.spawn(pwm_task(pwm_test)).unwrap();

    loop {
        Timer::after_secs(10).await;
        info!("Still alive");
    }
}
