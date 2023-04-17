//! This example runs on the STM32 LoRa Discovery board which has a builtin Semtech Sx127x radio
#![no_std]
#![no_main]
#![macro_use]
#![allow(dead_code)]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_lora::sx127x::*;
use embassy_lora::LoraTimer;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::rng::Rng;
use embassy_stm32::spi;
use embassy_stm32::time::khz;
use lorawan::default_crypto::DefaultFactory as Crypto;
use lorawan_device::async_device::{region, Device, JoinMode};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.mux = embassy_stm32::rcc::ClockSrc::HSI16;
    config.rcc.enable_hsi48 = true;
    let p = embassy_stm32::init(config);

    // SPI for sx127x
    let spi = spi::Spi::new(
        p.SPI1,
        p.PB3,
        p.PA7,
        p.PA6,
        p.DMA1_CH3,
        p.DMA1_CH2,
        khz(200),
        spi::Config::default(),
    );

    let cs = Output::new(p.PA15, Level::High, Speed::Low);
    let reset = Output::new(p.PC0, Level::High, Speed::Low);
    let _ = Input::new(p.PB1, Pull::None);

    let ready = Input::new(p.PB4, Pull::Up);
    let ready_pin = ExtiInput::new(ready, p.EXTI4);

    let radio = Sx127xRadio::new(spi, cs, reset, ready_pin, DummySwitch).await.unwrap();

    let region = region::Configuration::new(region::Region::EU868);
    let mut device: Device<_, Crypto, _, _> = Device::new(region, radio, LoraTimer::new(), Rng::new(p.RNG));

    defmt::info!("Joining LoRaWAN network");

    // TODO: Adjust the EUI and Keys according to your network credentials
    device
        .join(&JoinMode::OTAA {
            deveui: [0, 0, 0, 0, 0, 0, 0, 0],
            appeui: [0, 0, 0, 0, 0, 0, 0, 0],
            appkey: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        })
        .await
        .ok()
        .unwrap();
    defmt::info!("LoRaWAN network joined");

    defmt::info!("Sending 'PING'");
    device.send(b"PING", 1, false).await.ok().unwrap();
    defmt::info!("Message sent!");
}

pub struct DummySwitch;
impl RadioSwitch for DummySwitch {
    fn set_rx(&mut self) {}
    fn set_tx(&mut self) {}
}
