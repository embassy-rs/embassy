#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use defmt::panic;
use embassy::executor::{task, Executor};
use embassy::util::Forever;
use embedded_hal::digital::v2::*;
use futures::pin_mut;
use nrf52840_hal::clocks;
use nrf52840_hal::gpio;

use embassy_nrf::{interrupt, pac, rtc, spim};

#[task]
async fn run() {
    info!("running!");

    let p = unsafe { embassy_nrf::pac::Peripherals::steal() };
    let p0 = gpio::p0::Parts::new(p.P0);

    let pins = spim::Pins {
        sck: p0.p0_29.into_push_pull_output(gpio::Level::Low).degrade(),
        miso: Some(p0.p0_28.into_floating_input().degrade()),
        mosi: Some(p0.p0_30.into_push_pull_output(gpio::Level::Low).degrade()),
    };
    let config = spim::Config {
        pins,
        frequency: spim::Frequency::M16,
        mode: spim::MODE_0,
        orc: 0x00,
    };

    let mut ncs = p0.p0_31.into_push_pull_output(gpio::Level::High);
    let spim = spim::Spim::new(p.SPIM3, interrupt::take!(SPIM3), config);
    pin_mut!(spim);

    // Example on how to talk to an ENC28J60 chip

    // softreset
    cortex_m::asm::delay(10);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5);
    let tx = [0xFF];
    unwrap!(spim.as_mut().send_receive(&tx, &mut []).await);
    cortex_m::asm::delay(10);
    ncs.set_high().unwrap();

    cortex_m::asm::delay(100000);

    let mut rx = [0; 2];

    // read ESTAT
    cortex_m::asm::delay(5000);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5000);
    let tx = [0b000_11101, 0];
    unwrap!(spim.as_mut().send_receive(&tx, &mut rx).await);
    cortex_m::asm::delay(5000);
    ncs.set_high().unwrap();
    info!("estat: {=[?]}", rx);

    // Switch to bank 3
    cortex_m::asm::delay(10);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5);
    let tx = [0b100_11111, 0b11];
    unwrap!(spim.as_mut().send_receive(&tx, &mut rx).await);
    cortex_m::asm::delay(10);
    ncs.set_high().unwrap();

    // read EREVID
    cortex_m::asm::delay(10);
    ncs.set_low().unwrap();
    cortex_m::asm::delay(5);
    let tx = [0b000_10010, 0];
    unwrap!(spim.as_mut().send_receive(&tx, &mut rx).await);
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
