//! RTC peripheral abstraction
mod datetime;

#[cfg(feature = "low-power")]
use core::cell::Cell;

#[cfg(feature = "low-power")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "low-power")]
use embassy_sync::blocking_mutex::Mutex;

pub use self::datetime::{DateTime, DayOfWeek, Error as DateTimeError, RtcInstant};
use crate::rtc::datetime::day_of_week_to_u8;
use crate::time::Hertz;

/// refer to AN4759 to compare features of RTC2 and RTC3
#[cfg_attr(any(rtc_v1), path = "v1.rs")]
#[cfg_attr(
    any(
        rtc_v2f0, rtc_v2f2, rtc_v2f3, rtc_v2f4, rtc_v2f7, rtc_v2h7, rtc_v2l0, rtc_v2l1, rtc_v2l4, rtc_v2wb
    ),
    path = "v2.rs"
)]
#[cfg_attr(any(rtc_v3, rtc_v3u5), path = "v3.rs")]
mod _version;
#[allow(unused_imports)]
pub use _version::*;
use embassy_hal_internal::Peripheral;

use crate::peripherals::RTC;
use crate::rtc::sealed::Instance;

/// Errors that can occur on methods on [RtcClock]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RtcError {
    /// An invalid DateTime was given or stored on the hardware.
    InvalidDateTime(DateTimeError),

    /// The RTC clock is not running
    NotRunning,
}

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
        // For RM0433 we use BYPSHAD=1 to work around errata ES0392 2.19.1
        #[cfg(rcc_h7rm0433)]
        loop {
            let r = RTC::regs();
            let ss = r.ssr().read().ss();
            let dr = r.dr().read();
            let tr = r.tr().read();

            // If an RTCCLK edge occurs during read we may see inconsistent values
            // so read ssr again and see if it has changed. (see RM0433 Rev 7 46.3.9)
            let ss_after = r.ssr().read().ss();
            if ss == ss_after {
                let second = bcd2_to_byte((tr.st(), tr.su()));
                let minute = bcd2_to_byte((tr.mnt(), tr.mnu()));
                let hour = bcd2_to_byte((tr.ht(), tr.hu()));

                let weekday = dr.wdu();
                let day = bcd2_to_byte((dr.dt(), dr.du()));
                let month = bcd2_to_byte((dr.mt() as u8, dr.mu()));
                let year = bcd2_to_byte((dr.yt(), dr.yu())) as u16 + 1970_u16;

                return DateTime::from(year, month, day, weekday, hour, minute, second)
                    .map_err(RtcError::InvalidDateTime);
            }
        }

        #[cfg(not(rcc_h7rm0433))]
        {
            let r = RTC::regs();
            let tr = r.tr().read();
            let second = bcd2_to_byte((tr.st(), tr.su()));
            let minute = bcd2_to_byte((tr.mnt(), tr.mnu()));
            let hour = bcd2_to_byte((tr.ht(), tr.hu()));
            // Reading either RTC_SSR or RTC_TR locks the values in the higher-order
            // calendar shadow registers until RTC_DR is read.
            let dr = r.dr().read();

            let weekday = dr.wdu();
            let day = bcd2_to_byte((dr.dt(), dr.du()));
            let month = bcd2_to_byte((dr.mt() as u8, dr.mu()));
            let year = bcd2_to_byte((dr.yt(), dr.yu())) as u16 + 1970_u16;

            DateTime::from(year, month, day, weekday, hour, minute, second).map_err(RtcError::InvalidDateTime)
        }
    }
}

/// RTC Abstraction
pub struct Rtc {
    #[cfg(feature = "low-power")]
    stop_time: Mutex<CriticalSectionRawMutex, Cell<Option<RtcInstant>>>,
    #[cfg(not(feature = "low-power"))]
    _private: (),
}

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

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum RtcCalibrationCyclePeriod {
    /// 8-second calibration period
    Seconds8,
    /// 16-second calibration period
    Seconds16,
    /// 32-second calibration period
    Seconds32,
}

impl Default for RtcCalibrationCyclePeriod {
    fn default() -> Self {
        RtcCalibrationCyclePeriod::Seconds32
    }
}

impl Rtc {
    pub fn new(_rtc: impl Peripheral<P = RTC>, rtc_config: RtcConfig) -> Self {
        #[cfg(not(any(stm32l0, stm32f3, stm32l1, stm32f0, stm32f2)))]
        <RTC as crate::rcc::sealed::RccPeripheral>::enable_and_reset();

        let mut this = Self {
            #[cfg(feature = "low-power")]
            stop_time: Mutex::const_new(CriticalSectionRawMutex::new(), Cell::new(None)),
            #[cfg(not(feature = "low-power"))]
            _private: (),
        };

        let frequency = Self::frequency();
        let async_psc = ((frequency.0 / rtc_config.frequency.0) - 1) as u8;
        let sync_psc = (rtc_config.frequency.0 - 1) as u16;

        this.configure(async_psc, sync_psc);

        this
    }

    fn frequency() -> Hertz {
        let freqs = unsafe { crate::rcc::get_freqs() };
        freqs.rtc.unwrap()
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
        self::datetime::validate_datetime(&t).map_err(RtcError::InvalidDateTime)?;
        self.write(true, |rtc| {
            let (ht, hu) = byte_to_bcd2(t.hour() as u8);
            let (mnt, mnu) = byte_to_bcd2(t.minute() as u8);
            let (st, su) = byte_to_bcd2(t.second() as u8);

            let (dt, du) = byte_to_bcd2(t.day() as u8);
            let (mt, mu) = byte_to_bcd2(t.month() as u8);
            let yr = t.year() as u16;
            let yr_offset = (yr - 1970_u16) as u8;
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

    #[cfg(not(rtc_v2f2))]
    /// Return the current instant.
    pub fn instant(&self) -> Result<RtcInstant, RtcError> {
        let r = RTC::regs();
        let tr = r.tr().read();
        let subsecond = r.ssr().read().ss();
        let second = bcd2_to_byte((tr.st(), tr.su()));

        // Unlock the registers
        r.dr().read();

        RtcInstant::from(second, subsecond.try_into().unwrap())
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

    pub const BACKUP_REGISTER_COUNT: usize = RTC::BACKUP_REGISTER_COUNT;

    /// Read content of the backup register.
    ///
    /// The registers retain their values during wakes from standby mode or system resets. They also
    /// retain their value when Vdd is switched off as long as V_BAT is powered.
    pub fn read_backup_register(&self, register: usize) -> Option<u32> {
        RTC::read_backup_register(&RTC::regs(), register)
    }

    /// Set content of the backup register.
    ///
    /// The registers retain their values during wakes from standby mode or system resets. They also
    /// retain their value when Vdd is switched off as long as V_BAT is powered.
    pub fn write_backup_register(&self, register: usize, value: u32) {
        RTC::write_backup_register(&RTC::regs(), register, value)
    }
}

pub(crate) fn byte_to_bcd2(byte: u8) -> (u8, u8) {
    let mut bcd_high: u8 = 0;
    let mut value = byte;

    while value >= 10 {
        bcd_high += 1;
        value -= 10;
    }

    (bcd_high, ((bcd_high << 4) | value) as u8)
}

pub(crate) fn bcd2_to_byte(bcd: (u8, u8)) -> u8 {
    let value = bcd.1 | bcd.0 << 4;

    let tmp = ((value & 0xF0) >> 0x4) * 10;

    tmp + (value & 0x0F)
}

pub(crate) mod sealed {
    use crate::pac::rtc::Rtc;

    pub trait Instance {
        const BACKUP_REGISTER_COUNT: usize;

        #[cfg(feature = "low-power")]
        const EXTI_WAKEUP_LINE: usize;

        #[cfg(feature = "low-power")]
        type WakeupInterrupt: crate::interrupt::typelevel::Interrupt;

        fn regs() -> Rtc {
            crate::pac::RTC
        }

        /// Read content of the backup register.
        ///
        /// The registers retain their values during wakes from standby mode or system resets. They also
        /// retain their value when Vdd is switched off as long as V_BAT is powered.
        fn read_backup_register(rtc: &Rtc, register: usize) -> Option<u32>;

        /// Set content of the backup register.
        ///
        /// The registers retain their values during wakes from standby mode or system resets. They also
        /// retain their value when Vdd is switched off as long as V_BAT is powered.
        fn write_backup_register(rtc: &Rtc, register: usize, value: u32);

        // fn apply_config(&mut self, rtc_config: RtcConfig);
    }
}
