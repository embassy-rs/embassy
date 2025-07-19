//! Time driver using Periodic Interrupt Timer (PIT)
//!
//! This driver is used with the iMXRT1xxx parts.
//!
//! The PIT is run in lifetime mode. Timer 1 is chained to timer 0 to provide a free-running 64-bit timer.
//! The 64-bit timer is used to track how many ticks since boot.
//!
//! Timer 2 is used to count the remaining number of ticks until an alarm is triggered.
//!
//! # Non idealities
//!
//! This means that the PIT timer may be removed at some point.
//!
//! Unfortunately the timer has some amount of unintentional drift. While the `now()` value is monotonic (timers 0 and 1),
//! timer 2 must be stopped and started in the following cases:
//! - Every u32::MAX ticks or the number of ticks to the next alarm triggering.
//! - While a new alarm is set

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
            // three reads and calls now(), then the value in LTMR64L will be wrong when execution returns to
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
            r.set_perclk_clk_sel(nxp_pac::ccm::vals::PerclkClkSel::PERCLK_CLK_SEL_1);
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

        // Setting a new load value requires stopping the timer, which causes drift unfortunately. Try to minimize how long this takes.
        timer.tctrl().modify(|x| x.set_ten(false));
        timer.tflg().modify(|x| x.set_tif(true));

        // If the next alarm happens in more than u32::MAX ticks, then the interrupt handler will set the load value whe needed.
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
                // Must stop timer in case the deadline happens within the next u32::MAX ticks, or else the
                // deadline will be missed because the timer must expire before the new load value is applied.
                timer.tctrl().modify(|r| r.set_ten(false));

                let now = self.now();
                let timestamp = alarm.get();

                if timestamp <= now {
                    self.trigger_alarm(cs);
                } else {
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
