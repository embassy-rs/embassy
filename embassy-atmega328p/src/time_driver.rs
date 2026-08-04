//! Embassy time driver backed by the 16-bit Timer/Counter1.

use core::cell::{Cell, RefCell};
use core::ptr::{read_volatile, write_volatile};
use core::task::Waker;

use avr_device::interrupt::{CriticalSection, Mutex};
use embassy_time_driver::Driver;
use embassy_time_queue_utils::queue_generic::ConstGenericQueue;

#[cfg(all(feature = "time-driver-16mhz", feature = "time-driver-8mhz"))]
compile_error!("enable only one ATmega328P time-driver clock feature");

// Timer1 registers in the ATmega328P data address space.
const TIFR1: *mut u8 = 0x36 as *mut u8;
const TCCR1A: *mut u8 = 0x80 as *mut u8;
const TCCR1B: *mut u8 = 0x81 as *mut u8;
const TCNT1L: *mut u8 = 0x84 as *mut u8;
const TCNT1H: *mut u8 = 0x85 as *mut u8;
const OCR1AL: *mut u8 = 0x88 as *mut u8;
const OCR1AH: *mut u8 = 0x89 as *mut u8;
const TIMSK1: *mut u8 = 0x6f as *mut u8;

const TOV1: u8 = 1 << 0;
const OCF1A: u8 = 1 << 1;
const TOIE1: u8 = 1 << 0;
const OCIE1A: u8 = 1 << 1;
const CS11: u8 = 1 << 1;

const COUNTER_RANGE: u64 = 1 << 16;
const QUEUE_SIZE: usize = 8;

struct Timer1Driver {
    initialized: Mutex<Cell<bool>>,
    overflows: Mutex<Cell<u64>>,
    queue: Mutex<RefCell<ConstGenericQueue<QUEUE_SIZE>>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: Timer1Driver = Timer1Driver {
    initialized: Mutex::new(Cell::new(false)),
    overflows: Mutex::new(Cell::new(0)),
    queue: Mutex::new(RefCell::new(ConstGenericQueue::new())),
});

impl Driver for Timer1Driver {
    fn now(&self) -> u64 {
        avr_device::interrupt::free(|cs| self.now_locked(cs))
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        avr_device::interrupt::free(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now_locked(cs));
                while !self.set_alarm(cs, next) {
                    next = queue.next_expiration(self.now_locked(cs));
                }
            }
        });
    }
}

impl Timer1Driver {
    fn now_locked(&self, cs: CriticalSection<'_>) -> u64 {
        if !self.initialized.borrow(cs).get() {
            return 0;
        }

        let high = self.overflows.borrow(cs).get();
        let low = read_counter();

        // If the counter wrapped while interrupts were disabled, the overflow
        // ISR has not updated `overflows` yet. Account for the pending flag.
        if unsafe { read_volatile(TIFR1) } & TOV1 != 0 {
            let low = read_counter();
            (high + COUNTER_RANGE) | u64::from(low)
        } else {
            high | u64::from(low)
        }
    }

    fn set_alarm(&self, cs: CriticalSection<'_>, timestamp: u64) -> bool {
        let now = self.now_locked(cs);
        if timestamp <= now {
            disable_compare_interrupt();
            return false;
        }

        // A 16-bit compare value is only unambiguous within the current
        // counter cycle. For farther deadlines, the overflow ISR retries.
        if timestamp - now >= COUNTER_RANGE {
            disable_compare_interrupt();
            return true;
        }

        write_compare(timestamp as u16);
        unsafe {
            // Clear a stale compare flag before enabling its interrupt.
            write_volatile(TIFR1, OCF1A);
            modify(TIMSK1, OCIE1A, true);
        }

        // The counter may have passed the compare value during programming.
        if timestamp <= self.now_locked(cs) {
            disable_compare_interrupt();
            false
        } else {
            true
        }
    }

    #[cfg(target_arch = "avr")]
    fn trigger_alarm(&self, cs: CriticalSection<'_>) {
        let mut queue = self.queue.borrow(cs).borrow_mut();
        let mut next = queue.next_expiration(self.now_locked(cs));
        while !self.set_alarm(cs, next) {
            next = queue.next_expiration(self.now_locked(cs));
        }
    }

    #[cfg(target_arch = "avr")]
    fn on_overflow(&self) {
        avr_device::interrupt::free(|cs| {
            let high = self.overflows.borrow(cs);
            high.set(high.get() + COUNTER_RANGE);
            self.trigger_alarm(cs);
        });
    }

    #[cfg(target_arch = "avr")]
    fn on_compare(&self) {
        avr_device::interrupt::free(|cs| {
            disable_compare_interrupt();
            self.trigger_alarm(cs);
        });
    }
}

pub(crate) fn init() {
    avr_device::interrupt::free(|cs| unsafe {
        // Normal mode, stopped while configuring.
        write_volatile(TCCR1A, 0);
        write_volatile(TCCR1B, 0);
        write_counter(0);
        write_compare(0);
        write_volatile(TIFR1, TOV1 | OCF1A);
        write_volatile(TIMSK1, TOIE1);

        DRIVER.overflows.borrow(cs).set(0);
        DRIVER.initialized.borrow(cs).set(true);

        // Start Timer1 with a /8 prescaler. This produces a 2 MHz timebase at
        // 16 MHz CPU clock, or a 1 MHz timebase at 8 MHz CPU clock.
        write_volatile(TCCR1B, CS11);
    });
}

#[cfg(target_arch = "avr")]
#[avr_device::interrupt(atmega328p)]
fn TIMER1_OVF() {
    DRIVER.on_overflow();
}

#[cfg(target_arch = "avr")]
#[avr_device::interrupt(atmega328p)]
fn TIMER1_COMPA() {
    DRIVER.on_compare();
}

fn read_counter() -> u16 {
    unsafe {
        // Reading the low byte latches the high byte on classic AVR timers.
        let low = read_volatile(TCNT1L);
        let high = read_volatile(TCNT1H);
        u16::from_le_bytes([low, high])
    }
}

unsafe fn write_counter(value: u16) {
    let [low, high] = value.to_le_bytes();
    unsafe {
        // For 16-bit timer writes, the high byte must be written first.
        write_volatile(TCNT1H, high);
        write_volatile(TCNT1L, low);
    }
}

fn write_compare(value: u16) {
    let [low, high] = value.to_le_bytes();
    unsafe {
        write_volatile(OCR1AH, high);
        write_volatile(OCR1AL, low);
    }
}

fn disable_compare_interrupt() {
    unsafe { modify(TIMSK1, OCIE1A, false) }
}

unsafe fn modify(register: *mut u8, mask: u8, set: bool) {
    let current = unsafe { read_volatile(register) };
    let value = if set { current | mask } else { current & !mask };
    unsafe { write_volatile(register, value) };
}
