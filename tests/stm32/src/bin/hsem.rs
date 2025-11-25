// required-features: hsem
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::hsem::{HardwareSemaphore, HardwareSemaphoreInterruptHandler};
use embassy_stm32::peripherals::HSEM;

bind_interrupts!(struct Irqs{
    HSEM => HardwareSemaphoreInterruptHandler<HSEM>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = init();

    let hsem = HardwareSemaphore::new(p.HSEM, Irqs);

    //    if hsem.channel_for(SemaphoreNumber::Channel5).is_semaphore_locked() {
    //        defmt::panic!("Semaphore 5 already locked!")
    //    }
    //
    //    hsem.channel_for(SemaphoreNumber::Channel5).one_step_lock().unwrap();
    //    hsem.channel_for(SemaphoreNumber::Channel1).two_step_lock(0).unwrap();
    //
    //    hsem.channel_for(SemaphoreNumber::Channel5).unlock(0);

    #[cfg(feature = "stm32wb55rg")]
    let [_channel1, _channel2, mut channel5, _channel6] = hsem.split();
    #[cfg(not(feature = "stm32wb55rg"))]
    let [_channel1, _channel2, _channel3, _channel4, mut channel5, _channel6] = hsem.split();

    info!("Locking channel 5");

    let mutex = channel5.lock(0).await;

    info!("Locked channel 5");

    drop(mutex);

    info!("Unlocked channel 5");

    info!("Test OK");
    cortex_m::asm::bkpt();
}
