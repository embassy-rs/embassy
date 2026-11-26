//! Real Time Clock (RTC)
mod datetime;

#[cfg(all(feature = "low-power", not(feature = "_lp-time-driver")))]
mod low_power;
pub use self::datetime::RawDateTime;
#[cfg(feature = "jiff")]
pub use self::datetime::{InvalidDateTimeError, OutOfRangeError};
#[cfg(all(feature = "low-power", not(feature = "_lp-time-driver")))]
pub(crate) use self::low_power::{start_wakeup_alarm, stop_wakeup_alarm};
use crate::pac::rtc::regs::{Dr, Tr};
use crate::time::Hertz;

/// refer to AN4759 to compare features of RTC2 and RTC3
#[cfg_attr(rtc_v2, path = "v2.rs")]
#[cfg_attr(rtc_v3, path = "v3.rs")]
mod _version;
use core::marker::PhantomData;

#[allow(unused_imports)]
pub use _version::*;

use crate::Peri;
use crate::peripherals::RTC;

/// Reads the current date and time of the RTC.
///
/// Get one with [`Rtc::reader`]. It can be cloned and sent to other tasks, which can then read the
/// time while the [`Rtc`] is used to set it.
#[derive(Clone)]
pub struct RtcReader<'d> {
    _phantom: PhantomData<&'d ()>,
}

impl<'d> RtcReader<'d> {
    pub(self) const fn new() -> Self {
        Self { _phantom: PhantomData }
    }

    /// Read the current date and time.
    ///
    /// Ignores the day of week stored in the RTC.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDateTimeError`] if the RTC holds a date and time that does not exist, such as
    /// February 30.
    #[cfg(feature = "jiff")]
    pub fn read(&self) -> Result<jiff::civil::DateTime, InvalidDateTimeError> {
        // F2 and L1 Cat.1 have no subsecond counter: `subsecond` is 0, so `prediv_s` must be 0 too for the
        // second to read as just started.
        #[cfg(not(any(rtc_v2_f2, rtc_v2_l1_cat1)))]
        let prediv_s = RTC::regs().prer().read().prediv_s();
        #[cfg(any(rtc_v2_f2, rtc_v2_l1_cat1))]
        let prediv_s = 0;

        datetime::to_jiff(self.read_raw(), prediv_s)
    }

    /// Read the raw contents of the RTC calendar registers.
    ///
    /// See [`RawDateTime`]: the values are not validated.
    pub fn read_raw(&self) -> RawDateTime {
        self.read_regs(|dr, tr, ss| RawDateTime {
            year: bcd2_to_byte((dr.yt(), dr.yu())),
            month: bcd2_to_byte((dr.mt() as u8, dr.mu())),
            day: bcd2_to_byte((dr.dt(), dr.du())),
            day_of_week: dr.wdu(),
            hour: bcd2_to_byte((tr.ht(), tr.hu())),
            minute: bcd2_to_byte((tr.mnt(), tr.mnu())),
            second: bcd2_to_byte((tr.st(), tr.su())),
            subsecond: ss,
        })
    }

    /// Read the calendar registers, retrying until no RTCCLK edge occurs during the read.
    fn read_regs<R>(&self, f: impl FnOnce(Dr, Tr, u16) -> R) -> R {
        let r = RTC::regs();

        #[cfg(not(any(rtc_v2_f2, rtc_v2_l1_cat1)))]
        let read_ss = || r.ssr().read().ss();
        #[cfg(any(rtc_v2_f2, rtc_v2_l1_cat1))]
        let read_ss = || 0;

        let mut ss = read_ss();
        loop {
            let tr = r.tr().read();
            let dr = r.dr().read();
            let ss_after = read_ss();

            // If an RTCCLK edge occurs during read we may see inconsistent values
            // so read ssr again and see if it has changed. (see RM0433 Rev 7 46.3.9)
            if ss == ss_after {
                return f(dr, tr, ss as u16);
            } else {
                ss = ss_after
            }
        }
    }
}

/// RTC driver.
pub struct Rtc<'d> {
    _peri: Peri<'d, RTC>,
}

/// RTC configuration.
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq)]
pub struct RtcConfig {
    /// The subsecond counter frequency; default is 256
    ///
    /// If the RTC clock cannot be divided down to exactly this frequency with the 7-bit
    /// asynchronous prescaler, the nearest lower divider that divides the clock evenly is
    /// used instead, so the calendar still advances once per second.
    ///
    /// A high counter frequency may impact stop power consumption
    pub frequency: Hertz,
}

impl Default for RtcConfig {
    /// LSI with prescalers assuming 32.768 kHz.
    /// Raw sub-seconds in 1/256.
    fn default() -> Self {
        RtcConfig { frequency: Hertz(256) }
    }
}

/// Calibration cycle period.
#[derive(Default, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum RtcCalibrationCyclePeriod {
    /// 8-second calibration period
    Seconds8,
    /// 16-second calibration period
    Seconds16,
    /// 32-second calibration period
    #[default]
    Seconds32,
}

/// Prescaler register values (PREDIV_A, PREDIV_S) for an RTC clock of `clk` and a requested
/// subsecond frequency. The two dividers must multiply to exactly `clk` for the calendar to
/// advance once per second, and PREDIV_A is 7 bits, so the asynchronous divider is at most 128.
fn prescalers(clk: Hertz, subsecond: Hertz) -> (u8, u16) {
    let mut async_div = (clk.0 / subsecond.0).clamp(1, 128);
    while !clk.0.is_multiple_of(async_div) {
        async_div -= 1;
    }
    let sync_div = clk.0 / async_div;
    assert!(sync_div <= 1 << 15, "RTC clock too fast for the RTC prescalers");
    ((async_div - 1) as u8, (sync_div - 1) as u16)
}

impl<'d> Rtc<'d> {
    /// Create a new RTC instance.
    ///
    /// The date and time are kept. If the RTC does not already run with the prescalers for
    /// `rtc_config`, they are written, which restarts the current second.
    ///
    /// With the `low-power` feature and a time driver that is not a low-power timer,
    /// `embassy_stm32::init` already configures the RTC with `Config::rtc` to measure the time
    /// spent in Stop mode. Pass the same configuration here to keep it.
    pub fn new(peri: Peri<'d, RTC>, rtc_config: RtcConfig) -> Self {
        let this = Self::new_inner(peri, rtc_config);

        // A prescaler change makes the low-power anchor, kept in subsecond ticks, invalid.
        #[cfg(all(feature = "low-power", not(feature = "_lp-time-driver")))]
        low_power::invalidate_anchor();

        this
    }

    pub(self) fn new_inner(peri: Peri<'d, RTC>, rtc_config: RtcConfig) -> Self {
        #[cfg(not(any(stm32l0, stm32f3, stm32l1, stm32f0, stm32f2)))]
        critical_section::with(crate::rcc::enable_and_reset_with_cs_no_refcount::<RTC>);

        let mut this = Self { _peri: peri };

        let (async_psc, sync_psc) = prescalers(Self::frequency(), rtc_config.frequency);

        this.configure(async_psc, sync_psc);

        // Wait for the clock to update after initialization
        #[cfg(not(any(rtc_v2_f2, rtc_v2_l1_cat1)))]
        {
            let now = RtcReader::new().read_regs(|_, _, ss| ss);
            while now == RtcReader::new().read_regs(|_, _, ss| ss) {}
        }

        this
    }

    /// Get a reader for the current date and time.
    ///
    /// Use it to read the time from other tasks, while the `Rtc` is used to set it.
    pub fn reader(&self) -> RtcReader<'d> {
        RtcReader::new()
    }

    /// Read the current date and time. See [`RtcReader::read`].
    #[cfg(feature = "jiff")]
    pub fn read(&self) -> Result<jiff::civil::DateTime, InvalidDateTimeError> {
        RtcReader::new().read()
    }

    /// Read the raw contents of the RTC calendar registers. See [`RtcReader::read_raw`].
    pub fn read_raw(&self) -> RawDateTime {
        RtcReader::new().read_raw()
    }

    fn frequency() -> Hertz {
        unsafe { crate::rcc::get_freqs() }.rtc.to_hertz().unwrap()
    }

    /// Set the date and time.
    ///
    /// This function has second precision, and sets subseconds to 0.
    ///
    /// Writes a dummy day of week (Monday) to the RTC, which [`RtcReader::read`] ignores. Use
    /// [`set_raw`](Self::set_raw) if you need the hardware day of week.
    ///
    /// # Errors
    ///
    /// Returns [`OutOfRangeError`] if the year is not in `2000..=2099`.
    #[cfg(feature = "jiff")]
    pub fn set(&mut self, t: jiff::civil::DateTime) -> Result<(), OutOfRangeError> {
        self.set_raw(datetime::from_jiff(t)?);
        Ok(())
    }

    /// Write raw values to the RTC calendar registers.
    ///
    /// See [`RawDateTime`]: the values are not validated, and `subsecond` is ignored. This function sets subseconds to 0.
    pub fn set_raw(&mut self, t: RawDateTime) {
        self.write(true, |rtc| {
            let (ht, hu) = byte_to_bcd2(t.hour);
            let (mnt, mnu) = byte_to_bcd2(t.minute);
            let (st, su) = byte_to_bcd2(t.second);

            let (dt, du) = byte_to_bcd2(t.day);
            let (mt, mu) = byte_to_bcd2(t.month);
            let (yt, yu) = byte_to_bcd2(t.year);

            use crate::pac::rtc::vals::Ampm;

            rtc.tr().write(|w| {
                w.set_ht(ht);
                w.set_hu(hu);
                w.set_mnt(mnt);
                w.set_mnu(mnu);
                w.set_st(st);
                w.set_su(su);
                w.set_pm(Ampm::Am);
            });

            rtc.dr().write(|w| {
                w.set_dt(dt);
                w.set_du(du);
                w.set_mt(mt > 0);
                w.set_mu(mu);
                w.set_yt(yt);
                w.set_yu(yu);
                w.set_wdu(t.day_of_week);
            });
        });

        #[cfg(all(feature = "low-power", not(feature = "_lp-time-driver")))]
        low_power::invalidate_anchor();
    }

    /// Synchronize to a remote clock with a high degree of precision.
    ///
    /// `shift` should be adjusted from -1 to 1 second.
    #[cfg(not(any(rtc_v2_f2, rtc_v2_l1_cat1)))]
    pub fn synchronize(&mut self, shift: f32) {
        let prediv_s = RTC::regs().prer().read().prediv_s() as f32;
        let (add1s, subfs) = if shift > 0. {
            (true, ((1. - shift) * (1. + prediv_s)))
        } else if shift < 0. {
            (false, -shift * (1. + prediv_s))
        } else {
            return;
        };
        let subfs = subfs + 0.5; // round

        critical_section::with(|_| {
            while RTC::regs().ssr().read().ss() & 0x8000 != 0 || RTC::shpf() {}

            self.write(false, |rtc| {
                rtc.shiftr().write(|w| {
                    w.set_subfs(subfs.clamp(0., prediv_s) as u16);
                    w.set_add1s(add1s);
                });
            });
        });

        #[cfg(all(feature = "low-power", not(feature = "_lp-time-driver")))]
        low_power::invalidate_anchor();
    }

    /// Check if daylight savings time is active.
    pub fn get_daylight_savings(&self) -> bool {
        let cr = RTC::regs().cr().read();
        cr.bkp()
    }

    /// Enable/disable daylight savings time.
    pub fn set_daylight_savings(&mut self, daylight_savings: bool) {
        self.write(true, |rtc| {
            rtc.cr().modify(|w| w.set_bkp(daylight_savings));
        });
    }

    /// Number of backup registers of this instance.
    pub const BACKUP_REGISTER_COUNT: usize = RTC::BACKUP_REGISTER_COUNT;

    /// Read content of the backup register.
    ///
    /// The registers retain their values during wakes from standby mode or system resets. They also
    /// retain their value when Vdd is switched off as long as V_BAT is powered.
    #[cfg_attr(
        rtc_v3,
        doc = "\n\nAlways returns [`None`]. Use the [`TAMP`](crate::peripherals::TAMP) peripheral instead."
    )]
    pub fn read_backup_register(&self, register: usize) -> Option<u32> {
        RTC::read_backup_register(RTC::regs(), register)
    }

    /// Set content of the backup register.
    ///
    /// The registers retain their values during wakes from standby mode or system resets. They also
    /// retain their value when Vdd is switched off as long as V_BAT is powered.
    #[cfg_attr(
        rtc_v3,
        doc = "\n\nDoes not write to the register. Use the [`TAMP`](crate::peripherals::TAMP) peripheral instead."
    )]
    pub fn write_backup_register(&self, register: usize, value: u32) {
        RTC::write_backup_register(RTC::regs(), register, value)
    }
}

/// Split `byte` into its tens and units digits.
pub(crate) fn byte_to_bcd2(byte: u8) -> (u8, u8) {
    (byte / 10, byte % 10)
}

pub(crate) fn bcd2_to_byte(bcd: (u8, u8)) -> u8 {
    let value = bcd.1 | bcd.0 << 4;

    let tmp = ((value & 0xF0) >> 0x4) * 10;

    tmp + (value & 0x0F)
}

trait SealedInstance {
    const BACKUP_REGISTER_COUNT: usize;

    #[cfg(feature = "low-power")]
    #[cfg(not(feature = "_lp-time-driver"))]
    #[cfg(not(any(stm32c5, stm32wba, stm32u5, stm32u3, stm32u0, stm32g0)))]
    const EXTI_WAKEUP_LINE: usize;

    #[cfg(feature = "low-power")]
    type WakeupInterrupt: crate::interrupt::typelevel::Interrupt;

    fn regs() -> crate::pac::rtc::Rtc {
        crate::pac::RTC
    }

    /// Shift operation pending
    #[cfg(not(any(rtc_v2_f2, rtc_v2_l1_cat1)))]
    fn shpf() -> bool;

    /// Read content of the backup register.
    ///
    /// The registers retain their values during wakes from standby mode or system resets. They also
    /// retain their value when Vdd is switched off as long as V_BAT is powered.
    fn read_backup_register(rtc: crate::pac::rtc::Rtc, register: usize) -> Option<u32>;

    /// Set content of the backup register.
    ///
    /// The registers retain their values during wakes from standby mode or system resets. They also
    /// retain their value when Vdd is switched off as long as V_BAT is powered.
    fn write_backup_register(rtc: crate::pac::rtc::Rtc, register: usize, value: u32);

    // fn apply_config(&mut self, rtc_config: RtcConfig);
}

#[cfg(all(feature = "low-power", not(feature = "_lp-time-driver")))]
pub(crate) fn init_low_power(cs: critical_section::CriticalSection, config: RtcConfig) {
    // SAFETY: low-power mode shares the RTC registers with the user's `Rtc` without racing it,
    // see `low_power::write`.
    let mut rtc = Rtc::new_inner(unsafe { RTC::steal() }, config);
    rtc.enable_wakeup_line();
    stop_wakeup_alarm(cs);

    trace!("low power: stop with rtc configured");
}

#[cfg(test)]
mod prescaler_tests {
    use super::*;

    #[test]
    fn prescalers_keep_one_hertz() {
        // the usual cases are unchanged: LSE, and a 32 kHz LSI
        assert_eq!(prescalers(Hertz(32_768), Hertz(256)), (127, 255));
        assert_eq!(prescalers(Hertz(32_000), Hertz(256)), (124, 255));
        // a 40 kHz LSI (F0, F1, F3) needs 156.25 for 256 Hz, more than PREDIV_A holds
        assert_eq!(prescalers(Hertz(40_000), Hertz(256)), (124, 319));
        // a ratio that is not a whole number
        assert_eq!(prescalers(Hertz(32_768), Hertz(1000)), (31, 1023));
        for (clk, sub) in [(40_000, 256), (32_768, 1000), (1_000_000, 256), (37_000, 256)] {
            let (a, s) = prescalers(Hertz(clk), Hertz(sub));
            assert!(a <= 127);
            assert_eq!((a as u32 + 1) * (s as u32 + 1), clk);
        }
    }
}
