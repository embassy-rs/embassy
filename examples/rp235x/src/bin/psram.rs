//! This example tests an APS6404L PSRAM chip connected to the RP235x
//! It fills the PSRAM with alternating patterns and reads back a value
//!
//! In this example, the PSRAM CS is connected to Pin 0.

#![no_std]
#![no_main]

use core::slice;

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = embassy_rp::config::Config::default();
    let p = embassy_rp::init(config);
    let psram_config = embassy_rp::psram::Config::aps6404l();
    let psram = embassy_rp::psram::Psram::new(embassy_rp::qmi_cs1::QmiCs1::new(p.QMI_CS1, p.PIN_0), psram_config);

    let Ok(psram) = psram else {
        error!("PSRAM not found");
        loop {
            Timer::after_secs(1).await;
        }
    };

    let psram_slice = unsafe {
        let psram_ptr = psram.base_address();
        let slice: &'static mut [u8] = slice::from_raw_parts_mut(psram_ptr, psram.size() as usize);
        slice
    };

    loop {
        psram_slice.fill(0x55);
        info!("PSRAM filled with 0x55");
        let at_addr = psram_slice[0x100];
        info!("Read from PSRAM at address 0x100: 0x{:02x}", at_addr);
        Timer::after_secs(1).await;

        psram_slice.fill(0xAA);
        info!("PSRAM filled with 0xAA");
        let at_addr = psram_slice[0x100];
        info!("Read from PSRAM at address 0x100: 0x{:02x}", at_addr);
        Timer::after_secs(1).await;
    }
}
