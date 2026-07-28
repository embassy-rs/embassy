use core::fmt;
use core::marker::PhantomData;

use super::Instance;
use crate::eth::ptp::PtpTimestamp;

/// Ethernet MAC PTP subsecond increment.
///
/// The v2/v2a Ethernet MAC PTP clock is configured in nanosecond rollover
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
/// The v2/v2a Ethernet MAC PTP clock is configured in fine-update,
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
        clock.set_time(PtpTimestamp { seconds: 0, nanos: 0 });
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
        let (timestamp, subtract) = PtpTimestamp::from_offset_nanos(offset_nanos);
        apply_time_update::<T>(timestamp, subtract, TimeUpdate::Offset);
    }

    /// Set the live MAC PTP addend register.
    ///
    /// This adjusts the running clock frequency without changing
    /// [`PtpClock::nominal_addend`].
    pub fn set_addend(&self, addend: u32) {
        let mac = T::regs().ethernet_mac();
        mac.mactsar().write(|w| w.set_tsar(addend));
        while mac.mactscr().read().tsaddreg() {}
        mac.mactscr().modify(|w| w.set_tsaddreg(true));
        while mac.mactscr().read().tsaddreg() {}
    }

    fn configure(&self) {
        let mac = T::regs().ethernet_mac();
        mac.macier().modify(|w| w.set_tsie(false));
        mac.mactscr().modify(|w| w.set_tsena(false));
        mac.macssir().write(|w| {
            w.set_snsinc(0);
            w.set_ssinc(self.rate.increment.nanos());
        });
        self.set_addend(self.rate.nominal_addend);
        mac.mactscr().write(|w| {
            w.set_tsena(true);
            w.set_tscfupdt(true);
            w.set_tsctrlssr(true);
            w.set_tsver2ena(true);
            w.set_tsipv4ena(true);
            w.set_tsipv6ena(true);
            w.set_tsevntena(true);
            w.set_snaptypsel(0b01);
            w.set_txtsstsm(true);
        });
    }
}

fn read_timestamp<T: Instance>() -> PtpTimestamp {
    let mac = T::regs().ethernet_mac();
    loop {
        let seconds = mac.macstsr().read().tss();
        let nanos = mac.macstnr().read().tsss();
        if seconds == mac.macstsr().read().tss() {
            return PtpTimestamp { seconds, nanos };
        }
    }
}

fn apply_time_update<T: Instance>(timestamp: PtpTimestamp, subtract: bool, update: TimeUpdate) {
    write_time_update::<T>(timestamp, subtract);
    wait_timestamp_init_or_update_clear::<T>();
    T::regs().ethernet_mac().mactscr().modify(|w| match update {
        TimeUpdate::Init => w.set_tsinit(true),
        TimeUpdate::Offset => w.set_tsupdt(true),
    });
    wait_timestamp_init_or_update_clear::<T>();
}

fn write_time_update<T: Instance>(timestamp: PtpTimestamp, subtract: bool) {
    let mac = T::regs().ethernet_mac();
    mac.macstsur().write(|w| w.set_tss(timestamp.seconds));
    mac.macstnur().write(|w| {
        w.set_tsss(timestamp.nanos);
        w.set_addsub(subtract);
    });
}

fn wait_timestamp_init_or_update_clear<T: Instance>() {
    let mac = T::regs().ethernet_mac();
    while {
        let control = mac.mactscr().read();
        control.tsinit() || control.tsupdt()
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
