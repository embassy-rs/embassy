//! This example shows how to use the 2040 as an i2c slave.
#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{I2C0, I2C1, PIN_25};
use embassy_rp::{bind_interrupts, i2c, i2c_slave};
use embassy_time::{Duration, Timer};
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
            Ok(i2c_slave::Command::GeneralCall(len)) => info!("Device recieved general call write: {:x}", buf[..len]),
            Ok(i2c_slave::Command::Read) => loop {
                info!("Device received read");
                match dev.respond_to_read(&[state, state]).await {
                    Ok(x) => match x {
                        i2c_slave::ReadStatus::Done => break,
                        i2c_slave::ReadStatus::NeedMoreBytes => error!("need more bytes?"),
                        i2c_slave::ReadStatus::LeftoverBytes(x) => {
                            info!("tried to write {} extra bytes", x);
                            break;
                        }
                    },
                    Err(e) => {
                        error!("error while responding {}", e);
                        break;
                    }
                }
            },
            Ok(i2c_slave::Command::Write(len)) => info!("Device recieved write: {:x}", buf[..len]),
            Ok(i2c_slave::Command::WriteRead(len)) => {
                info!("device recieved write read: {:x}", buf[..len]);
                match buf[0] {
                    // Set the state
                    0xC2 => {
                        defmt::assert_eq!(len, 2);
                        state = buf[1];
                        match dev.respond_and_fill(&[state], 0x00).await {
                            Ok(read_status) => info!("set state status {}", read_status),
                            Err(e) => error!("error while responding to set state{}", e),
                        }
                    }
                    // Reset State
                    0xC8 => {
                        defmt::assert_eq!(len, 1);
                        state = 0;
                        match dev.respond_and_fill(&[state], 0x00).await {
                            Ok(read_status) => info!("reset state status {}", read_status),
                            Err(e) => error!("error while responding reset state {}", e),
                        }
                    }
                    // Read States
                    0xC9 => {
                        defmt::assert_eq!(len, 1);
                        match dev.respond_and_fill(&[state], 0x00).await {
                            Ok(read_status) => info!("read state status {}", read_status),
                            Err(e) => error!("error while responding to read state {}", e),
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
        for i in 0..1 {
            match con.write_read(DEV_ADDR, &[0xC2, i], &mut resp_buff).await {
                Ok(_) => info!("write_read response: {}", resp_buff),
                Err(e) => error!("Error writing {}", e),
            }

            Timer::after(Duration::from_millis(100)).await;
        }
        match con.write(DEV_ADDR, &[0xDE, 0xAD, 0xBE, 0xEF]).await {
            Ok(_) => info!("(1) wrote bytes"),
            Err(e) => error!("(1) Error writing {}", e),
        }
        match con.read(DEV_ADDR, &mut resp_buff).await {
            Ok(_) => info!("(2) read response: {:x}", resp_buff),
            Err(e) => error!("(2) Error writing {}", e),
        }
        match con.write_read(DEV_ADDR, &[0xC9], &mut resp_buff).await {
            Ok(_) => info!("(3) write_read response: {}", resp_buff),
            Err(e) => error!("(3) Error writing {}", e),
        }
        match con.read(DEV_ADDR, &mut resp_buff).await {
            Ok(_) => info!("(4) read response: {:x}", resp_buff),
            Err(e) => error!("(4) Error writing {}", e),
        }
        match con.write(DEV_ADDR, &[0xDA, 0xBB, 0xAD, 0x00]).await {
            Ok(_) => info!("(5) wrote bytes"),
            Err(e) => error!("(5) Error writing {}", e),
        }
        match con.write_read(DEV_ADDR, &[0xC8], &mut resp_buff).await {
            Ok(_) => info!("(6) write_read response: {}", resp_buff),
            Err(e) => error!("(6) Error writing {}", e),
        }
        Timer::after(Duration::from_micros(100)).await;
    }
}

#[embassy_executor::task]
async fn toggle_led(mut o: Output<'static, PIN_25>) {
    info!("Blinky start");

    loop {
        o.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    #[cfg(feature = "i2c_slave")]
    {
        let d_scl = p.PIN_11;
        let d_sda = p.PIN_10;
        let mut config = i2c_slave::Config::default();
        config.addr = DEV_ADDR as u16;
        let device = i2c_slave::I2cSlave::new(p.I2C1, d_scl, d_sda, Irqs, config);

        unwrap!(spawner.spawn(device_task(device)));
    }

    #[cfg(feature = "i2c_master")]
    {
        let c_scl = p.PIN_17;
        let c_sda = p.PIN_16;
        let mut config = i2c::Config::default();
        config.frequency = 400_000;
        let controller = i2c::I2c::new_async(p.I2C0, c_scl, c_sda, Irqs, config);

        unwrap!(spawner.spawn(controller_task(controller)));
    }

    let c_led = p.PIN_25;
    let output = Output::new(c_led, Level::Low);
    unwrap!(spawner.spawn(toggle_led(output)));
}
