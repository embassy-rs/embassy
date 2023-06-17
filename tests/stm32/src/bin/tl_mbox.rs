// required-features: ble

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_futures::poll_once;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32_wpan::ble::Ble;
use embassy_stm32_wpan::sys::Sys;
use embassy_stm32_wpan::{mm, TlMbox};
use embassy_time::{Duration, Timer};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

#[embassy_executor::task]
async fn run_mm_queue() {
    mm::MemoryManager::run_queue().await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    spawner.spawn(run_mm_queue()).unwrap();

    let config = Config::default();
    let mbox = TlMbox::init(p.IPCC, Irqs, config);

    let mut rx_buf = [0u8; 500];
    let ready_event = Sys::read().await;
    let _ = poll_once(Sys::read()); // clear rx not
    ready_event.write(&mut rx_buf).unwrap();

    info!("coprocessor ready {}", rx_buf);

    loop {
        let wireless_fw_info = mbox.wireless_fw_info();
        match wireless_fw_info {
            None => {}
            Some(fw_info) => {
                let version_major = fw_info.version_major();
                let version_minor = fw_info.version_minor();
                let subversion = fw_info.subversion();

                let sram2a_size = fw_info.sram2a_size();
                let sram2b_size = fw_info.sram2b_size();

                info!(
                    "version {}.{}.{} - SRAM2a {} - SRAM2b {}",
                    version_major, version_minor, subversion, sram2a_size, sram2b_size
                );

                break;
            }
        }

        Timer::after(Duration::from_millis(50)).await;
    }

    Sys::shci_c2_ble_init(Default::default()).await;

    info!("starting ble...");
    Ble::write(0x0c, &[]).await;

    info!("waiting for ble...");
    let ble_event = Ble::read().await;
    ble_event.write(&mut rx_buf).unwrap();

    info!("ble event: {}", rx_buf);

    // Timer::after(Duration::from_secs(3)).await;
    info!("Test OK");
    cortex_m::asm::bkpt();
}
