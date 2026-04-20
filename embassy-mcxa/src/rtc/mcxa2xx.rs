//! RTC DateTime driver.
use core::convert::Infallible;
use core::marker::PhantomData;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::{Peri, PeripheralType};
#[cfg(feature = "embedded-mcu-hal")]
use embedded_mcu_hal::time::{Datetime, DatetimeClock, DatetimeClockError, DatetimeFields, Month};
use maitake_sync::WaitCell;

use crate::clocks::{WakeGuard, with_clocks};
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::pac;
use crate::pac::rtc2xx::{Swr, Um};

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
    regs: pac::rtc2xx::Rtc,
    wait_cell: WaitCell,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::rtc2xx::Rtc {
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
    type Interrupt = crate::interrupt::typelevel::RTC;
}

/// Number of days in a standard year
const DAYS_IN_A_YEAR: u32 = 365;
/// Number of seconds in a day
const SECONDS_IN_A_DAY: u32 = 86400;
/// Number of seconds in an hour
const SECONDS_IN_A_HOUR: u32 = 3600;
/// Number of seconds in a minute
const SECONDS_IN_A_MINUTE: u32 = 60;
/// Unix epoch start year
const YEAR_RANGE_START: u16 = 1970;

/// Date and time structure for RTC operations
#[derive(Debug, Clone, Copy)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Copy, Clone)]
pub struct Config {
    #[allow(dead_code)]
    wakeup_select: bool,
    update_mode: Um,
    #[allow(dead_code)]
    supervisor_access: bool,
    compensation_interval: u8,
    compensation_time: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wakeup_select: false,
            update_mode: Um::Um0,
            supervisor_access: false,
            compensation_interval: 0,
            compensation_time: 0,
        }
    }
}

/// RTC interrupt enable flags
#[derive(Copy, Clone)]
pub struct RtcInterruptEnable;

impl RtcInterruptEnable {
    pub const RTC_TIME_INVALID_INTERRUPT_ENABLE: u32 = 1 << 0;
    pub const RTC_TIME_OVERFLOW_INTERRUPT_ENABLE: u32 = 1 << 1;
    pub const RTC_ALARM_INTERRUPT_ENABLE: u32 = 1 << 2;
    pub const RTC_SECONDS_INTERRUPT_ENABLE: u32 = 1 << 4;
}

/// Converts a DateTime structure to Unix timestamp (seconds since 1970-01-01)
///
/// # Arguments
///
/// * `datetime` - The date and time to convert
///
/// # Returns
///
/// Unix timestamp as u32
///
/// # Note
///
/// This function handles leap years correctly.
pub fn convert_datetime_to_seconds(datetime: &DateTime) -> u32 {
    let month_days: [u16; 13] = [0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

    let mut seconds = (datetime.year as u32 - 1970) * DAYS_IN_A_YEAR;
    seconds += (datetime.year as u32 / 4) - (1970 / 4);
    seconds += month_days[datetime.month as usize] as u32;
    seconds += datetime.day as u32 - 1;

    if (datetime.year & 3 == 0) && (datetime.month <= 2) {
        seconds -= 1;
    }

    seconds = seconds * SECONDS_IN_A_DAY
        + (datetime.hour as u32 * SECONDS_IN_A_HOUR)
        + (datetime.minute as u32 * SECONDS_IN_A_MINUTE)
        + datetime.second as u32;

    seconds
}

/// Converts Unix timestamp to DateTime structure
///
/// # Arguments
///
/// * `seconds` - Unix timestamp (seconds since 1970-01-01)
///
/// # Returns
///
/// DateTime structure with the converted date and time
///
/// # Note
///
/// This function handles leap years correctly.
pub fn convert_seconds_to_datetime(seconds: u32) -> DateTime {
    let mut seconds_remaining = seconds;
    let mut days = seconds_remaining / SECONDS_IN_A_DAY + 1;
    seconds_remaining %= SECONDS_IN_A_DAY;

    let hour = (seconds_remaining / SECONDS_IN_A_HOUR) as u8;
    seconds_remaining %= SECONDS_IN_A_HOUR;
    let minute = (seconds_remaining / SECONDS_IN_A_MINUTE) as u8;
    let second = (seconds_remaining % SECONDS_IN_A_MINUTE) as u8;

    let mut year = YEAR_RANGE_START;
    let mut days_in_year = DAYS_IN_A_YEAR;

    while days > days_in_year {
        days -= days_in_year;
        year += 1;

        days_in_year = if year.is_multiple_of(4) {
            DAYS_IN_A_YEAR + 1
        } else {
            DAYS_IN_A_YEAR
        };
    }

    let days_per_month = [
        31,
        if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) {
            29
        } else {
            28
        },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 1;
    for (m, month_days) in days_per_month.iter().enumerate() {
        let m = m + 1;
        if days <= *month_days as u32 {
            month = m;
            break;
        } else {
            days -= *month_days as u32;
        }
    }

    let day = days as u8;

    DateTime {
        year,
        month: month as u8,
        day,
        hour,
        minute,
        second,
    }
}

/// Minimal RTC handle for a specific instance I (store the zero-sized token like embassy)
pub struct Rtc<'a> {
    _inst: core::marker::PhantomData<&'a mut ()>,
    info: &'static Info,
    _freq: u32,
    _wg: Option<WakeGuard>,
}

impl<'a> Rtc<'a> {
    /// Create a new instance of the real time clock.
    pub fn new<T: Instance>(
        _inst: Peri<'a, T>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        config: Config,
    ) -> Self {
        let info = T::info();

        // The RTC is NOT gated by the MRCC, but we DO need to make sure the 16k clock
        // on the vsys domain is active
        let clocks = with_clocks(|c| c.clk_16k_vsys.clone());
        let clk = match clocks {
            None => panic!("Clocks have not been initialized"),
            Some(None) => panic!("Clocks initialized, but clk_16k_vsys not active"),
            Some(Some(clk)) => clk,
        };

        let mut inst = Self {
            info,
            _inst: PhantomData,
            _freq: clk.frequency,
            _wg: WakeGuard::for_power(&clk.power),
        };

        inst.set_configuration(&config);

        // Enable RTC interrupt
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        inst
    }

    fn set_configuration(&mut self, config: &Config) {
        self.info.regs().cr().modify(|w| w.set_swr(Swr::Swr1));
        self.info.regs().cr().modify(|w| w.set_swr(Swr::Swr0));
        self.info.regs().tsr().write(|w| w.0 = 1);

        self.info.regs().cr().modify(|w| w.set_um(config.update_mode));

        self.info.regs().tcr().modify(|w| {
            w.set_cir(config.compensation_interval);
            w.set_tcr(config.compensation_time);
        });
    }

    /// Set the current date and time
    ///
    /// # Arguments
    ///
    /// * `datetime` - The date and time to set
    ///
    /// # Note
    ///
    /// The datetime is converted to Unix timestamp and written to the time seconds register.
    pub fn set_datetime(&self, datetime: DateTime) {
        let seconds = convert_datetime_to_seconds(&datetime);
        self.info.regs().tsr().write(|w| w.0 = seconds);
    }

    /// Get the current date and time
    ///
    /// # Returns
    ///
    /// Current date and time as DateTime
    ///
    /// # Note
    ///
    /// Reads the current Unix timestamp from the time seconds register and converts it.
    pub fn get_datetime(&self) -> DateTime {
        let seconds = self.info.regs().tsr().read().0;
        convert_seconds_to_datetime(seconds)
    }

    /// Set the alarm date and time
    ///
    /// # Arguments
    ///
    /// * `alarm` - The date and time when the alarm should trigger
    ///
    /// # Note
    ///
    /// This function:
    /// - Clears any existing alarm by writing 0 to the alarm register
    /// - Waits for the clear operation to complete
    /// - Sets the new alarm time
    /// - Waits for the write operation to complete
    /// - Uses timeouts to prevent infinite loops
    /// - Enables the alarm interrupt after setting
    pub fn set_alarm(&self, alarm: DateTime) {
        let seconds = convert_datetime_to_seconds(&alarm);

        self.info.regs().tar().write(|w| w.0 = 0);
        let mut timeout = 10000;
        while self.info.regs().tar().read().0 != 0 && timeout > 0 {
            timeout -= 1;
        }

        self.info.regs().tar().write(|w| w.0 = seconds);

        let mut timeout = 10000;
        while self.info.regs().tar().read().0 != seconds && timeout > 0 {
            timeout -= 1;
        }

        self.set_interrupt(RtcInterruptEnable::RTC_ALARM_INTERRUPT_ENABLE);
    }

    /// Get the current alarm date and time
    ///
    /// # Returns
    ///
    /// Alarm date and time as DateTime
    ///
    /// # Note
    ///
    /// Reads the alarm timestamp from the time alarm register and converts it.
    pub fn get_alarm(&self) -> DateTime {
        let alarm_seconds = self.info.regs().tar().read().0;
        convert_seconds_to_datetime(alarm_seconds)
    }

    /// Start the RTC time counter
    ///
    /// # Note
    ///
    /// Sets the Time Counter Enable (TCE) bit in the status register.
    pub fn start(&self) {
        self.info.regs().sr().modify(|w| w.set_tce(true));
    }

    /// Stop the RTC time counter
    ///
    /// # Note
    ///
    /// Clears the Time Counter Enable (TCE) bit in the status register.
    pub fn stop(&self) {
        self.info.regs().sr().modify(|w| w.set_tce(false));
    }

    /// Enable specific RTC interrupts
    ///
    /// # Arguments
    ///
    /// * `mask` - Bitmask of interrupts to enable (use RtcInterruptEnable constants)
    ///
    /// # Note
    ///
    /// This function enables the specified interrupt types and resets the alarm occurred flag.
    /// Available interrupts:
    /// - Time Invalid Interrupt
    /// - Time Overflow Interrupt
    /// - Alarm Interrupt
    /// - Seconds Interrupt
    pub fn set_interrupt(&self, mask: u32) {
        if (RtcInterruptEnable::RTC_TIME_INVALID_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|w| w.set_tiie(true));
        }
        if (RtcInterruptEnable::RTC_TIME_OVERFLOW_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|w| w.set_toie(true));
        }
        if (RtcInterruptEnable::RTC_ALARM_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|w| w.set_taie(true));
        }
        if (RtcInterruptEnable::RTC_SECONDS_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|w| w.set_tsie(true));
        }
    }

    /// Disable specific RTC interrupts
    ///
    /// # Arguments
    ///
    /// * `mask` - Bitmask of interrupts to disable (use RtcInterruptEnable constants)
    ///
    /// # Note
    ///
    /// This function disables the specified interrupt types.
    pub fn disable_interrupt(&self, mask: u32) {
        if (RtcInterruptEnable::RTC_TIME_INVALID_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|w| w.set_tiie(false));
        }
        if (RtcInterruptEnable::RTC_TIME_OVERFLOW_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|w| w.set_toie(false));
        }
        if (RtcInterruptEnable::RTC_ALARM_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|w| w.set_taie(false));
        }
        if (RtcInterruptEnable::RTC_SECONDS_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|w| w.set_tsie(false));
        }
    }

    /// Clear the alarm interrupt flag
    ///
    /// # Note
    ///
    /// This function clears the Time Alarm Interrupt Enable bit.
    pub fn clear_alarm_flag(&self) {
        self.info.regs().ier().modify(|w| w.set_taie(false));
    }

    /// Wait for an RTC alarm to trigger.
    ///
    /// # Arguments
    ///
    /// * `alarm` - The date and time when the alarm should trigger
    ///
    /// This function will wait until the RTC alarm is triggered.
    /// If no alarm is scheduled, it will wait indefinitely until one is scheduled and triggered.
    pub async fn wait_for_alarm(&mut self, alarm: DateTime) {
        let wait = self.info.wait_cell().subscribe().await;

        self.set_alarm(alarm);
        self.start();

        // REVISIT: propagate error?
        let _ = wait.await;

        // Clear the interrupt and disable the alarm after waking up
        self.disable_interrupt(RtcInterruptEnable::RTC_ALARM_INTERRUPT_ENABLE);
    }
}

/// RTC interrupt handler
///
/// This struct implements the interrupt handler for RTC events.
impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        let rtc = pac::RTC0;
        // Check if this is actually a time alarm interrupt
        let sr = rtc.sr().read();
        if sr.taf() {
            rtc.ier().modify(|w| w.set_taie(false));
            T::PERF_INT_WAKE_INCR();
            T::info().wait_cell().wake();
        }
    }
}

#[cfg(feature = "embedded-mcu-hal")]
impl<'a> DatetimeClock for Rtc<'a> {
    fn now(&self) -> Result<Datetime, DatetimeClockError> {
        let dt = self.get_datetime();
        let month = match dt.month {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => return Err(DatetimeClockError::UnsupportedDatetime),
        };

        let fields = DatetimeFields {
            year: dt.year,
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
            year: datetime.year(),
            month: datetime.month() as u8,
            day: datetime.day(),
            hour: datetime.hour(),
            minute: datetime.minute(),
            second: datetime.second(),
        };
        self.set_datetime(dt);
        Ok(())
    }

    fn resolution_hz(&self) -> u32 {
        self._freq
    }
}

impl<'a> SetConfig for Rtc<'a> {
    type Config = Config;
    type ConfigError = Infallible;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_configuration(config);
        Ok(())
    }
}
