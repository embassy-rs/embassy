#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::Config as RccConfig;
use embassy_stm32_wpan::TlMbox;
use embassy_stm32_wpan::sub::mm;
use panic_probe as _;

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

#[embassy_executor::task]
async fn run_mm_queue(mut memory_manager: mm::MemoryManager<'static>) {
    memory_manager.run_queue().await;
}

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    /*
        How to make this work:

        - Obtain a NUCLEO-STM32WB55 from your preferred supplier.
        - Run the `fus_update` example: it installs the FUS and the wireless stack on its own,
          no external tool needed.
        - Run this example.

        Note: extended stack versions are not supported at this time. Do not attempt to install a stack with "extended" in the name.
    */

    let mut config = embassy_stm32::Config::default();
    config.rcc = RccConfig::new_wpan();
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let config = Config::default();
    let (_mac, mm) = TlMbox::wait_ready(p.IPCC, Irqs, config)
        .await
        .unwrap()
        .init_mac()
        .await
        .unwrap();

    spawner.spawn(run_mm_queue(mm).unwrap());

    //
    //    info!("starting ble...");
    //    mbox.ble.t_write(0x0c, &[]).await;
    //
    //    info!("waiting for ble...");
    //    let ble_event = mbox.ble.tl_read().await;
    //
    //    info!("ble event: {}", ble_event.payload());

    info!("Test OK");
    cortex_m::asm::bkpt();
}
