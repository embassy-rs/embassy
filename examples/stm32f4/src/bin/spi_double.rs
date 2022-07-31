#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::fmt::Write;
use core::str::from_utf8;

use defmt::{assert_eq, info, unwrap};
use embassy_stm32::executor::Spawner;
use embassy_stm32::spi::{Config, MasterSlave, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Peripherals;
use heapless::String;
use {defmt_rtt as _, panic_probe as _};

const FREQUENCY: u32 = 84_000_000;

fn config() -> embassy_stm32::Config {
    use embassy_stm32::Config;

    let mut config = Config::default();
    config.rcc.sys_ck = Some(Hertz(FREQUENCY));
    config
}

#[embassy_executor::main(config = "config()")]
async fn main(_spawner: Spawner, mut p: Peripherals) {
    info!("Emulation of double spi on same board, make sure sck of master and slave are connected!");

    // Slowest possible frequency is 328_125
    let spi_frequency = Hertz(FREQUENCY / 256);

    let mut spi = Spi::new(
        &mut p.SPI2,
        &mut p.PB13,
        &mut p.PB15,
        &mut p.PB14,
        &mut p.DMA1_CH4,
        &mut p.DMA1_CH3,
        spi_frequency,
        Default::default(),
    );

    let mut slave_config = Config::default();
    slave_config.master_slave = MasterSlave::Slave;
    // slave_config.mode = MODE_3;

    let mut spi_slave = Spi::new(
        &mut p.SPI1,
        &mut p.PB3,
        &mut p.PB5,
        &mut p.PB4,
        &mut p.DMA2_CH3,
        &mut p.DMA2_CH2,
        spi_frequency,
        slave_config,
    );

    for n in 0u32.. {
        let mut write_slave: String<128> = String::new();
        let mut write_master: String<128> = String::new();
        let mut read_master = [0; 128];

        let mut read_slave = [0; 128];
        core::write!(&mut write_master, "Hello from Master {}!\r\n", n).unwrap();
        core::write!(&mut write_slave, "Hello from Slave: {}!\r\n", n).unwrap();

        assert_eq!(write_slave.len(), write_master.len(), "make sure buffer sizes are same");

        let result = futures::future::join(
            // spi_slave should init first
            // or it would be not ready for first clock
            spi_slave.transfer(&mut read_slave[0..write_slave.len()], write_slave.as_bytes()),
            spi.transfer(&mut read_master[0..write_master.len()], write_master.as_bytes()),
        )
        .await;
        unwrap!(result.0);
        unwrap!(result.1);
        info!("read via spi master: {}", from_utf8(&read_master).unwrap());
        info!("read via spi slave: {}", from_utf8(&read_slave).unwrap());
    }
}
