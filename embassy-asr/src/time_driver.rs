//! `embassy-time` driver using the ASR6601 RTC.
//!
//! This follows the vendor LoRa timer implementation: the always-on calendar is
//! the monotonic clock and the cyclic counter provides one-shot wakeups.

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Waker;

use critical_section::{CriticalSection, Mutex};
use embassy_hal_internal::interrupt::{InterruptExt, Priority};
use embassy_time_driver::{Driver, TICK_HZ};
use embassy_time_queue_utils::Queue;

use crate::pac::{self, Interrupt, interrupt};

const TIMER_TICK_HZ: u64 = 32_768;
const MIN_ALARM_TICKS: u64 = 164; // Five milliseconds, as required by the vendor timer.

const RTC_SYNC_READY: u32 = 0x0dff;
const RTC_CALENDAR_ENABLE: u32 = 1 << 28;
const RTC_CYC_WAKE_ENABLE: u32 = 1 << 25;
const RTC_CYC_ENABLE: u32 = 1 << 24;
const RTC_CYC_INTERRUPT: u32 = 1 << 4;
const RTC_CYC_STATUS: u32 = 1 << 4;

const RCC_SR_ALL_DONE: u32 = 0x3f;
const RCC_SR_RTC_AON_DONE: u32 = 1 << 1;

// AFEC analog register 0x02 controls the 32 kHz oscillators. Clearing bits
// 13 and 14 enables XO32K, matching rcc_enable_oscillator(RCC_OSC_XO32K).
const AFEC_ANALOG_02: *mut u32 = 0x4000_8008 as *mut u32;

struct AsrTimeDriver {
    initialized: AtomicBool,
    alarm: Mutex<Cell<u64>>,
    queue: Mutex<RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: AsrTimeDriver = AsrTimeDriver {
    initialized: AtomicBool::new(false),
    alarm: Mutex::new(Cell::new(u64::MAX)),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

fn rtc() -> pac::Rtc {
    unsafe { pac::Rtc::steal() }
}

fn sync_ready() -> bool {
    rtc().sr1().read().bits() & RTC_SYNC_READY == RTC_SYNC_READY
}

fn wait_sync() {
    while !sync_ready() {}
}

fn read_stable(register: impl Fn(&pac::Rtc) -> u32) -> u32 {
    loop {
        let first = register(&rtc());
        if first == register(&rtc()) {
            return first;
        }
    }
}

fn is_leap_year(year: u32) -> bool {
    year % 4 == 0
}

fn calendar_ticks(time: u32, date: u32, subsecond: u32) -> u64 {
    let second = (time & 0x0f) + ((time >> 4) & 0x07) * 10;
    let minute = ((time >> 7) & 0x0f) + ((time >> 11) & 0x07) * 10;
    let hour = ((time >> 14) & 0x0f) + ((time >> 18) & 0x03) * 10;

    let day = (date & 0x0f) + ((date >> 4) & 0x03) * 10;
    let month = ((date >> 6) & 0x0f) + ((date >> 10) & 0x01) * 10;
    let year = 2000 + ((date >> 14) & 0x0f) + ((date >> 18) & 0x0f) * 10;

    const DAYS_BEFORE_MONTH: [u32; 12] =
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

    let years = year.saturating_sub(2000);
    let mut days = years * 365 + (years + 3) / 4;
    if (1..=12).contains(&month) {
        days += DAYS_BEFORE_MONTH[(month - 1) as usize];
        if month > 2 && is_leap_year(year) {
            days += 1;
        }
    }
    days += day.saturating_sub(1);

    let seconds = days as u64 * 86_400 + hour as u64 * 3_600 + minute as u64 * 60 + second as u64;
    seconds * TIMER_TICK_HZ + subsecond as u64
}

fn read_calendar_ticks() -> u64 {
    loop {
        let subsecond = rtc().sub_second_cnt().read().bits();
        let time = read_stable(|rtc| rtc.calendar_r().read().bits());
        let date = read_stable(|rtc| rtc.calendar_r_h().read().bits());

        // Retry across a subsecond wrap so the calendar and fractional part
        // always describe the same instant.
        if subsecond != 0 && subsecond == rtc().sub_second_cnt().read().bits() {
            return calendar_ticks(time, date, subsecond);
        }
    }
}

impl AsrTimeDriver {
    fn init(&'static self) {
        assert!(
            TICK_HZ == TIMER_TICK_HZ,
            "embassy-asr: time tick rate must be 32768 Hz"
        );

        let rcc = unsafe { pac::Rcc::steal() };
        rcc.cgr0().modify(|_, w| w.afec_clk_en().set_bit());

        unsafe {
            let value = core::ptr::read_volatile(AFEC_ANALOG_02);
            core::ptr::write_volatile(AFEC_ANALOG_02, value & !((1 << 13) | (1 << 14)));
        }

        // Reinitialize RTC and select XO32K while its functional clock is off.
        // The ordering and synchronization match rcc_enable_peripheral_clk().
        rcc.cgr1().modify(|_, w| w.rtc_clk_en().clear_bit());
        while rcc.sr().read().bits() & RCC_SR_ALL_DONE != RCC_SR_ALL_DONE {}
        while rcc.sr1().read().rtc_clk_en_sync().bit_is_set() {}
        rcc.cgr2()
            .modify(|_, w| w.rtc_aon_clk_en().clear_bit());
        while rcc.sr().read().bits() & RCC_SR_RTC_AON_DONE == 0 {}

        rcc.rst0().modify(|_, w| w.rtc_rst_n().clear_bit());
        rcc.rst0().modify(|_, w| w.rtc_rst_n().set_bit());
        cortex_m::asm::delay(5_000);

        rcc.cr1().modify(|_, w| w.rtc_clk_sel().xo32k());
        rcc.cgr1().modify(|_, w| w.rtc_clk_en().set_bit());
        while rcc.sr().read().bits() & RCC_SR_ALL_DONE != RCC_SR_ALL_DONE {}
        rcc.cgr2().modify(|_, w| w.rtc_aon_clk_en().set_bit());
        while rcc.sr().read().bits() & RCC_SR_RTC_AON_DONE == 0 {}

        wait_sync();
        unsafe {
            rtc().ctrl().write_with_zero(|w| w.bits(0));
            rtc().cr1().write_with_zero(|w| w.bits(0));
            rtc().sr().write_with_zero(|w| w.bits(0x7f));

            // Epoch: Saturday 2000-01-01 00:00:00.
            rtc().calendar().write_with_zero(|w| w.bits(0));
            rtc()
                .calendar_h()
                .write_with_zero(|w| w.bits((6 << 11) | (1 << 6) | 1));
        }
        wait_sync();
        rtc()
            .ctrl()
            .modify(|r, w| unsafe { w.bits(r.bits() | RTC_CALENDAR_ENABLE) });
        wait_sync();

        self.initialized.store(true, Ordering::Release);
        Interrupt::RTC.unpend();
        Interrupt::RTC.set_priority(Priority::P2);
        unsafe { Interrupt::RTC.enable() };
    }

    fn stop_alarm(&self) {
        wait_sync();
        rtc()
            .ctrl()
            .modify(|r, w| unsafe { w.bits(r.bits() & !RTC_CYC_ENABLE) });
        rtc()
            .cr1()
            .modify(|r, w| unsafe { w.bits(r.bits() & !RTC_CYC_INTERRUPT) });
    }

    fn on_interrupt(&self) {
        // RTC registers cross an asynchronous clock domain. Avoid spinning in
        // interrupt context; an uncleared level source will retrigger once the
        // synchronized status is readable.
        if !sync_ready() {
            return;
        }

        if rtc().sr().read().bits() & RTC_CYC_STATUS == 0 {
            return;
        }

        self.stop_alarm();
        wait_sync();
        unsafe {
            rtc()
                .sr()
                .write_with_zero(|w| w.bits(RTC_CYC_STATUS));
        }

        critical_section::with(|cs| self.trigger_alarm(cs));
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        self.stop_alarm();
        self.alarm.borrow(cs).set(timestamp);

        if timestamp == u64::MAX {
            return true;
        }

        let now = self.now();
        if timestamp <= now {
            self.alarm.borrow(cs).set(u64::MAX);
            return false;
        }

        let duration = (timestamp - now)
            .clamp(MIN_ALARM_TICKS, u32::MAX as u64) as u32;

        wait_sync();
        unsafe {
            rtc().cyc_max().write_with_zero(|w| w.bits(duration));
            rtc()
                .sr()
                .write_with_zero(|w| w.bits(RTC_CYC_STATUS));
        }
        wait_sync();
        rtc().ctrl().modify(|r, w| unsafe {
            w.bits(r.bits() | RTC_CYC_WAKE_ENABLE | RTC_CYC_ENABLE)
        });
        rtc()
            .cr1()
            .modify(|r, w| unsafe { w.bits(r.bits() | RTC_CYC_INTERRUPT) });

        if timestamp <= self.now() {
            self.stop_alarm();
            self.alarm.borrow(cs).set(u64::MAX);
            return false;
        }

        true
    }
}

impl Driver for AsrTimeDriver {
    fn now(&self) -> u64 {
        if self.initialized.load(Ordering::Acquire) {
            read_calendar_ticks()
        } else {
            0
        }
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                while !self.set_alarm(cs, next) {
                    next = queue.next_expiration(self.now());
                }
            }
        });
    }
}

pub(crate) fn init() {
    DRIVER.init();
}

#[interrupt]
fn RTC() {
    DRIVER.on_interrupt();
}
