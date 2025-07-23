//! Time driver using Periodic Interrupt Timer (PIT)
//!
//! This driver is used with the iMXRT1xxx parts.
//!
//! The PIT is run in lifetime mode. Timer 1 is chained to timer 0 to provide a free-running 64-bit timer.
//! The 64-bit timer is used to track how many ticks since boot.
//!
//! Timer 2 counts how many ticks there are within the current u32::MAX tick period. Timer 2 is restarted when
//! a new alarm is set (or every u32::MAX ticks). One caveat is that an alarm could be a few ticks late due to
//! restart. However the Cortex-M7 cores run at 500 MHz easily and the PIT will generally run at 1 MHz or lower.
//! Along with the fact that scheduling an alarm takes a critical section worst case an alarm may be a few
//! microseconds late.
//!
//! All PIT timers are clocked in lockstep, so the late start will not cause the now() count to drift.

use core::cell::{Cell, RefCell};
use core::task::Waker;

use critical_section::{CriticalSection, Mutex};
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_time_driver::Driver as _;
use embassy_time_queue_utils::Queue;

use crate::pac::{self, interrupt};

struct Driver {
    alarm: Mutex<Cell<u64>>,
    queue: Mutex<RefCell<Queue>>,
}

impl embassy_time_driver::Driver for Driver {
    fn now(&self) -> u64 {
        loop {
            // Even though reading LTMR64H will latch LTMR64L if another thread preempts between any of the
            // three reads and calls now() then the value in LTMR64L will be wrong when execution returns to
            // thread which was preempted.
            let hi = pac::PIT.ltmr64h().read().lth();
            let lo = pac::PIT.ltmr64l().read().ltl();
            let hi2 = pac::PIT.ltmr64h().read().lth();

            if hi == hi2 {
                // PIT timers always count down.
                return u64::MAX - ((hi as u64) << 32 | (lo as u64));
            }
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
        })
    }
}

impl Driver {
    fn init(&'static self) {
        // Disable PIT clock during mux configuration.
        pac::CCM.ccgr1().modify(|r| r.set_cg6(0b00));

        // TODO: This forces the PIT to be driven by the oscillator. However that isn't the only option as you
        // could divide the clock root by up to 64.
        pac::CCM.cscmr1().modify(|r| {
            // 1 MHz
            r.set_perclk_podf(pac::ccm::vals::PerclkPodf::DIVIDE_24);
            r.set_perclk_clk_sel(pac::ccm::vals::PerclkClkSel::PERCLK_CLK_SEL_1);
        });

        pac::CCM.ccgr1().modify(|r| r.set_cg6(0b11));

        // Disable clock during init.
        //
        // It is important that the PIT clock is prepared to not exceed limit (50 MHz on RT1011), or else
        // you will need to recover the device with boot mode switches when using any PIT registers.
        pac::PIT.mcr().modify(|w| {
            w.set_mdis(true);
        });

        pac::PIT.timer(0).ldval().write_value(u32::MAX);
        pac::PIT.timer(1).ldval().write_value(u32::MAX);
        pac::PIT.timer(2).ldval().write_value(0);
        pac::PIT.timer(3).ldval().write_value(0);

        pac::PIT.timer(1).tctrl().write(|w| {
            // In lifetime mode, timer 1 is chained to timer 0 to form a 64-bit timer.
            w.set_chn(true);
            w.set_ten(true);
            w.set_tie(false);
        });

        pac::PIT.timer(0).tctrl().write(|w| {
            w.set_chn(false);
            w.set_ten(true);
            w.set_tie(false);
        });

        pac::PIT.timer(2).tctrl().write(|w| {
            w.set_tie(true);
        });

        unsafe { interrupt::PIT.enable() };

        pac::PIT.mcr().write(|w| {
            w.set_mdis(false);
        });
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let alarm = self.alarm.borrow(cs);
        alarm.set(timestamp);

        let timer = pac::PIT.timer(2);
        let now = self.now();

        if timestamp <= now {
            alarm.set(u64::MAX);

            return false;
        }

        timer.tctrl().modify(|x| x.set_ten(false));
        timer.tflg().modify(|x| x.set_tif(true));

        // If the next alarm happens in more than u32::MAX cycles then the alarm will be restarted later.
        timer.ldval().write_value((timestamp - now) as u32);
        timer.tctrl().modify(|x| x.set_ten(true));

        true
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow_ref_mut(cs).next_expiration(self.now());

        while !self.set_alarm(cs, next) {
            next = self.queue.borrow_ref_mut(cs).next_expiration(self.now());
        }
    }

    fn on_interrupt(&self) {
        critical_section::with(|cs| {
            let timer = pac::PIT.timer(2);
            let alarm = self.alarm.borrow(cs);
            let interrupted = timer.tflg().read().tif();
            timer.tflg().write(|r| r.set_tif(true));

            if interrupted {
                // A new load value will not apply until the next timer expiration.
                //
                // The expiry may be up to u32::MAX cycles away, so the timer must be restarted.
                timer.tctrl().modify(|r| r.set_ten(false));

                let now = self.now();
                let timestamp = alarm.get();

                if timestamp <= now {
                    self.trigger_alarm(cs);
                } else {
                    // The alarm is not ready. Wait for u32::MAX cycles and check again or set the next alarm.
                    timer.ldval().write_value((timestamp - now) as u32);
                    timer.tctrl().modify(|r| r.set_ten(true));
                }
            }
        });
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: Driver = Driver {
    alarm: Mutex::new(Cell::new(0)),
    queue: Mutex::new(RefCell::new(Queue::new()))
});

pub(crate) fn init() {
    DRIVER.init();
}

#[cfg(feature = "rt")]
#[interrupt]
fn PIT() {
    DRIVER.on_interrupt();
}
