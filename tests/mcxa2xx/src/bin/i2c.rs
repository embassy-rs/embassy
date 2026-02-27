//! I2c controller + target

#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_mcxa::i2c::Async;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i2c::{controller, target};
use hal::peripherals::{LPI2C1, LPI2C2};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        LPI2C1 => target::InterruptHandler<LPI2C1>;
        LPI2C2 => controller::InterruptHandler<LPI2C2>;
    }
);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("I2C controller + target test");

    let mut config = target::Config::default();
    config.address = target::Address::Dual(0x13, 0x37);

    let target = target::I2c::new_async(p.LPI2C1, p.P1_1, p.P1_0, Irqs, config).unwrap();

    spawner.spawn(target_task(target).unwrap());

    let config = controller::Config::default();
    let mut i2c = controller::I2c::new_async(p.LPI2C2, p.P1_9, p.P1_8, Irqs, config).unwrap();

    let mut buf = [0];

    defmt::info!("Write read 0x13");
    i2c.async_write_read(0x13, &[13], &mut buf).await.unwrap();
    assert_eq!(buf[0], 13);
    defmt::info!("Write read 0x37");
    i2c.async_write_read(0x37, &[37], &mut buf).await.unwrap();
    assert_eq!(buf[0], 37);

    defmt::info!("Read 0x13");
    i2c.async_read(0x13, &mut buf).await.unwrap();
    assert_eq!(buf[0], 13);
    defmt::info!("Read 0x37");
    i2c.async_read(0x37, &mut buf).await.unwrap();
    assert_eq!(buf[0], 37);

    defmt::info!("Write 0x01");
    let error = i2c.async_write(0x01, &[0]).await.unwrap_err();
    assert_eq!(error, controller::IOError::AddressNack);

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}

#[embassy_executor::task]
async fn target_task(mut target: target::I2c<'static, Async>) {
    let mut addr0_value = [0];
    let mut addr1_value = [0];

    loop {
        defmt::debug!("Target listen");
        let request = target.async_listen().await.unwrap();
        defmt::info!("Received event {}", request);

        match request {
            target::Request::Read(0x13) => {
                defmt::dbg!(target.async_respond_to_read(&addr0_value).await.unwrap());
            }
            target::Request::Read(0x37) => {
                target.async_respond_to_read(&addr1_value).await.unwrap();
            }
            target::Request::Write(0x13) => {
                target.async_respond_to_write(&mut addr0_value).await.unwrap();
            }
            target::Request::Write(0x37) => {
                target.async_respond_to_write(&mut addr1_value).await.unwrap();
            }
            _ => {}
        }
    }
}
