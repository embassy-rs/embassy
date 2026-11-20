//! `embassy-time` driver using the ASR6601 LPTIM0.
//!
//! This follows the vendor `tremo_lptimer` programming model and the STM32
//! LPTIM time-driver pattern:
//! * LPTIM0 is clocked from the always-on XO32K at 32_768 Hz (verified against
//!   official `tremo.svd` v1.6.2 and the ASR6601 Reference Manual v1.5.0
//!   plus `tremo_lptimer.h`)
//! * The 16-bit counter free-runs with ARR = 0xFFFF; overflow (ARRM) extends
//!   it to 64 bit
//! * CMP provides one-shot wakeups. Far-future alarms are deferred until the
//!   overflow period brings them within range.

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use core::task::Waker;

use critical_section::{CriticalSection, Mutex};
use embassy_hal_internal::interrupt::{InterruptExt, Priority};
use embassy_time_driver::{Driver, TICK_HZ};
use embassy_time_queue_utils::Queue;

use crate::afec;
use crate::lptimer::{ISR_ARRM, ISR_ARROK, ISR_CFGROK, ISR_CMPM, ISR_CMPOK, ISR_CROK};
use crate::pac::{self, Interrupt, interrupt};
use crate::rcc::{self, Peripheral as RccPeripheral};

const TIMER_TICK_HZ: u64 = rcc::XO32K_HZ as u64;
const ARR_MAX: u16 = 0xFFFF;

const _: () = assert!(TICK_HZ == TIMER_TICK_HZ);

// The LPTIM flag bitmaps are shared with the main LPTIMER driver. They feed
// the private `wait_isr`/`wait_csr` helpers and the interrupt bitmap; the
// individual fields are also available through the PAC accessors.
const POLL_LIMIT: u32 = 1_000_000;

// If the alarm is more than ~75% of the 16-bit range in the future we defer
// enabling the compare interrupt until the next overflow. This avoids aliasing
// where the 16-bit compare value shadows an earlier period (same approach as
// `embassy-stm32/src/time_driver/lptim.rs`).
const COMPARE_THRESHOLD: u64 = 0xc000;

// Note: a compare value only a few ticks ahead of the running counter is
// missed for the current 2 s cycle on this silicon (the new value is not
// evaluated until the next overflow); such an alarm then fires up to one full
// period late. Keep short (sub-millisecond) deadlines in mind when using this
// driver.

struct LptimTimeDriver {
    initialized: AtomicBool,
    period: AtomicU32,
    alarm: Mutex<Cell<u64>>,
    queue: Mutex<RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: LptimTimeDriver = LptimTimeDriver {
    initialized: AtomicBool::new(false),
    period: AtomicU32::new(0),
    alarm: Mutex::new(Cell::new(u64::MAX)),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

fn lptim() -> pac::Lptim0 {
    unsafe { pac::Lptim0::steal() }
}

fn wait_isr(mask: u32) {
    // The mask form stays to match the vendor polling sequence bit-for-bit.
    for _ in 0..POLL_LIMIT {
        if lptim().isr().read().bits() & mask == mask {
            return;
        }
        core::hint::spin_loop();
    }
}

// CSR acknowledge for ARRM/CMPM, per the vendor wakeup example. Bounded so a
// wedged timer cannot hang the caller (or the time-driver ISR) forever; the
// normal path answers in a few cycles. The mask form matches the vendor
// wakeup example sequence.
fn wait_csr(mask: u32) {
    for _ in 0..POLL_LIMIT {
        if lptim().csr().read().bits() & mask == mask {
            return;
        }
        core::hint::spin_loop();
    }
}

impl LptimTimeDriver {
    fn init(&'static self) {
        // XO32K is in the always-on domain. Clear its AFEC power-down bits
        // (matches vendor `rcc_enable_oscillator(RCC_OSC_XO32K)`).
        afec::analog::with(|afec| {
            afec.analog_02().modify(|_, w| w.xo32k_power_down().enabled());
        });

        // Gate, reset, select XO32K, re-enable. Mirrors SDK
        // `rcc_enable_peripheral_clk` / `rcc_rst_peripheral` plus vendor 92 us
        // async-domain delay after reset release (handled in rcc helpers).
        let _ = rcc::disable_peripheral(RccPeripheral::Lptimer0);
        let _ = rcc::reset_peripheral(RccPeripheral::Lptimer0);

        // Select XO32K while functional clock is gated (vendor
        // `rcc_set_lptimer0_clk_source`). New PAC has proper variants.
        critical_section::with(|_| {
            // Gate functional clock and wait for sync clear if needed.
            let sync = || {
                unsafe { pac::Rcc::steal() }
                    .sr1()
                    .read()
                    .lptim0_clk_en_sync()
                    .bit_is_set()
            };
            if sync() {
                unsafe { pac::Rcc::steal() }
                    .cgr1()
                    .modify(|_, w| w.lptim0_clk_en().clear_bit());
                for _ in 0..POLL_LIMIT {
                    if !unsafe { pac::Rcc::steal() }
                        .sr1()
                        .read()
                        .lptim0_clk_en_sync()
                        .bit_is_set()
                    {
                        break;
                    }
                    core::hint::spin_loop();
                }
            }
            unsafe { pac::Rcc::steal() }.cr1().modify(|_, w| {
                w.lptim0_ext_clk_sel().clear_bit();
                w.lptim0_clk_sel().xo32k()
            });
        });

        // A bootloader may leave LPTIM0 clocked and counting with its own
        // interrupts armed. Don't panic if the always-on sync never asserts;
        // the functional gate is sufficient and every register below is
        // reprogrammed unconditionally to the needed configuration.
        let _ = rcc::enable_peripheral(RccPeripheral::Lptimer0);

        // Disarm interrupts before reconfiguring so no stale bootloader setup
        // can fire while registers are being reprogrammed.
        lptim().ier().modify(|_, w| {
            w.arrmie().clear_bit();
            w.cmpmie().clear_bit();
            w
        });
        Interrupt::LPTIM0.unpend();

        // Configure: internal clock, prescaler /1, no preload, no wave.
        wait_isr(ISR_CFGROK);
        lptim().cfgr().modify(|_, w| {
            w.countmode().clear_bit();
            w.preload().clear_bit();
            w.wavpol().clear_bit();
            w.presc().div1()
        });
        wait_isr(ISR_CFGROK);

        // Enable peripheral.
        lptim().cr().modify(|_, w| w.enable().set_bit());
        wait_isr(ISR_CROK);

        // ARR = max (`write_with_zero` is unsafe on ARR in the PAC).
        unsafe { lptim().arr().write_with_zero(|w| w.arr().bits(ARR_MAX)) };
        wait_isr(ISR_ARROK);

        // CMP = 0.
        unsafe { lptim().cmp().write_with_zero(|w| w.cmp().bits(0)) };
        wait_isr(ISR_CMPOK);

        // Clear pending flags through per-flag ICR accessors.
        unsafe {
            lptim().icr().write_with_zero(|w| {
                w.arrmcf().set_bit();
                w.cmpmcf().set_bit();
                w
            })
        };
        wait_csr(ISR_ARRM | ISR_CMPM);

        // Enable overflow interrupt; compare enabled on demand.
        lptim().ier().modify(|_, w| w.arrmie().set_bit());

        // Start continuous counting.
        lptim().cr().modify(|_, w| w.cntstrt().set_bit());
        wait_isr(ISR_CROK);

        self.period.store(0, Ordering::Release);
        // No second init is expected (hal::init runs once), but leave no
        // stale deadline behind if it ever re-runs: the queue itself belongs
        // to embassy-time and cannot be drained safely from here.
        critical_section::with(|cs| self.alarm.borrow(cs).set(u64::MAX));
        self.initialized.store(true, Ordering::Release);

        Interrupt::LPTIM0.unpend();
        Interrupt::LPTIM0.set_priority(Priority::P2);
        unsafe { Interrupt::LPTIM0.enable() };
    }

    fn now_inner(&self) -> u64 {
        // Called with interrupts masked to avoid tearing across overflow. The
        // counter itself can still wrap while we read it, so take two stable
        // reads and make sure the overflow flag did not change meanwhile. This
        // mirrors the STM32 LPTIM time driver.
        loop {
            let pending_before = lptim().isr().read().arrm().bit_is_set();
            let mut cnt = lptim().cnt().read().cnt().bits();
            loop {
                let again = lptim().cnt().read().cnt().bits();
                if again == cnt {
                    break;
                }
                cnt = again;
            }
            let pending_after = lptim().isr().read().arrm().bit_is_set();
            if pending_before == pending_after {
                let period = self.period.load(Ordering::Relaxed) + pending_after as u32;
                return ((period as u64) << 16) | u64::from(cnt);
            }
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        self.alarm.borrow(cs).set(timestamp);

        if timestamp == u64::MAX {
            lptim().ier().modify(|_, w| w.cmpmie().clear_bit());
            return true;
        }

        let now = self.now_inner();
        if timestamp <= now {
            lptim().ier().modify(|_, w| w.cmpmie().clear_bit());
            self.alarm.borrow(cs).set(u64::MAX);
            return false;
        }

        let cmp = (timestamp & 0xFFFF) as u16;
        unsafe { lptim().cmp().write_with_zero(|w| w.cmp().bits(cmp)) };
        wait_isr(ISR_CMPOK);

        let diff = timestamp - now;
        if diff < COMPARE_THRESHOLD {
            lptim().ier().modify(|_, w| w.cmpmie().set_bit());
        } else {
            lptim().ier().modify(|_, w| w.cmpmie().clear_bit());
        }

        if timestamp <= self.now_inner() {
            lptim().ier().modify(|_, w| w.cmpmie().clear_bit());
            self.alarm.borrow(cs).set(u64::MAX);
            return false;
        }

        true
    }

    /// Extend the 16-bit counter by one period.
    ///
    /// Hardware latches only one overflow flag, so an interrupt latency longer
    /// than one full 2-second period loses elapsed time. Keep the time-driver
    /// interrupt at a priority that cannot be blocked for that long.
    fn next_period(&self) {
        let period = self.period.load(Ordering::Relaxed) + 1;
        self.period.store(period, Ordering::Release);
        let t = (period as u64) << 16;

        critical_section::with(|cs| {
            let alarm = self.alarm.borrow(cs).get();
            if alarm != u64::MAX && alarm.saturating_sub(t) < COMPARE_THRESHOLD {
                lptim().ier().modify(|_, w| w.cmpmie().set_bit());
            }
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        lptim().ier().modify(|_, w| w.cmpmie().clear_bit());
        self.alarm.borrow(cs).set(u64::MAX);

        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now_inner());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now_inner());
        }
    }

    fn on_interrupt(&self) {
        // Keep the flag clear, period increment, and alarm re-arm in one
        // critical section. Otherwise a higher-priority interrupt calling
        // `Instant::now()` between clearing ARRM and incrementing `period`
        // could observe time going backwards by one 16-bit period.
        critical_section::with(|cs| {
            let pending = lptim().isr().read().bits() & lptim().ier().read().bits() & (ISR_ARRM | ISR_CMPM);
            if pending == 0 {
                return;
            }

            // Clear exactly the observed flags through per-flag ICR accessors.
            unsafe {
                lptim().icr().write_with_zero(|w| {
                    if pending & ISR_ARRM != 0 {
                        w.arrmcf().set_bit();
                    }
                    if pending & ISR_CMPM != 0 {
                        w.cmpmcf().set_bit();
                    }
                    w
                })
            };
            if pending & (ISR_ARRM | ISR_CMPM) != 0 {
                wait_csr(pending & (ISR_ARRM | ISR_CMPM));
            }

            if pending & ISR_ARRM != 0 {
                self.next_period();
            }
            if pending & ISR_CMPM != 0 {
                self.trigger_alarm(cs);
            }
        });
    }
}

impl Driver for LptimTimeDriver {
    fn now(&self) -> u64 {
        if !self.initialized.load(Ordering::Acquire) {
            return 0;
        }
        critical_section::with(|_| self.now_inner())
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
fn LPTIM0() {
    DRIVER.on_interrupt();
}
