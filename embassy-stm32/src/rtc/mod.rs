//! Real Time Clock (RTC)
mod datetime;

#[cfg(feature = "low-power")]
mod low_power;

#[cfg(feature = "low-power")]
use core::cell::Cell;

#[cfg(feature = "low-power")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "low-power")]
use embassy_sync::blocking_mutex::Mutex;

use self::datetime::{day_of_week_from_u8, day_of_week_to_u8};
pub use self::datetime::{DateTime, DayOfWeek, Error as DateTimeError};
use crate::pac::rtc::regs::{Dr, Tr};
use crate::time::Hertz;

/// refer to AN4759 to compare features of RTC2 and RTC3
#[cfg_attr(any(rtc_v1), path = "v1.rs")]
#[cfg_attr(
    any(
        rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
    ),
    path = "v2.rs"
)]
#[cfg_attr(any(rtc_v3, rtc_v3u5, rtc_v3l5), path = "v3.rs")]
mod _version;
#[allow(unused_imports)]
pub use _version::*;

use crate::peripherals::RTC;
use crate::Peri;

/// Errors that can occur on methods on [RtcClock]
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RtcError {
    /// An invalid DateTime was given or stored on the hardware.
    InvalidDateTime(DateTimeError),

    /// The current time could not be read
    ReadFailure,

    /// The RTC clock is not running
    NotRunning,
}

/// Provides immutable access to the current time of the RTC.
pub struct RtcTimeProvider {
    _private: (),
}

impl RtcTimeProvider {
    /// Return the current datetime.
    ///
    /// # Errors
    ///
    /// Will return an `RtcError::InvalidDateTime` if the stored value in the system is not a valid [`DayOfWeek`].
    pub fn now(&self) -> Result<DateTime, RtcError> {
        self.read(|dr, tr, _ss| {
            let second = bcd2_to_byte((tr.st(), tr.su()));
            let minute = bcd2_to_byte((tr.mnt(), tr.mnu()));
            let hour = bcd2_to_byte((tr.ht(), tr.hu()));

            let weekday = day_of_week_from_u8(dr.wdu()).map_err(RtcError::InvalidDateTime)?;
            let day = bcd2_to_byte((dr.dt(), dr.du()));
            let month = bcd2_to_byte((dr.mt() as u8, dr.mu()));
            let year = bcd2_to_byte((dr.yt(), dr.yu())) as u16 + 2000_u16;

            // Calculate second fraction and multiply to microseconds
            // Formula from RM0410
            #[cfg(not(rtc_v2f2))]
            let us = {
                let prediv = RTC::regs().prer().read().prediv_s() as f32;
                (((prediv - _ss as f32) / (prediv + 1.0)) * 1e6).min(999_999.0) as u32
            };
            #[cfg(rtc_v2f2)]
            let us = 0;

            DateTime::from(year, month, day, weekday, hour, minute, second, us).map_err(RtcError::InvalidDateTime)
        })
    }

    fn read<R>(&self, mut f: impl FnMut(Dr, Tr, u16) -> Result<R, RtcError>) -> Result<R, RtcError> {
        let r = RTC::regs();

        #[cfg(not(rtc_v2f2))]
        let read_ss = || r.ssr().read().ss();
        #[cfg(rtc_v2f2)]
        let read_ss = || 0;

        let mut ss = read_ss();
        for _ in 0..5 {
            let tr = r.tr().read();
            let dr = r.dr().read();
            let ss_after = read_ss();

            // If an RTCCLK edge occurs during read we may see inconsistent values
            // so read ssr again and see if it has changed. (see RM0433 Rev 7 46.3.9)
            if ss == ss_after {
                return f(dr, tr, ss.try_into().unwrap());
            } else {
                ss = ss_after
            }
        }

        Err(RtcError::ReadFailure)
    }
}

/// RTC driver.
pub struct Rtc {
    #[cfg(feature = "low-power")]
    stop_time: Mutex<CriticalSectionRawMutex, Cell<Option<low_power::RtcInstant>>>,
    _private: (),
}

/// RTC configuration.
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq)]
pub struct RtcConfig {
    /// The subsecond counter frequency; default is 256
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

impl Rtc {
    /// Create a new RTC instance.
    pub fn new(_rtc: Peri<'static, RTC>, rtc_config: RtcConfig) -> Self {
        #[cfg(not(any(stm32l0, stm32f3, stm32l1, stm32f0, stm32f2)))]
        crate::rcc::enable_and_reset::<RTC>();

        let mut this = Self {
            #[cfg(feature = "low-power")]
            stop_time: Mutex::const_new(CriticalSectionRawMutex::new(), Cell::new(None)),
            _private: (),
        };

        let frequency = Self::frequency();
        let async_psc = ((frequency.0 / rtc_config.frequency.0) - 1) as u8;
        let sync_psc = (rtc_config.frequency.0 - 1) as u16;

        this.configure(async_psc, sync_psc);

        // Wait for the clock to update after initialization
        #[cfg(not(rtc_v2f2))]
        {
            let now = this.time_provider().read(|_, _, ss| Ok(ss)).unwrap();
            while now == this.time_provider().read(|_, _, ss| Ok(ss)).unwrap() {}
        }

        this
    }

    fn frequency() -> Hertz {
        let freqs = unsafe { crate::rcc::get_freqs() };
        freqs.rtc.to_hertz().unwrap()
    }

    /// Acquire a [`RtcTimeProvider`] instance.
    pub const fn time_provider(&self) -> RtcTimeProvider {
        RtcTimeProvider { _private: () }
    }

    /// Set the datetime to a new value.
    ///
    /// # Errors
    ///
    /// Will return `RtcError::InvalidDateTime` if the datetime is not a valid range.
    pub fn set_datetime(&mut self, t: DateTime) -> Result<(), RtcError> {
        self.write(true, |rtc| {
            let (ht, hu) = byte_to_bcd2(t.hour());
            let (mnt, mnu) = byte_to_bcd2(t.minute());
            let (st, su) = byte_to_bcd2(t.second());

            let (dt, du) = byte_to_bcd2(t.day());
            let (mt, mu) = byte_to_bcd2(t.month());
            let yr = t.year();
            let yr_offset = (yr - 2000_u16) as u8;
            let (yt, yu) = byte_to_bcd2(yr_offset);

            use crate::pac::rtc::vals::Ampm;

            rtc.tr().write(|w| {
                w.set_ht(ht);
                w.set_hu(hu);
                w.set_mnt(mnt);
                w.set_mnu(mnu);
                w.set_st(st);
                w.set_su(su);
                w.set_pm(Ampm::AM);
            });

            rtc.dr().write(|w| {
                w.set_dt(dt);
                w.set_du(du);
                w.set_mt(mt > 0);
                w.set_mu(mu);
                w.set_yt(yt);
                w.set_yu(yu);
                w.set_wdu(day_of_week_to_u8(t.day_of_week()));
            });
        });

        Ok(())
    }

    /// Return the current datetime.
    ///
    /// # Errors
    ///
    /// Will return an `RtcError::InvalidDateTime` if the stored value in the system is not a valid [`DayOfWeek`].
    pub fn now(&self) -> Result<DateTime, RtcError> {
        self.time_provider().now()
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
        })
    }

    /// Number of backup registers of this instance.
    pub const BACKUP_REGISTER_COUNT: usize = RTC::BACKUP_REGISTER_COUNT;

    /// Read content of the backup register.
    ///
    /// The registers retain their values during wakes from standby mode or system resets. They also
    /// retain their value when Vdd is switched off as long as V_BAT is powered.
    pub fn read_backup_register(&self, register: usize) -> Option<u32> {
        RTC::read_backup_register(RTC::regs(), register)
    }

    /// Set content of the backup register.
    ///
    /// The registers retain their values during wakes from standby mode or system resets. They also
    /// retain their value when Vdd is switched off as long as V_BAT is powered.
    pub fn write_backup_register(&self, register: usize, value: u32) {
        RTC::write_backup_register(RTC::regs(), register, value)
    }
}

pub(crate) fn byte_to_bcd2(byte: u8) -> (u8, u8) {
    let mut bcd_high: u8 = 0;
    let mut value = byte;

    while value >= 10 {
        bcd_high += 1;
        value -= 10;
    }

    (bcd_high, ((bcd_high << 4) | value))
}

pub(crate) fn bcd2_to_byte(bcd: (u8, u8)) -> u8 {
    let value = bcd.1 | bcd.0 << 4;

    let tmp = ((value & 0xF0) >> 0x4) * 10;

    tmp + (value & 0x0F)
}

trait SealedInstance {
    const BACKUP_REGISTER_COUNT: usize;

    #[cfg(feature = "low-power")]
    #[cfg(not(any(stm32u5, stm32u0)))]
    const EXTI_WAKEUP_LINE: usize;

    #[cfg(feature = "low-power")]
    type WakeupInterrupt: crate::interrupt::typelevel::Interrupt;

    fn regs() -> crate::pac::rtc::Rtc {
        crate::pac::RTC
    }

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
