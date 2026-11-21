//! RTC DateTime driver.
use core::marker::PhantomData;
use core::mem::DropGuard;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::{Peri, PeripheralType};
#[cfg(feature = "embedded-mcu-hal")]
use embedded_mcu_hal::time::{Datetime, DatetimeClock, DatetimeClockError, DatetimeFields, Month as HalMonth};
use maitake_sync::WaitCell;

use crate::clocks::{Clock, WakeGuard, with_clocks};
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::pac;

/// RTC interrupt handler.
pub struct InterruptHandler<I: Instance> {
    _phantom: PhantomData<I>,
}

trait SealedInstance {
    fn info() -> &'static Info;

    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
}

/// Trait for RTC peripheral instances
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    type Interrupt: Interrupt;
}

struct Info {
    regs: pac::rtc::Rtc,
    wait_cell: WaitCell,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::rtc::Rtc {
        self.regs
    }

    #[inline(always)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
    }

    /// Wait with interrupts enabled for `DESYNCED` (`FFFEh`, RM 38.3.1.2-3)
    /// and, when `counters` is set, `STATUS[INVAL_BIT]` (RM 38.5.1.11) to clear.
    fn wait_ready(&self, counters: bool) -> Result<(), RtcError> {
        for _ in 0..SYNC_POLLS {
            let status = self.regs().status().read();
            if status.0 != DESYNCED && !(counters && status.inval_bit()) {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(RtcError::Other)
    }

    /// Run `f` inside a verified, freshly unlocked two-second write window.
    ///
    /// Call inside `critical_section::with` after [`Self::wait_ready`]. Locking
    /// first (`0x80`) avoids inheriting an expiring power-on/prior window and
    /// avoids back-to-back unlock sequences re-locking the block (RM 38.3.7
    /// Note 2). Both unlock and relock verify `STATUS[WRITE_PROT_EN]` (Note 1).
    fn with_unlocked<R>(&self, f: impl FnOnce(pac::rtc::Rtc) -> R) -> Result<R, RtcError> {
        let regs = self.regs();
        regs.status8().write_value(0x80);
        let status = regs.status().read();
        if status.0 == DESYNCED || !status.write_prot_en() {
            return Err(RtcError::Other);
        }

        for v in [0x00, 0x40, 0xC0, 0x80] {
            regs.status8().write_value(v);
        }
        let status = regs.status().read();
        if status.0 == DESYNCED || status.write_prot_en() {
            return Err(RtcError::Other);
        }

        let out = f(regs);

        regs.status8().write_value(0x80);
        let status = regs.status().read();
        if status.0 == DESYNCED || !status.write_prot_en() {
            return Err(RtcError::Other);
        }
        Ok(out)
    }
}

unsafe impl Sync for Info {}

impl SealedInstance for crate::peripherals::RTC0 {
    #[inline(always)]
    fn info() -> &'static Info {
        static INFO: Info = Info {
            regs: pac::RTC0,
            wait_cell: WaitCell::new(),
        };
        &INFO
    }

    const PERF_INT_INCR: fn() = crate::perf_counters::incr_interrupt_rtc0;
    const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::incr_interrupt_rtc0_wake;
}

impl Instance for crate::peripherals::RTC0 {
    type Interrupt = crate::interrupt::typelevel::RTC0;
}

/// Month
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Month {
    /// January
    January = 1,
    /// February
    February = 2,
    /// March
    March = 3,
    /// April
    April = 4,
    /// May
    May = 5,
    /// June
    June = 6,
    /// July
    July = 7,
    /// August
    August = 8,
    /// September
    September = 9,
    /// October
    October = 10,
    /// November
    November = 11,
    /// December
    December = 12,
}

impl From<u8> for Month {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::January,
            2 => Self::February,
            3 => Self::March,
            4 => Self::April,
            5 => Self::May,
            6 => Self::June,
            7 => Self::July,
            8 => Self::August,
            9 => Self::September,
            10 => Self::October,
            11 => Self::November,
            12 => Self::December,
            _ => unreachable!(),
        }
    }
}

impl From<Month> for u8 {
    fn from(value: Month) -> Self {
        value as u8
    }
}

impl Month {
    const fn from_register(value: u8) -> Result<Self, RtcError> {
        match value {
            1 => Ok(Self::January),
            2 => Ok(Self::February),
            3 => Ok(Self::March),
            4 => Ok(Self::April),
            5 => Ok(Self::May),
            6 => Ok(Self::June),
            7 => Ok(Self::July),
            8 => Ok(Self::August),
            9 => Ok(Self::September),
            10 => Ok(Self::October),
            11 => Ok(Self::November),
            12 => Ok(Self::December),
            _ => Err(RtcError::InvalidDateTime),
        }
    }
}

/// Day of the week
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Weekday {
    /// Sunday
    Sunday = 0,
    /// Monday
    Monday = 1,
    /// Tuesday
    Tuesday = 2,
    /// Wednesday
    Wednesday = 3,
    /// Thursday
    Thursday = 4,
    /// Friday
    Friday = 5,
    /// Saturday
    Saturday = 6,
}

impl From<u8> for Weekday {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Sunday,
            1 => Self::Monday,
            2 => Self::Tuesday,
            3 => Self::Wednesday,
            4 => Self::Thursday,
            5 => Self::Friday,
            6 => Self::Saturday,
            _ => unreachable!(),
        }
    }
}

impl From<Weekday> for u8 {
    fn from(value: Weekday) -> Self {
        value as u8
    }
}

impl Weekday {
    const fn from_register(value: u8) -> Result<Self, RtcError> {
        match value {
            0 => Ok(Self::Sunday),
            1 => Ok(Self::Monday),
            2 => Ok(Self::Tuesday),
            3 => Ok(Self::Wednesday),
            4 => Ok(Self::Thursday),
            5 => Ok(Self::Friday),
            6 => Ok(Self::Saturday),
            _ => Err(RtcError::InvalidDateTime),
        }
    }
}

/// Date and time structure for RTC operations
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DateTime {
    /// Year
    pub year: i16,
    /// Month
    pub month: Month,
    /// Day
    pub day: u8,
    /// Day of the week
    pub dow: Weekday,
    /// Hour
    pub hour: u8,
    /// Minute
    pub minute: u8,
    /// Second
    pub second: u8,
}

#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Clkout {
    /// No output clock
    #[default]
    None = 0,
    /// Fine 1Hz clock with both precise edges
    Fine = 1,
    /// 32768Hz or 16384Hz output
    Main = 2,
    /// Coarse 1Hz clock with both precise edges
    Coarse = 3,
}

impl From<Clkout> for u8 {
    fn from(value: Clkout) -> Self {
        value as u8
    }
}

#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
pub enum ClkSel {
    /// 16384Hz
    #[default]
    Clk16384,
    /// 32768Hz
    Clk32768,
}

impl From<ClkSel> for bool {
    fn from(value: ClkSel) -> Self {
        match value {
            ClkSel::Clk16384 => false,
            ClkSel::Clk32768 => true,
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
pub enum EnableDaylightSavings {
    /// No
    #[default]
    No,
    /// Yes
    Yes,
}

impl From<EnableDaylightSavings> for bool {
    fn from(value: EnableDaylightSavings) -> Self {
        match value {
            EnableDaylightSavings::No => false,
            EnableDaylightSavings::Yes => true,
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
pub enum Compensation {
    /// No compensation
    #[default]
    None,
    /// Coarse compensation
    Coarse {
        /// Duration in seconds over which the correction is applied.
        interval: u8,
        /// Correction value in terms of number of clock
        /// cycles of the RTC oscillator clock.
        correction: i8,
    },
    /// Fine compensation
    Fine {
        /// Integral correction value in terms of number of clock
        /// cycles of the RTC oscillator clock.
        integral: i8,
        /// Fractional correction value in terms of number of clock
        /// cycles of the fixed IRC clock
        fractional: u8,
    },
}

#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AlarmMatch {
    /// Alarm matches second, minute, and hour.
    #[default]
    Hour = 0,
    /// Alarm matches second, minute, hour, and day.
    Day = 1,
    /// Alarm matches second, minute, hour, day, and month.
    Month = 2,
    /// Alarm matches second, minute, hour, day, month, and year.
    Year = 3,
}

impl From<AlarmMatch> for u8 {
    fn from(value: AlarmMatch) -> Self {
        value as u8
    }
}

#[derive(Copy, Clone, Default)]
pub struct Config {
    /// Clkout selection
    clkout: Clkout,
    /// RTC Clock select
    clksel: ClkSel,
    /// Daylight savings select
    daylight_savings: EnableDaylightSavings,
    /// Alarm match. Selects which time and calendar counters are used
    /// for matching and will generate and alarm.
    alarm_match: AlarmMatch,
    /// Compensation method
    compensation: Compensation,
}

impl Config {
    /// Set the RTC clock select.
    pub fn with_clksel(mut self, clksel: ClkSel) -> Self {
        self.clksel = clksel;
        self
    }
}

/// CPU/bus poll budget when waiting out [`DESYNCED`] or `STATUS[INVAL_BIT]`.
const SYNC_POLLS: u32 = 1_000_000;

/// Value every RTC register reads back while the block is resynchronizing.
///
/// RM 38.3.1.2-3: when the system bus clock restarts after Stop/VBAT mode
/// (or when a newly selected clock starts), synchronization takes time (two
/// oscillator cycles out of VBAT mode). During synchronization no write is
/// allowed -- including `STATUS[WE]` unlock writes -- and any register read
/// returns FFFEh. If the clock selected by `CTRL[CLK_SEL]` is not running,
/// the block stays in FFFEh indefinitely until that oscillator starts or a
/// VBAT power-on reset occurs.
///
/// Treating FFFEh as data is actively dangerous rather than merely wrong:
/// `STATUS[INVAL_BIT]` (bit 0) is 0, `STATUS[WRITE_PROT_EN]` (bit 1) is 1, and
/// `ISR[ALM_IS]` (bit 2) is 1, so an unguarded read looks valid, an unverified
/// unlock during resync leaves registers locked for the next write, and an
/// unguarded alarm poll would complete immediately.
const DESYNCED: u16 = 0xfffe;

/// Value a counter register reads back while its contents are changing.
///
/// RM 38.5.1.11: STATUS[INVAL_BIT] is asserted for one oscillator clock either
/// side of the 1 Hz boundary, and "reading when STATUS[INVAL_BIT] is asserted
/// returns 0xFFFF. No transfer error is asserted." Decoded naively that is
/// hour=31, minute=63, second=63, month=15, dow=7.
///
/// This is a distinct sentinel from [`DESYNCED`]: FFFFh means the counters are
/// mid-update and the read should simply be retried, whereas FFFEh means the
/// register interface is synchronizing, possibly because its clock is stopped.
const INVALID: u16 = 0xffff;

/// Errors exclusive to HW initialization
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Clock configuration error.
    ClockSetup,
    /// The RTC register block is not accessible: reads return the
    /// synchronizing sentinel (RM 38.3.1.2) or a write did not take effect.
    RegisterAccess,
}

/// Errors exclusive for datetime.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum RtcError {
    /// Invalid datetime
    InvalidDateTime,
    /// Invalid DST year
    InvalidDstYear,
    /// Other error
    Other,
}

const _: () = {
    assert!(matches!(Month::from_register(1), Ok(Month::January)));
    assert!(matches!(Month::from_register(12), Ok(Month::December)));
    assert!(matches!(Month::from_register(0), Err(RtcError::InvalidDateTime)));
    assert!(matches!(Month::from_register(13), Err(RtcError::InvalidDateTime)));
    assert!(matches!(Weekday::from_register(0), Ok(Weekday::Sunday)));
    assert!(matches!(Weekday::from_register(6), Ok(Weekday::Saturday)));
    assert!(matches!(Weekday::from_register(7), Err(RtcError::InvalidDateTime)));
};

/// Provides read-only access to the current time, independently of alarm waits.
///
/// Created together with [`Rtc`]. Clones can be shared with other tasks without
/// allowing them to replace an outstanding alarm.
#[derive(Clone)]
pub struct RtcTimeProvider<'a> {
    info: &'static Info,
    _inst: PhantomData<&'a ()>,
    _wg: Option<WakeGuard>,
}

impl RtcTimeProvider<'_> {
    /// Return the current datetime.
    ///
    /// See [`Rtc::now`] for register synchronization and error behavior.
    pub fn now(&self) -> Result<DateTime, RtcError> {
        read_datetime(self.info)
    }
}

/// Exclusive RTC configuration and alarm handle.
pub struct Rtc<'a> {
    _inst: core::marker::PhantomData<&'a mut ()>,
    info: &'static Info,
    _freq: u32,
    _wg: Option<WakeGuard>,
}

impl<'a> Rtc<'a> {
    const BASE_YEAR: i16 = 2112;

    fn selected_clock(clksel: ClkSel) -> Result<Clock, SetupError> {
        // RTC uses the VBAT-domain outputs, not the system/core outputs.
        // A retained but unadvertised source is not an initialization opt-in.
        with_clocks(|c| match clksel {
            ClkSel::Clk16384 => c.clk_16k_vbat.clone(),
            ClkSel::Clk32768 => {
                #[cfg(not(feature = "rosc-32k-as-gpio"))]
                {
                    c.clk_32k_vbat.clone()
                }
                #[cfg(feature = "rosc-32k-as-gpio")]
                None
            }
        })
        .flatten()
        .ok_or(SetupError::ClockSetup)
    }

    /// Create an exclusive RTC handle and a cloneable, read-only time provider.
    ///
    /// Board clock setup must enable both the currently selected (possibly
    /// retained) source and the requested source. This does not start oscillators;
    /// it waits finitely for register synchronization before configuration.
    pub fn new<T: Instance>(
        _inst: Peri<'a, T>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        config: Config,
    ) -> Result<(Self, RtcTimeProvider<'a>), SetupError> {
        let info = T::info();

        // The RTC is NOT gated by the MRCC, but the clock selected by
        // CTRL[CLK_SEL] must actually be running: the register block stays in
        // its resynchronizing state for as long as it is not (RM 38.3.1.2).
        //
        // The RTC sits in the VBAT domain, so it is fed by clk_16k[2] /
        // osc32k[2] (RM p.81527), not the system-domain [0] outputs. Measured
        // on FRDM-MCXA577 with CLK_SEL=0: the counter runs with FROCLKE=0b100
        // alone, and stops with 0b001 or 0b010.
        let clk = Self::selected_clock(config.clksel)?;

        let mut inst = Self {
            info,
            _inst: PhantomData,
            _wg: WakeGuard::for_power(&clk.power),
            _freq: clk.frequency,
        };

        inst.set_configuration(&config)?;

        // Enable RTC interrupt
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let time_provider = RtcTimeProvider {
            info,
            _inst: PhantomData,
            _wg: inst._wg.clone(),
        };

        Ok((inst, time_provider))
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), SetupError> {
        let clk = Self::selected_clock(config.clksel)?;
        let wg = WakeGuard::for_power(&clk.power);

        self.info.wait_ready(false).map_err(|_| SetupError::RegisterAccess)?;

        critical_section::with(|_| self.set_configuration_inner(config))?;
        self._freq = clk.frequency;
        self._wg = wg;
        Ok(())
    }

    fn set_configuration_inner(&mut self, config: &Config) -> Result<(), SetupError> {
        // Refuse to touch a block we cannot read or write. Every write below
        // would be silently discarded, and callers would then be handed FFFEh
        // dressed up as register contents.
        if self.info.regs().ctrl().read().0 == DESYNCED {
            return Err(SetupError::RegisterAccess);
        }

        self.info
            .with_unlocked(|regs| {
                regs.ctrl().modify(|w| w.set_swr(true));
                regs.ctrl().modify(|w| w.set_swr(false));

                regs.ctrl().modify(|w| {
                    w.set_clkout(config.clkout.into());
                    w.set_clko_dis(config.clkout == Clkout::None);
                    w.set_clk_sel(config.clksel.into());
                    w.set_dst_en(config.daylight_savings.into());
                    w.set_alm_match(config.alarm_match.into());

                    match config.compensation {
                        Compensation::None => {
                            w.set_comp_en(false);
                            w.set_fineen(false);
                        }

                        Compensation::Coarse { .. } => {
                            w.set_comp_en(true);
                            w.set_fineen(false);
                        }

                        Compensation::Fine { .. } => {
                            w.set_comp_en(false);
                            w.set_fineen(true);
                        }
                    }
                });

                regs.compen().write(|w| match config.compensation {
                    Compensation::None => {}

                    Compensation::Coarse { interval, correction } => {
                        w.set_compen_val((interval as u16) << 8 | correction as u16);
                    }

                    Compensation::Fine { integral, fractional } => {
                        w.set_compen_val((integral as u16) << 8 | fractional as u16);
                    }
                });
            })
            .map_err(|_| SetupError::RegisterAccess)?;

        // Selecting a clock that is not running desynchronizes the block on the
        // spot, so confirm CLK_SEL actually took the requested value. Check the
        // sentinel first: bit 9 of FFFEh is set, so a failed write of
        // ClkSel::Clk32768 would otherwise read back as success.
        let ctrl = self.info.regs().ctrl().read();
        if ctrl.0 == DESYNCED || ctrl.clk_sel() != bool::from(config.clksel) {
            return Err(SetupError::RegisterAccess);
        }

        Ok(())
    }

    fn is_valid_datetime(t: DateTime) -> Result<(), RtcError> {
        if (t.year < (Self::BASE_YEAR - 128) || t.year > (Self::BASE_YEAR + 127))
            || t.day == 0
            || t.day > 31
            || t.hour > 23
            || t.minute > 59
            || t.second > 59
        {
            Err(RtcError::InvalidDateTime)
        } else {
            Ok(())
        }
    }

    fn is_valid_dst_year(&self, t: DateTime) -> Result<(), RtcError> {
        let now = self.now()?;

        if now.year != t.year {
            Err(RtcError::InvalidDstYear)
        } else {
            Ok(())
        }
    }

    /// Return the current datetime.
    ///
    /// Will block until we can access Datetime registers.
    ///
    /// # Errors
    ///
    /// Returns [`RtcError::InvalidDateTime`] if the RTC registers contain an invalid
    /// calendar or time field, such as while the peripheral is being reset.
    ///
    /// Returns [`RtcError::Other`] if the counters do not become stable, which
    /// indicates the RTC has no running clock.
    pub fn now(&self) -> Result<DateTime, RtcError> {
        read_datetime(self.info)
    }

    /// Set the datetime to a new value.
    ///
    /// # Errors
    ///
    /// Will return `RtcError::InvalidDateTime` if the datetime is not a valid range.
    /// Returns `RtcError::Other` if the counters are inaccessible or the update
    /// cannot be verified. An unsuccessful update may have changed some fields;
    /// the caller can retry setting the complete datetime.
    pub fn set_datetime(&mut self, t: DateTime) -> Result<(), RtcError> {
        Self::is_valid_datetime(t)?;

        let year = (t.year - Self::BASE_YEAR) as u8;
        let month = t.month.into();
        let dow = t.dow.into();
        let day = t.day;
        let hour = t.hour;
        let minute = t.minute;
        let second = t.second;

        self.info.wait_ready(true)?;

        // A time provider may read from another interrupt context. Serialize the
        // complete calendar update with its snapshot, not each register alone.
        critical_section::with(|_| {
            let status = self.info.regs().status().read();
            if status.0 == DESYNCED || status.inval_bit() {
                return Err(RtcError::Other);
            }

            self.info.with_unlocked(|regs| {
                regs.yearmon().write(|w| {
                    w.set_yrofst(year);
                    w.set_mon_cnt(month);
                });

                regs.days().write(|w| {
                    w.set_dow(dow);
                    w.set_day_cnt(day);
                });

                regs.hourmin().write(|w| {
                    w.set_hour_cnt(hour);
                    w.set_min_cnt(minute);
                });

                regs.seconds().write(|w| w.set_sec_cnt(second));
            })?;

            // A hardware counter update can still overlap the writes despite
            // interrupt masking. INVAL_BIT may already have cleared again, so
            // verify every field rather than trusting its final state alone.
            let s = self.info.regs().seconds().read();
            let hm = self.info.regs().hourmin().read();
            let d = self.info.regs().days().read();
            let ym = self.info.regs().yearmon().read();
            let status = self.info.regs().status().read();
            if status.0 == DESYNCED
                || status.inval_bit()
                || s.sec_cnt() != second
                || hm.hour_cnt() != hour
                || hm.min_cnt() != minute
                || d.dow() != dow
                || d.day_cnt() != day
                || ym.yrofst() != year
                || ym.mon_cnt() != month
            {
                return Err(RtcError::Other);
            }

            Ok(())
        })
    }

    /// Set the Daylight Savings start and end time.
    ///
    /// Note: only day, month, and hour are accounted for. The
    /// underlying HW is incapable of enabling DST for a future year.
    ///
    /// # Errors
    ///
    /// `RtcError::InvalidDateTime` if the either datetime is not a valid range.
    /// `RtcError::InvalidDstYear` if the either datetime contains a future or past year.
    /// `RtcError::Other` if the RTC register interface is inaccessible.
    pub fn set_dst(&mut self, start: DateTime, end: DateTime) -> Result<(), RtcError> {
        Self::is_valid_datetime(start)?;
        Self::is_valid_datetime(end)?;
        self.is_valid_dst_year(start)?;
        self.is_valid_dst_year(end)?;
        self.info.wait_ready(false)?;

        critical_section::with(|_| {
            self.info.with_unlocked(|regs| {
                regs.dst_hour().write(|w| {
                    w.set_dst_start_hour(start.hour - 1);
                    w.set_dst_end_hour(end.hour - 1);
                });

                regs.dst_month().write(|w| {
                    w.set_dst_start_month(start.month.into());
                    w.set_dst_end_month(end.month.into());
                });

                regs.dst_day().write(|w| {
                    w.set_dst_start_day(start.day);
                    w.set_dst_end_day(end.day);
                });
            })
        })
    }

    /// Set alarm to `t` and wait for the RTC alarm interrupt to trigger.
    ///
    /// The RTC is exclusively borrowed until this future completes or is dropped,
    /// preventing another caller from replacing the alarm. Use [`RtcTimeProvider`]
    /// to read the clock while waiting, without acquiring the RTC's mutex.
    /// Dropping the future makes a best-effort attempt to disable the alarm
    /// interrupt and clear its status while the RTC clock is running.
    ///
    /// # Errors
    ///
    /// Will return `RtcError::InvalidDateTime` if the datetime is not a valid range.
    /// Returns `RtcError::Other` if the RTC register interface is inaccessible.
    pub async fn wait_for_alarm(&mut self, t: DateTime) -> Result<(), RtcError> {
        Self::is_valid_datetime(t)?;

        let year = (t.year - Self::BASE_YEAR) as u8;
        let month = t.month.into();
        let day = t.day;
        let hour = t.hour;
        let minute = t.minute;
        let second = t.second;

        self.info.wait_ready(false)?;

        critical_section::with(|_| {
            self.info.with_unlocked(|regs| {
                regs.ier().modify(|w| w.set_alm_ie(false));

                regs.alm_yearmon().write(|w| {
                    w.set_alm_year(year);
                    w.set_alm_mon(month);
                });

                regs.alm_days().write(|w| {
                    w.set_alm_day(day);
                });

                regs.alm_hourmin().write(|w| {
                    w.set_alm_hour(hour);
                    w.set_alm_min(minute);
                });

                regs.alm_seconds().write(|w| w.set_alm_sec(second));
                // ALM_IS latches on a match even with ALM_IE clear, including daily
                // re-matches. Clear stale status after programming, before enabling.
                regs.isr().write(|w| w.set_alm_is(true));
                regs.ier().modify(|w| w.set_alm_ie(true));
            })
        })?;

        let info = self.info;
        let _cleanup = DropGuard::new(info, |info| {
            if info.wait_ready(false).is_ok() {
                let _ = critical_section::with(|_| {
                    info.with_unlocked(|regs| {
                        regs.ier().modify(|w| w.set_alm_ie(false));
                        regs.isr().write(|w| w.set_alm_is(true));
                    })
                });
            }
        });

        info.wait_cell()
            .wait_for(|| {
                let isr = info.regs().isr().read();
                // ALM_IS is bit 2 and bit 2 of the FFFEh synchronizing
                // sentinel is set, so an unguarded alm_is() reports a fired
                // alarm the instant the block loses its clock.
                isr.0 != DESYNCED && isr.alm_is()
            })
            .await
            .map_err(|_| RtcError::Other)
    }
}

fn read_datetime(info: &Info) -> Result<DateTime, RtcError> {
    // The four counters are separate registers, so a second boundary
    // falling between them would splice two different instants together.
    //
    // Waiting and sampling are deliberately separated. Waiting out the
    // boundary window is the only part that takes appreciable time - two
    // oscillator cycles, ~61 us at 32.768 kHz - so it runs with interrupts
    // enabled. Only the sample itself is masked, and that is a fixed five
    // register reads with no loop.
    let mut attempts = 0u32;
    let (second, hm, d, ym) = loop {
        attempts += 1;
        if attempts > 8 {
            return Err(RtcError::Other);
        }

        info.wait_ready(true)?;

        // Counters are stable. Take all four under one short critical
        // section so an interrupt cannot preempt us across a boundary,
        // re-checking DESYNCED and INVAL_BIT inside it because either can
        // assert between the wait above and the reads actually issuing.
        let sample = critical_section::with(|_| {
            let status = info.regs().status().read();
            if status.0 == DESYNCED || status.inval_bit() {
                return None;
            }
            let second = info.regs().seconds().read();
            let hm = info.regs().hourmin().read();
            let d = info.regs().days().read();
            let ym = info.regs().yearmon().read();

            if [second.0, hm.0, d.0, ym.0]
                .into_iter()
                .any(|raw| raw == DESYNCED || raw == INVALID)
            {
                return None;
            }
            Some((second.sec_cnt(), hm, d, ym))
        });

        if let Some(sample) = sample {
            break sample;
        }
    };

    let year = i16::from(ym.yrofst() as i8) + Rtc::BASE_YEAR;
    let month = Month::from_register(ym.mon_cnt())?;
    let dow = Weekday::from_register(d.dow())?;
    let day = d.day_cnt();
    let hour = hm.hour_cnt();
    let minute = hm.min_cnt();

    let datetime = DateTime {
        year,
        month,
        day,
        dow,
        hour,
        minute,
        second,
    };

    Rtc::is_valid_datetime(datetime)?;
    Ok(datetime)
}

/// RTC interrupt handler
///
/// This struct implements the interrupt handler for RTC events.
impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        let info = T::info();

        // An RTC alarm waking the MCU from Stop enters this handler before the
        // register interface finishes synchronizing (RM 38.3.1.2). Wait with
        // higher-priority interrupts enabled instead of re-entering on return.
        if info.regs().isr().read().0 == DESYNCED && info.wait_ready(false).is_err() {
            T::Interrupt::disable();
            return;
        }

        // Serialize the status read as well as the protected writes. Otherwise a
        // higher-priority owner could cancel/rearm after this ISR sampled the old
        // alarm, and this ISR would disable the new alarm or write after re-lock.
        let fired = critical_section::with(|_| {
            let status = info.regs().isr().read();

            if status.0 != DESYNCED && status.alm_is() {
                if info.with_unlocked(|r| r.ier().modify(|w| w.set_alm_ie(false))).is_err() {
                    T::Interrupt::disable();
                }
                true
            } else {
                false
            }
        });
        if fired {
            T::PERF_INT_WAKE_INCR();
            info.wait_cell().wake();
        }
    }
}

#[cfg(feature = "embedded-mcu-hal")]
/// Compute day of week from year/month/day using Sakamoto's algorithm.
fn day_of_week(year: i16, month: u8, day: u8) -> Weekday {
    const T: [u8; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year } as i32;
    let dow = (y + y / 4 - y / 100 + y / 400 + T[(month - 1) as usize] as i32 + day as i32) % 7;
    (dow as u8).into()
}

#[cfg(feature = "embedded-mcu-hal")]
impl<'a> DatetimeClock for Rtc<'a> {
    fn now(&self) -> Result<Datetime, DatetimeClockError> {
        let dt = self.now().map_err(|_| DatetimeClockError::UnsupportedDatetime)?;
        let month = match dt.month {
            Month::January => HalMonth::January,
            Month::February => HalMonth::February,
            Month::March => HalMonth::March,
            Month::April => HalMonth::April,
            Month::May => HalMonth::May,
            Month::June => HalMonth::June,
            Month::July => HalMonth::July,
            Month::August => HalMonth::August,
            Month::September => HalMonth::September,
            Month::October => HalMonth::October,
            Month::November => HalMonth::November,
            Month::December => HalMonth::December,
        };

        let fields = DatetimeFields {
            year: dt.year as u16,
            month: month,
            day: dt.day,
            hour: dt.hour,
            minute: dt.minute,
            second: dt.second,
            nanosecond: 0,
        };

        Datetime::new(fields).map_err(|_| DatetimeClockError::UnsupportedDatetime)
    }

    fn set(&mut self, datetime: Datetime) -> Result<(), DatetimeClockError> {
        let dt = DateTime {
            year: datetime.year() as i16,
            month: (datetime.month() as u8).into(),
            day: datetime.day(),
            dow: day_of_week(datetime.year() as i16, datetime.month() as u8, datetime.day()),
            hour: datetime.hour(),
            minute: datetime.minute(),
            second: datetime.second(),
        };
        self.set_datetime(dt)
            .map_err(|_| DatetimeClockError::UnsupportedDatetime)
    }

    fn resolution_hz(&self) -> u32 {
        self._freq
    }
}

impl<'a> SetConfig for Rtc<'a> {
    type Config = Config;
    type ConfigError = SetupError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_configuration(config)
    }
}
