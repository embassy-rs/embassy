#![no_std]
#![no_main]
#![macro_use]
#![allow(dead_code)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

use embassy_lora::{stm32wl::*, LoraTimer};
use embassy_stm32::{
    dma::NoDma,
    gpio::{Level, Output, Pin, Speed},
    interrupt, pac,
    rng::Rng,
    subghz::*,
    Peripherals,
};
use lorawan::default_crypto::DefaultFactory as Crypto;
use lorawan_device::async_device::{region, Device, JoinMode};

fn config() -> embassy_stm32::Config {
    let mut config = embassy_stm32::Config::default();
    config.rcc.mux = embassy_stm32::rcc::ClockSrc::HSI16;
    config.rcc.enable_lsi = true;
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: embassy::executor::Spawner, p: Peripherals) {
    unsafe { pac::RCC.ccipr().modify(|w| w.set_rngsel(0b01)) }

    let ctrl1 = Output::new(p.PC3.degrade(), Level::High, Speed::High);
    let ctrl2 = Output::new(p.PC4.degrade(), Level::High, Speed::High);
    let ctrl3 = Output::new(p.PC5.degrade(), Level::High, Speed::High);
    let rfs = RadioSwitch::new(ctrl1, ctrl2, ctrl3);

    let radio = SubGhz::new(p.SUBGHZSPI, p.PA5, p.PA7, p.PA6, NoDma, NoDma);

    let irq = interrupt::take!(SUBGHZ_RADIO);
    static mut RADIO_STATE: SubGhzState<'static> = SubGhzState::new();
    let radio = unsafe { SubGhzRadio::new(&mut RADIO_STATE, radio, rfs, irq) };

    let region = region::EU868::default().into();
    let mut device: Device<_, Crypto, _, _> =
        Device::new(region, radio, LoraTimer, Rng::new(p.RNG));

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
