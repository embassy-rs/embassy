//! This example runs on the RAK4631 WisBlock, which has an nRF52840 MCU and Semtech Sx126x radio.
#![no_std]
#![no_main]
#![macro_use]
#![allow(dead_code)]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_lora::sx126x::*;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pin as _, Pull};
use embassy_nrf::{interrupt, spim};
use embassy_time::{Duration, Timer};
use lorawan_device::async_device::radio::{Bandwidth, CodingRate, PhyRxTx, RfConfig, SpreadingFactor};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut spi_config = spim::Config::default();
    spi_config.frequency = spim::Frequency::M1; // M16 ???

    let mut radio = {
        let irq = interrupt::take!(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
        let spim = spim::Spim::new(p.TWISPI1, irq, p.P1_11, p.P1_13, p.P1_12, spi_config);

        let cs = Output::new(p.P1_10, Level::High, OutputDrive::Standard);
        let reset = Output::new(p.P1_06, Level::High, OutputDrive::Standard);
        let dio1 = Input::new(p.P1_15.degrade(), Pull::Down);
        let busy = Input::new(p.P1_14.degrade(), Pull::Down);
        let antenna_rx = Output::new(p.P1_05, Level::Low, OutputDrive::Standard);
        let antenna_tx = Output::new(p.P1_07, Level::Low, OutputDrive::Standard);

        match Sx126xRadio::new(spim, cs, reset, antenna_rx, antenna_tx, dio1, busy, false).await {
            Ok(r) => r,
            Err(err) => {
                info!("Sx126xRadio error = {}", err);
                return;
            }
        }
    };

    let mut debug_indicator = Output::new(p.P1_03, Level::Low, OutputDrive::Standard);
    let mut start_indicator = Output::new(p.P1_04, Level::Low, OutputDrive::Standard);

    start_indicator.set_high();
    Timer::after(Duration::from_secs(5)).await;
    start_indicator.set_low();

    match radio.lora.sleep().await {
        Ok(()) => info!("Sleep successful"),
        Err(err) => info!("Sleep unsuccessful = {}", err),
    }

    let rf_config = RfConfig {
        frequency: 903900000, // channel in Hz
        bandwidth: Bandwidth::_250KHz,
        spreading_factor: SpreadingFactor::_10,
        coding_rate: CodingRate::_4_8,
    };

    let mut buffer = [00u8; 100];

    // P2P receive
    match radio.rx(rf_config, &mut buffer).await {
        Ok((buffer_len, rx_quality)) => info!(
            "RX received = {:?} with length = {} rssi = {} snr = {}",
            &buffer[0..buffer_len],
            buffer_len,
            rx_quality.rssi(),
            rx_quality.snr()
        ),
        Err(err) => info!("RX error = {}", err),
    }

    match radio.lora.sleep().await {
        Ok(()) => info!("Sleep successful"),
        Err(err) => info!("Sleep unsuccessful = {}", err),
    }

    debug_indicator.set_high();
    Timer::after(Duration::from_secs(5)).await;
    debug_indicator.set_low();
}
