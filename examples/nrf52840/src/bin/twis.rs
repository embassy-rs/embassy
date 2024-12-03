//! TWIS example

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twis::{self, Command, Twis};
use embassy_nrf::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twis::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut config = twis::Config::default();
    config.address0 = 0x55; // Set i2c address
    let mut i2c = Twis::new(p.TWISPI0, Irqs, p.P0_03, p.P0_04, config);

    info!("Listening...");
    loop {
        let response = [1, 2, 3, 4, 5, 6, 7, 8];
        // This buffer is used if the i2c master performs a Write or WriteRead
        let mut buf = [0u8; 16];
        match i2c.listen(&mut buf).await {
            Ok(Command::Read) => {
                info!("Got READ command. Respond with data:\n{:?}\n", response);
                if let Err(e) = i2c.respond_to_read(&response).await {
                    error!("{:?}", e);
                }
            }
            Ok(Command::Write(n)) => info!("Got WRITE command with data:\n{:?}\n", buf[..n]),
            Ok(Command::WriteRead(n)) => {
                info!("Got WRITE/READ command with data:\n{:?}", buf[..n]);
                info!("Respond with data:\n{:?}\n", response);
                if let Err(e) = i2c.respond_to_read(&response).await {
                    error!("{:?}", e);
                }
            }
            Err(e) => error!("{:?}", e),
        }
    }
}
