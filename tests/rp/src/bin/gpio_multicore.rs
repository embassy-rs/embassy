#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{info, unwrap};
use embassy_executor::Executor;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::{PIN_0, PIN_1};
use embassy_rp::Peri;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static mut CORE1_STACK: Stack<1024> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static CHANNEL0: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();
static CHANNEL1: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(core1_task(p.PIN_1))));
        },
    );
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_task(p.PIN_0))));
}

#[embassy_executor::task]
async fn core0_task(p: Peri<'static, PIN_0>) {
    info!("CORE0 is running");

    let mut pin = Output::new(p, Level::Low);

    CHANNEL0.send(()).await;
    CHANNEL1.receive().await;

    pin.set_high();

    CHANNEL1.receive().await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}

#[embassy_executor::task]
async fn core1_task(p: Peri<'static, PIN_1>) {
    info!("CORE1 is running");

    CHANNEL0.receive().await;

    let mut pin = Input::new(p, Pull::None);
    let wait = pin.wait_for_rising_edge();

    CHANNEL1.send(()).await;

    wait.await;

    CHANNEL1.send(()).await;
}
