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

use crate::afec;
use crate::pac::{self, Interrupt, interrupt};
use crate::rcc::{self, Peripheral as RccPeripheral};

const TIMER_TICK_HZ: u64 = 32_768;
const MIN_ALARM_TICKS: u64 = 164; // Five milliseconds, as required by the vendor timer.

const RTC_SYNC_READY: u32 = 0x0dff;
const RTC_CALENDAR_ENABLE: u32 = 1 << 28;
const RTC_CYC_WAKE_ENABLE: u32 = 1 << 25;
const RTC_CYC_ENABLE: u32 = 1 << 24;
const RTC_CYC_INTERRUPT: u32 = 1 << 4;
const RTC_CYC_STATUS: u32 = 1 << 4;

// AFEC analog register 0x02 bits 13/14 power-gate XO32K. Clearing them matches
// `rcc_enable_oscillator(RCC_OSC_XO32K)`.
const ANALOG_XO32K_POWER_DOWN: u32 = (1 << 13) | (1 << 14);

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

    const DAYS_BEFORE_MONTH: [u32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

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
        assert!(TICK_HZ == TIMER_TICK_HZ, "embassy-asr: time tick rate must be 32768 Hz");

        // Shared AFEC analog helper also gates the AFEC clock.
        afec::analog::REG_02.clear_bits(ANALOG_XO32K_POWER_DOWN);

        // Reinitialize RTC and select XO32K while its functional clock is off.
        // Clock/reset helpers match `rcc_enable_peripheral_clk` / reset sequencing,
        // including the vendor 92 µs asynchronous-domain delay after reset release.
        rcc::disable_peripheral(RccPeripheral::Rtc).expect("embassy-asr: failed to disable RTC clock");
        rcc::reset_peripheral(RccPeripheral::Rtc).expect("embassy-asr: failed to reset RTC");

        unsafe { pac::Rcc::steal() }
            .cr1()
            .modify(|_, w| w.rtc_clk_sel().xo32k());

        rcc::enable_peripheral(RccPeripheral::Rtc).expect("embassy-asr: failed to enable RTC clock");

        wait_sync();
        unsafe {
            rtc().ctrl().write_with_zero(|w| w.bits(0));
            rtc().cr1().write_with_zero(|w| w.bits(0));
            rtc().sr().write_with_zero(|w| w.bits(0x7f));

            // Epoch: Saturday 2000-01-01 00:00:00.
            rtc().calendar().write_with_zero(|w| w.bits(0));
            rtc().calendar_h().write_with_zero(|w| w.bits((6 << 11) | (1 << 6) | 1));
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

    /// Disable the cyclic counter and its interrupt. Caller must ensure RTC
    /// register writes are allowed (`wait_sync` / `sync_ready`).
    fn disable_cyc(&self) {
        rtc()
            .ctrl()
            .modify(|r, w| unsafe { w.bits(r.bits() & !RTC_CYC_ENABLE) });
        rtc()
            .cr1()
            .modify(|r, w| unsafe { w.bits(r.bits() & !RTC_CYC_INTERRUPT) });
    }

    fn stop_alarm(&self) {
        wait_sync();
        self.disable_cyc();
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

        // Already synchronized above; avoid a second wait before disable.
        self.disable_cyc();
        // Clearing status requires a sync after the control-register writes.
        wait_sync();
        unsafe {
            rtc().sr().write_with_zero(|w| w.bits(RTC_CYC_STATUS));
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

        let duration = (timestamp - now).clamp(MIN_ALARM_TICKS, u32::MAX as u64) as u32;

        wait_sync();
        unsafe {
            rtc().cyc_max().write_with_zero(|w| w.bits(duration));
            rtc().sr().write_with_zero(|w| w.bits(RTC_CYC_STATUS));
        }
        wait_sync();
        rtc()
            .ctrl()
            .modify(|r, w| unsafe { w.bits(r.bits() | RTC_CYC_WAKE_ENABLE | RTC_CYC_ENABLE) });
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Pack BCD calendar date fields the same way the driver writes `calendar_h`.
    fn pack_date(year: u32, month: u32, day: u32, weekday: u32) -> u32 {
        let y = year - 2000;
        let y_ones = y % 10;
        let y_tens = y / 10;
        let m_ones = month % 10;
        let m_tens = month / 10;
        let d_ones = day % 10;
        let d_tens = day / 10;
        d_ones | (d_tens << 4) | (m_ones << 6) | (m_tens << 10) | (weekday << 11) | (y_ones << 14) | (y_tens << 18)
    }

    fn pack_time(hour: u32, minute: u32, second: u32) -> u32 {
        let s_ones = second % 10;
        let s_tens = second / 10;
        let m_ones = minute % 10;
        let m_tens = minute / 10;
        let h_ones = hour % 10;
        let h_tens = hour / 10;
        s_ones | (s_tens << 4) | (m_ones << 7) | (m_tens << 11) | (h_ones << 14) | (h_tens << 18)
    }

    #[test]
    fn epoch_is_zero() {
        let date = pack_date(2000, 1, 1, 6);
        assert_eq!(calendar_ticks(0, date, 0), 0);
        assert_eq!(date, (6 << 11) | (1 << 6) | 1);
    }

    #[test]
    fn one_second_and_subsecond() {
        let date = pack_date(2000, 1, 1, 6);
        let time = pack_time(0, 0, 1);
        assert_eq!(calendar_ticks(time, date, 0), TIMER_TICK_HZ);
        assert_eq!(calendar_ticks(0, date, 100), 100);
    }

    #[test]
    fn leap_day_2000() {
        let jan1 = pack_date(2000, 1, 1, 6);
        let mar1 = pack_date(2000, 3, 1, 3);
        let jan1_ticks = calendar_ticks(0, jan1, 0);
        let mar1_ticks = calendar_ticks(0, mar1, 0);
        // 2000 is a leap year in the RTC's simplified %4 rule: 31 + 29 days.
        assert_eq!(mar1_ticks - jan1_ticks, 60 * 86_400 * TIMER_TICK_HZ);
    }

    #[test]
    fn non_leap_feb_2001() {
        let jan1 = pack_date(2001, 1, 1, 1);
        let mar1 = pack_date(2001, 3, 1, 4);
        assert_eq!(
            calendar_ticks(0, mar1, 0) - calendar_ticks(0, jan1, 0),
            59 * 86_400 * TIMER_TICK_HZ
        );
    }
}
