#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::khz;
use embassy_stm32::timer::low_level;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut tim1 = low_level::Timer::new(p.TIM1);
    let mut tim2 = low_level::Timer::new(p.TIM2);

    tim1.set_frequency(khz(50));
    tim2.set_frequency(khz(100));

    low_level::synchronize([&mut tim1, &mut tim2]);

    tim1.set_frequency(khz(50));
}
