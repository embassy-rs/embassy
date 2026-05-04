#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i2c::target;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("I2C target example");

    let mut config = target::Config::default();
    config.address = target::Address::Single(42);

    // Other possible address configurations
    // config.address = target::Address::Dual(0x2a, 0x31);
    // config.address = target::Address::Range(0x20..0x30);

    let mut target = target::I2c::new_blocking(p.LPI2C3, p.P3_27, p.P3_28, config).unwrap();
    let mut buf = [0u8; 32];

    loop {
        let request = target.blocking_listen().unwrap();
        defmt::info!("Received event {}", request);
        match request {
            target::Request::Read(_addr) => {
                buf.fill(0x55);
                let count = target.blocking_respond_to_read(&buf).unwrap();
                defmt::info!("T [R]: {:02x} -> {:02x}", _addr, buf[..count]);
            }
            target::Request::Write(_addr) => {
                let count = target.blocking_respond_to_write(&mut buf).unwrap();
                defmt::info!("T [W]: {:02x} <- {:02x}", _addr, buf[..count]);
            }
            _ => {}
        }
    }
}
