use core::mem;
use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU8, Ordering};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::TICK_HZ;
use embassy_time_driver::{AlarmHandle, Driver};
use stm32_metapac::lptim::regs::IcrAdv;

use super::{AlarmState, ALARM_STATE_NEW};
use crate::interrupt::typelevel::Interrupt;
use crate::lptim::SealedInstance;
use crate::rcc::{self, SealedRccPeripheral};
use crate::{interrupt, peripherals};

#[cfg(time_driver_lptim1)]
type T = peripherals::LPTIM1;
#[cfg(time_driver_lptim2)]
type T = peripherals::LPTIM1;

foreach_interrupt! {
    (LPTIM1, lptim, $block:ident, GLOBAL, $irq:ident) => {
        #[cfg(time_driver_lptim1)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (LPTIM2, lptim, $block:ident, GLOBAL, $irq:ident) => {
        #[cfg(time_driver_lptim2)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
}

pub(crate) fn init(cs: CriticalSection) {
    DRIVER.init(cs)
}

/*impl RtcDriver {
    #[cfg(time_driver_lptim1)]
    fn init(&'static self, cs: critical_section::CriticalSection) {
        let r = T::regs();

        rcc::enable_and_reset_with_cs::<T>(cs);

        <T as SealedInstance>::Interrupt::unpend();
        unsafe {
            <T as SealedInstance>::Interrupt::enable();
        };

        let timer_freq = T::frequency();

        // disable timer to write to CFGR register
        r.cr().modify(|w| w.set_enable(false));
        // set counter to 0
        r.cnt().write(|w| w.set_cnt(0));

        // calculate the prescaler value
        let prescaler = if timer_freq.0 < TICK_HZ as u32 {
            panic!("lptim1 (= {}) is clocked too slow for desired TICK_HZ (= {})", timer_freq, TICK_HZ);
        } else if timer_freq.0 % TICK_HZ as u32 != 0 {
            panic!("frequency of lptim1 (= {}) must be a multiple of TICK_HZ (= {})", timer_freq, TICK_HZ);
        } else {
            use crate::pac::lptim::vals::Presc;
            match timer_freq.0 / TICK_HZ as u32 {
                1 => Presc::DIV1,
                2 => Presc::DIV2,
                4 => Presc::DIV4,
                8 => Presc::DIV8,
                16 => Presc::DIV16,
                32 => Presc::DIV32,
                64 => Presc::DIV64,
                128 => Presc::DIV128,
                _ => panic!("no valid prescaler value found for lptim1 (= {}) and TICK_HZ (= {})", timer_freq.0, TICK_HZ),
            }
        };
        // set the prescaler
        r.cfgr().write(|w| w.set_presc(prescaler));

        debug!("timer_freq: {}", timer_freq);


        // TODO: check what the URS stuff is for



        // enable timer to write do DIER
        r.cr().modify(|w| w.set_enable(true));
        for _ in 0..10 {

        }
        info!("dier: {}", r.dier().read().0);
        info!("isr: {}", r.isr().read().0);
        r.arr().write(|w| w.set_arr(u16::MAX));
        while r.isr().read().arrok() == false {}
        // Mid-way point
        r.ccr(0).write(|w| w.set_ccr(0x8000));
        // unpend interrupts
        r.icr().write(|w| {
            w.set_uecf(true);
            w.set_cccf(0, true);
            w.set_dierokcf(true);
        });
        // Enable overflow and half-overflow interrupts
        r.dier().write(|w| {
            w.set_ueie(true);
            w.set_ccie(0, true);
        });
        while r.isr().read().dierok() == false {}

        // start continuous mode
        r.cr().modify(|w| w.set_cntstrt(true));
    }

    fn on_interrupt(&self) {
        let r = regs_gp16();

        // XXX: reduce the size of this critical section ?
        critical_section::with(|cs| {
            #[cfg(not(time_driver_lptim1))]
            let sr = r.sr().read();
            #[cfg(time_driver_lptim1)]
            let isr = r.isr().read();

            let dier = r.dier().read();

            // Clear all interrupt flags. Bits in SR are "write 0 to clear", so write the bitwise NOT.
            // Other approaches such as writing all zeros, or RMWing won't work, they can
            // miss interrupts.
            #[cfg(not(time_driver_lptim1))]
            r.sr().write_value(regs::SrGp16(!sr.0));

            #[cfg(time_driver_lptim1)]
            r.icr().write_value(IcrAdv(isr.0));

            // Overflow
            #[cfg(not(time_driver_lptim1))]
            if sr.uif() {
                self.next_period();
            }
            #[cfg(time_driver_lptim1)]
            if isr.arrm() {
                self.next_period();
            }

            // Half overflow
            if isr.ccif(0) {
                self.next_period();
            }

            for n in 0..ALARM_COUNT {
                if isr.ccif(n + 1) && dier.ccie(n + 1) {
                    self.trigger_alarm(n, cs);
                }
            }
        })
    }
}*/
// TODO: let the user choose somehow
const ALARM_COUNT: usize = 1;

/// The LptimTimeDriver depends on a 16bit hardware counter. The software counters are increased on overflow.
/// The capture and compare channel is used for the alarms. It is not associated with any specific alarm, but
/// all alarms are checked (only on capture and compare match).
pub(crate) struct LptimTimeDriver {
    ticks_upper: AtomicU32,
    ticks_lower: AtomicU16,
    alarm_count: AtomicU8,
    alarms: Mutex<CriticalSectionRawMutex, [AlarmState; ALARM_COUNT]>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: LptimTimeDriver = LptimTimeDriver {
    ticks_upper: AtomicU32::new(0),
    ticks_lower: AtomicU16::new(0),
    alarm_count: AtomicU8::new(0),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), [ALARM_STATE_NEW; ALARM_COUNT]),
});

fn calc_now(ticks_upper: u32, ticks_lower: u16, counter: u16) -> u64 {
    ((ticks_upper as u64) << 32) + ((ticks_lower as u64) << 16) + (counter as u64)
}

fn calc_ticks_from_timestamp(timestamp: u64) -> (u32, u16) {
    let ticks_upper = (timestamp >> 32) as u32;
    let ticks_lower = ((timestamp & 0x00000000_FFFF0000) >> 16) as u16;
    (ticks_upper, ticks_lower)
}

fn calc_counter_from_timestamp(timestamp: u64) -> u16 {
    (timestamp & 0x00000000_0000FFFF) as u16
}

impl LptimTimeDriver {
    pub(crate) fn init(&'static self, cs: critical_section::CriticalSection) {
        let r = T::regs();

        rcc::enable_and_reset_with_cs::<T>(cs);

        <T as SealedInstance>::Interrupt::unpend();
        unsafe {
            <T as SealedInstance>::Interrupt::enable();
        }

        let timer_freq = T::frequency();

        // disable timer to write to CFGR register
        r.cr().modify(|w| w.set_enable(false));
        // set counter to 0
        r.cnt().write(|w| w.set_cnt(0));

        // calculate and set the prescaler value
        r.cfgr().write(|w| {
            w.set_presc(if timer_freq.0 < TICK_HZ as u32 {
                panic!(
                    "lptim (= {}) is clocked too slow for desired TICK_HZ (= {})",
                    timer_freq, TICK_HZ
                );
            } else if timer_freq.0 % TICK_HZ as u32 != 0 {
                panic!(
                    "frequency of lptim1 (= {}) must be a multiple of TICK_HZ (= {})",
                    timer_freq, TICK_HZ
                );
            } else {
                use crate::pac::lptim::vals::Presc;
                match timer_freq.0 / TICK_HZ as u32 {
                    1 => Presc::DIV1,
                    2 => Presc::DIV2,
                    4 => Presc::DIV4,
                    8 => Presc::DIV8,
                    16 => Presc::DIV16,
                    32 => Presc::DIV32,
                    64 => Presc::DIV64,
                    128 => Presc::DIV128,
                    _ => panic!(
                        "no valid prescaler value found for lptim1 (= {}) and TICK_HZ (= {})",
                        timer_freq.0, TICK_HZ
                    ),
                }
            })
        });

        // enable timer to write to DIER, CCR, ARR
        r.cr().modify(|w| w.set_enable(true));

        // set ARR
        r.arr().write(|w| w.set_arr(u16::MAX));
        // check that write is finished
        while r.isr().read().arrok() == false {}

        // unpend interrupts
        r.icr().write(|w| {
            w.set_arrmcf(true);
        });

        // Enable overflow interrupt
        r.dier().write(|w| {
            w.set_arrmie(true);
        });
        // check that write is finished
        while r.isr().read().dierok() == false {}

        // start continuous mode
        r.cr().modify(|w| w.set_cntstrt(true));
    }

    fn on_interrupt(&self) {
        // TODO: check all the memory ordering
        let r = T::regs();

        critical_section::with(|cs| {
            let isr = r.isr().read();

            // Clear all interrupts. Bits in ISR are "write 1 to clear", so write ISR to ICR.
            // Other approaches such as writing all zeros, or RMWing won't work, they can
            // miss interrupts.
            r.icr().write_value(IcrAdv(isr.0));

            let ticks_upper = self.ticks_upper.load(Ordering::Relaxed);
            let ticks_lower = self.ticks_lower.load(Ordering::Relaxed);

            // Overflow
            let now = if isr.arrm() {
                let new_ticks_lower = ticks_lower.overflowing_add(1);

                self.ticks_lower
                    .compare_exchange(ticks_lower, new_ticks_lower.0, Ordering::Relaxed, Ordering::Relaxed)
                    .expect("No one else is writing them so it shouldn't fail to write them.");

                // add 1 to ticks_upper if ticks_lower had an overflow
                let new_ticks_upper = if new_ticks_lower.1 {
                    // add 1 to ticks_upper, ignoring an overflow since it is very far in the future
                    let new_ticks_upper = ticks_upper.overflowing_add(1).0;

                    self.ticks_upper
                        .compare_exchange(ticks_upper, new_ticks_upper, Ordering::Relaxed, Ordering::Relaxed)
                        .expect("No one else is writing these so it shouldn't fail to write them.");
                    new_ticks_upper
                } else {
                    // no overflow -> ticks_upper stays the same
                    ticks_upper
                };

                // check if next alarms is before the next counter overflow
                self.set_ccp_interrupt_for_next_alarm_if_before_overflow(cs, new_ticks_upper, new_ticks_lower.0);

                calc_now(new_ticks_upper, new_ticks_lower.0, r.cnt().read().cnt())
            } else {
                calc_now(ticks_upper, ticks_lower, r.cnt().read().cnt())
            };

            // check if an alarm is ready
            for n in 0..ALARM_COUNT {
                self.trigger_alarm_if_ready(now, n, cs);
            }
        });
    }

    fn set_ccp_interrupt_if_before_overflow(&self, alarm_timestamp: u64, ticks_upper: u32, ticks_lower: u16) {
        // skip u64::MAX immediately
        if alarm_timestamp == u64::MAX {
            return;
        }

        let r = T::regs();

        let (alarm_ticks_upper, alarm_ticks_lower) = calc_ticks_from_timestamp(alarm_timestamp);
        if alarm_ticks_upper == ticks_upper && alarm_ticks_lower == ticks_lower {
            // alarm is before next counter overflow

            // set capture compare value
            r.ccr(0)
                .modify(|m| m.set_ccr(calc_counter_from_timestamp(alarm_timestamp)));

            // enable capture compare alarm
            r.dier().modify(|m| m.set_ccie(0, true));
            while r.isr().read().dierok() == false {}
        }
    }

    fn set_ccp_interrupt_for_next_alarm_if_before_overflow(
        &self,
        cs: CriticalSection,
        ticks_upper: u32,
        ticks_lower: u16,
    ) {
        let next_timestamp = self.next_alarm_timestamp(cs);
        self.set_ccp_interrupt_if_before_overflow(next_timestamp, ticks_upper, ticks_lower);
    }

    fn trigger_alarm_if_ready(&self, now: u64, n: usize, cs: CriticalSection) {
        let alarm = &self.alarms.borrow(cs)[n];

        if alarm.timestamp.get() <= now {
            alarm.timestamp.set(u64::MAX);
        }

        // Call the callback after clearing the alarm, so the callback can set another alarm.

        // safety:
        // - we can ignore the possibility of `f` being unset (null) because of the safety contract of `allocate_alarm`.
        // - other than that we only store valid function pointers into alarm.callback
        let f: fn(*mut ()) = unsafe { mem::transmute(alarm.callback.get()) };
        f(alarm.ctx.get());
    }

    fn next_alarm_timestamp(&self, cs: CriticalSection) -> u64 {
        self.alarms.borrow(cs).iter().map(|a| a.timestamp.get()).min().unwrap()
    }

    fn get_alarm<'a>(&'a self, cs: CriticalSection<'a>, alarm: AlarmHandle) -> &'a AlarmState {
        // safety: we're allowed to assume the AlarmState is created by us, and
        // we never create one that's out of bounds.
        unsafe { self.alarms.borrow(cs).get_unchecked(alarm.id() as usize) }
    }
}

impl Driver for LptimTimeDriver {
    fn now(&self) -> u64 {
        // TODO: check memory ordering

        let r = T::regs();
        loop {
            let ticks_lower = self.ticks_lower.load(Ordering::Acquire);

            let ticks_upper = self.ticks_upper.load(Ordering::Relaxed);

            let ticks_lower_2 = self.ticks_lower.load(Ordering::Acquire);

            if ticks_lower == ticks_lower_2 {
                return calc_now(ticks_upper, ticks_lower, r.cnt().read().cnt());
            }
        }
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        critical_section::with(|_| {
            let id = self.alarm_count.load(Ordering::Relaxed);
            if id < ALARM_COUNT as u8 {
                self.alarm_count.store(id + 1, Ordering::Relaxed);
                Some(AlarmHandle::new(id))
            } else {
                None
            }
        })
    }

    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        critical_section::with(|cs| {
            let alarm = self.get_alarm(cs, alarm);

            alarm.callback.set(callback as *const ());
            alarm.ctx.set(ctx);
        })
    }

    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        critical_section::with(|cs| {
            let r = T::regs();

            let n = alarm.id() as usize;
            let alarm = self.get_alarm(cs, alarm);
            alarm.timestamp.set(timestamp);

            let t = self.now();
            if timestamp <= t {
                // If alarm timestamp has passed the alarm will not fire.
                // Disarm the alarm and return `false` to indicate that.
                alarm.timestamp.set(u64::MAX);

                return false;
            }

            // recompute the ccp interrupt
            let (ticks_upper, ticks_lower) = calc_ticks_from_timestamp(t);
            self.set_ccp_interrupt_for_next_alarm_if_before_overflow(cs, ticks_upper, ticks_lower);

            // Reevaluate if the alarm timestamp is still in the future
            let t = self.now();
            if timestamp <= t {
                // If alarm timestamp has passed since we set it, we have a race condition and
                // the alarm may or may not have fired.
                // Disarm the alarm and return 'false' to indicate that.
                // It is the caller's responsibility to handle this ambiguity.
                r.dier().modify(|m| m.set_ccie(0, false));

                alarm.timestamp.set(u64::MAX);

                return false;
            }

            // We're confident the alarm will ring in the future
            true
        })
    }
}
