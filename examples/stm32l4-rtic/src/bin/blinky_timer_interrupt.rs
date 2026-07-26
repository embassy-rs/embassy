// A simple example of using RTIC and Embassy together with a timer HW interrupt
#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

#[rtic::app(device = embassy_stm32, peripherals = true)]
mod app {
    use embassy_stm32::gpio::{Level, Output, Speed};
    use embassy_stm32::pac;
    use embassy_stm32::peripherals::TIM2;
    use embassy_stm32::time::Hertz;
    use embassy_stm32::timer::low_level::{RoundTo, Timer};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Output<'static>,
        timer: Timer<'static, TIM2>,
    }

    #[init]
    fn init(_ctx: init::Context) -> (Shared, Local) {
        let device_config = embassy_stm32::Config::default();
        let stm32_peripherals = embassy_stm32::init(device_config);

        // Configure LED
        let led = Output::new(stm32_peripherals.PB14, Level::High, Speed::Low);

        // setup hw timer interrupt using the low_level timer API
        let timer = Timer::new(stm32_peripherals.TIM2);
        // 10Hz = 10 times per second = every 100ms
        timer.set_frequency(Hertz(10), RoundTo::Slower);
        timer.enable_update_interrupt(true);
        timer.set_autoreload_preload(true);
        timer.start();

        unsafe {
            // enable the timer interrupt in NVIC
            cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2);
        }

        (Shared {}, Local { led, timer })
    }

    // using a 'hardware task' to trigger blinking of an LED
    #[task(binds = TIM2, local = [led, timer])]
    fn tim2_handler(ctx: tim2_handler::Context) {
        // Clear the interrupt flag
        ctx.local.timer.clear_update_interrupt();
        ctx.local.led.toggle();
    }
}
