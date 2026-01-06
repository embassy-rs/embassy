#![no_std]
#![no_main]

//! STM32H7 Primary Core (CM4) Intercore Communication Example
//!
//! This example demonstrates reliable communication between the Cortex-M7 and
//! Cortex-M4 cores using a shared memory region
//!
//! The CM4 core handles:
//! - MPU configuration to make shared memory non-cacheable
//! - Clock initialization
//! - Toggling LED state in shared memory
//!
//! Usage:
//! 1. Flash the CM0+ (secondary) core binary first
//! 2. Then flash this CM4 (primary) core binary
//! 3. The system will start with CM4 toggling LED state and CM0+ responding by
//!    physically toggling the LED

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::ipcc::{Config as IPCCConfig, Ipcc, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::{Config, SharedData, bind_interrupts};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

#[unsafe(link_section = ".shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();
#[unsafe(link_section = ".shared_data")]
static LED_STATE: AtomicBool = AtomicBool::new(false);

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Initialize the CM4 core
    let p = embassy_stm32::init_primary(Config::default(), &SHARED_DATA);
    info!("CM4 core initialized");

    let ipcc = Ipcc::new(p.IPCC, Irqs, IPCCConfig::default());
    let [ch1, _ch2, _ch3, _ch4, _ch5, _ch6] = ipcc.split();
    let (mut tx, mut _rx) = ch1;

    info!("CM4: Starting main loop");
    loop {
        Timer::after_millis(1500).await;
        tx.send(|| {
            let new_led_state = !LED_STATE.load(Ordering::Relaxed);
            info!("CM4: Send! New LED state: {}", new_led_state);
            LED_STATE.store(new_led_state, Ordering::Relaxed);
        })
        .await;
    }
}
