//! This example shows how to share (async) I2C and SPI buses between multiple devices.

#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::{self, I2c, InterruptHandler};
use embassy_rp::peripherals::{I2C1, SPI1};
use embassy_rp::spi::{self, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

type Spi1Bus = Mutex<NoopRawMutex, Spi<'static, SPI1, spi::Async>>;
type I2c1Bus = Mutex<NoopRawMutex, I2c<'static, I2C1, i2c::Async>>;

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Here we go!");

    // Shared I2C bus
    let i2c = I2c::new_async(p.I2C1, p.PIN_15, p.PIN_14, Irqs, i2c::Config::default());
    static I2C_BUS: StaticCell<I2c1Bus> = StaticCell::new();
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

    spawner.must_spawn(i2c_task_a(i2c_bus));
    spawner.must_spawn(i2c_task_b(i2c_bus));

    // Shared SPI bus
    let spi_cfg = spi::Config::default();
    let spi = Spi::new(p.SPI1, p.PIN_10, p.PIN_11, p.PIN_12, p.DMA_CH0, p.DMA_CH1, spi_cfg);
    static SPI_BUS: StaticCell<Spi1Bus> = StaticCell::new();
    let spi_bus = SPI_BUS.init(Mutex::new(spi));

    // Chip select pins for the SPI devices
    let cs_a = Output::new(p.PIN_0, Level::High);
    let cs_b = Output::new(p.PIN_1, Level::High);

    spawner.must_spawn(spi_task_a(spi_bus, cs_a));
    spawner.must_spawn(spi_task_b(spi_bus, cs_b));
}

#[embassy_executor::task]
async fn i2c_task_a(i2c_bus: &'static I2c1Bus) {
    let i2c_dev = I2cDevice::new(i2c_bus);
    let _sensor = DummyI2cDeviceDriver::new(i2c_dev, 0xC0);
    loop {
        info!("i2c task A");
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn i2c_task_b(i2c_bus: &'static I2c1Bus) {
    let i2c_dev = I2cDevice::new(i2c_bus);
    let _sensor = DummyI2cDeviceDriver::new(i2c_dev, 0xDE);
    loop {
        info!("i2c task B");
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn spi_task_a(spi_bus: &'static Spi1Bus, cs: Output<'static>) {
    let spi_dev = SpiDevice::new(spi_bus, cs);
    let _sensor = DummySpiDeviceDriver::new(spi_dev);
    loop {
        info!("spi task A");
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn spi_task_b(spi_bus: &'static Spi1Bus, cs: Output<'static>) {
    let spi_dev = SpiDevice::new(spi_bus, cs);
    let _sensor = DummySpiDeviceDriver::new(spi_dev);
    loop {
        info!("spi task B");
        Timer::after_secs(1).await;
    }
}

// Dummy I2C device driver, using `embedded-hal-async`
struct DummyI2cDeviceDriver<I2C: embedded_hal_async::i2c::I2c> {
    _i2c: I2C,
}

impl<I2C: embedded_hal_async::i2c::I2c> DummyI2cDeviceDriver<I2C> {
    fn new(i2c_dev: I2C, _address: u8) -> Self {
        Self { _i2c: i2c_dev }
    }
}

// Dummy SPI device driver, using `embedded-hal-async`
struct DummySpiDeviceDriver<SPI: embedded_hal_async::spi::SpiDevice> {
    _spi: SPI,
}

impl<SPI: embedded_hal_async::spi::SpiDevice> DummySpiDeviceDriver<SPI> {
    fn new(spi_dev: SPI) -> Self {
        Self { _spi: spi_dev }
    }
}
