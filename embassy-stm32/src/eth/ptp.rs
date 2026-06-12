/// Ethernet MAC PTP timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PtpTimestamp {
    /// Seconds portion of the MAC timestamp.
    pub seconds: u32,
    /// Nanoseconds portion of the MAC timestamp.
    pub nanos: u32,
}

/// Ethernet MAC PTP subsecond increment.
///
/// The Ethernet clock API configures nanosecond rollover mode, where the
/// `MACSSIR.SSINC` field is the timestamp increment in nanoseconds.
#[cfg(feature = "ptp")]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PtpSubsecondIncrement {
    nanos: u8,
}

#[cfg(feature = "ptp")]
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
/// The clock is configured in fine-update, nanosecond rollover mode.
#[cfg(feature = "ptp")]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PtpClockConfig {
    /// Timestamp increment programmed into `MACSSIR.SSINC`.
    pub subsecond_increment: PtpSubsecondIncrement,
}

#[cfg(feature = "ptp")]
impl PtpClockConfig {
    /// Create a PTP clock configuration with a custom subsecond increment.
    pub const fn new(subsecond_increment: PtpSubsecondIncrement) -> Self {
        Self { subsecond_increment }
    }
}

#[cfg(feature = "ptp")]
impl Default for PtpClockConfig {
    fn default() -> Self {
        Self::new(PtpSubsecondIncrement::NANOS_8)
    }
}

#[cfg(feature = "ptp")]
mod imp {
    use core::marker::PhantomData;
    use core::sync::atomic::{AtomicU32, Ordering};
    use core::task::{Context, Poll};

    use embassy_net_driver::PacketMeta;
    use embassy_sync::waitqueue::AtomicWaker;

    use super::{PtpClockConfig, PtpSubsecondIncrement, PtpTimestamp};
    use crate::eth::Instance;

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

        /// Read the current MAC PTP time.
        pub fn now(&self) -> PtpTimestamp {
            let mac = T::regs().ethernet_mac();
            loop {
                let seconds = mac.macstsr().read().tss();
                let nanos = mac.macstnr().read().tsss();
                if seconds == mac.macstsr().read().tss() {
                    return PtpTimestamp { seconds, nanos };
                }
            }
        }

        /// Set the MAC PTP time.
        pub fn set_time(&self, timestamp: PtpTimestamp) {
            apply_time_update::<T>(timestamp, false, TimeUpdate::Init);
        }

        /// Step the MAC PTP time by `offset_nanos`.
        pub fn offset_time(&self, offset_nanos: i64) {
            let (timestamp, subtract) = timestamp_from_offset(offset_nanos);
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

    /// Shared Ethernet PTP packet timestamp store.
    ///
    /// The `TX` and `RX` const parameters are timestamp history depths. They do
    /// not need to match the Ethernet descriptor ring lengths. Pass this store
    /// to [`PacketQueue::new_with_ptp`](super::super::PacketQueue::new_with_ptp) or
    /// [`PacketQueue::init_with_ptp`](super::super::PacketQueue::init_with_ptp),
    /// then query it with the [`PacketMeta`] attached to RX and TX packets.
    pub struct PtpTimestampStore<const TX: usize, const RX: usize> {
        tx: [TimestampSlot; TX],
        rx: [TimestampSlot; RX],
        tx_waker: AtomicWaker,
    }

    impl<const TX: usize, const RX: usize> PtpTimestampStore<TX, RX> {
        /// Create an empty timestamp store.
        pub const fn new() -> Self {
            Self {
                tx: [const { TimestampSlot::new() }; TX],
                rx: [const { TimestampSlot::new() }; RX],
                tx_waker: AtomicWaker::new(),
            }
        }

        /// Get the transmit timestamp for a packet.
        ///
        /// Returns `None` if the timestamp has not been recorded, if the packet
        /// was not hardware timestamped, or if the history slot was overwritten.
        pub fn tx_timestamp(&self, meta: PacketMeta) -> Option<PtpTimestamp> {
            find_timestamp(&self.tx, meta.id)
        }

        /// Poll until the transmit timestamp for `meta` is available.
        ///
        /// This registers `cx` for future transmit completions. It can remain
        /// pending forever if the packet is not hardware timestamped or if its
        /// history slot is overwritten before it is read. Only one task should
        /// poll transmit timestamps from a store at a time.
        pub fn poll_tx_timestamp(&self, meta: PacketMeta, cx: &mut Context<'_>) -> Poll<PtpTimestamp> {
            if let Some(timestamp) = self.tx_timestamp(meta) {
                return Poll::Ready(timestamp);
            }

            self.tx_waker.register(cx.waker());

            match self.tx_timestamp(meta) {
                Some(timestamp) => Poll::Ready(timestamp),
                None => Poll::Pending,
            }
        }

        /// Get the receive timestamp for a packet.
        ///
        /// Returns `None` if the timestamp was not recorded or if the history
        /// slot was overwritten.
        pub fn rx_timestamp(&self, meta: PacketMeta) -> Option<PtpTimestamp> {
            find_timestamp(&self.rx, meta.id)
        }
    }

    impl<const TX: usize, const RX: usize> Default for PtpTimestampStore<TX, RX> {
        fn default() -> Self {
            Self::new()
        }
    }

    #[derive(Clone, Copy)]
    pub(crate) struct PtpTimestampSink {
        tx: Option<&'static [TimestampSlot]>,
        rx: Option<&'static [TimestampSlot]>,
        tx_waker: Option<&'static AtomicWaker>,
    }

    impl PtpTimestampSink {
        pub(crate) const fn new() -> Self {
            Self {
                tx: None,
                rx: None,
                tx_waker: None,
            }
        }

        pub(crate) const fn from_store<const TX: usize, const RX: usize>(
            store: &'static PtpTimestampStore<TX, RX>,
        ) -> Self {
            Self {
                tx: Some(&store.tx),
                rx: Some(&store.rx),
                tx_waker: Some(&store.tx_waker),
            }
        }

        pub(crate) fn tx(&self) -> TxPtpRing<'static> {
            TxPtpRing {
                slots: self.tx,
                waker: self.tx_waker,
            }
        }

        pub(crate) fn rx(&self) -> RxPtpRing<'static> {
            RxPtpRing { slots: self.rx }
        }
    }

    pub(crate) struct TxPtpRing<'a> {
        slots: Option<&'a [TimestampSlot]>,
        waker: Option<&'a AtomicWaker>,
    }

    impl TxPtpRing<'_> {
        pub(crate) fn enabled(&self) -> bool {
            self.slots.is_some()
        }

        pub(crate) fn store(&self, packet_id: u32, timestamp: Option<PtpTimestamp>) {
            if packet_id != 0 {
                if let (Some(slots), Some(timestamp)) = (self.slots, timestamp) {
                    if !slots.is_empty() {
                        slots[slot_index(slots, packet_id)].store(packet_id, timestamp);
                    }
                }
                if let Some(waker) = self.waker {
                    waker.wake();
                }
            }
        }
    }

    pub(crate) struct RxPtpRing<'a> {
        slots: Option<&'a [TimestampSlot]>,
    }

    impl RxPtpRing<'_> {
        pub(crate) fn store(&self, packet_id: u32, timestamp: Option<PtpTimestamp>) {
            if let (Some(slots), Some(timestamp)) = (self.slots, timestamp) {
                if packet_id != 0 && !slots.is_empty() {
                    slots[slot_index(slots, packet_id)].store(packet_id, timestamp);
                }
            }
        }
    }

    struct TimestampSlot {
        id: AtomicU32,
        seconds: AtomicU32,
        nanos: AtomicU32,
    }

    impl TimestampSlot {
        const fn new() -> Self {
            Self {
                id: AtomicU32::new(0),
                seconds: AtomicU32::new(0),
                nanos: AtomicU32::new(0),
            }
        }

        fn store(&self, packet_id: u32, timestamp: PtpTimestamp) {
            self.id.store(0, Ordering::Release);
            self.seconds.store(timestamp.seconds, Ordering::Relaxed);
            self.nanos.store(timestamp.nanos, Ordering::Relaxed);
            self.id.store(packet_id, Ordering::Release);
        }

        fn load(&self, packet_id: u32) -> Option<PtpTimestamp> {
            if packet_id == 0 || self.id.load(Ordering::Acquire) != packet_id {
                return None;
            }

            let timestamp = PtpTimestamp {
                seconds: self.seconds.load(Ordering::Relaxed),
                nanos: self.nanos.load(Ordering::Relaxed),
            };

            (self.id.load(Ordering::Acquire) == packet_id).then_some(timestamp)
        }
    }

    fn find_timestamp(slots: &[TimestampSlot], packet_id: u32) -> Option<PtpTimestamp> {
        if packet_id == 0 || slots.is_empty() {
            return None;
        }

        slots[slot_index(slots, packet_id)].load(packet_id)
    }

    fn slot_index(slots: &[TimestampSlot], packet_id: u32) -> usize {
        packet_id as usize % slots.len()
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

    fn timestamp_from_offset(offset_nanos: i64) -> (PtpTimestamp, bool) {
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

        (PtpTimestamp { seconds, nanos }, subtract)
    }
}

#[cfg(not(feature = "ptp"))]
mod imp {
    #[derive(Clone, Copy)]
    pub(crate) struct PtpTimestampSink {}

    impl PtpTimestampSink {
        pub(crate) const fn new() -> Self {
            Self {}
        }
    }
}

#[cfg(feature = "ptp")]
pub use imp::PtpClock;
pub(crate) use imp::PtpTimestampSink;
#[cfg(feature = "ptp")]
pub use imp::PtpTimestampStore;
#[cfg(feature = "ptp")]
pub(crate) use imp::{RxPtpRing, TxPtpRing};

#[cfg(all(test, feature = "ptp"))]
mod tests {
    use super::{PtpSubsecondIncrement, imp::ClockRate};

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
