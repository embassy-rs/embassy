#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _; // global logger
use panic_probe as _;

use embassy_stm32 as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    loop {
        //defmt::info!("loop!");
    }
}
