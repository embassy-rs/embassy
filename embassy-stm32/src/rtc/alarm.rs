use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use embassy_sync::waitqueue::AtomicWaker;
use stm32_metapac::rtc::regs::{Alrmr, Alrmssr};
#[cfg(rtc_v3)]
use stm32_metapac::rtc::vals::{Alrmf, Calrf};
use stm32_metapac::rtc::vals::{AlrmrMsk, AlrmrPm, AlrmrWdsel};

use crate::interrupt;
use crate::peripherals::RTC;
use crate::rtc::SealedInstance;

use super::datetime::day_of_week_to_u8;
use super::{byte_to_bcd2, DayOfWeek, Rtc};

cfg_if::cfg_if!(
    if #[cfg(rtc_v2f2)] {
        const ALARM_COUNT: usize = 1;
    } else {
        const ALARM_COUNT: usize = 2;
    }
);

static RTC_WAKERS: [AtomicWaker; ALARM_COUNT] = [const { AtomicWaker::new() }; ALARM_COUNT];

/// RTC alarm index
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Alarm {
    /// Alarm A
    A = 0,
    // stm32wb0 also doesn't have alarm B?
    #[cfg(not(any(rtc_v2f2)))]
    /// Alarm B
    B = 1,
}

impl Alarm {
    /// Get alarm index (0..1)
    pub fn index(&self) -> usize {
        *self as usize
    }
}

/// Kind of date used in alarm match.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AlarmDate {
    /// Match on day of month.
    DayOfMonth(u8),
    /// Match on weekday.
    DayOfWeek(DayOfWeek),
}

/// RTC alarm specification. Alarm is fired when a match occurs.
#[derive(Clone, Debug, PartialEq)]
pub struct RtcAlarmMatch {
    /// If not [`None`], match subsecond to this value. If [`None`], alarm matches when second is incremented.
    pub subsecond: Option<u16>,
    /// If not [`None`], match second to this value.
    pub second: Option<u8>,
    /// If not [`None`], match minute to this value.
    pub minute: Option<u8>,
    /// If not [`None`], match hour to this value.
    pub hour: Option<u8>,
    /// Set to `true` when value from [`hour`] field is PM, if AM or 24h format set to `false`.
    pub hour_is_pm: bool,
    /// If not [`None`], match date to this value.
    pub date: Option<AlarmDate>,
}

impl RtcAlarmMatch {
    /// Creates an alarm that fires everytime the second in incremented
    pub fn every_second() -> Self {
        Self {
            subsecond: None,
            second: None,
            minute: None,
            hour: None,
            hour_is_pm: false,
            date: None,
        }
    }

    pub(crate) fn to_alrmr(&self) -> Alrmr {
        let mut alrmr = Alrmr::default();

        if let Some(second) = self.second {
            alrmr.set_msk1(AlrmrMsk::TO_MATCH);
            let (st, su) = byte_to_bcd2(second);
            alrmr.set_st(st);
            alrmr.set_su(su);
        } else {
            alrmr.set_msk1(AlrmrMsk::NOT_MATCH);
        }

        if let Some(minute) = self.minute {
            alrmr.set_msk2(AlrmrMsk::TO_MATCH);
            let (mnt, mnu) = byte_to_bcd2(minute);
            alrmr.set_mnt(mnt);
            alrmr.set_mnu(mnu);
        } else {
            alrmr.set_msk2(AlrmrMsk::NOT_MATCH);
        }

        if let Some(hour) = self.hour {
            alrmr.set_msk3(AlrmrMsk::TO_MATCH);
            if self.hour_is_pm {
                alrmr.set_pm(AlrmrPm::PM);
            } else {
                alrmr.set_pm(AlrmrPm::AM);
            }
            let (ht, hu) = byte_to_bcd2(hour);
            alrmr.set_ht(ht);
            alrmr.set_hu(hu);
        } else {
            alrmr.set_msk3(AlrmrMsk::NOT_MATCH);
        }

        if let Some(date) = self.date {
            alrmr.set_msk4(AlrmrMsk::TO_MATCH);

            let (date, wdsel) = match date {
                AlarmDate::DayOfMonth(date) => (date, AlrmrWdsel::DATE_UNITS),
                AlarmDate::DayOfWeek(day_of_week) => (day_of_week_to_u8(day_of_week), AlrmrWdsel::WEEK_DAY),
            };
            alrmr.set_wdsel(wdsel);
            let (dt, du) = byte_to_bcd2(date);
            alrmr.set_dt(dt);
            alrmr.set_du(du);
        } else {
            alrmr.set_msk4(AlrmrMsk::NOT_MATCH);
        }

        alrmr
    }

    pub(crate) fn to_alrmssr(&self) -> Alrmssr {
        let mut alrmssr = Alrmssr::default();

        if let Some(subsecond) = self.subsecond {
            alrmssr.set_maskss(0b1111); // only implement matching all bits
            alrmssr.set_ss(subsecond);
        } else {
            alrmssr.set_maskss(0);
        }

        alrmssr
    }
}

impl Rtc {
    /// Set alarm spec for specified alarm.
    pub fn set_alarm(&mut self, alarm: Alarm, spec: RtcAlarmMatch) {
        let alrmr = spec.to_alrmr();
        let alrmrss = spec.to_alrmssr();

        self.write(false, |r| {
            // disable alarm
            r.cr().modify(|w| w.set_alre(alarm.index(), false));

            // wait until update is allowed
            #[cfg(rtc_v3)]
            while !r.icsr().read().alrwf(alarm.index()) {}

            #[cfg(any(
                rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
            ))]
            while !r.isr().read().alrwf(alarm.index()) {}

            r.alrmr(alarm.index()).write_value(alrmr);
            r.alrmssr(alarm.index()).write_value(alrmrss);
        });
    }

    /// Wait until the specified alarm fires.
    pub async fn wait_for_alarm(&mut self, alarm: Alarm) {
        RtcAlarmFuture::new(alarm).await
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct RtcAlarmFuture {
    alarm: Alarm,
}

impl RtcAlarmFuture {
    fn new(alarm: Alarm) -> Self {
        critical_section::with(|_| {
            RTC::write(false, |rtc| {
                // enable interrupt
                rtc.cr().modify(|w| {
                    w.set_alre(alarm.index(), true);
                    w.set_alrie(alarm.index(), true);
                });

                // clear pending bit
                #[cfg(rtc_v3)]
                rtc.scr().write(|w| w.set_calrf(alarm.index(), Calrf::CLEAR));

                #[cfg(any(
                    rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
                ))]
                rtc.isr().modify(|w| w.set_alrf(alarm.index(), false))
            });

            use crate::pac::EXTI;
            EXTI.rtsr(0).modify(|w| w.set_line(RTC::EXTI_ALARM_LINE, true));
            EXTI.imr(0).modify(|w| w.set_line(RTC::EXTI_ALARM_LINE, true));
        });

        Self { alarm }
    }
}

impl Drop for RtcAlarmFuture {
    fn drop(&mut self) {
        // clear interrupt
        critical_section::with(|_| {
            RTC::write(false, |rtc| {
                rtc.cr().modify(|w| {
                    w.set_alrie(self.alarm.index(), false);
                });
            });
        })
    }
}

impl Future for RtcAlarmFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        RTC_WAKERS[self.alarm as usize].register(cx.waker());
        defmt::info!("poll rtc future");

        if !crate::pac::RTC.cr().read().alrie(self.alarm.index()) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

unsafe fn on_irq() {
    #[cfg(feature = "low-power")]
    crate::low_power::on_wakeup_irq();

    let reg = crate::pac::RTC;
    #[cfg(rtc_v3)]
    let misr = reg.misr().read();

    #[cfg(any(
        rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
    ))]
    let isr = reg.isr().read();

    // defmt::info!("RTC IRQ (misr = {})", misr.0);

    for i in 0..ALARM_COUNT {
        #[cfg(rtc_v3)]
        let has_fired = misr.alrmf(i) == Alrmf::MATCH;

        #[cfg(any(
            rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
        ))]
        let has_fired = isr.alrf(i);

        if has_fired {
            // clear pending bit
            #[cfg(rtc_v3)]
            reg.scr().write(|w| w.set_calrf(i, Calrf::CLEAR));

            #[cfg(any(
                rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
            ))]
            reg.isr().modify(|w| w.set_alrf(i, false));

            RTC::write(false, |regs| {
                // disable the interrupt, this way the future knows the irq fired
                regs.cr().modify(|w| w.set_alrie(i, false));
            });

            // wake task
            RTC_WAKERS[i].wake();
        }
    }

    // the RTC exti line is configurable on some variants, other do not need
    // the pending bit reset

    #[cfg(not(any(exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_h5, exti_h50)))]
    crate::pac::EXTI.pr(0).write(|w| w.set_line(RTC::EXTI_ALARM_LINE, true));

    #[cfg(any(exti_c0, exti_u0, exti_l5, exti_u5, exti_h5, exti_h50))]
    crate::pac::EXTI
        .rpr(0)
        .write(|w| w.set_line(RTC::EXTI_ALARM_LINE, true));
}

// TODO figure out IRQs for all variants..

#[cfg(any(stm32f0, stm32g0, stm32l0, stm32u0, stm32u5))]
foreach_interrupt! {
    (RTC, rtc, $block:ident, TAMP, $irq:ident) => {
        #[interrupt]
        #[allow(non_snake_case)]
        unsafe fn $irq() {
            on_irq();
        }
    };
}

#[cfg(any(stm32f1, stm32f3, stm32f4, stm32g4))]
foreach_interrupt! {
    (RTC, rtc, $block:ident, ALARM, $irq:ident) => {
        #[interrupt]
        #[allow(non_snake_case)]
        unsafe fn $irq() {
            on_irq();
        }
    };
}
