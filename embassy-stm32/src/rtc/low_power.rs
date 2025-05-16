use super::{bcd2_to_byte, DateTimeError, Rtc, RtcError};
use crate::peripherals::RTC;
use crate::rtc::SealedInstance;

/// Represents an instant in time that can be substracted to compute a duration
pub(super) struct RtcInstant {
    /// 0..59
    second: u8,
    /// 0..256
    subsecond: u16,
}

impl RtcInstant {
    #[cfg(not(rtc_v2f2))]
    const fn from(second: u8, subsecond: u16) -> Result<Self, DateTimeError> {
        if second > 59 {
            Err(DateTimeError::InvalidSecond)
        } else {
            Ok(Self { second, subsecond })
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for RtcInstant {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "{}:{}",
            self.second,
            RTC::regs().prer().read().prediv_s() - self.subsecond,
        )
    }
}

#[cfg(feature = "time")]
impl core::ops::Sub for RtcInstant {
    type Output = embassy_time::Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        use embassy_time::{Duration, TICK_HZ};

        let second = if self.second < rhs.second {
            self.second + 60
        } else {
            self.second
        };

        let psc = RTC::regs().prer().read().prediv_s() as u32;

        let self_ticks = second as u32 * (psc + 1) + (psc - self.subsecond as u32);
        let other_ticks = rhs.second as u32 * (psc + 1) + (psc - rhs.subsecond as u32);
        let rtc_ticks = self_ticks - other_ticks;

        Duration::from_ticks(((rtc_ticks * TICK_HZ as u32) / (psc + 1)) as u64)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum WakeupPrescaler {
    Div2 = 2,
    Div4 = 4,
    Div8 = 8,
    Div16 = 16,
}

#[cfg(any(
    stm32f4, stm32l0, stm32g4, stm32l4, stm32l5, stm32wb, stm32h5, stm32g0, stm32u5, stm32u0
))]
impl From<WakeupPrescaler> for crate::pac::rtc::vals::Wucksel {
    fn from(val: WakeupPrescaler) -> Self {
        use crate::pac::rtc::vals::Wucksel;

        match val {
            WakeupPrescaler::Div2 => Wucksel::DIV2,
            WakeupPrescaler::Div4 => Wucksel::DIV4,
            WakeupPrescaler::Div8 => Wucksel::DIV8,
            WakeupPrescaler::Div16 => Wucksel::DIV16,
        }
    }
}

#[cfg(any(
    stm32f4, stm32l0, stm32g4, stm32l4, stm32l5, stm32wb, stm32h5, stm32g0, stm32u5, stm32u0
))]
impl From<crate::pac::rtc::vals::Wucksel> for WakeupPrescaler {
    fn from(val: crate::pac::rtc::vals::Wucksel) -> Self {
        use crate::pac::rtc::vals::Wucksel;

        match val {
            Wucksel::DIV2 => WakeupPrescaler::Div2,
            Wucksel::DIV4 => WakeupPrescaler::Div4,
            Wucksel::DIV8 => WakeupPrescaler::Div8,
            Wucksel::DIV16 => WakeupPrescaler::Div16,
            _ => unreachable!(),
        }
    }
}

impl WakeupPrescaler {
    pub fn compute_min(val: u32) -> Self {
        *[
            WakeupPrescaler::Div2,
            WakeupPrescaler::Div4,
            WakeupPrescaler::Div8,
            WakeupPrescaler::Div16,
        ]
        .iter()
        .find(|psc| **psc as u32 > val)
        .unwrap_or(&WakeupPrescaler::Div16)
    }
}

impl Rtc {
    /// Return the current instant.
    fn instant(&self) -> Result<RtcInstant, RtcError> {
        self.time_provider().read(|_, tr, ss| {
            let second = bcd2_to_byte((tr.st(), tr.su()));

            RtcInstant::from(second, ss).map_err(RtcError::InvalidDateTime)
        })
    }

    /// start the wakeup alarm and with a duration that is as close to but less than
    /// the requested duration, and record the instant the wakeup alarm was started
    pub(crate) fn start_wakeup_alarm(
        &self,
        requested_duration: embassy_time::Duration,
        cs: critical_section::CriticalSection,
    ) {
        use embassy_time::{Duration, TICK_HZ};

        #[cfg(any(rtc_v3, rtc_v3u5, rtc_v3l5))]
        use crate::pac::rtc::vals::Calrf;

        // Panic if the rcc mod knows we're not using low-power rtc
        #[cfg(any(rcc_wb, rcc_f4, rcc_f410))]
        unsafe { crate::rcc::get_freqs() }.rtc.to_hertz().unwrap();

        let requested_duration = requested_duration.as_ticks().clamp(0, u32::MAX as u64);
        let rtc_hz = Self::frequency().0 as u64;
        let rtc_ticks = requested_duration * rtc_hz / TICK_HZ;
        let prescaler = WakeupPrescaler::compute_min((rtc_ticks / u16::MAX as u64) as u32);

        // adjust the rtc ticks to the prescaler and subtract one rtc tick
        let rtc_ticks = rtc_ticks / prescaler as u64;
        let rtc_ticks = rtc_ticks.clamp(0, (u16::MAX - 1) as u64).saturating_sub(1) as u16;

        self.write(false, |regs| {
            regs.cr().modify(|w| w.set_wute(false));

            #[cfg(any(
                rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
            ))]
            {
                regs.isr().modify(|w| w.set_wutf(false));
                while !regs.isr().read().wutwf() {}
            }

            #[cfg(any(rtc_v3, rtc_v3u5, rtc_v3l5))]
            {
                regs.scr().write(|w| w.set_cwutf(Calrf::CLEAR));
                while !regs.icsr().read().wutwf() {}
            }

            regs.cr().modify(|w| w.set_wucksel(prescaler.into()));
            regs.wutr().write(|w| w.set_wut(rtc_ticks));
            regs.cr().modify(|w| w.set_wute(true));
            regs.cr().modify(|w| w.set_wutie(true));
        });

        let instant = self.instant().unwrap();
        trace!(
            "rtc: start wakeup alarm for {} ms (psc: {}, ticks: {}) at {}",
            Duration::from_ticks(rtc_ticks as u64 * TICK_HZ * prescaler as u64 / rtc_hz).as_millis(),
            prescaler as u32,
            rtc_ticks,
            instant,
        );

        assert!(self.stop_time.borrow(cs).replace(Some(instant)).is_none())
    }

    /// stop the wakeup alarm and return the time elapsed since `start_wakeup_alarm`
    /// was called, otherwise none
    pub(crate) fn stop_wakeup_alarm(&self, cs: critical_section::CriticalSection) -> Option<embassy_time::Duration> {
        use crate::interrupt::typelevel::Interrupt;
        #[cfg(any(rtc_v3, rtc_v3u5, rtc_v3l5))]
        use crate::pac::rtc::vals::Calrf;

        let instant = self.instant().unwrap();
        if RTC::regs().cr().read().wute() {
            trace!("rtc: stop wakeup alarm at {}", instant);

            self.write(false, |regs| {
                regs.cr().modify(|w| w.set_wutie(false));
                regs.cr().modify(|w| w.set_wute(false));

                #[cfg(any(
                    rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
                ))]
                regs.isr().modify(|w| w.set_wutf(false));

                #[cfg(any(rtc_v3, rtc_v3u5, rtc_v3l5))]
                regs.scr().write(|w| w.set_cwutf(Calrf::CLEAR));

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

        self.stop_time.borrow(cs).take().map(|stop_time| instant - stop_time)
    }

    pub(crate) fn enable_wakeup_line(&self) {
        use crate::interrupt::typelevel::Interrupt;

        <RTC as crate::rtc::SealedInstance>::WakeupInterrupt::unpend();
        unsafe { <RTC as crate::rtc::SealedInstance>::WakeupInterrupt::enable() };

        #[cfg(not(any(stm32u5, stm32u0)))]
        {
            use crate::pac::EXTI;
            EXTI.rtsr(0).modify(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));

            #[cfg(not(stm32wb))]
            {
                EXTI.imr(0).modify(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));
            }
            #[cfg(stm32wb)]
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
    }
}
