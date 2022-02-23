use core::cell::Cell;
use core::sync::atomic::{compiler_fence, AtomicU32, AtomicU8, Ordering};
use core::{mem, ptr};
use critical_section::CriticalSection;
use embassy::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy::blocking_mutex::CriticalSectionMutex as Mutex;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::time::driver::{AlarmHandle, Driver};

use crate::interrupt;
use crate::pac;

// RTC timekeeping works with something we call "periods", which are time intervals
// of 2^23 ticks. The RTC counter value is 24 bits, so one "overflow cycle" is 2 periods.
//
// A `period` count is maintained in parallel to the RTC hardware `counter`, like this:
// - `period` and `counter` start at 0
// - `period` is incremented on overflow (at counter value 0)
// - `period` is incremented "midway" between overflows (at counter value 0x800000)
//
// Therefore, when `period` is even, counter is in 0..0x7fffff. When odd, counter is in 0x800000..0xFFFFFF
// This allows for now() to return the correct value even if it races an overflow.
//
// To get `now()`, `period` is read first, then `counter` is read. If the counter value matches
// the expected range for the `period` parity, we're done. If it doesn't, this means that
// a new period start has raced us between reading `period` and `counter`, so we assume the `counter` value
// corresponds to the next period.
//
// `period` is a 32bit integer, so It overflows on 2^32 * 2^23 / 32768 seconds of uptime, which is 34865 years.

fn calc_now(period: u32, counter: u32) -> u64 {
    ((period as u64) << 23) + ((counter ^ ((period & 1) << 23)) as u64)
}

fn compare_n(n: usize) -> u32 {
    1 << (n + 16)
}

#[cfg(tests)]
mod test {
    use super::*;

    #[test]
    fn test_calc_now() {
        assert_eq!(calc_now(0, 0x000000), 0x0_000000);
        assert_eq!(calc_now(0, 0x000001), 0x0_000001);
        assert_eq!(calc_now(0, 0x7FFFFF), 0x0_7FFFFF);
        assert_eq!(calc_now(1, 0x7FFFFF), 0x1_7FFFFF);
        assert_eq!(calc_now(0, 0x800000), 0x0_800000);
        assert_eq!(calc_now(1, 0x800000), 0x0_800000);
        assert_eq!(calc_now(1, 0x800001), 0x0_800001);
        assert_eq!(calc_now(1, 0xFFFFFF), 0x0_FFFFFF);
        assert_eq!(calc_now(2, 0xFFFFFF), 0x1_FFFFFF);
        assert_eq!(calc_now(1, 0x000000), 0x1_000000);
        assert_eq!(calc_now(2, 0x000000), 0x1_000000);
    }
}

struct AlarmState {
    timestamp: Cell<u64>,

    // This is really a Option<(fn(*mut ()), *mut ())>
    // but fn pointers aren't allowed in const yet
    callback: Cell<*const ()>,
    ctx: Cell<*mut ()>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
            callback: Cell::new(ptr::null()),
            ctx: Cell::new(ptr::null_mut()),
        }
    }
}

const ALARM_COUNT: usize = 3;

cfg_if::cfg_if! {
    if #[cfg(feature = "time-driver-rtc1")] {
        pub mod rtc;
        pub use rtc as driver_impl;
    } else if #[cfg(feature = "time-driver-timer1")] {
        pub mod timer;
        pub use timer as driver_impl;
    }
}
