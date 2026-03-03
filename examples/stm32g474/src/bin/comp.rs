#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::COMP2;
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32::comp::{self, Comp};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    COMP1_2_3 => comp::InterruptHandler<COMP2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    info!("Hello World!");

    let mut cfg = comp::Config::default();
    cfg.inverting_input = comp::InvertingInput::Vref; // <--- About 1.2V
    let comp = Comp::new(p.COMP2, p.PA7, Irqs, cfg);

    for i in 0u64.. {
        defmt::println!("{}: output_level: {}", i, comp.output_level());
        Timer::after_millis(100).await;
    }
}
