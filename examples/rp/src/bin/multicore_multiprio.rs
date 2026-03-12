//! Example showing both cores running thread-mode and interrupt executors
//! This example needs to be run
//! with:
//!
//! ```sh
//! cargo run --release --no-default-features --features=custom-executor --bin multicore_multiprio
//! ```
//! Output will be logged via a USB serial port device
//! Note this example will not work with the regular executors in embassy_executor
#![no_std]
#![no_main]

use embassy_executor::{Spawner, main};
use embassy_rp::executor::{Executor, InterruptExecutor};
use embassy_rp::interrupt::{InterruptExt, Priority};
use embassy_rp::multicore::{CoreId, Stack, current_core, spawn_core1};
use embassy_rp::peripherals::USB;
use embassy_rp::{bind_interrupts, interrupt, usb};
use embassy_time::{Duration, Ticker, Timer};
use log::*;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static mut CORE1_STACK: Stack<4096> = Stack::new();
static CORE1_LOW: StaticCell<Executor> = StaticCell::new();
static CORE0_HIGH: InterruptExecutor = InterruptExecutor::new();
static CORE1_HIGH: InterruptExecutor = InterruptExecutor::new();

#[interrupt]
unsafe fn SWI_IRQ_0() {
    // Only necessay because we're using SWI_IRQ_0 on both cores. We could
    // Chose to use SWI_IRQ_1 on one core instead and have unique IRQ handlers
    match current_core() {
        CoreId::Core0 => unsafe { CORE0_HIGH.on_interrupt() },
        CoreId::Core1 => unsafe { CORE1_HIGH.on_interrupt() },
    }
}

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

#[main(executor = "Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let usb_driver = usb::Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(usb_driver).unwrap());
    spawner.spawn(low_task().unwrap());

    Timer::after_millis(200).await;

    interrupt::SWI_IRQ_0.set_priority(Priority::P2);
    let spawner_high = CORE0_HIGH.start(interrupt::SWI_IRQ_0);
    spawner_high.spawn(high_task().unwrap());

    Timer::after_millis(10).await;

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            interrupt::SWI_IRQ_0.set_priority(Priority::P2);
            let spawner_high = CORE1_HIGH.start(interrupt::SWI_IRQ_0);
            spawner_high.spawn(high_task().unwrap());

            let executor_low = CORE1_LOW.init(Executor::new());
            executor_low.run(|spawner| spawner.spawn(low_task().unwrap()));
        },
    );
}

#[embassy_executor::task(pool_size = 2)]
async fn high_task() {
    let cpuid = current_core();
    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        info!("[high] Core {:?} is receiving interrupts", cpuid);
        ticker.next().await;
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn low_task() {
    let cpuid = current_core();
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        info!("[low] Core {:?} is running", cpuid);
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn logger_task(driver: usb::Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
