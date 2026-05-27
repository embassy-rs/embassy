//! This example shows how to use an stm32 as both a master and a slave.
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Address, OwnAddresses, SlaveCommandKind};
use embassy_stm32::mode::Async;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, dma, i2c, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA1_CHANNEL2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
    DMA1_CHANNEL3 => dma::InterruptHandler<peripherals::DMA1_CH3>;
    DMA1_CHANNEL4 => dma::InterruptHandler<peripherals::DMA1_CH4>;
});

const DEV_ADDR: u8 = 0x42;

#[embassy_executor::task]
async fn device_task(mut dev: i2c::I2c<'static, Async, i2c::MultiMaster>) -> ! {
    info!("Device start");

    let mut state = 0;

    loop {
        let mut buf = [0u8; 128];
        match dev.listen().await {
            Ok(i2c::SlaveCommand {
                kind: SlaveCommandKind::Read,
                address: Address::SevenBit(DEV_ADDR),
            }) => match dev.respond_to_read(&[state]).await {
                Ok(i2c::SendStatus::LeftoverBytes(x)) => info!("tried to write {} extra bytes", x),
                Ok(i2c::SendStatus::Done) => {}
                Err(e) => error!("error while responding {}", e),
            },
            Ok(i2c::SlaveCommand {
                kind: SlaveCommandKind::Write,
                address: Address::SevenBit(DEV_ADDR),
            }) => match dev.respond_to_write(&mut buf).await {
                Ok(len) => {
                    info!("Device received write: {}", buf[..len]);

                    if match buf[0] {
                        // Set the state
                        0xC2 => {
                            state = buf[1];
                            true
                        }
                        // Reset State
                        0xC8 => {
                            state = 0;
                            true
                        }
                        x => {
                            error!("Invalid Write Read {:x}", x);
                            false
                        }
                    } {
                        match dev.respond_to_read(&[state]).await {
                            Ok(read_status) => info!(
                                "This read is part of a write/read transaction. The response read status {}",
                                read_status
                            ),
                            Err(i2c::Error::Timeout) => {
                                info!("The device only performed a write and it not also do a read")
                            }
                            Err(e) => error!("error while responding {}", e),
                        }
                    }
                }
                Err(e) => error!("error while receiving {}", e),
            },
            Ok(i2c::SlaveCommand { address, .. }) => {
                defmt::unreachable!(
                    "The slave matched address: {}, which it was not configured for",
                    address
                );
            }
            Err(e) => error!("{}", e),
        }
    }
}

#[embassy_executor::task]
async fn controller_task(mut con: i2c::I2c<'static, Async, i2c::Master>) {
    info!("Controller start");

    loop {
        let mut resp_buff = [0u8; 1];
        for i in 0..10 {
            match con.write_read(DEV_ADDR, &[0xC2, i], &mut resp_buff).await {
                Ok(_) => {
                    info!("write_read response: {}", resp_buff);
                    defmt::assert_eq!(resp_buff[0], i);
                }
                Err(e) => error!("Error writing {}", e),
            }

            Timer::after_millis(100).await;
        }
        match con.read(DEV_ADDR, &mut resp_buff).await {
            Ok(_) => {
                info!("read response: {}", resp_buff);
                // assert that the state is the last index that was written
                defmt::assert_eq!(resp_buff[0], 9);
            }
            Err(e) => error!("Error writing {}", e),
        }
        match con.write_read(DEV_ADDR, &[0xC8], &mut resp_buff).await {
            Ok(_) => {
                info!("write_read response: {}", resp_buff);
                // assert that the state has been reset
                defmt::assert_eq!(resp_buff[0], 0);
            }
            Err(e) => error!("Error writing {}", e),
        }
        Timer::after_millis(100).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut config = i2c::Config::default();
    config.frequency = Hertz::khz(400);

    let d_addr_config = i2c::SlaveAddrConfig {
        addr: OwnAddresses::OA1(Address::SevenBit(DEV_ADDR)),
        general_call: false,
    };
    let d_sda = p.PA8;
    let d_scl = p.PA9;
    let device =
        i2c::I2c::new(p.I2C2, d_scl, d_sda, p.DMA1_CH1, p.DMA1_CH2, Irqs, config).into_slave_multimaster(d_addr_config);

    spawner.spawn(unwrap!(device_task(device)));

    let c_sda = p.PB8;
    let c_scl = p.PB7;
    let controller = i2c::I2c::new(p.I2C1, c_sda, c_scl, p.DMA1_CH3, p.DMA1_CH4, Irqs, config);

    spawner.spawn(unwrap!(controller_task(controller)));
}
