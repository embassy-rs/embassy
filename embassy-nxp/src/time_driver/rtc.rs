use core::cell::{Cell, RefCell};
use core::task::Waker;

use critical_section::CriticalSection;
use embassy_hal_internal::interrupt::{InterruptExt, Priority};
use embassy_sync::blocking_mutex::CriticalSectionMutex as Mutex;
use embassy_time_driver::{Driver, time_driver_impl};
use embassy_time_queue_utils::Queue;

use crate::pac::{PMC, RTC, SYSCON, interrupt, pmc, rtc};

struct AlarmState {
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

pub struct RtcDriver {
    alarms: Mutex<AlarmState>,
    queue: Mutex<RefCell<Queue>>,
}

time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    alarms: Mutex::new(AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
});
impl RtcDriver {
    fn init(&'static self) {
        let syscon = SYSCON;
        let pmc = PMC;
        let rtc = RTC;

        syscon.ahbclkctrl0().modify(|w| w.set_rtc(true));

        // By default the RTC enters software reset. If for some reason it is
        // not in reset, we enter and them promptly leave.q
        rtc.ctrl().modify(|w| w.set_swreset(true));
        rtc.ctrl().modify(|w| w.set_swreset(false));

        // Select clock source - either XTAL or FRO
        // pmc.rtcosc32k().write(|w| w.set_sel(pmc::vals::Sel::XTAL32K));
        pmc.rtcosc32k().write(|w| w.set_sel(pmc::vals::Sel::FRO32K));

        // Start the RTC peripheral
        rtc.ctrl().modify(|w| w.set_rtc_osc_pd(rtc::vals::RtcOscPd::POWER_UP));

        //reset/clear(?) counter
        rtc.count().modify(|w| w.set_val(0));
        //en rtc main counter
        rtc.ctrl().modify(|w| w.set_rtc_en(true));
        rtc.ctrl().modify(|w| w.set_rtc1khz_en(true));
        // subsec counter enable
        rtc.ctrl()
            .modify(|w| w.set_rtc_subsec_ena(rtc::vals::RtcSubsecEna::POWER_UP));

        // enable irq
        unsafe {
            interrupt::RTC.set_priority(Priority::from(3));
            interrupt::RTC.enable();
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let rtc = RTC;
        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);
        let now = self.now();

        if timestamp <= now {
            alarm.timestamp.set(u64::MAX);
            return false;
        }

        //time diff in sub-sec not ticks (32kHz)
        let diff = timestamp - now;
        let sec = (diff / 32768) as u32;
        let subsec = (diff % 32768) as u32;

        let current_sec = rtc.count().read().val();
        let target_sec = current_sec.wrapping_add(sec as u32);

        rtc.match_().write(|w| w.set_matval(target_sec));
        rtc.wake().write(|w| {
            let ms = (subsec * 1000) / 32768;
            w.set_val(ms as u16)
        });

        if subsec > 0 {
            let ms = (subsec * 1000) / 32768;
            rtc.wake().write(|w| w.set_val(ms as u16));
        }

        rtc.ctrl().modify(|w| {
            w.set_alarm1hz(false);
            w.set_wake1khz(rtc::vals::Wake1khz::RUN)
        });
        true
    }

    fn on_interrupt(&self) {
        critical_section::with(|cs| {
            let rtc = RTC;
            let flags = rtc.ctrl().read();
            if flags.alarm1hz() == false {
                rtc.ctrl().modify(|w| w.set_alarm1hz(true));
                self.trigger_alarm(cs);
            }

            if flags.wake1khz() == rtc::vals::Wake1khz::RUN {
                rtc.ctrl().modify(|w| w.set_wake1khz(rtc::vals::Wake1khz::TIMEOUT));
                self.trigger_alarm(cs);
            }
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(u64::MAX);
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        if next == u64::MAX {
            // no scheduled events, skipping
            return;
        }
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
            if next == u64::MAX {
                //no next event found after retry
                return;
            }
        }
    }
}

impl Driver for RtcDriver {
    fn now(&self) -> u64 {
        let rtc = RTC;

        loop {
            let sec1 = rtc.count().read().val() as u64;
            let sub1 = rtc.subsec().read().subsec() as u64;
            let sec2 = rtc.count().read().val() as u64;
            let sub2 = rtc.subsec().read().subsec() as u64;

            if sec1 == sec2 && sub1 == sub2 {
                return sec1 * 32768 + sub1;
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
#[interrupt]
fn RTC() {
    DRIVER.on_interrupt();
}

pub fn init() {
    DRIVER.init();
}
