use core::fmt;
use core::marker::PhantomData;

use embassy_net_driver::Timestamp as PtpTimestamp;

use super::Instance;

fn from_offset_nanos(offset_nanos: i64) -> (PtpTimestamp, bool) {
    let subtract = offset_nanos < 0;
    let nanos = if subtract {
        offset_nanos.unsigned_abs()
    } else {
        offset_nanos as u64
    };
    let seconds = nanos / 1_000_000_000;
    let (seconds, nanos) = if seconds > u64::from(u32::MAX) {
        (u32::MAX, 999_999_999)
    } else {
        (seconds as u32, (nanos % 1_000_000_000) as u32)
    };

    (PtpTimestamp::from_seconds_and_nanos(seconds, nanos), subtract)
}

/// Ethernet MAC PTP subsecond increment.
///
/// The v1b Ethernet MAC PTP clock is configured in nanosecond rollover
/// mode, where the subsecond increment field is an integer number of
/// nanoseconds.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PtpSubsecondIncrement {
    nanos: u8,
}

impl PtpSubsecondIncrement {
    /// 8 ns timestamp increment.
    pub const NANOS_8: Self = Self { nanos: 8 };

    /// Create a subsecond increment from integer nanoseconds.
    pub const fn from_nanos(nanos: u8) -> Option<Self> {
        if nanos == 0 { None } else { Some(Self { nanos }) }
    }

    /// Return the integer nanosecond increment.
    pub const fn nanos(self) -> u8 {
        self.nanos
    }
}

/// Ethernet MAC PTP clock configuration.
///
/// The v1b Ethernet MAC PTP clock is configured in fine-update,
/// nanosecond rollover mode.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PtpClockConfig {
    /// Timestamp subsecond increment.
    pub subsecond_increment: PtpSubsecondIncrement,
}

impl PtpClockConfig {
    /// Create a PTP clock configuration with a custom subsecond increment.
    pub const fn new(subsecond_increment: PtpSubsecondIncrement) -> Self {
        Self { subsecond_increment }
    }
}

impl Default for PtpClockConfig {
    fn default() -> Self {
        Self::new(PtpSubsecondIncrement::NANOS_8)
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(super) struct ClockRate {
    pub(super) increment: PtpSubsecondIncrement,
    pub(super) nominal_addend: u32,
}

impl ClockRate {
    pub(super) fn from_hclk(hclk_hz: u32, increment: PtpSubsecondIncrement) -> Self {
        let denominator = u64::from(hclk_hz) * u64::from(increment.nanos());
        assert!(denominator != 0);

        let numerator = (1u64 << 32) * 1_000_000_000;
        let addend = (numerator + denominator / 2) / denominator;
        assert!(addend != 0 && addend <= u64::from(u32::MAX));

        Self {
            increment,
            nominal_addend: addend as u32,
        }
    }
}

#[derive(Clone, Copy)]
enum TimeUpdate {
    Init,
    Offset,
}

/// Handle for the Ethernet MAC PTP clock.
#[derive(Debug)]
pub struct PtpClock<T: Instance> {
    rate: ClockRate,
    _peri: PhantomData<T>,
}

/// Read-only provider for the Ethernet MAC PTP time.
pub struct PtpTimeProvider<T: Instance> {
    _peri: PhantomData<T>,
}

impl<T: Instance> Clone for PtpTimeProvider<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<T: Instance> fmt::Debug for PtpTimeProvider<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PtpTimeProvider").finish()
    }
}

impl<T: Instance> PtpTimeProvider<T> {
    const fn new() -> Self {
        Self { _peri: PhantomData }
    }

    /// Read the current MAC PTP time.
    pub fn now(&self) -> PtpTimestamp {
        read_timestamp::<T>()
    }
}

impl<T: Instance> PtpClock<T> {
    pub(crate) fn start(config: PtpClockConfig) -> Self {
        let hclk = unwrap!(unsafe { crate::rcc::get_freqs() }.hclk1.to_hertz());
        let rate = ClockRate::from_hclk(hclk.0, config.subsecond_increment);
        let clock = Self {
            rate,
            _peri: PhantomData,
        };
        clock.configure();
        clock.set_time(PtpTimestamp {
            seconds: 0,
            quarter_nanos: 0,
        });
        debug!(
            "eth ptp clock hclk={} increment={}ns addend={:#010x}",
            hclk.0,
            rate.increment.nanos(),
            rate.nominal_addend
        );
        clock
    }

    /// Return the configured subsecond increment.
    pub fn subsecond_increment(&self) -> PtpSubsecondIncrement {
        self.rate.increment
    }

    /// Return the nominal timestamp addend for the configured HCLK.
    pub fn nominal_addend(&self) -> u32 {
        self.rate.nominal_addend
    }

    /// Return a read-only provider for the MAC PTP time.
    pub fn time_provider(&self) -> PtpTimeProvider<T> {
        PtpTimeProvider::new()
    }

    /// Read the current MAC PTP time.
    pub fn now(&self) -> PtpTimestamp {
        read_timestamp::<T>()
    }

    /// Set the MAC PTP time.
    pub fn set_time(&self, timestamp: PtpTimestamp) {
        apply_time_update::<T>(timestamp, false, TimeUpdate::Init);
    }

    /// Step the MAC PTP time by `offset_nanos`.
    pub fn offset_time(&self, offset_nanos: i64) {
        let (timestamp, subtract) = from_offset_nanos(offset_nanos);
        apply_time_update::<T>(timestamp, subtract, TimeUpdate::Offset);
    }

    /// Set the live MAC PTP addend register.
    ///
    /// This adjusts the running clock frequency without changing
    /// [`PtpClock::nominal_addend`].
    pub fn set_addend(&self, addend: u32) {
        let ptp = T::regs().ethernet_ptp();
        ptp.ptptsar().write(|w| w.set_tsa(addend));
        while ptp.ptptscr().read().ttsaru() {}
        ptp.ptptscr().modify(|w| w.set_ttsaru(true));
        while ptp.ptptscr().read().ttsaru() {}
    }

    fn configure(&self) {
        // Disable timestamp interrupt (v1b uses MACIMR.TSTIM, 1 = masked)
        T::regs()
            .ethernet_mac()
            .macimr()
            .modify(|w| w.set_tstim(crate::pac::eth::vals::Tstim::Masked));

        let ptp = T::regs().ethernet_ptp();
        ptp.ptptscr().modify(|w| w.set_tse(false));
        ptp.ptpssir().write(|w| w.set_stssi(self.rate.increment.nanos()));
        self.set_addend(self.rate.nominal_addend);
        ptp.ptptscr().write(|w| {
            w.set_tse(true);
            w.set_tsfcu(true);
            w.set_tsssr(true);
            w.set_tsptppsv2e(true);
            w.set_tssipv4fe(true);
            w.set_tssipv6fe(true);
            w.set_tsseme(true);
            w.set_tscnt(0b01);
        });
    }
}

fn read_timestamp<T: Instance>() -> PtpTimestamp {
    let ptp = T::regs().ethernet_ptp();
    loop {
        let seconds = ptp.ptptshr().read().sts();
        let nanos = ptp.ptptslr().read().stss();
        if seconds == ptp.ptptshr().read().sts() {
            return PtpTimestamp::from_seconds_and_nanos(seconds, nanos);
        }
    }
}

fn apply_time_update<T: Instance>(timestamp: PtpTimestamp, subtract: bool, update: TimeUpdate) {
    write_time_update::<T>(timestamp, subtract);
    wait_timestamp_init_or_update_clear::<T>();
    T::regs().ethernet_ptp().ptptscr().modify(|w| match update {
        TimeUpdate::Init => w.set_tssti(true),
        TimeUpdate::Offset => w.set_tsstu(true),
    });
    wait_timestamp_init_or_update_clear::<T>();
}

fn write_time_update<T: Instance>(timestamp: PtpTimestamp, subtract: bool) {
    let ptp = T::regs().ethernet_ptp();
    ptp.ptptshur().write(|w| w.set_tsus(timestamp.seconds));
    ptp.ptptslur().write(|w| {
        w.set_tsuss(timestamp.nanos());
        w.set_tsupns(subtract);
    });
}

fn wait_timestamp_init_or_update_clear<T: Instance>() {
    let ptp = T::regs().ethernet_ptp();
    while {
        let control = ptp.ptptscr().read();
        control.tssti() || control.tsstu()
    } {}
}

#[cfg(test)]
mod tests {
    use super::{ClockRate, PtpSubsecondIncrement};

    #[test]
    fn addend_for_200mhz_8ns() {
        let rate = ClockRate::from_hclk(200_000_000, PtpSubsecondIncrement::NANOS_8);
        assert_eq!(rate.nominal_addend, 0xa000_0000);
    }

    #[test]
    fn addend_for_200mhz_6ns() {
        let rate = ClockRate::from_hclk(200_000_000, PtpSubsecondIncrement::from_nanos(6).unwrap());
        assert_eq!(rate.nominal_addend, 0xd555_5555);
    }

    #[test]
    #[should_panic]
    fn addend_rejects_impossible_rate() {
        ClockRate::from_hclk(200_000_000, PtpSubsecondIncrement::from_nanos(5).unwrap());
    }
}
