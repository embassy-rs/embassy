#![no_std]
#![no_main]

use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::interrupt;
use embassy_nrf::rtc::Rtc;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use portable_atomic::AtomicU64;
use {defmt_rtt as _, panic_probe as _};

// 64 bit counter which will never overflow.
static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);
static RTC: Mutex<CriticalSectionRawMutex, RefCell<Option<Rtc<'static>>>> = Mutex::new(RefCell::new(None));

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::println!("nRF54L15 RTC example");
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P2_09, Level::High, OutputDrive::Standard);
    // Counter resolution is 125 ms.
    let mut rtc = Rtc::new(p.RTC10, (1 << 12) - 1).unwrap();
    rtc.enable_interrupt(embassy_nrf::rtc::Interrupt::Tick, true);
    rtc.enable_event(embassy_nrf::rtc::Interrupt::Tick);
    rtc.enable();
    RTC.lock(|r| {
        let mut rtc_borrow = r.borrow_mut();
        *rtc_borrow = Some(rtc);
    });

    let mut last_counter_val = 0;
    loop {
        let current = TICK_COUNTER.load(core::sync::atomic::Ordering::Relaxed);
        if current != last_counter_val {
            led.toggle();
            last_counter_val = current;
        }
    }
}

#[interrupt]
fn RTC10() {
    // For 64-bit, we do not need to worry about overflowing, at least not for realistic program
    // lifetimes.
    TICK_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    RTC.lock(|r| {
        let mut rtc_borrow = r.borrow_mut();
        rtc_borrow
            .as_mut()
            .unwrap()
            .reset_event(embassy_nrf::rtc::Interrupt::Tick);
    });
}
