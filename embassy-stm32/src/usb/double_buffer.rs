//! Software state of double-buffered bulk endpoints.

// IN: one packet is advertised while software prepares the other.
// Once hardware flow control is active, releasing two packets before either
// completes can make DTOG_TX == SW_BUF (NAK). The ISR releases the queued
// second packet, also keeping each CTR notification unambiguous.
#[derive(Clone, Copy)]
pub(super) struct TxQueue {
    queued: u8,
    next: u8,
}

impl TxQueue {
    pub(super) const fn new() -> Self {
        Self { queued: 0, next: 0 }
    }

    pub(super) fn free_buffer(&self) -> Option<usize> {
        (self.queued < 2).then_some(self.next as usize)
    }

    // Call only after filling free_buffer(). Returns whether to release it now.
    pub(super) fn enqueue(&mut self) -> bool {
        assert!(self.queued < 2);
        self.queued += 1;
        self.next ^= 1;
        self.queued == 1
    }

    // Returns whether another prepared buffer should be released to hardware.
    pub(super) fn complete(&mut self) -> bool {
        if self.queued != 0 {
            self.queued -= 1;
        }
        self.queued != 0
    }
}

// RM0008 COUNTn_RX: allocation occupies bits 15:10; received bytes are 9:0.
pub(super) const fn received_len(count: u16) -> usize {
    (count & 0x03ff) as usize
}

pub(super) const fn receive_capacity(count: u16) -> u16 {
    count & !0x03ff
}

// OUT: only one receive buffer is advertised at a time, keeping CTR_RX
// unambiguous. The ISR can advertise the other buffer before the application
// reads the first packet. Once both are full, read() returns a buffer to USB.
#[derive(Clone, Copy)]
pub(super) struct RxQueue {
    ready: u8,
    head: u8,
    armed: bool,
}

impl RxQueue {
    pub(super) const fn new() -> Self {
        Self {
            ready: 0,
            head: 0,
            armed: true,
        }
    }

    pub(super) fn next_packet(&self) -> Option<usize> {
        (self.ready != 0).then_some(self.head as usize)
    }

    fn arm_if_free(&mut self) -> bool {
        if !self.armed && self.ready < 2 {
            self.armed = true;
            true
        } else {
            false
        }
    }

    // The advertised buffer completed. Returns whether to advertise another.
    pub(super) fn complete(&mut self) -> bool {
        if !self.armed {
            return false;
        }
        self.armed = false;
        self.ready += 1;
        self.arm_if_free()
    }

    // Call after copying the packet and restoring its receive capacity.
    pub(super) fn consume(&mut self) -> bool {
        assert!(self.ready != 0);
        self.ready -= 1;
        self.head ^= 1;
        self.arm_if_free()
    }
}

#[cfg(test)]
mod tests {
    use super::{RxQueue, TxQueue, receive_capacity, received_len};

    #[test]
    fn two_packets_then_backpressure() {
        let mut queue = TxQueue::new();
        assert_eq!(queue.free_buffer(), Some(0));
        assert!(queue.enqueue());
        assert_eq!(queue.free_buffer(), Some(1));
        assert!(!queue.enqueue()); // defer release until the first CTR_TX
        assert_eq!(queue.free_buffer(), None);
        assert!(queue.complete());
        assert_eq!(queue.free_buffer(), Some(0));
        assert!(!queue.enqueue());
        assert!(queue.complete());
        assert!(!queue.complete());
        assert_eq!(queue.free_buffer(), Some(1));
        assert!(queue.enqueue());
    }

    #[test]
    fn released_buffers_follow_hardware_order() {
        let mut queue = TxQueue::new();
        let mut buffers = [None; 2];
        let mut dtog = 0;
        let mut sw_buf = 0;
        let mut received = 0;
        for packet in 0..100 {
            let buffer = queue.free_buffer().unwrap();
            assert!(buffers[buffer].is_none()); // never overwrite a queued packet
            buffers[buffer] = Some(packet);
            if queue.enqueue() {
                sw_buf ^= 1;
            }
            if queue.free_buffer().is_none() {
                assert_ne!(dtog, sw_buf); // peripheral owns the advertised buffer
                assert_eq!(buffers[dtog].take(), Some(received));
                received += 1;
                dtog ^= 1; // successful IN transaction
                assert_eq!(dtog, sw_buf); // hardware must NAK until ISR handoff
                assert!(queue.complete());
                sw_buf ^= 1;
            }
        }
        assert_ne!(dtog, sw_buf);
        assert_eq!(buffers[dtog].take(), Some(received));
        dtog ^= 1;
        assert!(!queue.complete());
        assert_eq!(dtog, sw_buf);
        assert_eq!(buffers, [None; 2]);
        assert_eq!(queue.free_buffer(), Some(dtog));
    }

    #[test]
    fn completion_before_second_write_and_reset() {
        let mut queue = TxQueue::new();
        assert!(queue.enqueue());
        assert!(!queue.complete());
        assert_eq!(queue.free_buffer(), Some(1));
        assert!(queue.enqueue());
        queue = TxQueue::new(); // reset, disable or clear halt
        assert_eq!(queue.free_buffer(), Some(0));
        assert!(queue.enqueue());
        assert!(!queue.complete());
        assert!(!queue.complete()); // a discarded completion can't underflow
    }

    #[test]
    fn receive_count_preserves_capacity_and_decodes_packets() {
        for (capacity, len) in [(0x8400, 0), (0x8400, 64), (0x1000, 3)] {
            let count = capacity | len;
            assert_eq!(received_len(count), len as usize);
            assert_eq!(receive_capacity(count), capacity);
        }
    }

    #[test]
    fn two_packets_backpressure_and_drain() {
        let mut queue = RxQueue::new();
        assert_eq!(queue.next_packet(), None);
        assert!(queue.complete()); // allow buffer 1 while buffer 0 is waiting
        assert_eq!(queue.next_packet(), Some(0));
        assert!(!queue.complete()); // both full: don't advertise either
        assert!(!queue.complete()); // no third completion is possible
        assert!(queue.consume()); // buffer 0 becomes available to USB
        assert_eq!(queue.next_packet(), Some(1));
        assert!(!queue.consume()); // USB already owns buffer 0: don't release twice
        assert_eq!(queue.next_packet(), None);
        assert!(queue.complete());
        assert_eq!(queue.next_packet(), Some(0));
    }

    #[test]
    fn consumption_before_next_interrupt_and_reset() {
        let mut queue = RxQueue::new();
        assert!(queue.complete());
        assert!(!queue.consume()); // buffer 1 is in flight, leave SW_BUF alone
        assert!(queue.complete());
        assert_eq!(queue.next_packet(), Some(1));
        // A too-small application buffer leaves the packet queued for retry.
        assert_eq!(queue.next_packet(), Some(1));
        queue = RxQueue::new();
        assert_eq!(queue.next_packet(), None);
        assert!(queue.complete());
        assert_eq!(queue.next_packet(), Some(0));
    }

    #[test]
    fn selectors_never_allow_overwriting_an_unread_packet() {
        let mut queue = RxQueue::new();
        let mut buffers = [None; 2];
        let mut dtog = 0;
        let mut sw_buf = 1;
        let mut received = 0;
        for packet in 0..100 {
            assert_ne!(dtog, sw_buf);
            assert!(buffers[dtog].is_none());
            buffers[dtog] = Some(packet);
            dtog ^= 1;
            assert_eq!(dtog, sw_buf); // NAK until completion interrupt
            if queue.complete() {
                sw_buf ^= 1;
            } else {
                let head = queue.next_packet().unwrap();
                assert_eq!(buffers[head].take(), Some(received));
                received += 1;
                assert!(queue.consume());
                sw_buf ^= 1;
            }
        }
        let head = queue.next_packet().unwrap();
        assert_eq!(buffers[head].take(), Some(received));
        assert!(!queue.consume()); // hardware already has a free buffer
        assert_eq!(buffers, [None; 2]);
        assert_eq!(queue.next_packet(), None);
        assert_ne!(dtog, sw_buf);
    }
}
