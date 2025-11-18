//! RTC DateTime driver.
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_hal_internal::{Peri, PeripheralType};

use crate::clocks::with_clocks;
use crate::pac;
use crate::pac::rtc0::cr::Um;

type Regs = pac::rtc0::RegisterBlock;

static ALARM_TRIGGERED: AtomicBool = AtomicBool::new(false);

// Token-based instance pattern like embassy-imxrt
pub trait Instance: PeripheralType {
    fn ptr() -> *const Regs;
}

/// Token for RTC0
pub type Rtc0 = crate::peripherals::RTC0;
impl Instance for crate::peripherals::RTC0 {
    #[inline(always)]
    fn ptr() -> *const Regs {
        pac::Rtc0::ptr()
    }
}

const DAYS_IN_A_YEAR: u32 = 365;
const SECONDS_IN_A_DAY: u32 = 86400;
const SECONDS_IN_A_HOUR: u32 = 3600;
const SECONDS_IN_A_MINUTE: u32 = 60;
const YEAR_RANGE_START: u16 = 1970;

#[derive(Debug, Clone, Copy)]
pub struct RtcDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
#[derive(Copy, Clone)]
pub struct RtcConfig {
    #[allow(dead_code)]
    wakeup_select: bool,
    update_mode: Um,
    #[allow(dead_code)]
    supervisor_access: bool,
    compensation_interval: u8,
    compensation_time: u8,
}

#[derive(Copy, Clone)]
pub struct RtcInterruptEnable;
impl RtcInterruptEnable {
    pub const RTC_TIME_INVALID_INTERRUPT_ENABLE: u32 = 1 << 0;
    pub const RTC_TIME_OVERFLOW_INTERRUPT_ENABLE: u32 = 1 << 1;
    pub const RTC_ALARM_INTERRUPT_ENABLE: u32 = 1 << 2;
    pub const RTC_SECONDS_INTERRUPT_ENABLE: u32 = 1 << 4;
}

pub fn convert_datetime_to_seconds(datetime: &RtcDateTime) -> u32 {
    let month_days: [u16; 13] = [0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

    let mut seconds = (datetime.year as u32 - 1970) * DAYS_IN_A_YEAR;
    seconds += (datetime.year as u32 / 4) - (1970 / 4);
    seconds += month_days[datetime.month as usize] as u32;
    seconds += datetime.day as u32 - 1;

    if (datetime.year & 3 == 0) && (datetime.month <= 2) {
        seconds -= 1;
    }

    seconds = seconds * SECONDS_IN_A_DAY
        + (datetime.hour as u32 * SECONDS_IN_A_HOUR)
        + (datetime.minute as u32 * SECONDS_IN_A_MINUTE)
        + datetime.second as u32;

    seconds
}

pub fn convert_seconds_to_datetime(seconds: u32) -> RtcDateTime {
    let mut seconds_remaining = seconds;
    let mut days = seconds_remaining / SECONDS_IN_A_DAY + 1;
    seconds_remaining %= SECONDS_IN_A_DAY;

    let hour = (seconds_remaining / SECONDS_IN_A_HOUR) as u8;
    seconds_remaining %= SECONDS_IN_A_HOUR;
    let minute = (seconds_remaining / SECONDS_IN_A_MINUTE) as u8;
    let second = (seconds_remaining % SECONDS_IN_A_MINUTE) as u8;

    let mut year = YEAR_RANGE_START;
    let mut days_in_year = DAYS_IN_A_YEAR;

    while days > days_in_year {
        days -= days_in_year;
        year += 1;

        days_in_year = if year.is_multiple_of(4) {
            DAYS_IN_A_YEAR + 1
        } else {
            DAYS_IN_A_YEAR
        };
    }

    let days_per_month = [
        31,
        if year.is_multiple_of(4) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 1;
    for (m, month_days) in days_per_month.iter().enumerate() {
        let m = m + 1;
        if days <= *month_days as u32 {
            month = m;
            break;
        } else {
            days -= *month_days as u32;
        }
    }

    let day = days as u8;

    RtcDateTime {
        year,
        month: month as u8,
        day,
        hour,
        minute,
        second,
    }
}

pub fn get_default_config() -> RtcConfig {
    RtcConfig {
        wakeup_select: false,
        update_mode: Um::Um0,
        supervisor_access: false,
        compensation_interval: 0,
        compensation_time: 0,
    }
}
/// Minimal RTC handle for a specific instance I (store the zero-sized token like embassy)
pub struct Rtc<'a, I: Instance> {
    _inst: core::marker::PhantomData<&'a mut I>,
}

impl<'a, I: Instance> Rtc<'a, I> {
    /// initialize RTC
    pub fn new(_inst: Peri<'a, I>, config: RtcConfig) -> Self {
        let rtc = unsafe { &*I::ptr() };

        // The RTC is NOT gated by the MRCC, but we DO need to make sure the 16k clock
        // on the vsys domain is active
        let clocks = with_clocks(|c| c.clk_16k_vsys.clone());
        match clocks {
            None => panic!("Clocks have not been initialized"),
            Some(None) => panic!("Clocks initialized, but clk_16k_vsys not active"),
            Some(Some(_)) => {}
        }

        /* RTC reset */
        rtc.cr().modify(|_, w| w.swr().set_bit());
        rtc.cr().modify(|_, w| w.swr().clear_bit());
        rtc.tsr().write(|w| unsafe { w.bits(1) });

        rtc.cr().modify(|_, w| w.um().variant(config.update_mode));

        rtc.tcr().modify(|_, w| unsafe {
            w.cir()
                .bits(config.compensation_interval)
                .tcr()
                .bits(config.compensation_time)
        });

        Self {
            _inst: core::marker::PhantomData,
        }
    }

    pub fn set_datetime(&self, datetime: RtcDateTime) {
        let rtc = unsafe { &*I::ptr() };
        let seconds = convert_datetime_to_seconds(&datetime);
        rtc.tsr().write(|w| unsafe { w.bits(seconds) });
    }

    pub fn get_datetime(&self) -> RtcDateTime {
        let rtc = unsafe { &*I::ptr() };
        let seconds = rtc.tsr().read().bits();
        convert_seconds_to_datetime(seconds)
    }

    pub fn set_alarm(&self, alarm: RtcDateTime) {
        let rtc = unsafe { &*I::ptr() };
        let seconds = convert_datetime_to_seconds(&alarm);

        rtc.tar().write(|w| unsafe { w.bits(0) });
        let mut timeout = 10000;
        while rtc.tar().read().bits() != 0 && timeout > 0 {
            timeout -= 1;
        }

        rtc.tar().write(|w| unsafe { w.bits(seconds) });

        let mut timeout = 10000;
        while rtc.tar().read().bits() != seconds && timeout > 0 {
            timeout -= 1;
        }
    }

    pub fn get_alarm(&self) -> RtcDateTime {
        let rtc = unsafe { &*I::ptr() };
        let alarm_seconds = rtc.tar().read().bits();
        convert_seconds_to_datetime(alarm_seconds)
    }

    pub fn start(&self) {
        let rtc = unsafe { &*I::ptr() };
        rtc.sr().modify(|_, w| w.tce().set_bit());
    }

    pub fn stop(&self) {
        let rtc = unsafe { &*I::ptr() };
        rtc.sr().modify(|_, w| w.tce().clear_bit());
    }

    pub fn set_interrupt(&self, mask: u32) {
        let rtc = unsafe { &*I::ptr() };

        if (RtcInterruptEnable::RTC_TIME_INVALID_INTERRUPT_ENABLE & mask) != 0 {
            rtc.ier().modify(|_, w| w.tiie().tiie_1());
        }
        if (RtcInterruptEnable::RTC_TIME_OVERFLOW_INTERRUPT_ENABLE & mask) != 0 {
            rtc.ier().modify(|_, w| w.toie().toie_1());
        }
        if (RtcInterruptEnable::RTC_ALARM_INTERRUPT_ENABLE & mask) != 0 {
            rtc.ier().modify(|_, w| w.taie().taie_1());
        }
        if (RtcInterruptEnable::RTC_SECONDS_INTERRUPT_ENABLE & mask) != 0 {
            rtc.ier().modify(|_, w| w.tsie().tsie_1());
        }

        ALARM_TRIGGERED.store(false, Ordering::SeqCst);
    }

    pub fn disable_interrupt(&self, mask: u32) {
        let rtc = unsafe { &*I::ptr() };

        if (RtcInterruptEnable::RTC_TIME_INVALID_INTERRUPT_ENABLE & mask) != 0 {
            rtc.ier().modify(|_, w| w.tiie().tiie_0());
        }
        if (RtcInterruptEnable::RTC_TIME_OVERFLOW_INTERRUPT_ENABLE & mask) != 0 {
            rtc.ier().modify(|_, w| w.toie().toie_0());
        }
        if (RtcInterruptEnable::RTC_ALARM_INTERRUPT_ENABLE & mask) != 0 {
            rtc.ier().modify(|_, w| w.taie().taie_0());
        }
        if (RtcInterruptEnable::RTC_SECONDS_INTERRUPT_ENABLE & mask) != 0 {
            rtc.ier().modify(|_, w| w.tsie().tsie_0());
        }
    }

    pub fn clear_alarm_flag(&self) {
        let rtc = unsafe { &*I::ptr() };
        rtc.ier().modify(|_, w| w.taie().clear_bit());
    }

    pub fn is_alarm_triggered(&self) -> bool {
        ALARM_TRIGGERED.load(Ordering::Relaxed)
    }
}

pub fn on_interrupt() {
    let rtc = unsafe { &*pac::Rtc0::ptr() };
    // Check if this is actually a time alarm interrupt
    let sr = rtc.sr().read();
    if sr.taf().bit_is_set() {
        rtc.ier().modify(|_, w| w.taie().clear_bit());
        ALARM_TRIGGERED.store(true, Ordering::SeqCst);
    }
}

pub struct RtcHandler;
impl crate::interrupt::typelevel::Handler<crate::interrupt::typelevel::RTC> for RtcHandler {
    unsafe fn on_interrupt() {
        on_interrupt();
    }
}
