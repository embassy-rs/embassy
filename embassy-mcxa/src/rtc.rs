//! RTC DateTime driver.
use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitCell;

use crate::clocks::{WakeGuard, with_clocks};
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::pac;
use crate::pac::rtc0::cr::Um;

/// RTC interrupt handler.
pub struct InterruptHandler<I: Instance> {
    _phantom: PhantomData<I>,
}

trait SealedInstance {
    fn info() -> &'static Info;
}

/// Trait for RTC peripheral instances
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    type Interrupt: Interrupt;
    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
}

struct Info {
    regs: *const pac::rtc0::RegisterBlock,
    wait_cell: WaitCell,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> &pac::rtc0::RegisterBlock {
        unsafe { &*self.regs }
    }

    #[inline(always)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
    }
}

unsafe impl Sync for Info {}

/// Token for RTC0
pub type Rtc0 = crate::peripherals::RTC0;
impl SealedInstance for crate::peripherals::RTC0 {
    #[inline(always)]
    fn info() -> &'static Info {
        static INFO: Info = Info {
            regs: pac::Rtc0::ptr(),
            wait_cell: WaitCell::new(),
        };
        &INFO
    }
}

impl Instance for crate::peripherals::RTC0 {
    type Interrupt = crate::interrupt::typelevel::RTC;
    const PERF_INT_INCR: fn() = crate::perf_counters::incr_interrupt_rtc0;
    const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::incr_interrupt_rtc0_wake;
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
pub struct RtcDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
#[derive(Copy, Clone)]
pub struct RtcConfig {
    #[allow(dead_code)]
    wakeup_select: bool,
    update_mode: Um,
    #[allow(dead_code)]
    supervisor_access: bool,
    compensation_interval: u8,
    compensation_time: u8,
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
pub fn convert_datetime_to_seconds(datetime: &RtcDateTime) -> u32 {
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
/// RtcDateTime structure with the converted date and time
///
/// # Note
///
/// This function handles leap years correctly.
pub fn convert_seconds_to_datetime(seconds: u32) -> RtcDateTime {
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

    RtcDateTime {
        year,
        month: month as u8,
        day,
        hour,
        minute,
        second,
    }
}

/// Returns default RTC configuration
///
/// # Returns
///
/// RtcConfig with sensible default values:
/// - No wakeup selection
/// - Update mode 0 (immediate updates)
/// - No supervisor access restriction
/// - No compensation
pub fn get_default_config() -> RtcConfig {
    RtcConfig {
        wakeup_select: false,
        update_mode: Um::Um0,
        supervisor_access: false,
        compensation_interval: 0,
        compensation_time: 0,
    }
}
/// Minimal RTC handle for a specific instance I (store the zero-sized token like embassy)
pub struct Rtc<'a> {
    _inst: core::marker::PhantomData<&'a mut ()>,
    info: &'static Info,
    _wg: Option<WakeGuard>,
}

impl<'a> Rtc<'a> {
    /// Create a new instance of the real time clock.
    pub fn new<T: Instance>(
        _inst: Peri<'a, T>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        config: RtcConfig,
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

        // RTC reset
        info.regs().cr().modify(|_, w| w.swr().set_bit());
        info.regs().cr().modify(|_, w| w.swr().clear_bit());
        info.regs().tsr().write(|w| unsafe { w.bits(1) });

        info.regs().cr().modify(|_, w| w.um().variant(config.update_mode));

        info.regs().tcr().modify(|_, w| unsafe {
            w.cir()
                .bits(config.compensation_interval)
                .tcr()
                .bits(config.compensation_time)
        });

        // Enable RTC interrupt
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self {
            _inst: core::marker::PhantomData,
            info,
            _wg: WakeGuard::for_power(&clk.power),
        }
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
    pub fn set_datetime(&self, datetime: RtcDateTime) {
        let seconds = convert_datetime_to_seconds(&datetime);
        self.info.regs().tsr().write(|w| unsafe { w.bits(seconds) });
    }

    /// Get the current date and time
    ///
    /// # Returns
    ///
    /// Current date and time as RtcDateTime
    ///
    /// # Note
    ///
    /// Reads the current Unix timestamp from the time seconds register and converts it.
    pub fn get_datetime(&self) -> RtcDateTime {
        let seconds = self.info.regs().tsr().read().bits();
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
    pub fn set_alarm(&self, alarm: RtcDateTime) {
        let seconds = convert_datetime_to_seconds(&alarm);

        self.info.regs().tar().write(|w| unsafe { w.bits(0) });
        let mut timeout = 10000;
        while self.info.regs().tar().read().bits() != 0 && timeout > 0 {
            timeout -= 1;
        }

        self.info.regs().tar().write(|w| unsafe { w.bits(seconds) });

        let mut timeout = 10000;
        while self.info.regs().tar().read().bits() != seconds && timeout > 0 {
            timeout -= 1;
        }

        self.set_interrupt(RtcInterruptEnable::RTC_ALARM_INTERRUPT_ENABLE);
    }

    /// Get the current alarm date and time
    ///
    /// # Returns
    ///
    /// Alarm date and time as RtcDateTime
    ///
    /// # Note
    ///
    /// Reads the alarm timestamp from the time alarm register and converts it.
    pub fn get_alarm(&self) -> RtcDateTime {
        let alarm_seconds = self.info.regs().tar().read().bits();
        convert_seconds_to_datetime(alarm_seconds)
    }

    /// Start the RTC time counter
    ///
    /// # Note
    ///
    /// Sets the Time Counter Enable (TCE) bit in the status register.
    pub fn start(&self) {
        self.info.regs().sr().modify(|_, w| w.tce().set_bit());
    }

    /// Stop the RTC time counter
    ///
    /// # Note
    ///
    /// Clears the Time Counter Enable (TCE) bit in the status register.
    pub fn stop(&self) {
        self.info.regs().sr().modify(|_, w| w.tce().clear_bit());
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
            self.info.regs().ier().modify(|_, w| w.tiie().tiie_1());
        }
        if (RtcInterruptEnable::RTC_TIME_OVERFLOW_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|_, w| w.toie().toie_1());
        }
        if (RtcInterruptEnable::RTC_ALARM_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|_, w| w.taie().taie_1());
        }
        if (RtcInterruptEnable::RTC_SECONDS_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|_, w| w.tsie().tsie_1());
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
            self.info.regs().ier().modify(|_, w| w.tiie().tiie_0());
        }
        if (RtcInterruptEnable::RTC_TIME_OVERFLOW_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|_, w| w.toie().toie_0());
        }
        if (RtcInterruptEnable::RTC_ALARM_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|_, w| w.taie().taie_0());
        }
        if (RtcInterruptEnable::RTC_SECONDS_INTERRUPT_ENABLE & mask) != 0 {
            self.info.regs().ier().modify(|_, w| w.tsie().tsie_0());
        }
    }

    /// Clear the alarm interrupt flag
    ///
    /// # Note
    ///
    /// This function clears the Time Alarm Interrupt Enable bit.
    pub fn clear_alarm_flag(&self) {
        self.info.regs().ier().modify(|_, w| w.taie().clear_bit());
    }

    /// Wait for an RTC alarm to trigger.
    ///
    /// # Arguments
    ///
    /// * `alarm` - The date and time when the alarm should trigger
    ///
    /// This function will wait until the RTC alarm is triggered.
    /// If no alarm is scheduled, it will wait indefinitely until one is scheduled and triggered.
    pub async fn wait_for_alarm(&mut self, alarm: RtcDateTime) {
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
        unsafe {
            let rtc = &*pac::Rtc0::ptr();
            // Check if this is actually a time alarm interrupt
            let sr = rtc.sr().read();
            if sr.taf().bit_is_set() {
                rtc.ier().modify(|_, w| w.taie().clear_bit());
                T::PERF_INT_WAKE_INCR();
                T::info().wait_cell().wake();
            }
        }
    }
}
