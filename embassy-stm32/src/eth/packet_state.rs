#[cfg(feature = "ptp")]
mod imp {
    use embassy_net_driver::PacketMeta;

    use super::super::ptp::{PtpTimestamp, PtpTimestampSink, RxPtpRing, TxPtpRing};

    pub(crate) struct PacketState<const TX: usize, const RX: usize> {
        tx_meta: [PacketMeta; TX],
        tx_in_flight: [bool; TX],
        rx_meta: [PacketMeta; RX],
        ptp: PtpTimestampSink,
    }

    impl<const TX: usize, const RX: usize> PacketState<TX, RX> {
        pub(crate) const fn new(ptp: PtpTimestampSink) -> Self {
            Self {
                tx_meta: [const { PacketMeta::EMPTY }; TX],
                tx_in_flight: [false; TX],
                rx_meta: [const { PacketMeta::EMPTY }; RX],
                ptp,
            }
        }

        pub(crate) fn split(&mut self) -> (TxPacketStateRing<'_>, RxPacketStateRing<'_>) {
            (
                TxPacketStateRing {
                    ptp: self.ptp.tx(),
                    meta: &mut self.tx_meta,
                    next_tx: PacketMeta::EMPTY,
                    in_flight: &mut self.tx_in_flight,
                },
                RxPacketStateRing {
                    ptp: self.ptp.rx(),
                    meta: &mut self.rx_meta,
                    next_rx_id: 1,
                },
            )
        }
    }

    pub(crate) struct TxPacketStateRing<'a> {
        ptp: TxPtpRing<'a>,
        meta: &'a mut [PacketMeta],
        next_tx: PacketMeta,
        in_flight: &'a mut [bool],
    }

    impl TxPacketStateRing<'_> {
        pub(crate) fn set_meta(&mut self, meta: PacketMeta) {
            self.next_tx = meta;
        }

        pub(crate) fn commit(&mut self, index: usize) {
            self.in_flight[index] = true;
            self.meta[index] = self.next_tx;
            self.next_tx = PacketMeta::EMPTY;
        }

        pub(crate) fn pending(&self, index: usize) -> bool {
            self.in_flight[index]
        }

        pub(crate) fn timestamp_enabled(&self) -> bool {
            self.ptp.enabled() && self.next_tx.id != 0
        }

        pub(crate) fn next_id(&self) -> u32 {
            self.next_tx.id
        }

        pub(crate) fn complete(&mut self, index: usize, timestamp: Option<PtpTimestamp>) {
            self.ptp.store(self.id(index), timestamp);
            self.clear(index);
        }

        pub(crate) fn id(&self, index: usize) -> u32 {
            self.meta[index].id
        }

        fn clear(&mut self, index: usize) {
            self.in_flight[index] = false;
            self.meta[index] = PacketMeta::EMPTY;
        }
    }

    pub(crate) struct RxPacketStateRing<'a> {
        ptp: RxPtpRing<'a>,
        meta: &'a mut [PacketMeta],
        next_rx_id: u32,
    }

    impl RxPacketStateRing<'_> {
        pub(crate) fn capture(&mut self, index: usize, timestamp: Option<PtpTimestamp>) {
            let packet_id = self.ensure_rx(index);
            self.ptp.store(packet_id, timestamp);
        }

        pub(crate) fn meta(&self, index: usize) -> PacketMeta {
            self.meta[index]
        }

        pub(crate) fn clear(&mut self, index: usize) {
            self.meta[index] = PacketMeta::EMPTY;
        }

        fn ensure_rx(&mut self, index: usize) -> u32 {
            if self.meta[index].id == 0 {
                self.meta[index].id = self.next_rx_id;
                self.next_rx_id = self.next_rx_id.wrapping_add(1).max(1);
            }
            self.meta[index].id
        }
    }
}

#[cfg(not(feature = "ptp"))]
mod imp {
    use core::marker::PhantomData;

    use embassy_net_driver::PacketMeta;

    use super::super::ptp::{PtpTimestamp, PtpTimestampSink};

    pub(crate) struct PacketState<const TX: usize, const RX: usize>;

    impl<const TX: usize, const RX: usize> PacketState<TX, RX> {
        pub(crate) const fn new(_ptp: PtpTimestampSink) -> Self {
            Self
        }

        pub(crate) fn split(&mut self) -> (TxPacketStateRing<'_>, RxPacketStateRing<'_>) {
            (TxPacketStateRing(PhantomData), RxPacketStateRing(PhantomData))
        }
    }

    pub(crate) struct TxPacketStateRing<'a>(PhantomData<&'a mut ()>);

    impl TxPacketStateRing<'_> {
        pub(crate) fn set_meta(&mut self, meta: PacketMeta) {
            let _ = meta;
        }

        pub(crate) fn commit(&mut self, index: usize) {
            let _ = index;
        }
    }

    pub(crate) struct RxPacketStateRing<'a>(PhantomData<&'a mut ()>);

    impl RxPacketStateRing<'_> {
        pub(crate) fn capture(&mut self, index: usize, timestamp: Option<PtpTimestamp>) {
            let _ = (index, timestamp);
        }

        pub(crate) fn meta(&self, index: usize) -> PacketMeta {
            let _ = index;
            PacketMeta::EMPTY
        }

        pub(crate) fn clear(&mut self, index: usize) {
            let _ = index;
        }
    }
}

pub(crate) use imp::{PacketState, RxPacketStateRing, TxPacketStateRing};
