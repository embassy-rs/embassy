// required-features: ble

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::tl_mbox::{Config, TlMbox};
use embassy_stm32::{bind_interrupts, tl_mbox};
use embassy_time::{Duration, Timer};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => tl_mbox::ReceiveInterruptHandler;
    IPCC_C1_TX => tl_mbox::TransmitInterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    let config = Config::default();
    let mbox = TlMbox::new(p.IPCC, Irqs, config);

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

    info!("Test OK");
    cortex_m::asm::bkpt();
}
