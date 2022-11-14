//! TWIS example

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::interrupt;
use embassy_nrf::twis::{self, Command, Twis};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    let mut config = twis::Config::default();
    // Set i2c address
    config.address0 = 0x55;
    let mut i2c = Twis::new(p.TWISPI0, irq, p.P0_03, p.P0_04, config);

    info!("Listening...");
    loop {
        let mut buf = [0u8; 16];
        let tx_buf = [1, 2, 3, 4, 5, 6, 7, 8];
        match i2c.listen(&mut buf).await {
            Ok(Command::Read) => {
                info!("Got READ command. Writing back data:\n{:?}\n", tx_buf);
                if let Err(e) = i2c.write(&tx_buf).await {
                    error!("{:?}", e);
                }
            }
            Ok(Command::Write(n)) => info!("Got WRITE command with data:\n{:?}\n", buf[..n]),
            Ok(Command::WriteRead(n)) => {
                info!("Got WRITE/READ command with data:\n{:?}", buf[..n]);
                info!("Writing back data:\n{:?}\n", tx_buf);
                if let Err(e) = i2c.write(&tx_buf).await {
                    error!("{:?}", e);
                }
            }
            Err(e) => error!("{:?}", e),
        }
    }
}
