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
use crate::pac::{self, Interrupt, interrupt};
use crate::rcc::{self, Peripheral as RccPeripheral};

const TIMER_TICK_HZ: u64 = 32_768;
const ARR_MAX: u16 = 0xFFFF;

// Vendor `tremo_lptimer.h` bit definitions (also in RM). ISR/CSR have proper
// PAC accessors in Rimpampa/ASR6601-PAC@svd; IER/ICR/ARR/CMP/CNT/SR1 are raw.
const ISR_CMPM: u32 = 1 << 0;
const ISR_ARRM: u32 = 1 << 1;
const ISR_CMPOK: u32 = 1 << 3;
const ISR_ARROK: u32 = 1 << 4;
const ISR_CFGROK: u32 = 1 << 7;
const ISR_CROK: u32 = 1 << 8;

const IER_CMPM: u32 = 1 << 0;
const IER_ARRM: u32 = 1 << 1;

const CFGR_PRESC_MASK: u32 = 0xe00;

const CR_ENABLE: u32 = 1 << 0;
const CR_CNTSTRT: u32 = 1 << 2;

// AFEC analog register 0x02 bits 13/14 power-gate XO32K. Clearing them matches
// `rcc_enable_oscillator(RCC_OSC_XO32K)`.
const ANALOG_XO32K_POWER_DOWN: u32 = (1 << 13) | (1 << 14);

const POLL_LIMIT: u32 = 1_000_000;

// If the alarm is more than ~75% of the 16-bit range in the future we defer
// enabling the compare interrupt until the next overflow. This avoids aliasing
// where the 16-bit compare value shadows an earlier period (same approach as
// `embassy-stm32/src/time_driver/lptim.rs`).
const COMPARE_THRESHOLD: u64 = 0xc000;

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

fn lptim() -> pac::Lptimer0 {
    unsafe { pac::Lptimer0::steal() }
}

fn wait_isr(mask: u32) {
    for _ in 0..POLL_LIMIT {
        if lptim().isr().read().bits() & mask == mask {
            return;
        }
        core::hint::spin_loop();
    }
}

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
        assert!(TICK_HZ == TIMER_TICK_HZ, "embassy-asr: time tick rate must be 32768 Hz");

        // XO32K is in the always-on domain. Clear its AFEC power-down bits
        // (matches vendor `rcc_enable_oscillator(RCC_OSC_XO32K)`).
        afec::analog::REG_02.clear_bits(ANALOG_XO32K_POWER_DOWN);

        // Gate, reset, select XO32K, re-enable. Mirrors SDK
        // `rcc_enable_peripheral_clk` / `rcc_rst_peripheral` plus vendor 92 us
        // async-domain delay after reset release (handled in rcc helpers).
        let _ = rcc::disable_peripheral(RccPeripheral::Lptimer0);
        let _ = rcc::reset_peripheral(RccPeripheral::Lptimer0);

        // Select XO32K while functional clock is gated (vendor
        // `rcc_set_lptimer0_clk_source`). New PAC has proper variants.
        critical_section::with(|_| {
            // Gate functional clock and wait for sync clear if needed.
            let sync = || unsafe { pac::Rcc::steal() }.sr1().read().lptimer0_clk_en_sync().bit_is_set();
            if sync() {
                unsafe { pac::Rcc::steal() }
                    .cgr1()
                    .modify(|_, w| w.lptimer0_clk_en().clear_bit());
                for _ in 0..POLL_LIMIT {
                    if !unsafe { pac::Rcc::steal() }.sr1().read().lptimer0_clk_en_sync().bit_is_set() {
                        break;
                    }
                    core::hint::spin_loop();
                }
            }
            unsafe { pac::Rcc::steal() }.cr1().modify(|_, w| {
                w.lptimer0_extclk_sel().clear_bit();
                w.lptimer0_clk_sel().xo32k()
            });
        });

        // A bootloader may leave LPTIM0 clocked and counting with its own
        // interrupts armed. Don't panic if the always-on sync never asserts;
        // the functional gate is sufficient and every register below is
        // reprogrammed unconditionally to the needed configuration.
        let _ = rcc::enable_peripheral(RccPeripheral::Lptimer0);

        // Disarm interrupts before reconfiguring so no stale bootloader setup
        // can fire while registers are being reprogrammed.
        lptim().ier().modify(|r, w| unsafe { w.bits(r.bits() & !(IER_ARRM | IER_CMPM)) });
        Interrupt::LPTIMER0.unpend();

        // Configure: internal clock, prescaler /1, no preload, no wave.
        wait_isr(ISR_CFGROK);
        lptim().cfgr().modify(|_, w| {
            w.countmode().clear_bit();
            w.preload().clear_bit();
            w.wavpol().clear_bit()
        });
        wait_isr(ISR_CFGROK);
        // PRESC /1 is missing in PAC/SVD; program raw bits from tremo_lptimer.h.
        lptim().cfgr().modify(|r, w| unsafe {
            w.bits((r.bits() & !CFGR_PRESC_MASK) | 0x0)
        });
        wait_isr(ISR_CFGROK);

        // Enable peripheral.
        lptim().cr().modify(|_, w| w.enable().set_bit());
        wait_isr(ISR_CROK);

        // ARR = max.
        unsafe { lptim().arr().write_with_zero(|w| w.bits(ARR_MAX as u32)) };
        wait_isr(ISR_ARROK);

        // CMP = 0.
        unsafe { lptim().cmp().write_with_zero(|w| w.bits(0)) };
        wait_isr(ISR_CMPOK);

        // Clear pending flags.
        unsafe { lptim().icr().write_with_zero(|w| w.bits(ISR_ARRM | ISR_CMPM)) };
        wait_csr(ISR_ARRM | ISR_CMPM);

        // Enable overflow interrupt; compare enabled on demand.
        lptim().ier().modify(|r, w| unsafe { w.bits(r.bits() | IER_ARRM) });

        // Start continuous counting (CNTSTRT missing in PAC; raw from header).
        lptim().cr().modify(|r, w| unsafe { w.bits(r.bits() | CR_CNTSTRT) });
        wait_isr(ISR_CROK);

        // Silence unused-constant warning for ENABLE mask (we use PAC accessor).
        let _ = CR_ENABLE;

        self.period.store(0, Ordering::Release);
        self.initialized.store(true, Ordering::Release);

        Interrupt::LPTIMER0.unpend();
        Interrupt::LPTIMER0.set_priority(Priority::P2);
        unsafe { Interrupt::LPTIMER0.enable() };
    }

    fn now_inner(&self) -> u64 {
        // Called with interrupts masked to avoid tearing across overflow.
        let period = self.period.load(Ordering::Relaxed);
        let cnt = lptim().cnt().read().bits() as u16 as u64;
        let pending = if lptim().isr().read().bits() & ISR_ARRM != 0 { 1 } else { 0 };
        ((period as u64 + pending as u64) << 16) | cnt
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        self.alarm.borrow(cs).set(timestamp);

        if timestamp == u64::MAX {
            lptim().ier().modify(|r, w| unsafe { w.bits(r.bits() & !IER_CMPM) });
            return true;
        }

        let now = self.now_inner();
        if timestamp <= now {
            lptim().ier().modify(|r, w| unsafe { w.bits(r.bits() & !IER_CMPM) });
            self.alarm.borrow(cs).set(u64::MAX);
            return false;
        }

        let cmp = (timestamp & 0xFFFF) as u32;
        unsafe { lptim().cmp().write_with_zero(|w| w.bits(cmp)) };
        wait_isr(ISR_CMPOK);

        let diff = timestamp - now;
        if diff < COMPARE_THRESHOLD {
            lptim().ier().modify(|r, w| unsafe { w.bits(r.bits() | IER_CMPM) });
        } else {
            lptim().ier().modify(|r, w| unsafe { w.bits(r.bits() & !IER_CMPM) });
        }

        if timestamp <= self.now_inner() {
            lptim().ier().modify(|r, w| unsafe { w.bits(r.bits() & !IER_CMPM) });
            self.alarm.borrow(cs).set(u64::MAX);
            return false;
        }

        true
    }

    fn next_period(&self) {
        let period = self.period.load(Ordering::Relaxed) + 1;
        self.period.store(period, Ordering::Release);
        let t = (period as u64) << 16;

        critical_section::with(|cs| {
            let alarm = self.alarm.borrow(cs).get();
            if alarm != u64::MAX && alarm.saturating_sub(t) < COMPARE_THRESHOLD {
                lptim().ier().modify(|r, w| unsafe { w.bits(r.bits() | IER_CMPM) });
            }
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        lptim().ier().modify(|r, w| unsafe { w.bits(r.bits() & !IER_CMPM) });
        self.alarm.borrow(cs).set(u64::MAX);

        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now_inner());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now_inner());
        }
    }

    fn on_interrupt(&self) {
        let pending = lptim().isr().read().bits() & lptim().ier().read().bits() & (ISR_ARRM | ISR_CMPM);
        if pending == 0 {
            return;
        }

        unsafe { lptim().icr().write_with_zero(|w| w.bits(pending)) };
        if pending & (ISR_ARRM | ISR_CMPM) != 0 {
            wait_csr(pending & (ISR_ARRM | ISR_CMPM));
        }

        if pending & ISR_ARRM != 0 {
            self.next_period();
        }
        if pending & ISR_CMPM != 0 {
            critical_section::with(|cs| self.trigger_alarm(cs));
        }
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
fn LPTIMER0() {
    DRIVER.on_interrupt();
}
