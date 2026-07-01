#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use embassy_stm32::gpio::{Input, Pull};
use {defmt_rtt as _, panic_probe as _};

// ~20 ms debounce at 12 MHz HSI (default reset clock).
const DEBOUNCE_CYCLES: u32 = 240_000;

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    // B1 user button on NUCLEO-C092RC: PC13, active low.
    let button = Input::new(p.PC13, Pull::Up);

    let mut prev = button.is_high();

    loop {
        cortex_m::asm::delay(DEBOUNCE_CYCLES);
        let state = button.is_high();
        if state != prev {
            prev = state;
            if state {
                info!("Released");
            } else {
                info!("Pressed");
            }
        }
    }
}
