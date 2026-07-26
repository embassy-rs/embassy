//! i2c-loopback-async — runs the I2C target and async controller on the
//! same MCXA577. Wire P0_20↔P3_20 (SDA), P0_21↔P3_21 (SCL), with pull-ups.

#![no_std]
#![no_main]

#[path = "../i2c_loopback.rs"]
mod i2c_loopback;

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i2c::controller::{self, I2c, InterruptHandler as ControllerIH, Speed};
use hal::i2c::target::InterruptHandler as TargetIH;
use hal::peripherals::{LPI2C0, LPI2C3};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        LPI2C0 => ControllerIH<LPI2C0>;
        LPI2C3 => TargetIH<LPI2C3>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);
    defmt::info!("i2c-loopback-async start");

    let target = i2c_loopback::target_task(p.LPI2C3, p.P3_21, p.P3_20, Irqs);

    let mut ccfg = controller::Config::default();
    ccfg.speed = Speed::Standard;
    let mut ctrl = I2c::new_async(p.LPI2C0, p.P0_21, p.P0_20, Irqs, ccfg).unwrap();

    let suite = async {
        i2c_loopback::harness::run(&mut ctrl).await;
        defmt::info!("== done — bkpt ==");
        cortex_m::asm::bkpt();
        core::future::pending::<()>().await
    };

    match select(target, suite).await {
        Either::First(_) => defmt::info!("target task exited"),
        Either::Second(_) => defmt::info!("suite exited"),
    }
}
