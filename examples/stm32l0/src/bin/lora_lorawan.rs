//! This example runs on the STM32 LoRa Discovery board, which has a builtin Semtech Sx1276 radio.
//! It demonstrates LoRaWAN join functionality.
#![no_std]
#![no_main]
#![macro_use]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_lora::iv::Stm32l0InterfaceVariant;
use embassy_lora::LoraTimer;
use embassy_stm32::exti::{Channel, ExtiInput};
use embassy_stm32::gpio::{Input, Level, Output, Pin, Pull, Speed};
use embassy_stm32::rng::Rng;
use embassy_stm32::time::khz;
use embassy_stm32::{bind_interrupts, peripherals, rng, spi};
use embassy_time::Delay;
use lora_phy::mod_params::*;
use lora_phy::sx1276_7_8_9::SX1276_7_8_9;
use lora_phy::LoRa;
use lorawan::default_crypto::DefaultFactory as Crypto;
use lorawan_device::async_device::lora_radio::LoRaRadio;
use lorawan_device::async_device::{region, Device, JoinMode};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG_LPUART1 => rng::InterruptHandler<peripherals::RNG>;
});

const LORAWAN_REGION: region::Region = region::Region::EU868; // warning: set this appropriately for the region

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.mux = embassy_stm32::rcc::ClockSrc::HSI16;
    config.rcc.enable_hsi48 = true;
    let p = embassy_stm32::init(config);

    let mut spi_config = spi::Config::default();
    spi_config.frequency = khz(200);

    // SPI for sx1276
    let spi = spi::Spi::new(p.SPI1, p.PB3, p.PA7, p.PA6, p.DMA1_CH3, p.DMA1_CH2, spi_config);

    let nss = Output::new(p.PA15.degrade(), Level::High, Speed::Low);
    let reset = Output::new(p.PC0.degrade(), Level::High, Speed::Low);

    let irq_pin = Input::new(p.PB4.degrade(), Pull::Up);
    let irq = ExtiInput::new(irq_pin, p.EXTI4.degrade());

    let iv = Stm32l0InterfaceVariant::new(nss, reset, irq, None, None).unwrap();

    let mut delay = Delay;

    let lora = {
        match LoRa::new(SX1276_7_8_9::new(BoardType::Stm32l0Sx1276, spi, iv), true, &mut delay).await {
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
