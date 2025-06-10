//! This example shows how to use the 2040 as an i2c slave.
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{I2C0, I2C1};
use embassy_rp::{bind_interrupts, i2c, i2c_slave};
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

const DEV_ADDR: u8 = 0x42;

#[embassy_executor::task]
async fn device_task(mut dev: i2c_slave::I2cSlave<'static, I2C1>) -> ! {
    info!("Device start");

    let mut state = 0;

    loop {
        let mut buf = [0u8; 128];
        match dev.listen(&mut buf).await {
            Ok(i2c_slave::Command::GeneralCall(len)) => info!("Device received general call write: {}", buf[..len]),
            Ok(i2c_slave::Command::Read) => loop {
                match dev.respond_to_read(&[state]).await {
                    Ok(x) => match x {
                        i2c_slave::ReadStatus::Done => break,
                        i2c_slave::ReadStatus::NeedMoreBytes => (),
                        i2c_slave::ReadStatus::LeftoverBytes(x) => {
                            info!("tried to write {} extra bytes", x);
                            break;
                        }
                    },
                    Err(e) => error!("error while responding {}", e),
                }
            },
            Ok(i2c_slave::Command::Write(len)) => info!("Device received write: {}", buf[..len]),
            Ok(i2c_slave::Command::WriteRead(len)) => {
                info!("device received write read: {:x}", buf[..len]);
                match buf[0] {
                    // Set the state
                    0xC2 => {
                        state = buf[1];
                        match dev.respond_and_fill(&[state], 0x00).await {
                            Ok(read_status) => info!("response read status {}", read_status),
                            Err(e) => error!("error while responding {}", e),
                        }
                    }
                    // Reset State
                    0xC8 => {
                        state = 0;
                        match dev.respond_and_fill(&[state], 0x00).await {
                            Ok(read_status) => info!("response read status {}", read_status),
                            Err(e) => error!("error while responding {}", e),
                        }
                    }
                    x => error!("Invalid Write Read {:x}", x),
                }
            }
            Err(e) => error!("{}", e),
        }
    }
}

#[embassy_executor::task]
async fn controller_task(mut con: i2c::I2c<'static, I2C0, i2c::Async>) {
    info!("Controller start");

    loop {
        let mut resp_buff = [0u8; 2];
        for i in 0..10 {
            match con.write_read(DEV_ADDR, &[0xC2, i], &mut resp_buff).await {
                Ok(_) => info!("write_read response: {}", resp_buff),
                Err(e) => error!("Error writing {}", e),
            }

            Timer::after_millis(100).await;
        }
        match con.read(DEV_ADDR, &mut resp_buff).await {
            Ok(_) => info!("read response: {}", resp_buff),
            Err(e) => error!("Error writing {}", e),
        }
        match con.write_read(DEV_ADDR, &[0xC8], &mut resp_buff).await {
            Ok(_) => info!("write_read response: {}", resp_buff),
            Err(e) => error!("Error writing {}", e),
        }
        Timer::after_millis(100).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let d_sda = p.PIN_2;
    let d_scl = p.PIN_3;
    let mut config = i2c_slave::Config::default();
    config.addr = DEV_ADDR as u16;
    let device = i2c_slave::I2cSlave::new(p.I2C1, d_scl, d_sda, Irqs, config);

    unwrap!(spawner.spawn(device_task(device)));

    let c_sda = p.PIN_0;
    let c_scl = p.PIN_1;
    let mut config = i2c::Config::default();
    config.frequency = 1_000_000;
    let controller = i2c::I2c::new_async(p.I2C0, c_scl, c_sda, Irqs, config);

    unwrap!(spawner.spawn(controller_task(controller)));
}
