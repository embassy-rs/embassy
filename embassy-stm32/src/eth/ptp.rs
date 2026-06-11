/// Ethernet MAC packet timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PtpTimestamp {
    /// Seconds portion of the MAC timestamp.
    pub seconds: u32,
    /// Nanoseconds portion of the MAC timestamp.
    pub nanos: u32,
}

#[cfg(all(feature = "ptp", any(eth_v2, eth_v2a)))]
mod imp {
    use core::sync::atomic::{AtomicU32, Ordering};

    use embassy_net_driver::PacketMeta;

    use super::PtpTimestamp;

    /// Shared PTP packet timestamp history.
    ///
    /// The `TX` and `RX` const parameters are timestamp history depths. They do
    /// not need to match the Ethernet descriptor ring lengths.
    pub struct PtpTimestampStore<const TX: usize, const RX: usize> {
        tx: [TimestampSlot; TX],
        rx: [TimestampSlot; RX],
    }

    impl<const TX: usize, const RX: usize> PtpTimestampStore<TX, RX> {
        /// Create an empty timestamp store.
        pub const fn new() -> Self {
            Self {
                tx: [const { TimestampSlot::new() }; TX],
                rx: [const { TimestampSlot::new() }; RX],
            }
        }

        /// Get the transmit timestamp recorded for `meta`.
        ///
        /// Returns `None` if the timestamp has not been recorded, if the packet
        /// was not hardware timestamped, or if the history slot was overwritten.
        pub fn tx_timestamp(&self, meta: PacketMeta) -> Option<PtpTimestamp> {
            find_timestamp(&self.tx, meta.id)
        }

        /// Get the receive timestamp recorded for `meta`.
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
    pub(crate) struct PtpStorage {
        tx: Option<&'static [TimestampSlot]>,
        rx: Option<&'static [TimestampSlot]>,
    }

    impl PtpStorage {
        pub(crate) const fn new() -> Self {
            Self { tx: None, rx: None }
        }

        pub(crate) const fn new_with_store<const TX: usize, const RX: usize>(
            store: &'static PtpTimestampStore<TX, RX>,
        ) -> Self {
            Self {
                tx: Some(&store.tx),
                rx: Some(&store.rx),
            }
        }

        pub(crate) fn tx(&self) -> TxPtpRing<'static> {
            TxPtpRing { slots: self.tx }
        }

        pub(crate) fn rx(&self) -> RxPtpRing<'static> {
            RxPtpRing { slots: self.rx }
        }
    }

    pub(crate) struct TxPtpRing<'a> {
        slots: Option<&'a [TimestampSlot]>,
    }

    impl TxPtpRing<'_> {
        pub(crate) fn enabled(&self) -> bool {
            self.slots.is_some()
        }

        pub(crate) fn store(&self, packet_id: u32, timestamp: Option<PtpTimestamp>) {
            if let (Some(slots), Some(timestamp)) = (self.slots, timestamp) {
                if packet_id != 0 && !slots.is_empty() {
                    slots[slot_index(slots, packet_id)].store(packet_id, timestamp);
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
}

#[cfg(not(feature = "ptp"))]
mod imp {
    use core::marker::PhantomData;

    use super::PtpTimestamp;

    #[derive(Clone, Copy)]
    pub(crate) struct PtpStorage {}

    impl PtpStorage {
        pub(crate) const fn new() -> Self {
            Self {}
        }

        pub(crate) fn tx(&self) -> TxPtpRing<'static> {
            TxPtpRing { _lifetime: PhantomData }
        }

        pub(crate) fn rx(&self) -> RxPtpRing<'static> {
            RxPtpRing { _lifetime: PhantomData }
        }
    }

    pub(crate) struct TxPtpRing<'a> {
        _lifetime: PhantomData<&'a ()>,
    }

    impl TxPtpRing<'_> {
        #[cfg(any(eth_v2, eth_v2a))]
        pub(crate) fn enabled(&self) -> bool {
            false
        }

        pub(crate) fn store(&self, packet_id: u32, timestamp: Option<PtpTimestamp>) {
            let _ = (packet_id, timestamp);
        }
    }

    pub(crate) struct RxPtpRing<'a> {
        _lifetime: PhantomData<&'a ()>,
    }

    impl RxPtpRing<'_> {
        pub(crate) fn store(&self, packet_id: u32, timestamp: Option<PtpTimestamp>) {
            let _ = (packet_id, timestamp);
        }
    }
}

#[cfg(all(feature = "ptp", any(eth_v2, eth_v2a)))]
pub use imp::PtpTimestampStore;
pub(crate) use imp::{PtpStorage, RxPtpRing, TxPtpRing};
