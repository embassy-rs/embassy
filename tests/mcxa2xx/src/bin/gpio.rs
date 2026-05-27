#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_mcxa::gpio::{self, Async, Input, Output};
use embassy_mcxa::{bind_interrupts, peripherals};
use hal::config::Config;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    GPIO2 => gpio::InterruptHandler<peripherals::GPIO2>;
    }
);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = Config::default();
    let p = hal::init(config);

    defmt::info!("Gpio test");

    let mut output = Output::new(
        p.P1_8,
        embassy_mcxa::gpio::Level::Low,
        embassy_mcxa::gpio::DriveStrength::Normal,
        embassy_mcxa::gpio::SlewRate::Slow,
    );

    spawner.spawn(wait(Input::new_async(p.P2_4, Irqs, embassy_mcxa::gpio::Pull::Down)).unwrap());

    embassy_time::Timer::after_millis(40).await;
    output.set_high();
    assert!(output.is_set_high());
    embassy_time::Timer::after_millis(40).await;
    output.set_low();
    assert!(output.is_set_low());
    embassy_time::Timer::after_millis(40).await;
    output.set_high();
    assert!(output.is_set_high());
    embassy_time::Timer::after_millis(40).await;
    output.set_low();
    assert!(output.is_set_low());
    embassy_time::Timer::after_millis(40).await;
    output.set_high();
    assert!(output.is_set_high());
    embassy_time::Timer::after_millis(40).await;

    unreachable!("The wait task failed to see the output values");
}

#[embassy_executor::task]
async fn wait(mut input: Input<'static, Async>) {
    assert!(input.is_low());

    input.wait_for_high().await;

    embassy_time::Timer::after_millis(10).await;
    assert!(input.is_high());
    embassy_time::Timer::after_millis(10).await;

    input.wait_for_low().await;

    embassy_time::Timer::after_millis(10).await;
    assert!(input.is_low());
    embassy_time::Timer::after_millis(10).await;

    input.wait_for_rising_edge().await;

    embassy_time::Timer::after_millis(10).await;
    assert!(input.is_high());
    embassy_time::Timer::after_millis(10).await;

    input.wait_for_falling_edge().await;

    embassy_time::Timer::after_millis(10).await;
    assert!(input.is_low());
    embassy_time::Timer::after_millis(10).await;

    input.wait_for_any_edge().await;

    embassy_time::Timer::after_millis(10).await;
    assert!(input.is_high());

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
