//! This example runs on the STM32 LoRa Discovery board which has a builtin Semtech Sx127x radio
#![no_std]
#![no_main]
#![macro_use]
#![allow(dead_code)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy_lora::{sx127x::*, LoraTimer};
use embassy_stm32::{
    dbgmcu::Dbgmcu,
    dma::NoDma,
    exti::ExtiInput,
    gpio::{Input, Level, Output, Pull, Speed},
    rcc,
    rng::Rng,
    spi,
    time::U32Ext,
    Peripherals,
};
use lorawan_device::async_device::{region, Device, JoinMode};
use lorawan_encoding::default_crypto::DefaultFactory as Crypto;

fn config() -> embassy_stm32::Config {
    let mut config = embassy_stm32::Config::default();
    config.rcc = config.rcc.clock_src(embassy_stm32::rcc::ClockSrc::HSI16);
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: embassy::executor::Spawner, mut p: Peripherals) {
    unsafe {
        Dbgmcu::enable_all();
    }

    let mut rcc = rcc::Rcc::new(p.RCC);
    let _ = rcc.enable_hsi48(&mut p.SYSCFG, p.CRS);

    // SPI for sx127x
    let spi = spi::Spi::new(
        p.SPI1,
        p.PB3,
        p.PA7,
        p.PA6,
        NoDma,
        NoDma,
        200_000.hz(),
        spi::Config::default(),
    );

    let cs = Output::new(p.PA15, Level::High, Speed::Low);
    let reset = Output::new(p.PC0, Level::High, Speed::Low);
    let _ = Input::new(p.PB1, Pull::None);

    let ready = Input::new(p.PB4, Pull::Up);
    let ready_pin = ExtiInput::new(ready, p.EXTI4);

    let radio = Sx127xRadio::new(
        spi,
        cs,
        reset,
        ready_pin,
        DummySwitch,
        &mut embassy::time::Delay,
    )
    .unwrap();

    let region = region::EU868::default().into();
    let mut radio_buffer = [0; 256];
    let mut device: Device<'_, _, Crypto, _, _> = Device::new(
        region,
        radio,
        LoraTimer,
        Rng::new(p.RNG),
        &mut radio_buffer[..],
    );

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
