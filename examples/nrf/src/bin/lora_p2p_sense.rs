//! This example runs on the RAK4631 WisBlock, which has an nRF52840 MCU and Semtech Sx126x radio.
#![no_std]
#![no_main]
#![macro_use]
#![feature(type_alias_impl_trait)]
#![feature(alloc_error_handler)]
#![allow(incomplete_features)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_lora::sx126x::*;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin as _, Pull};
use embassy_nrf::temp::Temp;
use embassy_nrf::{interrupt, spim};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher};
use embassy_time::{Duration, Timer};
use lorawan_device::async_device::radio::{Bandwidth, CodingRate, PhyRxTx, RfConfig, SpreadingFactor, TxConfig};
use {defmt_rtt as _, panic_probe as _, panic_probe as _};

// Sensor packet constants
const TEMPERATURE_UID: u8 = 0x01;
const MOTION_UID: u8 = 0x02;

// Message bus: queue of 2, 1 subscriber (Lora P2P), 2 publishers (temperature, motion detection)
static MESSAGE_BUS: PubSubChannel<CriticalSectionRawMutex, Message, 2, 1, 2> = PubSubChannel::new();

#[derive(Clone, defmt::Format)]
enum Message {
    Temperature(i32),
    MotionDetected,
}

#[embassy_executor::task]
async fn temperature_task(
    mut temperature: Temp<'static>,
    publisher: Publisher<'static, CriticalSectionRawMutex, Message, 2, 1, 2>,
) {
    Timer::after(Duration::from_secs(45)).await; // stabilize for 45 seconds

    let mut temperature_reporting_threshhold = 10;

    loop {
        let value = temperature.read().await;
        let mut temperature_val = value.to_num::<i32>();

        info!("Temperature: {}", temperature_val);

        // only report every 2 degree Celsius drops, from 9 through 5, but starting at 3 always report

        if temperature_val == 8 || temperature_val == 6 || temperature_val == 4 {
            temperature_val += 1;
        }

        if temperature_reporting_threshhold > temperature_val
            && (temperature_val == 9 || temperature_val == 7 || temperature_val == 5)
        {
            temperature_reporting_threshhold = temperature_val;
            publisher.publish(Message::Temperature(temperature_val)).await;
        } else if temperature_val <= 3 {
            publisher.publish(Message::Temperature(temperature_val)).await;
        }

        Timer::after(Duration::from_secs(20 * 60)).await;
    }
}

#[embassy_executor::task]
async fn motion_detection_task(
    mut pir_pin: Input<'static, AnyPin>,
    publisher: Publisher<'static, CriticalSectionRawMutex, Message, 2, 1, 2>,
) {
    Timer::after(Duration::from_secs(30)).await; // stabilize for 30 seconds

    loop {
        // wait for motion detection
        pir_pin.wait_for_low().await;
        publisher.publish(Message::MotionDetected).await;

        // wait a minute before setting up for more motion detection
        Timer::after(Duration::from_secs(60)).await;
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
    spi_config.frequency = spim::Frequency::M1; // M16 ???

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

    // set up for the temperature task
    let temperature_irq = interrupt::take!(TEMP);
    let temperature = Temp::new(p.TEMP, temperature_irq);

    // set the motion detection pin
    let pir_pin = Input::new(p.P0_10.degrade(), Pull::Up);

    let mut start_indicator = Output::new(p.P1_04, Level::Low, OutputDrive::Standard);
    let mut debug_indicator = Output::new(p.P1_03, Level::Low, OutputDrive::Standard);

    start_indicator.set_high();
    Timer::after(Duration::from_secs(5)).await;
    start_indicator.set_low();

    match radio.lora.sleep().await {
        Ok(()) => info!("Sleep successful"),
        Err(err) => info!("Sleep unsuccessful = {}", err),
    }

    unwrap!(spawner.spawn(temperature_task(temperature, temperature_publisher)));
    unwrap!(spawner.spawn(motion_detection_task(pir_pin, motion_detection_publisher)));

    loop {
        let message = lora_tx_subscriber.next_message_pure().await;

        let tx_config = TxConfig {
            // 11 byte maximum payload for Bandwidth 125 and SF 10
            pw: 20, // up to 20 // 5 ???
            rf: RfConfig {
                frequency: 903900000, // channel in Hz, not MHz
                bandwidth: Bandwidth::_250KHz,
                spreading_factor: SpreadingFactor::_10,
                coding_rate: CodingRate::_4_8,
            },
        };

        let mut buffer = [TEMPERATURE_UID, 0xffu8, MOTION_UID, 0x00u8];
        match message {
            Message::Temperature(temperature) => buffer[1] = temperature as u8,
            Message::MotionDetected => buffer[3] = 0x01u8,
        };

        // crypto for text ???
        match radio.tx(tx_config, &buffer).await {
            Ok(ret_val) => info!("TX ret_val = {}", ret_val),
            Err(err) => info!("TX error = {}", err),
        }

        match radio.lora.sleep().await {
            Ok(()) => info!("Sleep successful"),
            Err(err) => info!("Sleep unsuccessful = {}", err),
        }

        debug_indicator.set_high();
        Timer::after(Duration::from_secs(5)).await;
        debug_indicator.set_low();
    }
}
