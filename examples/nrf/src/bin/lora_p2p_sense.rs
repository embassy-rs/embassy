//! This example runs on the RAK4631 WisBlock, which has an nRF52840 MCU and Semtech Sx126x radio.
//! Other nrf/sx126x combinations may work with appropriate pin modifications.
//! It demonstates LORA P2P functionality in conjunction with example lora_p2p_report.rs.
#![no_std]
#![no_main]
#![macro_use]
#![feature(type_alias_impl_trait)]
#![feature(alloc_error_handler)]
#![allow(incomplete_features)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_lora::sx126x::*;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pin as _, Pull};
use embassy_nrf::{interrupt, spim};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher};
use embassy_time::{Duration, Timer};
use lorawan_device::async_device::radio::{Bandwidth, CodingRate, PhyRxTx, RfConfig, SpreadingFactor, TxConfig};
use {defmt_rtt as _, panic_probe as _, panic_probe as _};

// Message bus: queue of 2, 1 subscriber (Lora P2P), 2 publishers (temperature, motion detection)
static MESSAGE_BUS: PubSubChannel<CriticalSectionRawMutex, Message, 2, 1, 2> = PubSubChannel::new();

#[derive(Clone, defmt::Format)]
enum Message {
    Temperature(i32),
    MotionDetected,
}

#[embassy_executor::task]
async fn temperature_task(publisher: Publisher<'static, CriticalSectionRawMutex, Message, 2, 1, 2>) {
    // Publish a fake temperature every 43 seconds, minimizing LORA traffic.
    loop {
        Timer::after(Duration::from_secs(43)).await;
        publisher.publish(Message::Temperature(9)).await;
    }
}

#[embassy_executor::task]
async fn motion_detection_task(publisher: Publisher<'static, CriticalSectionRawMutex, Message, 2, 1, 2>) {
    // Publish a fake motion detection every 79 seconds, minimizing LORA traffic.
    loop {
        Timer::after(Duration::from_secs(79)).await;
        publisher.publish(Message::MotionDetected).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    // set up to funnel temperature and motion detection events to the Lora Tx task
    let mut lora_tx_subscriber = unwrap!(MESSAGE_BUS.subscriber());
    let temperature_publisher = unwrap!(MESSAGE_BUS.publisher());
    let motion_detection_publisher = unwrap!(MESSAGE_BUS.publisher());

    let mut spi_config = spim::Config::default();
    spi_config.frequency = spim::Frequency::M16;

    let mut radio = {
        let irq = interrupt::take!(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
        let spim = spim::Spim::new(p.TWISPI1, irq, p.P1_11, p.P1_13, p.P1_12, spi_config);

        let cs = Output::new(p.P1_10.degrade(), Level::High, OutputDrive::Standard);
        let reset = Output::new(p.P1_06.degrade(), Level::High, OutputDrive::Standard);
        let dio1 = Input::new(p.P1_15.degrade(), Pull::Down);
        let busy = Input::new(p.P1_14.degrade(), Pull::Down);
        let antenna_rx = Output::new(p.P1_05.degrade(), Level::Low, OutputDrive::Standard);
        let antenna_tx = Output::new(p.P1_07.degrade(), Level::Low, OutputDrive::Standard);

        match Sx126xRadio::new(spim, cs, reset, antenna_rx, antenna_tx, dio1, busy, false).await {
            Ok(r) => r,
            Err(err) => {
                info!("Sx126xRadio error = {}", err);
                return;
            }
        }
    };

    let mut start_indicator = Output::new(p.P1_04, Level::Low, OutputDrive::Standard);

    start_indicator.set_high();
    Timer::after(Duration::from_secs(5)).await;
    start_indicator.set_low();

    match radio.lora.sleep().await {
        Ok(()) => info!("Sleep successful"),
        Err(err) => info!("Sleep unsuccessful = {}", err),
    }

    unwrap!(spawner.spawn(temperature_task(temperature_publisher)));
    unwrap!(spawner.spawn(motion_detection_task(motion_detection_publisher)));

    loop {
        let message = lora_tx_subscriber.next_message_pure().await;

        let tx_config = TxConfig {
            // 11 byte maximum payload for Bandwidth 125 and SF 10
            pw: 10, // up to 20
            rf: RfConfig {
                frequency: 903900000, // channel in Hz, not MHz
                bandwidth: Bandwidth::_250KHz,
                spreading_factor: SpreadingFactor::_10,
                coding_rate: CodingRate::_4_8,
            },
        };

        let mut buffer = [0x00u8];
        match message {
            Message::Temperature(temperature) => buffer[0] = temperature as u8,
            Message::MotionDetected => buffer[0] = 0x01u8,
        };

        // unencrypted
        match radio.tx(tx_config, &buffer).await {
            Ok(ret_val) => info!("TX ret_val = {}", ret_val),
            Err(err) => info!("TX error = {}", err),
        }

        match radio.lora.sleep().await {
            Ok(()) => info!("Sleep successful"),
            Err(err) => info!("Sleep unsuccessful = {}", err),
        }
    }
}
