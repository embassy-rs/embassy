//! Time Driver
//!
//! Uses the CLINT MTIMER peripheral to manage time.
//! This is intended to work on both a single-hart and dual-hart configuration.
//!
//! In the case of dual-hart, hart 0 will always be the owner of time-keeping,
//! and is solely responsible for handling timer interrupts and waking tasks
//! as appropriate on both harts' executors.
use core::cell::RefCell;

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

embassy_time_driver::time_driver_impl!(static DRIVER: MtimerDriver = MtimerDriver {
    queue: Mutex::new(RefCell::new(Queue::new()))
});

#[riscv_rt::core_interrupt(crate::pac::interrupt::CoreInterrupt::MachineTimer)]
fn machine_timer_handler() {
    DRIVER.on_interrupt()
}

struct MtimerDriver {
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

impl MtimerDriver {
    fn on_interrupt(&self) {
        clint().mtimer().mtimecmp0().write(u64::MAX);

        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();

            let mut next = queue.next_expiration(self.now());
            while !self.set_alarm(next) {
                next = queue.next_expiration(self.now());
            }
        });
    }

    fn set_alarm(&self, ts: u64) -> bool {
        // Timestamp is in the past, so can't set the alarm
        if ts <= self.now() {
            false
        // Otherwise try to set the alarm but double check the ts isn't in the past again
        } else {
            clint().mtimer().mtimecmp0().write(ts);
            ts > self.now()
        }
    }
}

pub(crate) fn init() {
    // CLINT is used for timer interrupts which is necessary for time keeping
    if !crate::sysinfo::SysInfo::soc_config().has_clint() {
        panic!("CLINT must be supported for time-driver to work");
    }

    // Ensure only hart 0 initializes time-driver
    assert_eq!(riscv::register::mhartid::read(), 0);

    // Set the compare value far, far in the future so interrupt won't trigger yet
    clint().mtimer().mtimecmp0().write(u64::MAX);

    // SAFETY: It is okay to enable mtimer interrupts here
    unsafe { clint().mtimer().enable() };
}

impl Driver for MtimerDriver {
    fn now(&self) -> u64 {
        clint().mtimer().mtime().read()
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                while !self.set_alarm(next) {
                    next = queue.next_expiration(self.now());
                }
            }
        })
    }
}

fn clint() -> crate::pac::Clint {
    // SAFETY: We are the only ones who use mtimecmp0 and mtimer, so we can manage it safely
    unsafe { crate::pac::Clint::steal() }
}
