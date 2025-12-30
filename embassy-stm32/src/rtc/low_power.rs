use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use embassy_time::{Duration, Instant, TICK_HZ};

use super::Rtc;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::rtc::vals::Wucksel;
use crate::peripherals::RTC;
use crate::rtc::{RtcTimeProvider, SealedInstance};

fn wucksel_compute_min(val: u32) -> (Wucksel, u32) {
    *[
        (Wucksel::DIV2, 2),
        (Wucksel::DIV4, 4),
        (Wucksel::DIV8, 8),
        (Wucksel::DIV16, 16),
    ]
    .iter()
    .find(|(_, psc)| *psc as u32 > val)
    .unwrap_or(&(Wucksel::DIV16, 16))
}

impl Rtc {
    pub(super) fn calc_epoch(&self) -> DateTime<Utc> {
        let now: NaiveDateTime = RtcTimeProvider::new().now().unwrap().into();

        now.and_utc() - TimeDelta::microseconds(Instant::now().as_micros().try_into().unwrap())
    }

    /// start the wakeup alarm and with a duration that is as close to but less than
    /// the requested duration, and record the instant the wakeup alarm was started
    pub(crate) fn start_wakeup_alarm(&mut self, requested_duration: embassy_time::Duration) {
        // Panic if the rcc mod knows we're not using low-power rtc
        #[cfg(any(rcc_wb, rcc_f4, rcc_f410))]
        unsafe { crate::rcc::get_freqs() }.rtc.to_hertz().unwrap();

        let requested_duration = requested_duration.as_ticks().clamp(0, u32::MAX as u64);
        let rtc_hz = Self::frequency().0 as u64;
        let rtc_ticks = requested_duration * rtc_hz / TICK_HZ;
        let (wucksel, prescaler) = wucksel_compute_min((rtc_ticks / u16::MAX as u64) as u32);

        // adjust the rtc ticks to the prescaler and subtract one rtc tick
        let rtc_ticks = rtc_ticks / prescaler as u64;
        let rtc_ticks = rtc_ticks.clamp(0, (u16::MAX - 1) as u64).saturating_sub(1) as u16;

        self.write(false, |regs| {
            regs.cr().modify(|w| w.set_wute(false));

            #[cfg(rtc_v2)]
            {
                regs.isr().modify(|w| w.set_wutf(false));
                while !regs.isr().read().wutwf() {}
            }

            #[cfg(rtc_v3)]
            {
                regs.scr().write(|w| w.set_cwutf(crate::pac::rtc::vals::Calrf::CLEAR));
                while !regs.icsr().read().wutwf() {}
            }

            regs.cr().modify(|w| w.set_wucksel(wucksel));
            regs.wutr().write(|w| w.set_wut(rtc_ticks));
            regs.cr().modify(|w| w.set_wute(true));
            regs.cr().modify(|w| w.set_wutie(true));
        });

        trace!(
            "rtc: start wakeup alarm for {} ms (psc: {}, ticks: {})",
            Duration::from_ticks(rtc_ticks as u64 * TICK_HZ * prescaler as u64 / rtc_hz).as_millis(),
            prescaler as u32,
            rtc_ticks,
        );
    }

    /// stop the wakeup alarm and return the time elapsed since `start_wakeup_alarm`
    /// was called, otherwise none
    pub(crate) fn stop_wakeup_alarm(&mut self) -> embassy_time::Instant {
        if RTC::regs().cr().read().wute() {
            trace!("rtc: stop wakeup alarm");

            self.write(false, |regs| {
                regs.cr().modify(|w| w.set_wutie(false));
                regs.cr().modify(|w| w.set_wute(false));

                #[cfg(rtc_v2)]
                regs.isr().modify(|w| w.set_wutf(false));
                #[cfg(rtc_v3)]
                regs.scr().write(|w| w.set_cwutf(crate::pac::rtc::vals::Calrf::CLEAR));

                // Check RM for EXTI and/or NVIC section, "Event event input mapping" or "EXTI interrupt/event mapping" or something similar,
                // there is a table for every "Event input" / "EXTI Line".
                // If you find the EXTI line related to "RTC wakeup" marks as "Configurable" (not "Direct"),
                // then write 1 to related field of Pending Register, to clean it's pending state.
                #[cfg(any(exti_v1, stm32h7, stm32wb))]
                crate::pac::EXTI
                    .pr(0)
                    .modify(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));

                <RTC as crate::rtc::SealedInstance>::WakeupInterrupt::unpend();
            });
        }

        let datetime: NaiveDateTime = RtcTimeProvider::new().now().expect("failed to read now").into();
        let offset = datetime.and_utc() - self.epoch;

        Instant::from_micros(offset.num_microseconds().unwrap().try_into().unwrap())
    }

    pub(super) fn enable_wakeup_line(&mut self) {
        <RTC as crate::rtc::SealedInstance>::WakeupInterrupt::unpend();
        unsafe { <RTC as crate::rtc::SealedInstance>::WakeupInterrupt::enable() };

        #[cfg(not(any(stm32u5, stm32u0, stm32wba)))]
        {
            use crate::pac::EXTI;
            EXTI.rtsr(0).modify(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));

            #[cfg(not(any(stm32wb, stm32wl5x)))]
            {
                EXTI.imr(0).modify(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));
            }
            #[cfg(any(stm32wb, stm32wl5x))]
            {
                EXTI.cpu(0).imr(0).modify(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));
            }
        }
        #[cfg(stm32u5)]
        {
            use crate::pac::RCC;
            RCC.srdamr().modify(|w| w.set_rtcapbamen(true));
            RCC.apb3smenr().modify(|w| w.set_rtcapbsmen(true));
        }
        #[cfg(stm32wba)]
        {
            use crate::pac::RCC;
            // RCC.srdamr().modify(|w| w.set_rtcapbamen(true));
            RCC.apb7smenr().modify(|w| w.set_rtcapbsmen(true));
        }
    }
}
