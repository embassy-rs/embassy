#![no_std]
#![no_main]

//! STM32H7 dual-core HSEM notification example, CM4 side.
//!
//! Waits for notifications on HSEM channel 5 with `listen()` and toggles the yellow LED (LD2, PE1)
//! on each one. The CM7 (`stm32h755cm7/src/bin/hsem_notify.rs`) notifies every 500 ms and toggles
//! the green LED (LD1, PB0), so both LEDs should blink together.
//!
//! Usage: flash this binary, then the CM7 binary (`hsem_notify`).

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::hsem::{HardwareSemaphore, HardwareSemaphoreInterruptHandler};
use embassy_stm32::{SharedData, bind_interrupts, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    HSEM2 => HardwareSemaphoreInterruptHandler<peripherals::HSEM>;
});

#[unsafe(link_section = ".ram_d3")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init_secondary(&SHARED_DATA);

    let hsem = HardwareSemaphore::new(p.HSEM, Irqs);
    let [_, _, _, _, mut channel5, _] = hsem.split();
    let mut led = Output::new(p.PE1, Level::Low, Speed::Low);

    let mut count = 0u32;
    loop {
        channel5.listen().await;
        led.toggle();
        count = count.wrapping_add(1);
        info!("woken by channel 5 ({})", count);
    }
}
