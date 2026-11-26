use core::cell::Cell;

use critical_section::{CriticalSection, Mutex};
use embassy_time::{Duration, Instant, TICK_HZ};

use super::{Rtc, RtcReader, SealedInstance, bcd2_to_byte};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::rtc::vals::Wucksel;
use crate::peripherals::RTC;

const SECONDS_PER_DAY: u32 = 86_400;

/// The longest wakeup alarm `start_wakeup_alarm` programs: `u16::MAX` periods of the 1 Hz `ck_spre`.
const MAX_STOP_SECONDS: u32 = u16::MAX as u32;

/// Age in seconds after which the anchor is moved to the current time when entering stop mode.
///
/// The time elapsed since the anchor is measured from the RTC time of day, so it is only
/// known modulo a day. Moving the anchor at least this often keeps the span from the anchor
/// to the next wakeup below a day, with an hour of margin for the time driver clock drifting
/// while it measures the anchor's age.
const MAX_ANCHOR_AGE_SECONDS: u32 = SECONDS_PER_DAY - MAX_STOP_SECONDS - 3_600;

/// A point in time known both as an RTC time of day and as an embassy-time instant.
#[derive(Clone, Copy)]
struct Anchor {
    /// Time of day, in RTC subsecond ticks.
    time_of_day: u32,
    instant: Instant,
}

/// The number of RTC subsecond ticks per second, `PREDIV_S + 1`.
fn subsecond_hz() -> u32 {
    RTC::regs().prer().read().prediv_s() as u32 + 1
}

/// The current time of day, in RTC subsecond ticks.
fn time_of_day(subsecond_hz: u32) -> u32 {
    let ticks = RtcReader::new().read_regs(|_, tr, ss| {
        let hour = bcd2_to_byte((tr.ht(), tr.hu())) as u32;
        let minute = bcd2_to_byte((tr.mnt(), tr.mnu())) as u32;
        let second = bcd2_to_byte((tr.st(), tr.su())) as u32;
        let seconds = (hour * 60 + minute) * 60 + second;

        // SS counts down from PREDIV_S. Right after a shift operation it can be above
        // PREDIV_S, and the time is then before `seconds`.
        seconds as i64 * subsecond_hz as i64 + (subsecond_hz - 1) as i64 - ss as i64
    });

    ticks.rem_euclid(SECONDS_PER_DAY as i64 * subsecond_hz as i64) as u32
}

/// The embassy-time ticks from time of day `from` to time of day `to`, both in RTC subsecond
/// ticks, assuming less than a day passed between them.
fn ticks_between(from: u32, to: u32, subsecond_hz: u32) -> u64 {
    let ticks_per_day = SECONDS_PER_DAY as u64 * subsecond_hz as u64;
    let rtc_ticks = (to as u64 + ticks_per_day - from as u64) % ticks_per_day;
    rtc_ticks * TICK_HZ / subsecond_hz as u64
}

fn wucksel_compute_min(val: u32, rtc_hz: u32) -> (Wucksel, u32) {
    *[
        (Wucksel::Div2, 2),
        (Wucksel::Div4, 4),
        (Wucksel::Div8, 8),
        (Wucksel::Div16, 16),
        (Wucksel::ClockSpare, rtc_hz),
    ]
    .iter()
    .find(|(_, psc)| *psc as u32 > val)
    .unwrap_or(&(Wucksel::ClockSpare, rtc_hz))
}

/// The anchor the time spent in Stop mode is measured from. `None` until the first Stop, and
/// after the RTC time or prescalers change.
static ANCHOR: Mutex<Cell<Option<Anchor>>> = Mutex::new(Cell::new(None));

/// Forget the anchor, because the RTC time or prescalers changed.
pub(super) fn invalidate_anchor() {
    critical_section::with(|cs| ANCHOR.borrow(cs).set(None));
}

/// Start the wakeup alarm with a duration that is as close to but less than the requested duration,
/// minus `exit_latency` so clock restoration finishes by the target deadline.
///
/// `now` is the current time of the time driver, which must be stopped right after this
/// call and stay stopped until `stop_wakeup_alarm`.
pub(crate) fn start_wakeup_alarm(
    cs: CriticalSection,
    requested_duration: Duration,
    now: Instant,
    exit_latency: Duration,
) {
    let anchor = ANCHOR.borrow(cs);
    let max_anchor_age = Duration::from_secs(MAX_ANCHOR_AGE_SECONDS as u64);
    if anchor
        .get()
        .is_none_or(|a| now.saturating_duration_since(a.instant) > max_anchor_age)
    {
        anchor.set(Some(Anchor {
            time_of_day: time_of_day(subsecond_hz()),
            instant: now,
        }));
    }

    let rtc_hz: u32 = Rtc::frequency().0;
    let requested_duration = requested_duration
        .as_ticks()
        .saturating_sub(exit_latency.as_ticks())
        .clamp(0, u32::MAX as u64);
    let rtc_ticks: u32 = (requested_duration * rtc_hz as u64 / TICK_HZ).clamp(0, u32::MAX as u64) as u32;
    let (wucksel, prescaler) = wucksel_compute_min(rtc_ticks / u16::MAX as u32, rtc_hz);

    // adjust the rtc ticks to the prescaler and subtract one rtc tick
    let rtc_ticks: u32 = rtc_ticks / prescaler;
    let rtc_ticks = rtc_ticks.clamp(0, (u16::MAX - 1) as u32).saturating_sub(1).max(1) as u16;

    write(|regs| {
        regs.cr().modify(|w| w.set_wute(false));

        #[cfg(rtc_v2)]
        {
            regs.isr().modify(|w| w.set_wutf(false));
            while !regs.isr().read().wutwf() {}
        }

        #[cfg(rtc_v3)]
        {
            regs.scr().write(|w| w.set_cwutf(crate::pac::rtc::vals::Calrf::Clear));
            while !regs.icsr().read().wutwf() {}
        }

        regs.cr().modify(|w| w.set_wucksel(wucksel));
        regs.wutr().write(|w| w.set_wut(rtc_ticks));
        regs.cr().modify(|w| w.set_wute(true));
        regs.cr().modify(|w| w.set_wutie(true));
    });

    trace!(
        "rtc: start wakeup alarm for {} ms (psc: {}, ticks: {})",
        Duration::from_ticks(rtc_ticks as u64 * TICK_HZ * prescaler as u64 / rtc_hz as u64).as_millis(),
        prescaler as u32,
        rtc_ticks,
    );
}

/// Stop the wakeup alarm and return the current time as measured by the RTC, or `None`
/// if `start_wakeup_alarm` has not been called since the RTC time was last changed.
pub(crate) fn stop_wakeup_alarm(cs: CriticalSection) -> Option<Instant> {
    if RTC::regs().cr().read().wute() {
        trace!("rtc: stop wakeup alarm");

        write(|regs| {
            regs.cr().modify(|w| w.set_wutie(false));
            regs.cr().modify(|w| w.set_wute(false));

            #[cfg(rtc_v2)]
            regs.isr().modify(|w| w.set_wutf(false));
            #[cfg(rtc_v3)]
            regs.scr().write(|w| w.set_cwutf(crate::pac::rtc::vals::Calrf::Clear));

            // Check RM for EXTI and/or NVIC section, "Event event input mapping" or "EXTI interrupt/event mapping" or something similar,
            // there is a table for every "Event input" / "EXTI Line".
            // If you find the EXTI line related to "RTC wakeup" marks as "Configurable" (not "Direct"),
            // then write 1 to related field of Pending Register, to clean it's pending state.
            #[cfg(any(exti_v1, stm32h7, stm32wb))]
            crate::pac::EXTI
                .pr(0)
                .write(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));

            <RTC as crate::rtc::SealedInstance>::WakeupInterrupt::unpend();
        });
    }

    let anchor = ANCHOR.borrow(cs).get()?;
    let subsecond_hz = subsecond_hz();
    let elapsed = ticks_between(anchor.time_of_day, time_of_day(subsecond_hz), subsecond_hz);
    Some(anchor.instant + Duration::from_ticks(elapsed))
}

/// Write the wakeup timer registers.
///
/// The time driver calls this with the time stopped, from the executor's idle path in a critical
/// section, so it never interleaves with the `Rtc` owner's register writes.
fn write(f: impl FnOnce(crate::pac::rtc::Rtc)) {
    // SAFETY: see above, this doesn't race the user's `Rtc`.
    let mut rtc = Rtc {
        _peri: unsafe { RTC::steal() },
    };
    rtc.write(false, f)
}

impl Rtc<'_> {
    pub(super) fn enable_wakeup_line(&mut self) {
        <RTC as crate::rtc::SealedInstance>::WakeupInterrupt::unpend();
        unsafe { <RTC as crate::rtc::SealedInstance>::WakeupInterrupt::enable() };

        // The RTC is a direct EXTI line on G0, U0, U3, U5 and WBA, unmasked at reset.
        #[cfg(not(any(stm32u5, stm32u3, stm32u0, stm32wba, stm32g0)))]
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
        #[cfg(all(feature = "low-power", stm32u5))]
        <RTC as crate::rcc::SealedRccPeripheral>::RCC_INFO.enable_stop_and_autonomous();
        #[cfg(all(feature = "low-power", stm32wba))]
        crate::pac::RCC.apb7smenr().modify(|w| w.set_rtcapbsmen(true));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_anchor_age_leaves_room_for_the_longest_stop() {
        assert!(MAX_ANCHOR_AGE_SECONDS > 3_600);
        assert!(MAX_ANCHOR_AGE_SECONDS + MAX_STOP_SECONDS < SECONDS_PER_DAY);
    }

    #[test]
    fn ticks_between_same_day() {
        let hz = 256;
        assert_eq!(ticks_between(0, 0, hz), 0);
        assert_eq!(ticks_between(10 * hz, 12 * hz, hz), 2 * TICK_HZ);
        assert_eq!(ticks_between(10 * hz, 10 * hz + 1, hz), TICK_HZ / 256);
    }

    #[test]
    fn ticks_between_across_midnight() {
        let hz = 256;
        let day = SECONDS_PER_DAY * hz;
        // 23:59:59 to 00:00:01
        assert_eq!(ticks_between(day - hz, hz, hz), 2 * TICK_HZ);
        assert_eq!(ticks_between(day - 1, 0, hz), TICK_HZ / 256);
    }

    #[test]
    fn ticks_between_fastest_subsecond_clock() {
        // PREDIV_S at its maximum, 32768 ticks per second: a day of ticks overflows u32 when added.
        let hz = 32_768;
        let day = SECONDS_PER_DAY * hz;
        assert_eq!(
            ticks_between(day - 1, day - 2, hz),
            (day as u64 - 1) * TICK_HZ / hz as u64
        );
        assert_eq!(ticks_between(1, 0, hz), (day as u64 - 1) * TICK_HZ / hz as u64);
    }
}
