use embassy_net_driver::PacketMeta;

use super::ptp::{PtpStorage, PtpTimestamp, RxPtpRing, TxPtpRing};

pub(crate) struct PacketStateStorage<const TX: usize, const RX: usize> {
    #[cfg(feature = "ptp")]
    tx_meta: [PacketMeta; TX],
    #[cfg(feature = "ptp")]
    tx_in_flight: [bool; TX],
    #[cfg(feature = "ptp")]
    rx_meta: [PacketMeta; RX],
    ptp: PtpStorage,
}

impl<const TX: usize, const RX: usize> PacketStateStorage<TX, RX> {
    pub(crate) const fn new(ptp: PtpStorage) -> Self {
        Self {
            #[cfg(feature = "ptp")]
            tx_meta: [const { PacketMeta::EMPTY }; TX],
            #[cfg(feature = "ptp")]
            tx_in_flight: [false; TX],
            #[cfg(feature = "ptp")]
            rx_meta: [const { PacketMeta::EMPTY }; RX],
            ptp,
        }
    }

    pub(crate) fn rings(&mut self) -> (TxPacketStateRing<'_>, RxPacketStateRing<'_>) {
        (
            TxPacketStateRing {
                ptp: self.ptp.tx(),
                #[cfg(feature = "ptp")]
                meta: &mut self.tx_meta,
                #[cfg(feature = "ptp")]
                next_tx: PacketMeta::EMPTY,
                #[cfg(feature = "ptp")]
                in_flight: &mut self.tx_in_flight,
            },
            RxPacketStateRing {
                ptp: self.ptp.rx(),
                #[cfg(feature = "ptp")]
                meta: &mut self.rx_meta,
                #[cfg(feature = "ptp")]
                next_rx_id: 1,
            },
        )
    }
}

pub(crate) struct TxPacketStateRing<'a> {
    ptp: TxPtpRing<'a>,
    #[cfg(feature = "ptp")]
    meta: &'a mut [PacketMeta],
    #[cfg(feature = "ptp")]
    next_tx: PacketMeta,
    #[cfg(feature = "ptp")]
    in_flight: &'a mut [bool],
}

impl TxPacketStateRing<'_> {
    pub(crate) fn set_meta(&mut self, meta: PacketMeta) {
        #[cfg(feature = "ptp")]
        {
            self.next_tx = meta;
        }
        #[cfg(not(feature = "ptp"))]
        let _ = meta;
    }

    pub(crate) fn commit(&mut self, index: usize) {
        #[cfg(feature = "ptp")]
        {
            self.in_flight[index] = true;
            self.meta[index] = self.next_tx;
            self.next_tx = PacketMeta::EMPTY;
        }
        #[cfg(not(feature = "ptp"))]
        let _ = index;
    }

    #[cfg(any(eth_v2, eth_v2a))]
    pub(crate) fn pending(&self, index: usize) -> bool {
        #[cfg(feature = "ptp")]
        {
            self.in_flight[index]
        }
        #[cfg(not(feature = "ptp"))]
        {
            let _ = index;
            false
        }
    }

    #[cfg(any(eth_v2, eth_v2a))]
    pub(crate) fn timestamp_enabled(&self) -> bool {
        self.ptp.enabled() && self.next_id() != 0
    }

    #[cfg(any(eth_v2, eth_v2a))]
    pub(crate) fn next_id(&self) -> u32 {
        #[cfg(feature = "ptp")]
        {
            self.next_tx.id
        }
        #[cfg(not(feature = "ptp"))]
        {
            0
        }
    }

    pub(crate) fn complete(&mut self, index: usize, timestamp: Option<PtpTimestamp>) {
        self.ptp.store(self.id(index), timestamp);
        self.clear(index);
    }

    #[cfg(any(eth_v2, eth_v2a))]
    pub(crate) fn id(&self, index: usize) -> u32 {
        #[cfg(feature = "ptp")]
        {
            self.meta[index].id
        }
        #[cfg(not(feature = "ptp"))]
        {
            let _ = index;
            0
        }
    }

    fn clear(&mut self, index: usize) {
        #[cfg(feature = "ptp")]
        {
            self.in_flight[index] = false;
            self.meta[index] = PacketMeta::EMPTY;
        }
        #[cfg(not(feature = "ptp"))]
        let _ = index;
    }
}

pub(crate) struct RxPacketStateRing<'a> {
    ptp: RxPtpRing<'a>,
    #[cfg(feature = "ptp")]
    meta: &'a mut [PacketMeta],
    #[cfg(feature = "ptp")]
    next_rx_id: u32,
}

impl RxPacketStateRing<'_> {
    pub(crate) fn capture(&mut self, index: usize, timestamp: Option<PtpTimestamp>) {
        let packet_id = self.ensure_rx(index);
        self.ptp.store(packet_id, timestamp);
    }

    pub(crate) fn meta(&self, index: usize) -> PacketMeta {
        #[cfg(feature = "ptp")]
        {
            self.meta[index]
        }
        #[cfg(not(feature = "ptp"))]
        {
            let _ = index;
            PacketMeta::EMPTY
        }
    }

    pub(crate) fn clear(&mut self, index: usize) {
        #[cfg(feature = "ptp")]
        {
            self.meta[index] = PacketMeta::EMPTY;
        }
        #[cfg(not(feature = "ptp"))]
        let _ = index;
    }

    fn ensure_rx(&mut self, index: usize) -> u32 {
        #[cfg(feature = "ptp")]
        {
            if self.meta[index].id == 0 {
                self.meta[index].id = self.next_rx_id;
                self.next_rx_id = self.next_rx_id.wrapping_add(1).max(1);
            }
            self.meta[index].id
        }
        #[cfg(not(feature = "ptp"))]
        {
            let _ = index;
            0
        }
    }
}
