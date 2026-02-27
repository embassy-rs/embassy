#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i2c::target::{self, InterruptHandler};
use hal::peripherals::LPI2C3;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        LPI2C3 => InterruptHandler<LPI2C3>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("I2C target example");

    let mut config = target::Config::default();
    config.address = target::Address::Range(0x20..0x30);

    // Other possible address configurations
    // config.address = target::Address::Single(0x2a);
    // config.address = target::Address::Dual(0x2a, 0x31);
    // config.address = target::Address::Range(0x20..0x30);

    let mut target =
        target::I2c::new_async_with_dma(p.LPI2C3, p.P3_27, p.P3_28, p.DMA_CH0, p.DMA_CH1, Irqs, config).unwrap();
    let mut buf = [0u8; 256];

    loop {
        let request = target.async_listen().await.unwrap();
        defmt::info!("Received event {}", request);
        match request {
            target::Request::Read(_addr) => {
                buf.fill(0x55);
                let count = target.async_respond_to_read(&buf).await.unwrap();
                defmt::info!("T [R]: {:02x} -> {:02x}", _addr, buf[..count]);
            }
            target::Request::Write(_addr) => {
                let count = target.async_respond_to_write(&mut buf).await.unwrap();
                defmt::info!("T [W]: {:02x} <- {:02x}", _addr, buf[..count]);
            }
            _ => {}
        }
    }
}
