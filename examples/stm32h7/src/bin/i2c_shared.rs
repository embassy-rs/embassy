#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::*;
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::mode::Async;
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Timer};
use embedded_hal_1::i2c::I2c as _;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const TMP117_ADDR: u8 = 0x48;
const TMP117_TEMP_RESULT: u8 = 0x00;

const SHTC3_ADDR: u8 = 0x70;
const SHTC3_WAKEUP: [u8; 2] = [0x35, 0x17];
const SHTC3_MEASURE_RH_FIRST: [u8; 2] = [0x5c, 0x24];
const SHTC3_SLEEP: [u8; 2] = [0xb0, 0x98];

static I2C_BUS: StaticCell<NoopMutex<RefCell<I2c<'static, Async, i2c::Master>>>> = StaticCell::new();

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::task]
async fn temperature(mut i2c: I2cDevice<'static, NoopRawMutex, I2c<'static, Async, i2c::Master>>) {
    let mut data = [0u8; 2];

    loop {
        match i2c.write_read(TMP117_ADDR, &[TMP117_TEMP_RESULT], &mut data) {
            Ok(()) => {
                let temp = f32::from(i16::from_be_bytes(data)) * 7.8125 / 1000.0;
                info!("Temperature {}", temp);
            }
            Err(_) => error!("I2C Error"),
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy_executor::task]
async fn humidity(mut i2c: I2cDevice<'static, NoopRawMutex, I2c<'static, Async, i2c::Master>>) {
    let mut data = [0u8; 6];

    loop {
        // Wakeup
        match i2c.write(SHTC3_ADDR, &SHTC3_WAKEUP) {
            Ok(()) => Timer::after(Duration::from_millis(20)).await,
            Err(_) => error!("I2C Error"),
        }

        // Measurement
        match i2c.write(SHTC3_ADDR, &SHTC3_MEASURE_RH_FIRST) {
            Ok(()) => Timer::after(Duration::from_millis(5)).await,
            Err(_) => error!("I2C Error"),
        }

        // Result
        match i2c.read(SHTC3_ADDR, &mut data) {
            Ok(()) => Timer::after(Duration::from_millis(5)).await,
            Err(_) => error!("I2C Error"),
        }

        // Sleep
        match i2c.write(SHTC3_ADDR, &SHTC3_SLEEP) {
            Ok(()) => {
                let (bytes, _) = data.split_at(core::mem::size_of::<i16>());
                let rh = f32::from(u16::from_be_bytes(bytes.try_into().unwrap())) * 100.0 / 65536.0;
                info!("Humidity: {}", rh);
            }
            Err(_) => error!("I2C Error"),
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let i2c = I2c::new(p.I2C1, p.PB8, p.PB9, Irqs, p.DMA1_CH4, p.DMA1_CH5, Default::default());
    let i2c_bus = NoopMutex::new(RefCell::new(i2c));
    let i2c_bus = I2C_BUS.init(i2c_bus);

    // Device 1, using embedded-hal-async compatible driver for TMP117
    let i2c_dev1 = I2cDevice::new(i2c_bus);
    spawner.spawn(temperature(i2c_dev1).unwrap());

    // Device 2, using embedded-hal-async compatible driver for SHTC3
    let i2c_dev2 = I2cDevice::new(i2c_bus);
    spawner.spawn(humidity(i2c_dev2).unwrap());
}
