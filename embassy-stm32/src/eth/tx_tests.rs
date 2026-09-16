use core::mem::MaybeUninit;

use xarxa_driver::{PacketBuf, TxTimestamp};

use super::*;
use crate::eth::PacketQueue;

// Model DMA write-back without touching peripheral registers.
fn complete(descriptor: &TDes, id: u32) {
    #[cfg(any(eth_v1b, eth_v1c))]
    {
        descriptor.tdes6.set(id * 10);
        descriptor.tdes7.set(id);
        descriptor.tdes0.set(TXDESC_0_TTSS);
    }
    #[cfg(any(eth_v2, eth_v2a, eth_v2b))]
    {
        descriptor.tdes0.set(id * 10);
        descriptor.tdes1.set(id);
        descriptor.tdes3.set(EMAC_TDES3_TTSS);
    }
}

fn submit(ring: &mut TDesRing<'_>, id: u32, request_timestamp: bool) {
    let mut packet = PacketBuf::try_new().unwrap();
    packet.meta_mut().id = id;
    packet.meta_mut().request_timestamp = request_timestamp;
    ring.buffers[ring.index] = Some(packet);
    complete(&ring.descriptors[ring.index], id);
    ring.index = (ring.index + 1) % ring.len();
    ring.in_flight += 1;
}

#[test]
fn reclaims_without_a_timestamp_consumer_even_when_reports_are_full() {
    let mut queue = MaybeUninit::<PacketQueue<2, 2>>::uninit();
    PacketQueue::init(&mut queue);
    let mut queue = unsafe { queue.assume_init() };
    let mut ring = TDesRing {
        descriptors: &mut queue.tx_desc,
        buffers: &mut queue.tx_buf,
        index: 0,
        in_flight: 0,
        timestamps: heapless::Deque::new(),
    };
    for id in 1..=8 {
        submit(&mut ring, id, true);
        assert!(ring.can_transmit());
        assert_eq!(ring.in_flight, 0);
        assert!(ring.buffers.iter().all(Option::is_none));
    }
    for id in 1..=4 {
        assert_eq!(
            ring.poll_timestamp(),
            Some(TxTimestamp {
                id,
                timestamp: Timestamp::from_seconds_and_nanos(id, id * 10),
            })
        );
    }
    assert_eq!(ring.poll_timestamp(), None);

    // Ring reuse must not produce old reports; ordinary traffic needs no consumer.
    for id in 9..=12 {
        submit(&mut ring, id, false);
        ring.reclaim();
        assert!(ring.buffers.iter().all(Option::is_none));
        assert_eq!(ring.poll_timestamp(), None);
    }
    submit(&mut ring, 13, true);
    assert_eq!(ring.poll_timestamp().unwrap().id, 13);
}

#[test]
fn drains_reports_without_overflow_or_reclaiming_dma_owned_packets() {
    let mut descriptors = [const { TDes::new() }; 7];
    let mut buffers = [const { None }; 7];
    let mut ring = TDesRing {
        descriptors: &mut descriptors,
        buffers: &mut buffers,
        index: 0,
        in_flight: 0,
        timestamps: heapless::Deque::new(),
    };
    for id in 1..=4 {
        submit(&mut ring, id, true);
        ring.reclaim();
    }
    submit(&mut ring, 5, false);
    for id in 6..=10 {
        submit(&mut ring, id, true);
    }
    let index = ring.index;
    submit(&mut ring, 11, true);
    #[cfg(any(eth_v1b, eth_v1c))]
    ring.descriptors[index].tdes0.set(TXDESC_0_OWN);
    #[cfg(any(eth_v2, eth_v2a, eth_v2b))]
    ring.descriptors[index].tdes3.set(EMAC_DES3_OWN);
    for id in [1, 2, 3, 4, 6, 7, 8, 9, 10] {
        assert_eq!(ring.poll_timestamp().unwrap().id, id);
    }
    assert_eq!(ring.poll_timestamp(), None);
    ring.reclaim();
    assert!(ring.buffers[index].is_some());
    complete(&ring.descriptors[index], 11);
    assert_eq!(ring.poll_timestamp().unwrap().id, 11);
    assert_eq!(ring.poll_timestamp(), None);
    assert!(ring.can_transmit());
    assert!(ring.buffers.iter().all(Option::is_none));
}
