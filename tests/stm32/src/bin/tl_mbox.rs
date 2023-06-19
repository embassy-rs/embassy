// required-features: ble

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use core::mem;

use common::*;
use embassy_executor::Spawner;
use embassy_futures::poll_once;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32_wpan::{mm, TlMbox};
use embassy_time::{Duration, Timer};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

#[embassy_executor::task]
async fn run_mm_queue(memory_manager: mm::MemoryManager) {
    memory_manager.run_queue().await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    let config = Config::default();
    let mbox = TlMbox::init(p.IPCC, Irqs, config);

    spawner.spawn(run_mm_queue(mbox.mm_subsystem)).unwrap();

    let ready_event = mbox.sys_subsystem.read().await;
    let _ = poll_once(mbox.sys_subsystem.read()); // clear rx not

    info!("coprocessor ready {}", ready_event.payload());

    // test memory manager
    mem::drop(ready_event);

    let fw_info = mbox.sys_subsystem.wireless_fw_info().unwrap();
    let version_major = fw_info.version_major();
    let version_minor = fw_info.version_minor();
    let subversion = fw_info.subversion();

    let sram2a_size = fw_info.sram2a_size();
    let sram2b_size = fw_info.sram2b_size();

    info!(
        "version {}.{}.{} - SRAM2a {} - SRAM2b {}",
        version_major, version_minor, subversion, sram2a_size, sram2b_size
    );

    Timer::after(Duration::from_millis(50)).await;

    mbox.sys_subsystem.shci_c2_ble_init(Default::default()).await;

    info!("starting ble...");
    mbox.ble_subsystem.write(0x0c, &[]).await;

    info!("waiting for ble...");
    let ble_event = mbox.ble_subsystem.read().await;

    info!("ble event: {}", ble_event.payload());

    Timer::after(Duration::from_millis(150)).await;
    info!("Test OK");
    cortex_m::asm::bkpt();
}
