#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_nrf::gpio::{Level, Output};
use embassy_nrf::peripherals::Peripherals;
use embassy_traits::spi::FullDuplex;
use example_common::*;

use cortex_m_rt::entry;
use defmt::panic;
use embassy::executor::{task, Executor};
use embassy::util::Forever;
use embedded_hal::digital::v2::*;
use futures::pin_mut;
use nrf52840_hal::clocks;

use embassy_nrf::{interrupt, pac, rtc, spim};

#[task]
async fn run() {
    info!("running!");

    let mut p = unsafe { Peripherals::steal() };

    let config = spim::Config {
        frequency: spim::Frequency::M16,
        mode: spim::MODE_0,
        orc: 0x00,
    };

    let mut irq = interrupt::take!(SPIM3);
    let spim = spim::Spim::new(p.spim3, irq, p.p0_29, p.p0_28, p.p0_30, config);
    pin_mut!(spim);

    let mut ncs = Output::new(p.p0_31, Level::High);

    // Example on how to talk to an ENC28J60 chip

    // softreset
    cortex_m::asm::delay(10);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5);
    let tx = [0xFF];
    unwrap!(spim.as_mut().read_write(&mut [], &tx).await);
    cortex_m::asm::delay(10);
    ncs.set_high().unwrap();

    cortex_m::asm::delay(100000);

    let mut rx = [0; 2];

    // read ESTAT
    cortex_m::asm::delay(5000);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5000);
    let tx = [0b000_11101, 0];
    unwrap!(spim.as_mut().read_write(&mut rx, &tx).await);
    cortex_m::asm::delay(5000);
    ncs.set_high().unwrap();
    info!("estat: {=[?]}", rx);

    // Switch to bank 3
    cortex_m::asm::delay(10);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5);
    let tx = [0b100_11111, 0b11];
    unwrap!(spim.as_mut().read_write(&mut rx, &tx).await);
    cortex_m::asm::delay(10);
    ncs.set_high().unwrap();

    // read EREVID
    cortex_m::asm::delay(10);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5);
    let tx = [0b000_10010, 0];
    unwrap!(spim.as_mut().read_write(&mut rx, &tx).await);
    cortex_m::asm::delay(10);
    ncs.set_high().unwrap();

    info!("erevid: {=[?]}", rx);
}

static RTC: Forever<rtc::RTC<pac::RTC1>> = Forever::new();
static ALARM: Forever<rtc::Alarm<pac::RTC1>> = Forever::new();
static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = unwrap!(embassy_nrf::pac::Peripherals::take());

    clocks::Clocks::new(p.CLOCK)
        .enable_ext_hfosc()
        .set_lfclk_src_external(clocks::LfOscConfiguration::NoExternalNoBypass)
        .start_lfclk();

    let rtc = RTC.put(rtc::RTC::new(p.RTC1, interrupt::take!(RTC1)));
    rtc.start();

    unsafe { embassy::time::set_clock(rtc) };

    let alarm = ALARM.put(rtc.alarm0());
    let executor = EXECUTOR.put(Executor::new());
    executor.set_alarm(alarm);

    executor.run(|spawner| {
        unwrap!(spawner.spawn(run()));
    });
}
