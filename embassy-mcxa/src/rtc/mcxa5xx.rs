//! RTC DateTime driver.
use core::marker::PhantomData;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitCell;

use crate::clocks::{WakeGuard, with_clocks};
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::pac;
use crate::pac::rtc5xx::Swr;

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
    regs: pac::rtc5xx::Rtc,
    wait_cell: WaitCell,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::rtc5xx::Rtc {
        self.regs
    }

    #[inline(always)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
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

/// Errors exclusive to HW initialization
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Clock configuration error.
    ClockSetup,
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

/// RTC driver.
pub struct Rtc<'a> {
    _inst: core::marker::PhantomData<&'a mut ()>,
    info: &'static Info,
    _wg: Option<WakeGuard>,
}

impl<'a> Rtc<'a> {
    const BASE_YEAR: i16 = 2112;

    /// Create a new instance of the real time clock.
    pub fn new<T: Instance>(
        _inst: Peri<'a, T>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        config: Config,
    ) -> Result<Self, SetupError> {
        let info = T::info();

        // The RTC is NOT gated by the MRCC, but we DO need to make
        // sure either the 16k clock or the 32k clock is active.
        let clocks = if config.clksel == ClkSel::Clk16384 {
            with_clocks(|c| c.clk_16k_vsys.clone())
        } else {
            with_clocks(|c| c.clk_32k_vsys.clone())
        };

        let clk = clocks.flatten().ok_or(SetupError::ClockSetup)?;

        let mut inst = Self {
            info,
            _inst: PhantomData,
            _wg: WakeGuard::for_power(&clk.power),
        };

        inst.set_configuration(&config)?;

        // Enable RTC interrupt
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Ok(inst)
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), SetupError> {
        self.disable_write_protect();

        self.info.regs().ctrl().modify(|w| w.set_swr(Swr::ASSERTED));
        self.info.regs().ctrl().modify(|w| w.set_swr(Swr::CLEARED));

        self.info.regs().ctrl().modify(|w| {
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

        self.info.regs().compen().write(|w| match config.compensation {
            Compensation::None => {}

            Compensation::Coarse { interval, correction } => {
                w.set_compen_val((interval as u16) << 8 | correction as u16);
            }

            Compensation::Fine { integral, fractional } => {
                w.set_compen_val((integral as u16) << 8 | fractional as u16);
            }
        });

        self.enable_write_protect();

        Ok(())
    }

    fn disable_write_protect(&mut self) {
        self.info.regs().status8().write_value(0b00);
        self.info.regs().status8().write_value(0b01);
        self.info.regs().status8().write_value(0b11);
        self.info.regs().status8().write_value(0b10);
    }

    fn enable_write_protect(&mut self) {
        self.info.regs().status8().write_value(0b10);
    }

    fn is_valid_datetime(&self, t: DateTime) -> Result<(), RtcError> {
        if (t.year < (Self::BASE_YEAR - 128) || t.year > (Self::BASE_YEAR + 127))
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
    pub fn now(&self) -> Result<DateTime, RtcError> {
        let ym = self.info.regs().yearmon().read();
        let d = self.info.regs().days().read();
        let hm = self.info.regs().hourmin().read();
        let second = self.info.regs().seconds().read().sec_cnt();

        let year = (i16::from(ym.yrofst() as i8) + Self::BASE_YEAR).into();
        let month = ym.mon_cnt().into();
        let dow = d.dow().into();
        let day = d.day_cnt();
        let hour = hm.hour_cnt();
        let minute = hm.min_cnt();

        Ok(DateTime {
            year,
            month,
            day,
            dow,
            hour,
            minute,
            second,
        })
    }

    /// Set the datetime to a new value.
    ///
    /// # Errors
    ///
    /// Will return `RtcError::InvalidDateTime` if the datetime is not a valid range.
    pub fn set_datetime(&mut self, t: DateTime) -> Result<(), RtcError> {
        self.is_valid_datetime(t)?;

        let year = (t.year - Self::BASE_YEAR) as u8;
        let month = t.month.into();
        let dow = t.dow.into();
        let day = t.day;
        let hour = t.hour;
        let minute = t.minute;
        let second = t.second;

        self.disable_write_protect();

        self.info.regs().yearmon().write(|w| {
            w.set_yrofst(year);
            w.set_mon_cnt(month);
        });

        self.info.regs().days().write(|w| {
            w.set_dow(dow);
            w.set_day_cnt(day);
        });

        self.info.regs().hourmin().write(|w| {
            w.set_hour_cnt(hour);
            w.set_min_cnt(minute);
        });

        self.info.regs().seconds().write(|w| w.set_sec_cnt(second));

        self.enable_write_protect();

        Ok(())
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
    pub fn set_dst(&mut self, start: DateTime, end: DateTime) -> Result<(), RtcError> {
        self.is_valid_datetime(start)?;
        self.is_valid_datetime(end)?;
        self.is_valid_dst_year(start)?;
        self.is_valid_dst_year(end)?;

        self.disable_write_protect();

        self.info.regs().dst_hour().write(|w| {
            w.set_dst_start_hour(start.hour - 1);
            w.set_dst_end_hour(end.hour - 1);
        });

        self.info.regs().dst_month().write(|w| {
            w.set_dst_start_month(start.month.into());
            w.set_dst_end_month(end.month.into());
        });

        self.info.regs().dst_day().write(|w| {
            w.set_dst_start_day(start.day);
            w.set_dst_end_day(end.day);
        });

        self.enable_write_protect();

        Ok(())
    }

    /// Set alarm to `t` and wait for the RTC alarm interrup to trigger.
    ///
    /// # Errors
    ///
    /// Will return `RtcError::InvalidDateTime` if the datetime is not a valid range.
    pub async fn wait_for_alarm(&mut self, t: DateTime) -> Result<(), RtcError> {
        self.is_valid_datetime(t)?;

        let year = (t.year - Self::BASE_YEAR) as u8;
        let month = t.month.into();
        let day = t.day;
        let hour = t.hour;
        let minute = t.minute;
        let second = t.second;

        self.disable_write_protect();

        self.info.regs().alm_yearmon().write(|w| {
            w.set_alm_year(year);
            w.set_alm_mon(month);
        });

        self.info.regs().alm_days().write(|w| {
            w.set_alm_day(day);
        });

        self.info.regs().alm_hourmin().write(|w| {
            w.set_alm_hour(hour);
            w.set_alm_min(minute);
        });

        self.info.regs().alm_seconds().write(|w| w.set_alm_sec(second));

        self.info
            .wait_cell()
            .wait_for(|| {
                self.info.regs().ier().modify(|w| w.set_alm_ie(true));
                self.info.regs().isr().read().alm_is()
            })
            .await
            .map_err(|_| RtcError::Other)?;

        self.info.regs().isr().write(|w| w.set_alm_is(true));

        self.enable_write_protect();

        Ok(())
    }
}

/// RTC interrupt handler
///
/// This struct implements the interrupt handler for RTC events.
impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();

        // Check if this is actually a time alarm interrupt
        let status = T::info().regs().isr().read();

        if status.alm_is() {
            T::info().regs().ier().modify(|w| w.set_alm_ie(false));
            T::PERF_INT_WAKE_INCR();
            T::info().wait_cell().wake();
        }
    }
}

impl<'a> SetConfig for Rtc<'a> {
    type Config = Config;
    type ConfigError = SetupError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_configuration(config)
    }
}
