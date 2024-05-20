//! This example test the RP Pico W on board LED.
//!
//! It does not work with the RP Pico board. See blinky.rs.

#![no_std]
#![no_main]

use core::cell::RefCell;
use core::future::Future;
use core::ops::DerefMut;

use bt_hci::cmd::{AsyncCmd, SyncCmd};
use bt_hci::{
    data, param, Controller, ControllerCmdAsync, ControllerCmdSync, ControllerToHostPacket, ReadHci, WithIndicator,
    WriteHci,
};
use cyw43_pio::PioSpi;
use defmt::{todo, *};
use embassy_executor::{Executor, Spawner};
use embassy_futures::join::join3;
use embassy_futures::yield_now;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use embedded_io_async::Read;
use static_cell::StaticCell;
use trouble_host::adapter::{Adapter, HostResources};
use trouble_host::advertise::{AdStructure, AdvertiseConfig, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE};
use trouble_host::attribute::{AttributeTable, Characteristic, CharacteristicProp, Service, Uuid};
use trouble_host::PacketQos;
use {defmt_rtt as _, embassy_time as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let fw = include_bytes!("../../../../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../../../../cyw43-firmware/43439A0_clm.bin");
    let btfw = include_bytes!("../../../../cyw43-firmware/43439A0_btfw.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download 43439A0.bin --format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download 43439A0_clm.bin --format bin --chip RP2040 --base-address 0x10140000
    //let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 224190) };
    //let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(&mut pio.common, pio.sm0, pio.irq0, cs, p.PIN_24, p.PIN_29, p.DMA_CH0);

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, mut runner) = cyw43::new_with_bluetooth(state, pwr, spi, fw, btfw).await;
    //unwrap!(spawner.spawn(wifi_task(runner)));
    //control.init(clm, false, true).await;

    Timer::after_millis(1000).await;

    let mut buf = [0u8; 512];
    let n = runner.hci_read(&mut buf).await;
    info!("read: {:02x}", &buf[..n]);

    let pkt = &[0x01, 0x14, 0x0c, 0x00];
    //let pkt = &[0x03, 0x00, 0x00, 0x01, 0x14, 0x0c, 0x00];
    runner.hci_write(pkt).await;
}
