// required-features: hsem
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::hsem::HardwareSemaphore;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = init();

    let mut hsem = HardwareSemaphore::new(p.HSEM);

    if hsem.is_semaphore_locked(5) {
        defmt::panic!("Semaphore 5 already locked!")
    }

    hsem.one_step_lock(5).unwrap();
    hsem.two_step_lock(1, 0).unwrap();

    info!("Test OK");
    cortex_m::asm::bkpt();
}
