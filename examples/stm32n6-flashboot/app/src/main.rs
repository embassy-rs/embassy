#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::rcc::XspiClkSrc;
use embassy_stm32::{bind_interrupts, interrupt};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    pub struct Irqs {
        EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
    }
);

#[embassy_executor::task]
async fn button_task(mut button: ExtiInput<'static, Async>, mut green_led: Output<'static>) {
    loop {
        button.wait_for_rising_edge().await;
        info!("Button pressed — green LED on");
        green_led.set_high();

        button.wait_for_falling_edge().await;
        info!("Button released — green LED off");
        green_led.set_low();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("App starting");

    let mut config = embassy_stm32::Config::default();
    config.rcc.mux.xspi2sel = XspiClkSrc::PER;
    config.rcc.vddio3_1v8 = true;
    let p = embassy_stm32::init(config);

    info!("HAL initialized");

    let mut red_led = Output::new(p.PG10, Level::Low, Speed::Low);
    let green_led = Output::new(p.PO1, Level::Low, Speed::Low);
    let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Down, Irqs);

    spawner.spawn(button_task(button, green_led).unwrap());

    info!("Blinking red LED, press button for green LED");
    loop {
        red_led.set_high();
        Timer::after_millis(500).await;
        red_led.set_low();
        Timer::after_millis(500).await;
    }
}
