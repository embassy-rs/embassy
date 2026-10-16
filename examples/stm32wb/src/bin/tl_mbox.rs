#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::Config as RccConfig;
use embassy_stm32_wpan::TlMbox;
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    /*
        How to make this work:

        - Obtain a NUCLEO-STM32WB55 from your preferred supplier.
        - Run the `fus_update` example: it installs the FUS and the BLE + MAC 802.15.4
          wireless stack on its own, no external tool needed.
        - Run this example.

        Note: extended stack versions are not supported at this time. Do not attempt to install a stack with "extended" in the name.
    */

    let mut config = embassy_stm32::Config::default();
    config.rcc = RccConfig::new_wpan();
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let config = Config::default();
    let mbox = TlMbox::wait_ready(p.IPCC, Irqs, config)
        .await
        .expect("failed to init tl mbox");

    loop {
        let wireless_fw_info = mbox.sys.wireless_fw_info();
        match wireless_fw_info {
            None => info!("not yet initialized"),
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

        Timer::after_millis(50).await;
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
