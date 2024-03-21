//! Real Time Clock (RTC)
mod datetime;

#[cfg(feature = "low-power")]
use core::cell::Cell;

#[cfg(feature = "low-power")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "low-power")]
use embassy_sync::blocking_mutex::Mutex;

#[cfg(not(rtc_v2f2))]
use self::datetime::RtcInstant;
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
use embassy_hal_internal::Peripheral;

use crate::peripherals::RTC;
use crate::rtc::sealed::Instance;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum WakeupPrescaler {
    Div2 = 2,
    Div4 = 4,
    Div8 = 8,
    Div16 = 16,
}

#[cfg(any(stm32wb, stm32f4, stm32l0, stm32g4, stm32l5, stm32g0))]
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

#[cfg(any(stm32wb, stm32f4, stm32l0, stm32g4, stm32l5, stm32g0))]
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

#[cfg(feature = "low-power")]
impl WakeupPrescaler {
    pub fn compute_min(val: u32) -> Self {
        *[
            WakeupPrescaler::Div2,
            WakeupPrescaler::Div4,
            WakeupPrescaler::Div8,
            WakeupPrescaler::Div16,
        ]
        .iter()
        .skip_while(|psc| **psc as u32 <= val)
        .next()
        .unwrap_or(&WakeupPrescaler::Div16)
    }
}

/// Errors that can occur on methods on [RtcClock]
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
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
    #[cfg(not(rtc_v2f2))]
    pub(crate) fn instant(&self) -> Result<RtcInstant, RtcError> {
        self.read(|_, tr, ss| {
            let second = bcd2_to_byte((tr.st(), tr.su()));

            RtcInstant::from(second, ss).map_err(RtcError::InvalidDateTime)
        })
    }

    /// Return the current datetime.
    ///
    /// # Errors
    ///
    /// Will return an `RtcError::InvalidDateTime` if the stored value in the system is not a valid [`DayOfWeek`].
    pub fn now(&self) -> Result<DateTime, RtcError> {
        self.read(|dr, tr, _| {
            let second = bcd2_to_byte((tr.st(), tr.su()));
            let minute = bcd2_to_byte((tr.mnt(), tr.mnu()));
            let hour = bcd2_to_byte((tr.ht(), tr.hu()));

            let weekday = day_of_week_from_u8(dr.wdu()).map_err(RtcError::InvalidDateTime)?;
            let day = bcd2_to_byte((dr.dt(), dr.du()));
            let month = bcd2_to_byte((dr.mt() as u8, dr.mu()));
            let year = bcd2_to_byte((dr.yt(), dr.yu())) as u16 + 2000_u16;

            DateTime::from(year, month, day, weekday, hour, minute, second).map_err(RtcError::InvalidDateTime)
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

        return Err(RtcError::ReadFailure);
    }
}

/// RTC driver.
pub struct Rtc {
    #[cfg(feature = "low-power")]
    stop_time: Mutex<CriticalSectionRawMutex, Cell<Option<RtcInstant>>>,
    #[cfg(not(feature = "low-power"))]
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
    /// Create a new RTC instance.
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

        // Wait for the clock to update after initialization
        #[cfg(not(rtc_v2f2))]
        {
            let now = this.instant().unwrap();

            while this.instant().unwrap().subsecond == now.subsecond {}
        }

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
        self.write(true, |rtc| {
            let (ht, hu) = byte_to_bcd2(t.hour() as u8);
            let (mnt, mnu) = byte_to_bcd2(t.minute() as u8);
            let (st, su) = byte_to_bcd2(t.second() as u8);

            let (dt, du) = byte_to_bcd2(t.day() as u8);
            let (mt, mu) = byte_to_bcd2(t.month() as u8);
            let yr = t.year() as u16;
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

    #[cfg(not(rtc_v2f2))]
    /// Return the current instant.
    fn instant(&self) -> Result<RtcInstant, RtcError> {
        self.time_provider().instant()
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
        RTC::read_backup_register(&RTC::regs(), register)
    }

    /// Set content of the backup register.
    ///
    /// The registers retain their values during wakes from standby mode or system resets. They also
    /// retain their value when Vdd is switched off as long as V_BAT is powered.
    pub fn write_backup_register(&self, register: usize, value: u32) {
        RTC::write_backup_register(&RTC::regs(), register, value)
    }

    #[cfg(feature = "low-power")]
    /// start the wakeup alarm and wtih a duration that is as close to but less than
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
        unsafe { crate::rcc::get_freqs() }.rtc.unwrap();

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

    #[cfg(feature = "low-power")]
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

                #[cfg(all(stm32g0))]
                crate::pac::EXTI
                    .rpr(0)
                    .modify(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));
                #[cfg(all(not(stm32g0), not(stm32l5)))]
                crate::pac::EXTI
                    .pr(0)
                    .modify(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));

                #[cfg(stm32l5)]
                crate::pac::EXTI
                    .fpr(0)
                    .modify(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));

                <RTC as crate::rtc::sealed::Instance>::WakeupInterrupt::unpend();
            });
        }

        self.stop_time.borrow(cs).take().map(|stop_time| instant - stop_time)
    }

    #[cfg(feature = "low-power")]
    pub(crate) fn enable_wakeup_line(&self) {
        use crate::interrupt::typelevel::Interrupt;
        use crate::pac::EXTI;

        <RTC as crate::rtc::sealed::Instance>::WakeupInterrupt::unpend();
        unsafe { <RTC as crate::rtc::sealed::Instance>::WakeupInterrupt::enable() };

        EXTI.rtsr(0).modify(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));
        EXTI.imr(0).modify(|w| w.set_line(RTC::EXTI_WAKEUP_LINE, true));
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
