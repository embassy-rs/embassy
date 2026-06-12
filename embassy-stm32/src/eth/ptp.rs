/// Ethernet MAC PTP timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PtpTimestamp {
    /// Seconds portion of the MAC timestamp.
    pub seconds: u32,
    /// Nanoseconds portion of the MAC timestamp.
    pub nanos: u32,
}

#[cfg(feature = "ptp")]
mod imp {
    use core::{
        sync::atomic::{AtomicU32, Ordering},
        task::{Context, Poll},
    };

    use embassy_net_driver::PacketMeta;
    use embassy_sync::waitqueue::AtomicWaker;

    use super::PtpTimestamp;

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

pub(crate) use imp::PtpTimestampSink;
#[cfg(feature = "ptp")]
pub use imp::PtpTimestampStore;
#[cfg(feature = "ptp")]
pub(crate) use imp::{RxPtpRing, TxPtpRing};
