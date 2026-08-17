//! High-resolution and low-power time drivers for Embassy.
#![allow(clippy::new_without_default)]

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Waker;

use apollo3_pac as pac;
use cortex_m::peripheral::NVIC;
use critical_section::{CriticalSection, Mutex};
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;
use pac::{Interrupt, interrupt};

#[cfg(all(feature = "_stimer", feature = "time-driver-ctimer0"))]
compile_error!(
    "embassy-ambiq: enable at most one time-driver backend — \
     `time-driver-stimer-lfrc` / `time-driver-stimer-xtal` OR `time-driver-ctimer0`."
);

struct AlarmState {
    /// Timestamp at which to fire the alarm. `u64::MAX` if no alarm is scheduled.
    timestamp: Cell<u64>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
        }
    }
}

pub(crate) struct Apollo3TimeDriver {
    /// 32-bit overflow counter. Appended to the 32-bit hardware counter to form a 64-bit tick count.
    overflows: AtomicU32,
    /// Next hardware compare deadline.
    alarm: Mutex<AlarmState>,
    /// Pending timer wakers, owned by the executor's time queue.
    queue: Mutex<RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: Apollo3TimeDriver = Apollo3TimeDriver {
    overflows: AtomicU32::new(0),
    alarm: Mutex::new(AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

/// STIMER BACKEND

#[cfg(feature = "_stimer")]
impl Apollo3TimeDriver {
    pub(crate) fn init() {
        let ctimer = pac::CTIMER;

        // Clear and freeze the counter while (re)configuring.
        ctimer.stcfg().write(|w| {
            w.set_clear(pac::ctimer::vals::Clear::Clear);
            w.set_freeze(pac::ctimer::vals::Freeze::Freeze);
        });

        // Clock source (and thus tick rate) is fixed by the selected feature.
        ctimer.stcfg().write(|w| {
            #[cfg(feature = "_xtal-div1")]
            w.set_clksel(pac::ctimer::vals::Clksel::XtalDiv1); // 32768 Hz
            #[cfg(feature = "_xtal-div2")]
            w.set_clksel(pac::ctimer::vals::Clksel::XtalDiv2); // 16384 Hz
            #[cfg(feature = "_xtal-div32")]
            w.set_clksel(pac::ctimer::vals::Clksel::XtalDiv32); // 1024 Hz
            #[cfg(feature = "_lfrc")]
            w.set_clksel(pac::ctimer::vals::Clksel::LfrcDiv1); // ~1024 Hz (uncalibrated)

            w.set_compare_a_en(true);
            w.set_clear(pac::ctimer::vals::Clear::Run);
            w.set_freeze(pac::ctimer::vals::Freeze::Thaw);
        });

        // Enable the Compare-A interrupt once, leaving it enabled. The ISR only clears the
        // status and re-arms by writing a new absolute compare. Overflow feeds the 64-bit now().
        ctimer.stmintclr().write(|w| w.set_comparea(true));
        ctimer.stminten().modify(|w| {
            w.set_comparea(true);
            w.set_overflow(true);
        });

        // Enable NVIC interrupts
        unsafe {
            NVIC::unmask(Interrupt::STIMER);
            NVIC::unmask(Interrupt::STIMER_CMPR0);
        }
    }

    fn on_overflow(&self) {
        self.overflows.fetch_add(1, Ordering::Relaxed);
        pac::CTIMER.stmintclr().write(|w| w.set_overflow(true));
    }

    fn on_compare0(&self) {
        // Clear the status only; Compare-A stays enabled (see init).
        pac::CTIMER.stmintclr().write(|w| w.set_comparea(true));

        critical_section::with(|cs| {
            self.alarm.borrow(cs).timestamp.set(u64::MAX);
            self.trigger_alarm(cs);
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let alarm = self.alarm.borrow(cs);
        alarm.timestamp.set(timestamp);

        let now = self.now();
        if timestamp <= now {
            alarm.timestamp.set(u64::MAX);
            return false;
        }

        let diff = timestamp - now;
        let diff_u32 = if diff > 0x7FFF_FFFF { 0x7FFF_FFFF } else { diff as u32 };

        // SCMPR0 takes a DELTA ("offset from NOW"): the hardware adds it to the COUNTER in the
        // STIMER clock domain. Write the raw diff
        pac::CTIMER.scmpr0().write_value(diff_u32);
        pac::CTIMER.stmintclr().write(|w| w.set_comparea(true));

        true
    }
}

#[cfg(feature = "_stimer")]
impl Driver for Apollo3TimeDriver {
    fn now(&self) -> u64 {
        critical_section::with(|_| {
            let high = self.overflows.load(Ordering::Relaxed);
            let low = pac::CTIMER.sttmr().read();

            if pac::CTIMER.stmintstat().read().overflow() {
                let low = pac::CTIMER.sttmr().read();
                (((high + 1) as u64) << 32) | (low as u64)
            } else {
                ((high as u64) << 32) | (low as u64)
            }
        })
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

#[cfg(feature = "_stimer")]
#[interrupt]
fn STIMER() {
    DRIVER.on_overflow();
}

#[cfg(feature = "_stimer")]
#[interrupt]
fn STIMER_CMPR0() {
    DRIVER.on_compare0();
}

/// CTIMER0 BACKEND (High Speed)

#[cfg(feature = "time-driver-ctimer0")]
impl Apollo3TimeDriver {
    pub(crate) fn init() {
        let ctimer = pac::CTIMER;

        const _ASSERT_TICK: () = assert!(
            embassy_time_driver::TICK_HZ == 1_000_000,
            "embassy-time tick-hz must be 1_000_000 for ctimer0 (HFRC/16 scaled to 1MHz)"
        );

        // Stop the timer
        ctimer.ctrl0().write(|w| {
            w.set_tmra0en(false);
            w.set_tmrb0en(false);
            w.set_tmra0clr(pac::ctimer::vals::Clear::Clear);
            w.set_tmrb0clr(pac::ctimer::vals::Clear::Clear);
        });

        // Configure CTIMER0 as a single 32-bit timer, clocked by HFRC/16 (3 MHz)
        ctimer.ctrl0().modify(|w| {
            w.set_ctlink0(pac::ctimer::vals::Ctlink::_32bitTimer);
            w.set_tmra0clk(pac::ctimer::vals::Tmra0clk::HfrcDiv16); // 3 MHz
            w.set_tmra0fn(pac::ctimer::vals::Tmrfn::Continuous);
            w.set_tmra0ie0(true); // REQUIRED to generate the compare 0 interrupt
        });

        // Initialize compare registers far into the future to prevent an immediate interrupt on boot
        ctimer.cmpra0().write_value(pac::ctimer::regs::Cmpra0(0xFFFF));
        ctimer.cmprb0().write_value(pac::ctimer::regs::Cmprb0(0xFFFF));

        // Enable interrupt on Compare A0
        ctimer.inten().modify(|w| w.set_ctmra0c0int(true));

        // Start the timer
        ctimer.ctrl0().modify(|w| {
            w.set_tmra0en(true);
            w.set_tmrb0en(true); // 32-bit mode requires B side to run too
            w.set_tmra0clr(pac::ctimer::vals::Clear::Run);
            w.set_tmrb0clr(pac::ctimer::vals::Clear::Run);
        });

        // Globally enable CTIMER0 (A and B sides)
        ctimer.globen().modify(|w| {
            w.set_ena0(pac::ctimer::vals::En::Lco);
            w.set_enb0(pac::ctimer::vals::En::Lco);
        });

        // Enable NVIC interrupts
        unsafe {
            NVIC::unmask(Interrupt::CTIMER);
        }
    }

    fn on_interrupt(&self) {
        let ctimer = pac::CTIMER;
        let stat = ctimer.intstat().read();

        if stat.ctmra0c0int() || stat.ctmrb0c0int() {
            // Clear ALL CTIMER interrupts just in case
            ctimer.intclr().write(|w| {
                w.set_ctmra0c0int(true);
                w.set_ctmrb0c0int(true);
                w.set_ctmra0c1int(true);
                w.set_ctmrb0c1int(true);
            });
            ctimer.inten().modify(|w| w.set_ctmra0c0int(false)); // Disable compare int

            critical_section::with(|cs| {
                self.alarm.borrow(cs).timestamp.set(u64::MAX);
                self.trigger_alarm(cs);
            });
        }
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let alarm = self.alarm.borrow(cs);
        alarm.timestamp.set(timestamp);

        let now = self.now();
        if timestamp <= now {
            pac::CTIMER.inten().modify(|w| w.set_ctmra0c0int(false));
            alarm.timestamp.set(u64::MAX);
            return false;
        }

        // Calculate diff in 1MHz ticks, then scale to 3MHz hardware ticks
        let diff = timestamp.saturating_sub(now);
        let diff_hw = diff.saturating_mul(3);
        let diff_u32 = if diff_hw > 0x7FFF_FFFF {
            0x7FFF_FFFF
        } else {
            diff_hw as u32
        };

        let current = pac::CTIMER.tmr0().read().0;
        let target = current.wrapping_add(diff_u32);

        // 32-bit timer mode requires the target to be split across A and B registers
        pac::CTIMER
            .cmpra0()
            .write_value(pac::ctimer::regs::Cmpra0(target & 0xFFFF));
        pac::CTIMER
            .cmprb0()
            .write_value(pac::ctimer::regs::Cmprb0(target >> 16));

        pac::CTIMER.intclr().write(|w| w.set_ctmra0c0int(true));
        pac::CTIMER.inten().modify(|w| w.set_ctmra0c0int(true));

        true
    }
}

#[cfg(feature = "time-driver-ctimer0")]
impl Driver for Apollo3TimeDriver {
    fn now(&self) -> u64 {
        critical_section::with(|_| {
            let low = pac::CTIMER.tmr0().read();
            let high = self.overflows.load(Ordering::Relaxed);

            // hardware ticks at 3MHz. Convert to 1MHz ticks (divide by 3).
            let hw_ticks = ((high as u64) << 32) | (low.0 as u64);
            hw_ticks / 3
        })
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

#[cfg(feature = "time-driver-ctimer0")]
#[interrupt]
fn CTIMER() {
    DRIVER.on_interrupt();
}
