use core::cell::{Cell, RefCell};
use core::task::Waker;

use critical_section::CriticalSection;
use embassy_hal_internal::interrupt::{InterruptExt, Priority};
use embassy_sync::blocking_mutex::CriticalSectionMutex as Mutex;
use embassy_time_driver::{time_driver_impl, Driver};
use embassy_time_queue_utils::Queue;
use lpc55_pac::{interrupt, PMC, RTC, SYSCON};
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
        let syscon = unsafe { &*SYSCON::ptr() };
        let pmc = unsafe { &*PMC::ptr() };
        let rtc = unsafe { &*RTC::ptr() };

        syscon.ahbclkctrl0.modify(|_, w| w.rtc().enable());

        // By default the RTC enters software reset. If for some reason it is
        // not in reset, we enter and them promptly leave.q
        rtc.ctrl.modify(|_, w| w.swreset().set_bit());
        rtc.ctrl.modify(|_, w| w.swreset().clear_bit());

        // Select clock source - either XTAL or FRO
        // pmc.rtcosc32k.write(|w| w.sel().xtal32k());
        pmc.rtcosc32k.write(|w| w.sel().fro32k());

        // Start the RTC peripheral
        rtc.ctrl.modify(|_, w| w.rtc_osc_pd().power_up());

        // rtc.ctrl.modify(|_, w| w.rtc_en().clear_bit()); // EXTRA

        //reset/clear(?) counter
        rtc.count.reset();
        //en rtc main counter
        rtc.ctrl.modify(|_, w| w.rtc_en().set_bit());
        rtc.ctrl.modify(|_, w| w.rtc1khz_en().set_bit());
        // subsec counter enable
        rtc.ctrl.modify(|_, w| w.rtc_subsec_ena().set_bit());

        // enable irq
        unsafe {
            interrupt::RTC.set_priority(Priority::from(3));
            interrupt::RTC.enable();
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let rtc = unsafe { &*RTC::ptr() };
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

        let current_sec = rtc.count.read().val().bits();
        let target_sec = current_sec.wrapping_add(sec as u32);

        rtc.match_.write(|w| unsafe { w.matval().bits(target_sec) });
        rtc.wake.write(|w| unsafe {
            let ms = (subsec * 1000) / 32768;
            w.val().bits(ms as u16)
        });
        if subsec > 0 {
            let ms = (subsec * 1000) / 32768;
            rtc.wake.write(|w| unsafe { w.val().bits(ms as u16) });
        }
        rtc.ctrl.modify(|_, w| w.alarm1hz().clear_bit().wake1khz().clear_bit());
        true
    }

    fn on_interrupt(&self) {
        critical_section::with(|cs| {
            let rtc = unsafe { &*RTC::ptr() };
            let flags = rtc.ctrl.read();
            if flags.alarm1hz().bit_is_clear() {
                rtc.ctrl.modify(|_, w| w.alarm1hz().set_bit());
                self.trigger_alarm(cs);
            }

            if flags.wake1khz().bit_is_clear() {
                rtc.ctrl.modify(|_, w| w.wake1khz().set_bit());
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
        let rtc = unsafe { &*RTC::ptr() };

        loop {
            let sec1 = rtc.count.read().val().bits() as u64;
            let sub1 = rtc.subsec.read().subsec().bits() as u64;
            let sec2 = rtc.count.read().val().bits() as u64;
            let sub2 = rtc.subsec.read().subsec().bits() as u64;

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
#[cortex_m_rt::interrupt]
fn RTC() {
    DRIVER.on_interrupt();
}

pub fn init() {
    DRIVER.init();
}
