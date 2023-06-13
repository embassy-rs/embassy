// required-features: ble

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::Config;
use embassy_stm32_wpan::TlMbox;
use embassy_time::{Duration, Timer};

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => embassy_stm32_wpan::ReceiveInterruptHandler;
    IPCC_C1_TX => embassy_stm32_wpan::TransmitInterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    let config = Config::default();
    let mbox = TlMbox::init(p.IPCC, Irqs, config);

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

    //    let mut rc = RadioCoprocessor::new(mbox);
    //
    //    let response = rc.read().await;
    //    info!("coprocessor ready {}", response);
    //
    //    rc.write(&[0x01, 0x03, 0x0c, 0x00, 0x00]);
    //    let response = rc.read().await;
    //    info!("ble reset rsp {}", response);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
