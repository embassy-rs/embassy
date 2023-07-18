//! This example runs on the RAK4631 WisBlock, which has an nRF52840 MCU and Semtech Sx126x radio.
//! Other nrf/sx126x combinations may work with appropriate pin modifications.
//! It demonstrates LORA CAD functionality.
#![no_std]
#![no_main]
#![macro_use]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_lora::iv::GenericSx126xInterfaceVariant;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pin as _, Pull};
use embassy_nrf::{bind_interrupts, peripherals, spim};
use embassy_time::{Delay, Duration, Timer};
use lora_phy::mod_params::*;
use lora_phy::sx1261_2::SX1261_2;
use lora_phy::LoRa;
use {defmt_rtt as _, panic_probe as _};

const LORA_FREQUENCY_IN_HZ: u32 = 903_900_000; // warning: set this appropriately for the region

bind_interrupts!(struct Irqs {
    SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => spim::InterruptHandler<peripherals::TWISPI1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut spi_config = spim::Config::default();
    spi_config.frequency = spim::Frequency::M16;

    let spim = spim::Spim::new(p.TWISPI1, Irqs, p.P1_11, p.P1_13, p.P1_12, spi_config);

    let nss = Output::new(p.P1_10.degrade(), Level::High, OutputDrive::Standard);
    let reset = Output::new(p.P1_06.degrade(), Level::High, OutputDrive::Standard);
    let dio1 = Input::new(p.P1_15.degrade(), Pull::Down);
    let busy = Input::new(p.P1_14.degrade(), Pull::Down);
    let rf_switch_rx = Output::new(p.P1_05.degrade(), Level::Low, OutputDrive::Standard);
    let rf_switch_tx = Output::new(p.P1_07.degrade(), Level::Low, OutputDrive::Standard);

    let iv =
        GenericSx126xInterfaceVariant::new(nss, reset, dio1, busy, Some(rf_switch_rx), Some(rf_switch_tx)).unwrap();

    let mut delay = Delay;

    let mut lora = {
        match LoRa::new(SX1261_2::new(BoardType::Rak4631Sx1262, spim, iv), false, &mut delay).await {
            Ok(l) => l,
            Err(err) => {
                info!("Radio error = {}", err);
                return;
            }
        }
    };

    let mut debug_indicator = Output::new(p.P1_03, Level::Low, OutputDrive::Standard);
    let mut start_indicator = Output::new(p.P1_04, Level::Low, OutputDrive::Standard);

    start_indicator.set_high();
    Timer::after(Duration::from_secs(5)).await;
    start_indicator.set_low();

    let mdltn_params = {
        match lora.create_modulation_params(
            SpreadingFactor::_10,
            Bandwidth::_250KHz,
            CodingRate::_4_8,
            LORA_FREQUENCY_IN_HZ,
        ) {
            Ok(mp) => mp,
            Err(err) => {
                info!("Radio error = {}", err);
                return;
            }
        }
    };

    match lora.prepare_for_cad(&mdltn_params, true).await {
        Ok(()) => {}
        Err(err) => {
            info!("Radio error = {}", err);
            return;
        }
    };

    match lora.cad().await {
        Ok(cad_activity_detected) => {
            if cad_activity_detected {
                info!("cad successful with activity detected")
            } else {
                info!("cad successful without activity detected")
            }
            debug_indicator.set_high();
            Timer::after(Duration::from_secs(5)).await;
            debug_indicator.set_low();
        }
        Err(err) => info!("cad unsuccessful = {}", err),
    }
}
