//! Timer driver.
use core::cell::{Cell, RefCell};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

#[cfg(all(feature = "_rp235x", feature = "time-driver-timer1"))]
use embassy_rp::clocks;
#[cfg(all(feature = "_rp235x", feature = "time-driver-mtime"))]
use embassy_rp::clocks;
#[cfg(all(feature = "_rp235x", feature = "time-driver-aot"))]
use pac::POWMAN as TIMER;
#[cfg(all(feature = "_rp235x", feature = "time-driver-mtime"))]
use pac::SIO as TIMER;
#[cfg(feature = "rp2040")]
use pac::TIMER;
#[cfg(all(
    feature = "_rp235x",
    not(any(
        feature = "time-driver-timer1",
        feature = "time-driver-mtime",
        feature = "time-driver-aot"
    ))
))]
use pac::TIMER0 as TIMER;
#[cfg(all(feature = "_rp235x", feature = "time-driver-timer1"))]
use pac::TIMER1 as TIMER;

use crate::interrupt::InterruptExt;
use crate::{interrupt, pac};

struct AlarmState {
    timestamp: Cell<u64>,
}
unsafe impl Send for AlarmState {}

struct TimerDriver {
    alarms: Mutex<CriticalSectionRawMutex, AlarmState>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimerDriver = TimerDriver{
    alarms:  Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState {
        timestamp: Cell::new(0),
    }),
    queue: Mutex::new(RefCell::new(Queue::new()))
});

impl Driver for TimerDriver {
    #[cfg(not(any(feature = "time-driver-aot", feature = "time-driver-mtime")))]
    fn now(&self) -> u64 {
        loop {
            let hi = TIMER.timerawh().read();
            let lo = TIMER.timerawl().read();
            let hi2 = TIMER.timerawh().read();
            if hi == hi2 {
                return (hi as u64) << 32 | (lo as u64);
            }
        }
    }

    #[cfg(all(feature = "_rp235x", feature = "time-driver-aot"))]
    fn now(&self) -> u64 {
        use timer_aon::TICKS_PER_LPOSC_TICK;
        let now_lpo = loop {
            let hi = TIMER.read_time_upper().read();
            let lo = TIMER.read_time_lower().read();
            let hi2 = TIMER.read_time_upper().read();
            if hi == hi2 {
                break (hi as u64) << 32 | (lo as u64);
            }
        };
        now_lpo * TICKS_PER_LPOSC_TICK
    }

    #[cfg(all(feature = "_rp235x", feature = "time-driver-mtime"))]
    fn now(&self) -> u64 {
        let now_mtime = loop {
            let timehi = TIMER.mtimeh();
            let timelo = TIMER.mtime();
            let hi2 = timehi.read();
            let lo = timelo.read();
            let hi = timehi.read();
            if hi == hi2 {
                break (hi as u64) << 32 | (lo as u64);
            }
        };
        now_mtime
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
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

impl TimerDriver {
    #[cfg(not(any(feature = "time-driver-aot", feature = "time-driver-mtime")))]
    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let n = 0;
        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);

        // Arm it.
        // Note that we're not checking the high bits at all. This means the irq may fire early
        // if the alarm is more than 72 minutes (2^32 us) in the future. This is OK, since on irq fire
        // it is checked if the alarm time has passed.
        TIMER.alarm(n).write_value(timestamp as u32);

        let now = self.now();
        if timestamp <= now {
            // If alarm timestamp has passed the alarm will not fire.
            // Disarm the alarm and return `false` to indicate that.
            TIMER.armed().write(|w| w.set_armed(1 << n));

            alarm.timestamp.set(u64::MAX);

            false
        } else {
            true
        }
    }

    #[cfg(feature = "time-driver-mtime")]
    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);

        // Arm it.

        let mtime_cmp = TIMER.mtimecmp();
        let mtime_cmp_h = TIMER.mtimecmph();
        mtime_cmp.write_value(u32::MAX);
        mtime_cmp_h.write_value((timestamp >> 32) as u32);
        mtime_cmp.write_value(timestamp as u32);

        let now = self.now();
        if timestamp <= now {
            // If alarm timestamp has passed the alarm will not fire.
            // Disarm the alarm and return `false` to indicate that.

            alarm.timestamp.set(u64::MAX);

            false
        } else {
            // stays armed
            true
        }
    }

    #[cfg(not(any(feature = "time-driver-aot", feature = "time-driver-mtime")))]
    fn check_alarm(&self) {
        let n = 0;
        critical_section::with(|cs| {
            // clear the irq
            TIMER.intr().write(|w| w.set_alarm(n, true));

            let alarm = &self.alarms.borrow(cs);
            let timestamp = alarm.timestamp.get();
            if timestamp <= self.now() {
                self.trigger_alarm(cs)
            } else {
                // Not elapsed, arm it again.
                // This can happen if it was set more than 2^32 us in the future.
                TIMER.alarm(n).write_value(timestamp as u32);
            }
        });
    }

    #[cfg(feature = "time-driver-mtime")]
    fn check_alarm(&self) {
        critical_section::with(|cs| {
            // clear the irq
            let alarm = &self.alarms.borrow(cs);
            let timestamp = alarm.timestamp.get();
            if timestamp <= self.now() {
                self.trigger_alarm(cs)
            } else {
                // stays armed
            }
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self
            .queue
            .borrow(cs)
            .borrow_mut()
            .next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self
                .queue
                .borrow(cs)
                .borrow_mut()
                .next_expiration(self.now());
        }
    }
}

/// safety: must be called exactly once at bootup
pub unsafe fn init() {
    // init alarms
    #[cfg(all(feature = "_rp235x", feature = "time-driver-timer1"))]
    {
        let timer1_cycles = clocks::clk_ref_freq() / embassy_time_driver::TICK_HZ as u32;
        assert!(timer1_cycles < 512);
        assert!(timer1_cycles > 0);
        assert!(timer1_cycles * embassy_time_driver::TICK_HZ as u32 == clocks::clk_ref_freq());
        pac::TICKS.timer1_cycles().write(|w| w.0 = timer1_cycles);
        pac::TICKS.timer1_ctrl().write(|w| w.set_enable(true));
    };

    #[cfg(all(feature = "_rp235x", feature = "time-driver-aot"))]
    timer_aon::initialize_aon_timer();

    #[cfg(all(feature = "_rp235x", feature = "time-driver-mtime"))]
    {
        let timer_cycles = clocks::clk_ref_freq() / embassy_time_driver::TICK_HZ as u32;
        assert!(timer_cycles < 512);
        assert!(timer_cycles > 0);
        assert!(timer_cycles * embassy_time_driver::TICK_HZ as u32 == clocks::clk_ref_freq());
        pac::TICKS.riscv_cycles().write(|w| w.0 = timer_cycles);
        pac::TICKS.riscv_ctrl().write(|w| w.0 = 1);
        TIMER.mtime().write_value(0);
        TIMER.mtimeh().write_value(0);
    }

    critical_section::with(|cs| {
        let alarm = DRIVER.alarms.borrow(cs);
        alarm.timestamp.set(u64::MAX);
    });

    #[cfg(feature = "rp2040")]
    {
        // enable irq
        TIMER.inte().write(|w| {
            w.set_alarm(0, true);
        });
        interrupt::TIMER_IRQ_0.enable();
    }
    #[cfg(feature = "_rp235x")]
    {
        #[cfg(all(
            feature = "_rp235x",
            not(any(
                feature = "time-driver-timer1",
                feature = "time-driver-mtime",
                feature = "time-driver-aot"
            ))
        ))]
        {
            // enable irq
            TIMER.inte().write(|w| {
                w.set_alarm(0, true);
            });
            interrupt::TIMER0_IRQ_0.enable();
        }
        #[cfg(feature = "time-driver-timer1")]
        {
            // enable irq
            TIMER.inte().write(|w| {
                w.set_alarm(0, true);
            });
            interrupt::TIMER1_IRQ_0.enable();
        }
        #[cfg(feature = "time-driver-aot")]
        {
            use timer_aon::PowmanIntValue;
            let inte = TIMER.inte();
            let mut inte_value = inte.read();
            inte_value.set_timer(true);
            inte.write_value_key(inte_value);
            interrupt::POWMAN_IRQ_TIMER.enable();
        }
        #[cfg(feature = "time-driver-mtime")]
        {
            // enable irq
            interrupt::SIO_IRQ_MTIMECMP.enable();
        }
    }
}

#[cfg(all(feature = "rt", feature = "rp2040"))]
#[interrupt]
fn TIMER_IRQ_0() {
    DRIVER.check_alarm()
}

#[cfg(all(
    feature = "_rp235x",
    not(any(
        feature = "time-driver-timer1",
        feature = "time-driver-mtime",
        feature = "time-driver-aot"
    ))
))]
#[interrupt]
fn TIMER0_IRQ_0() {
    DRIVER.check_alarm()
}

#[cfg(all(feature = "rt", feature = "_rp235x", feature = "time-driver-timer1"))]
#[interrupt]
fn TIMER1_IRQ_0() {
    DRIVER.check_alarm()
}

#[cfg(all(feature = "rt", feature = "_rp235x", feature = "time-driver-aot"))]
#[interrupt]
fn POWMAN_IRQ_TIMER() {
    DRIVER.check_alarm()
}

#[cfg(all(feature = "_rp235x", feature = "time-driver-mtime"))]
#[interrupt]
fn SIO_IRQ_MTIMECMP() {
    DRIVER.check_alarm()
}

#[cfg(all(feature = "_rp235x", feature = "time-driver-aot"))]
mod timer_aon {
    use super::TIMER;
    use embassy_rp::pac::powman;
    use powman::regs;
    use regs::{AlarmTime15to0, AlarmTime31to16, AlarmTime47to32, AlarmTime63to48, Int, Timer};

    const POWMAN_KEY: u32 = 0x5afeu32 << 16;
    pub const LPOSC_TICK: u64 = 1000;
    pub const TICKS_PER_LPOSC_TICK: u64 = embassy_time_driver::TICK_HZ / LPOSC_TICK;

    /// [static_assert]: http://en.cppreference.com/w/cpp/language/static_assert
    /// macro taken from crate "static assertion" v1.1.0 (copied to avoid dependancy)
    macro_rules! const_assert {
        ($x:expr $(,)?) => {
            #[allow(unknown_lints, clippy::eq_op)]
            const _: [(); 0 - !{
                const ASSERT: bool = $x;
                ASSERT
            } as usize] = [];
        };
    }
    // The Embassy system tick needs to be an integer multiple of the Low Power oscillator tick
    const_assert!(TICKS_PER_LPOSC_TICK * LPOSC_TICK == embassy_time_driver::TICK_HZ);

    pub fn initialize_aon_timer() {
        let timer_reg = TIMER.timer();
        let mut reset_timer = regs::Timer(0);
        reset_timer.set_nonsec_write(true);
        timer_reg.write_value_key(reset_timer);

        let mut clear_timer = Timer(0);
        clear_timer.set_clear(true);
        clear_timer.set_nonsec_write(true);
        timer_reg.write_value_key(clear_timer);

        let mut timer_reg_value = timer_reg.read();
        timer_reg_value.set_use_lposc(true);
        timer_reg_value.set_nonsec_write(true);
        timer_reg.write_value_key(timer_reg_value);
        timer_reg_value.set_run(true);
        timer_reg_value.set_alarm_enab(true); // enable alarm
        timer_reg_value.set_pwrup_on_alarm(true);
        timer_reg.write_value_key(timer_reg_value);
    }

    use super::TimerDriver;
    use critical_section::CriticalSection;
    use embassy_time_driver::Driver;
    impl TimerDriver {
        pub fn set_aon_alarm_value(&self, timestamp_ticks: u64) {
            let timestamp_lposc_ticks = timestamp_ticks / TICKS_PER_LPOSC_TICK;

            let timer_reg = TIMER.timer();
            let mut timer_reg_value = timer_reg.read();
            let timer_reg_value_o = timer_reg_value;
            timer_reg_value.set_alarm_enab(false); // disable alarm
            timer_reg_value.set_nonsec_write(true);
            timer_reg.write_value_key(timer_reg_value);

            let timestamp_15to0 = ((timestamp_lposc_ticks >> 0) & 0xFFFF) as u16;
            let timestamp_31to16 = ((timestamp_lposc_ticks >> 16) & 0xFFFF) as u16;
            let timestamp_47to32 = ((timestamp_lposc_ticks >> 32) & 0xFFFF) as u16;
            let timestamp_64to48 = ((timestamp_lposc_ticks >> 48) & 0xFFFF) as u16;
            let mut a = AlarmTime15to0(POWMAN_KEY);
            a.set_alarm_time_15to0(timestamp_15to0);
            TIMER.alarm_time_15to0().write_value(a);
            let mut a = AlarmTime31to16(POWMAN_KEY);
            a.set_alarm_time_31to16(timestamp_31to16);
            TIMER.alarm_time_31to16().write_value(a);
            let mut a = AlarmTime47to32(POWMAN_KEY);
            a.set_alarm_time_47to32(timestamp_47to32);
            TIMER.alarm_time_47to32().write_value(a);
            let mut a = AlarmTime63to48(POWMAN_KEY);
            a.set_alarm_time_63to48(timestamp_64to48);
            TIMER.alarm_time_63to48().write_value(a);

            timer_reg.write_value_key(timer_reg_value_o);
        }

        pub fn check_alarm(&self) {
            critical_section::with(|cs| {
                let alarm = &self.alarms.borrow(cs);
                let timestamp = alarm.timestamp.get();
                if timestamp <= self.now() {
                    self.trigger_alarm(cs)
                } else {
                    // Not elapsed, arm it again.
                    self.set_aon_alarm_value(timestamp);
                }
            });

            // clear the irq
            let timer_reg = TIMER.timer();
            let mut timer_reg_value = timer_reg.read();
            timer_reg_value.set_alarm_enab(true); // enable alarm
            timer_reg_value.set_alarm(true); // reset alarm
            timer_reg_value.set_nonsec_write(true);
            timer_reg.write_value_key(timer_reg_value);

            let inte = TIMER.inte();
            let mut inte_value = inte.read();
            inte_value.set_timer(true);
            inte.write_value_key(inte_value);
        }

        pub fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
            // timestamp in µSec due to the definition of the constant TICK_HZ: u64 = 1_000_000; in ticks.rs

            let alarm = &self.alarms.borrow(cs);
            alarm.timestamp.set(timestamp);

            // Arm it.
            self.set_aon_alarm_value(timestamp);

            let now = self.now();
            if timestamp <= now {
                // If alarm timestamp has passed the alarm will not fire.
                let timer_reg = TIMER.timer();
                // Disarm the alarm and return `false` to indicate that.
                let mut timer_reg_value = timer_reg.read();
                timer_reg_value.set_alarm_enab(true); // enable alarm
                timer_reg_value.set_pwrup_on_alarm(true);
                timer_reg_value.set_alarm(true); // reset alarm (just in case)
                timer_reg_value.set_nonsec_write(true);
                timer_reg.write_value_key(timer_reg_value);
                alarm.timestamp.set(u64::MAX);
                false
            } else {
                true
            }
        }
    }

    pub trait PowmanTimerValue {
        fn write_value_key(&self, val: Timer);
    }

    impl<A> PowmanTimerValue for rp_pac::common::Reg<Timer, A>
    where
        A: rp_pac::common::Write,
    {
        #[inline(always)]
        fn write_value_key(&self, val: Timer) {
            let timer_val = rp_pac::powman::regs::Timer(val.0 | POWMAN_KEY);
            self.write_value(timer_val);
        }
    }

    pub trait PowmanIntValue {
        fn write_value_key(&self, val: Int);
    }

    impl<A> PowmanIntValue for rp_pac::common::Reg<Int, A>
    // where T: Copy, A: rp_pac::common::Write
    where
        A: rp_pac::common::Write,
    {
        #[inline(always)]
        fn write_value_key(&self, val: Int) {
            let int_val = rp_pac::powman::regs::Int(val.0 | POWMAN_KEY);
            self.write_value(int_val);
        }
    }
}
