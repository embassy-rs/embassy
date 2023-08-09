//! RTC peripheral abstraction
mod datetime;

pub use self::datetime::{DateTime, DayOfWeek, Error as DateTimeError};

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

/// RTC Abstraction
pub struct Rtc {
    rtc_config: RtcConfig,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum RtcClockSource {
    /// 00: No clock
    NoClock = 0b00,
    /// 01: LSE oscillator clock used as RTC clock
    LSE = 0b01,
    /// 10: LSI oscillator clock used as RTC clock
    LSI = 0b10,
    /// 11: HSE oscillator clock divided by 32 used as RTC clock
    HSE = 0b11,
}

#[derive(Copy, Clone, PartialEq)]
pub struct RtcConfig {
    /// Asynchronous prescaler factor
    /// This is the asynchronous division factor:
    /// ck_apre frequency = RTCCLK frequency/(PREDIV_A+1)
    /// ck_apre drives the subsecond register
    async_prescaler: u8,
    /// Synchronous prescaler factor
    /// This is the synchronous division factor:
    /// ck_spre frequency = ck_apre frequency/(PREDIV_S+1)
    /// ck_spre must be 1Hz
    sync_prescaler: u16,
}

impl Default for RtcConfig {
    /// LSI with prescalers assuming 32.768 kHz.
    /// Raw sub-seconds in 1/256.
    fn default() -> Self {
        RtcConfig {
            async_prescaler: 127,
            sync_prescaler: 255,
        }
    }
}

impl RtcConfig {
    /// Set the asynchronous prescaler of RTC config
    pub fn async_prescaler(mut self, prescaler: u8) -> Self {
        self.async_prescaler = prescaler;
        self
    }

    /// Set the synchronous prescaler of RTC config
    pub fn sync_prescaler(mut self, prescaler: u16) -> Self {
        self.sync_prescaler = prescaler;
        self
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
        RTC::enable_peripheral_clk();

        let mut rtc_struct = Self { rtc_config };

        Self::enable();

        rtc_struct.configure(rtc_config);
        rtc_struct.rtc_config = rtc_config;

        rtc_struct
    }

    /// Set the datetime to a new value.
    ///
    /// # Errors
    ///
    /// Will return `RtcError::InvalidDateTime` if the datetime is not a valid range.
    pub fn set_datetime(&mut self, t: DateTime) -> Result<(), RtcError> {
        self::datetime::validate_datetime(&t).map_err(RtcError::InvalidDateTime)?;
        self.write(true, |rtc| self::datetime::write_date_time(rtc, t));

        Ok(())
    }

    /// Return the current datetime.
    ///
    /// # Errors
    ///
    /// Will return an `RtcError::InvalidDateTime` if the stored value in the system is not a valid [`DayOfWeek`].
    pub fn now(&self) -> Result<DateTime, RtcError> {
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

        self::datetime::datetime(year, month, day, weekday, hour, minute, second).map_err(RtcError::InvalidDateTime)
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

    pub fn get_config(&self) -> RtcConfig {
        self.rtc_config
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

        fn regs() -> Rtc {
            crate::pac::RTC
        }

        fn enable_peripheral_clk() {}

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
