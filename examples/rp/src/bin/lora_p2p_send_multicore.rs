//! This example runs on the Raspberry Pi Pico with a Waveshare board containing a Semtech Sx1262 radio.
//! It demonstrates LORA P2P send functionality using the second core, with data provided by the first core.

#![no_std]
#![no_main]
#![macro_use]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Executor;
use embassy_lora::iv::GenericSx126xInterfaceVariant;
use embassy_rp::gpio::{AnyPin, Input, Level, Output, Pin, Pull};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::SPI1;
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Delay, Duration, Timer};
use lora_phy::mod_params::*;
use lora_phy::sx1261_2::SX1261_2;
use lora_phy::LoRa;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static CHANNEL: Channel<CriticalSectionRawMutex, [u8; 3], 1> = Channel::new();

const LORA_FREQUENCY_IN_HZ: u32 = 903_900_000; // warning: set this appropriately for the region

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());

    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let clk = p.PIN_10;
    let spi = Spi::new(p.SPI1, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Config::default());

    let nss = Output::new(p.PIN_3.degrade(), Level::High);
    let reset = Output::new(p.PIN_15.degrade(), Level::High);
    let dio1 = Input::new(p.PIN_20.degrade(), Pull::None);
    let busy = Input::new(p.PIN_2.degrade(), Pull::None);

    let iv = GenericSx126xInterfaceVariant::new(nss, reset, dio1, busy, None, None).unwrap();

    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner| unwrap!(spawner.spawn(core1_task(spi, iv))));
    });

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_task())));
}

#[embassy_executor::task]
async fn core0_task() {
    info!("Hello from core 0");
    loop {
        CHANNEL.send([0x01u8, 0x02u8, 0x03u8]).await;
        Timer::after(Duration::from_millis(60 * 1000)).await;
    }
}

#[embassy_executor::task]
async fn core1_task(
    spi: Spi<'static, SPI1, Async>,
    iv: GenericSx126xInterfaceVariant<Output<'static, AnyPin>, Input<'static, AnyPin>>,
) {
    info!("Hello from core 1");
    let mut delay = Delay;

    let mut lora = {
        match LoRa::new(
            SX1261_2::new(BoardType::RpPicoWaveshareSx1262, spi, iv),
            false,
            &mut delay,
        )
        .await
        {
            Ok(l) => l,
            Err(err) => {
                info!("Radio error = {}", err);
                return;
            }
        }
    };

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

    let mut tx_pkt_params = {
        match lora.create_tx_packet_params(4, false, true, false, &mdltn_params) {
            Ok(pp) => pp,
            Err(err) => {
                info!("Radio error = {}", err);
                return;
            }
        }
    };

    loop {
        let buffer: [u8; 3] = CHANNEL.receive().await;
        match lora.prepare_for_tx(&mdltn_params, 20, false).await {
            Ok(()) => {}
            Err(err) => {
                info!("Radio error = {}", err);
                return;
            }
        };

        match lora.tx(&mdltn_params, &mut tx_pkt_params, &buffer, 0xffffff).await {
            Ok(()) => {
                info!("TX DONE");
            }
            Err(err) => {
                info!("Radio error = {}", err);
                return;
            }
        };

        match lora.sleep(&mut delay).await {
            Ok(()) => info!("Sleep successful"),
            Err(err) => info!("Sleep unsuccessful = {}", err),
        }
    }
}
