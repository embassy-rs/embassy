//! This example shows how to communicate asynchronous using i2c with external chip.
//!
//! It's using embassy's functions directly instead of traits from embedded_hal_async::i2c::I2c.
//! While most of i2c devices are addressed using 7 bits, an extension allows 10 bits too.

#![no_std]
#![no_main]

use defmt::*;
use embassy_rp::i2c::InterruptHandler;
use {defmt_rtt as _, panic_probe as _};

// Our anonymous hypotetical temperature sensor could be:
// a 12-bit sensor, with 100ms startup time, range of -40*C - 125*C, and precision 0.25*C
// It requires no configuration or calibration, works with all i2c bus speeds,
// never stretches clock or does anything complicated. Replies with one u16.
// It requires only one write to take it out of suspend mode, and stays on.
// Often result would be just on 12 bits, but here we'll simplify it to 16.

enum UncomplicatedSensorId {
    A(UncomplicatedSensorU8),
    B(UncomplicatedSensorU16),
}
enum UncomplicatedSensorU8 {
    First = 0x48,
}
enum UncomplicatedSensorU16 {
    Other = 0x0049,
}

impl Into<u16> for UncomplicatedSensorU16 {
    fn into(self) -> u16 {
        self as u16
    }
}
impl Into<u16> for UncomplicatedSensorU8 {
    fn into(self) -> u16 {
        0x48
    }
}
impl From<UncomplicatedSensorId> for u16 {
    fn from(t: UncomplicatedSensorId) -> Self {
        match t {
            UncomplicatedSensorId::A(x) => x.into(),
            UncomplicatedSensorId::B(x) => x.into(),
        }
    }
}

embassy_rp::bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<embassy_rp::peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_task_spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default());
    let sda = p.PIN_14;
    let scl = p.PIN_15;
    let config = embassy_rp::i2c::Config::default();
    let mut bus = embassy_rp::i2c::I2c::new_async(p.I2C1, scl, sda, Irqs, config);

    const WAKEYWAKEY: u16 = 0xBABE;
    let mut result: [u8; 2] = [0, 0];
    // wait for sensors to initialize
    embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;

    let _res_1 = bus
        .write_async(UncomplicatedSensorU8::First, WAKEYWAKEY.to_be_bytes())
        .await;
    let _res_2 = bus
        .write_async(UncomplicatedSensorU16::Other, WAKEYWAKEY.to_be_bytes())
        .await;

    loop {
        let s1 = UncomplicatedSensorId::A(UncomplicatedSensorU8::First);
        let s2 = UncomplicatedSensorId::B(UncomplicatedSensorU16::Other);
        let sensors = [s1, s2];
        for sensor in sensors {
            if bus.read_async(sensor, &mut result).await.is_ok() {
                info!("Result {}", u16::from_be_bytes(result.into()));
            }
        }
        embassy_time::Timer::after(embassy_time::Duration::from_millis(200)).await;
    }
}
