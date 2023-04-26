//! This example runs on the RAK4631 WisBlock, which has an nRF52840 MCU and Semtech Sx126x radio.
//! Other nrf/sx126x combinations may work with appropriate pin modifications.
//! It demonstrates LoRaWAN join functionality.
#![no_std]
#![no_main]
#![macro_use]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_lora::iv::GenericSx126xInterfaceVariant;
use embassy_lora::LoraTimer;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pin as _, Pull};
use embassy_nrf::rng::Rng;
use embassy_nrf::{bind_interrupts, peripherals, rng, spim};
use embassy_time::Delay;
use lora_phy::mod_params::*;
use lora_phy::sx1261_2::SX1261_2;
use lora_phy::LoRa;
use lorawan::default_crypto::DefaultFactory as Crypto;
use lorawan_device::async_device::lora_radio::LoRaRadio;
use lorawan_device::async_device::{region, Device, JoinMode};
use {defmt_rtt as _, panic_probe as _};

const LORAWAN_REGION: region::Region = region::Region::EU868; // warning: set this appropriately for the region

bind_interrupts!(struct Irqs {
    SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => spim::InterruptHandler<peripherals::TWISPI1>;
    RNG => rng::InterruptHandler<peripherals::RNG>;
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

    let lora = {
        match LoRa::new(SX1261_2::new(BoardType::Rak4631Sx1262, spim, iv), true, &mut delay).await {
            Ok(l) => l,
            Err(err) => {
                info!("Radio error = {}", err);
                return;
            }
        }
    };

    let radio = LoRaRadio::new(lora);
    let region: region::Configuration = region::Configuration::new(LORAWAN_REGION);
    let mut device: Device<_, Crypto, _, _> = Device::new(region, radio, LoraTimer::new(), Rng::new(p.RNG, Irqs));

    defmt::info!("Joining LoRaWAN network");

    // TODO: Adjust the EUI and Keys according to your network credentials
    match device
        .join(&JoinMode::OTAA {
            deveui: [0, 0, 0, 0, 0, 0, 0, 0],
            appeui: [0, 0, 0, 0, 0, 0, 0, 0],
            appkey: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        })
        .await
    {
        Ok(()) => defmt::info!("LoRaWAN network joined"),
        Err(err) => {
            info!("Radio error = {}", err);
            return;
        }
    };
}
