#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{info, unwrap};
use embassy_executor::Executor;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::spinlock_mutex::SpinlockRawMutex;
use embassy_sync::channel::Channel;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static mut CORE1_STACK: Stack<1024> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static CHANNEL0: Channel<SpinlockRawMutex<0>, bool, 1> = Channel::new();
static CHANNEL1: Channel<SpinlockRawMutex<1>, bool, 1> = Channel::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(core1_task())));
        },
    );
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_task())));
}

#[embassy_executor::task]
async fn core0_task() {
    info!("CORE0 is running");
    let ping = true;
    CHANNEL0.send(ping).await;
    let pong = CHANNEL1.receive().await;
    assert_eq!(ping, pong);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

#[embassy_executor::task]
async fn core1_task() {
    info!("CORE1 is running");
    let ping = CHANNEL0.receive().await;
    CHANNEL1.send(ping).await;
}
