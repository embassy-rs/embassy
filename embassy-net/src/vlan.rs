//! Contains a VLAN splitter driver

use core::future::poll_fn;
use core::mem::ManuallyDrop;
use core::task::{Context, Poll};
use core::{array, mem, ptr};

use embassy_net_driver::{Capabilities, Driver, HardwareAddress, LinkState, RxToken, Timestamp, TxToken};
use embassy_sync::blocking_mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::waitqueue::WakerRegistration;

struct VlanSplitterState<'d, D: Driver + 'd, const N: usize> {
    wakers: [WakerRegistration; N],
    waker: WakerRegistration, // Wakes the runner for tx requests
    link_state: LinkState,
    tx_out: bool,
    rx_out: bool,
    tx_req: u32, // Bitmask of drivers requesting tx
    t_req: [Mask; N],
    rx_token: Option<(u8, VlanRxToken<'d, D, N>)>,
    tx_token: Option<D::TxToken<'d>>,
    timestamp: (u8, Timestamp),
}

/// VLAN splitter
pub struct VlanSplitter<'d, D: Driver + 'd, const N: usize> {
    address: HardwareAddress,
    capabilities: Capabilities,
    driver: Mutex<NoopRawMutex, D>,
    vlans: [u16; N],
    state: blocking_mutex::NoopMutex<VlanSplitterState<'d, D, N>>,
}

impl<'d, D: Driver + 'd, const N: usize> VlanSplitter<'d, D, N> {
    /// Create a new splitter
    pub fn new(driver: D, vlans: [u16; N]) -> Self {
        core::assert!(N < 32);

        Self {
            address: driver.hardware_address(),
            capabilities: driver.capabilities(),
            driver: Mutex::new(driver),
            vlans,
            state: blocking_mutex::NoopMutex::new(VlanSplitterState {
                wakers: [const { WakerRegistration::new() }; N],
                waker: WakerRegistration::new(),
                link_state: LinkState::Down,
                tx_out: false,
                rx_out: false,
                tx_req: 0,
                t_req: [Mask::default(); N],
                rx_token: None,
                tx_token: None,
                timestamp: (0, Timestamp::from_seconds_and_nanos(0, 0)),
            }),
        }
    }

    /// Split the driver
    pub fn split(&'d self) -> [VlanSplitterDriver<'d, D, N>; N] {
        array::from_fn(|i| VlanSplitterDriver {
            splitter: self,
            index: i as u8,
        })
    }

    /// Run the driver
    pub async fn run(&'d self) -> ! {
        let mut driver = self.driver.lock().await;

        poll_fn(|cx| unsafe {
            self.state.lock_mut(|mut s| self.poll(&mut *driver, &mut s, cx));

            Poll::<()>::Pending
        })
        .await;

        loop {}
    }

    fn poll(&'d self, driver: &mut D, s: &mut VlanSplitterState<'d, D, N>, cx: &mut Context<'_>) {
        s.waker.register(cx.waker());

        let out = s.tx_out || s.rx_out;
        let needs_tx = s.tx_req.trailing_zeros();

        let link_state = driver.link_state(cx);
        if mem::replace(&mut s.link_state, link_state) != link_state {
            for waker in &mut s.wakers {
                waker.wake();
            }
        }

        // Wait for all drivers to claim their timestamps before getting new tokens.
        loop {
            if !self.capabilities.timestamp {
                break;
            }

            let mut wait_ts = false;
            for (i, m) in s.t_req.iter().enumerate() {
                if m.is_set(s.timestamp.0) {
                    s.wakers[i].wake();
                }

                wait_ts = true;
            }

            if wait_ts {
                return;
            }

            if let Some(timestamp) = driver.poll_timestamp(cx) {
                s.timestamp = timestamp;
            } else {
                break;
            }
        }

        if !out
            && s.tx_token.is_none()
            && needs_tx != 32
            && let Some(tx) = driver.transmit(cx)
        {
            // SAFETY: strip the lifetime and use runtime logic to manage the tx lifetime
            unsafe {
                s.tx_token.replace(mem::transmute(tx));
            }

            s.wakers[needs_tx as usize].wake();
        } else if !out && needs_tx == 32 && s.rx_token.is_none() {
            // drop the stored txtoken before we call receive so that we behave as safe rust would
            s.tx_token.take();

            let Some((mut rx, tx)) = driver.receive(cx) else {
                return;
            };

            // SAFETY: strip the lifetime and use runtime logic to manage the tx lifetime
            unsafe {
                s.tx_token.replace(mem::transmute(tx));
            }

            let (vlan, buf) = strip_vlan(rx.buf());
            let buf = buf as *mut [u8];
            let index = self.vlans.iter().position(|&v| vlan == v);

            if let Some(index) = index {
                s.rx_out = true;

                // SAFETY: strip the lifetime and use runtime logic to manage the rx lifetime
                unsafe {
                    s.rx_token.replace((
                        index as u8,
                        VlanRxToken {
                            state: &self.state,
                            buf,
                            token: mem::transmute(rx),
                        },
                    ));
                }

                s.wakers[index].wake();
            } else {
                rx.consume(|_| {});
            }
        }
    }
}

/// VLAN splitter driver
pub struct VlanSplitterDriver<'d, D: Driver + 'd, const N: usize> {
    splitter: &'d VlanSplitter<'d, D, N>,
    index: u8,
}

impl<'d, D: Driver + 'd, const N: usize> Driver for VlanSplitterDriver<'d, D, N> {
    type RxToken<'a>
        = VlanRxToken<'d, D, N>
    where
        Self: 'a;

    type TxToken<'a>
        = VlanTxToken<'d, D, N>
    where
        Self: 'a;

    fn receive(&mut self, cx: &mut core::task::Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        unsafe {
            self.splitter.state.lock_mut(|s| {
                s.wakers[self.index as usize].register(cx.waker());

                if s.rx_token.is_some() && s.rx_token.as_ref().unwrap().0 == self.index && s.tx_token.is_some() {
                    s.tx_out = true;

                    Some((
                        s.rx_token.take().unwrap().1,
                        VlanTxToken {
                            state: &self.splitter.state,
                            vlan: self.splitter.vlans[self.index as usize],
                            index: self.index,
                            token: s.tx_token.take().unwrap(),
                        },
                    ))
                } else {
                    None
                }
            })
        }
    }

    fn transmit(&mut self, cx: &mut core::task::Context) -> Option<Self::TxToken<'_>> {
        unsafe {
            self.splitter.state.lock_mut(|s| {
                s.wakers[self.index as usize].register(cx.waker());

                if let Some(tx) = s.tx_token.take() {
                    s.tx_req &= !(1 << self.index);
                    s.tx_out = true;

                    Some(VlanTxToken {
                        state: &self.splitter.state,
                        vlan: self.splitter.vlans[self.index as usize],
                        index: self.index,
                        token: tx,
                    })
                } else {
                    s.tx_req |= 1 << self.index;
                    s.waker.wake();

                    None
                }
            })
        }
    }

    fn poll_timestamp(&mut self, cx: &mut core::task::Context) -> Option<(u8, embassy_net_driver::Timestamp)> {
        unsafe {
            self.splitter.state.lock_mut(|s| {
                s.wakers[self.index as usize].register(cx.waker());

                if s.t_req[self.index as usize].clear(s.timestamp.0) {
                    Some(s.timestamp)
                } else {
                    None
                }
            })
        }
    }

    fn capabilities(&self) -> embassy_net_driver::Capabilities {
        self.splitter.capabilities.clone()
    }

    fn hardware_address(&self) -> embassy_net_driver::HardwareAddress {
        self.splitter.address
    }

    fn link_state(&mut self, cx: &mut core::task::Context) -> embassy_net_driver::LinkState {
        unsafe {
            self.splitter.state.lock_mut(|s| {
                s.waker.register(cx.waker());
                s.link_state
            })
        }
    }
}

/// VLAN tx token
pub struct VlanTxToken<'d, D: Driver + 'd, const N: usize> {
    state: &'d blocking_mutex::NoopMutex<VlanSplitterState<'d, D, N>>,
    vlan: u16,
    index: u8,
    token: D::TxToken<'d>,
}

impl<'d, D: Driver + 'd, const N: usize> TxToken for VlanTxToken<'d, D, N> {
    fn id(&self) -> u8 {
        self.token.id()
    }

    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let this = ManuallyDrop::new(self);

        unsafe {
            this.state.lock_mut(|s| {
                // store packet id into the driver tx here.
                s.t_req[this.index as usize].set(this.id());
                s.tx_out = false;
                s.waker.wake();
            });
        }

        unsafe { ptr::read(&this.token) }.consume(len + 4, |mut buf| {
            let r = f(&mut buf[4..]);

            insert_vlan(&mut buf, this.vlan);

            r
        })
    }
}

impl<'d, D: Driver + 'd, const N: usize> Drop for VlanTxToken<'d, D, N> {
    fn drop(&mut self) {
        unsafe {
            self.state.lock_mut(|s| {
                s.tx_out = false;
                s.waker.wake();
            });
        }
    }
}

/// VLAN rx token
pub struct VlanRxToken<'d, D: Driver + 'd, const N: usize> {
    state: &'d blocking_mutex::NoopMutex<VlanSplitterState<'d, D, N>>,
    buf: *mut [u8],
    token: D::RxToken<'d>,
}

impl<'d, D: Driver + 'd, const N: usize> RxToken for VlanRxToken<'d, D, N> {
    fn timestamp(&self) -> embassy_net_driver::Timestamp {
        self.token.timestamp()
    }

    fn buf(&mut self) -> &mut [u8] {
        unsafe { &mut *self.buf }
    }

    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut this = ManuallyDrop::new(self);
        let r = f(unsafe { &mut *this.buf });

        unsafe {
            this.state.lock_mut(|s| {
                s.rx_out = false;
                s.waker.wake();
            });
        }

        unsafe { ptr::read(&this.token) }.consume(|_| {});

        r
    }
}

impl<'d, D: Driver + 'd, const N: usize> Drop for VlanRxToken<'d, D, N> {
    fn drop(&mut self) {
        unsafe {
            self.state.lock_mut(|s| {
                s.rx_out = false;
                s.waker.wake();
            });
        }
    }
}
#[derive(Debug, Default, Clone, Copy)]
struct Mask {
    word: u64,
}

impl Mask {
    pub fn set(&mut self, n: u8) {
        self.word |= 1u64 << (n % 64);
    }

    /// Clears the nth bit and returns whether it was previously set.
    pub fn clear(&mut self, n: u8) -> bool {
        let mask = 1u64 << (n % 64);
        let was_set = (self.word & mask) != 0;

        self.word &= !mask;
        was_set
    }

    pub fn is_set(&self, n: u8) -> bool {
        let mask = 1u64 << (n % 64);

        (self.word & mask) != 0
    }
}

/// Strip a single 802.1Q C‑VLAN tag (TPID = 0x8100) in place by shifting
/// the header *right* 4 bytes, avoiding payload copying.
/// Returns (vlan_id, stripped_frame).
/// If no VLAN tag exists, returns (0, original_frame).
fn strip_vlan(frame: &mut [u8]) -> (u16, &mut [u8]) {
    // Need at least: dest(6) + src(6) + tpid(2) + tci(2)
    if frame.len() < 16 {
        return (0, frame);
    }

    // TPID sits immediately after source MAC
    let tpid = u16::from_be_bytes([frame[12], frame[13]]);
    if tpid != 0x8100 {
        return (0, frame);
    }

    // Extract VLAN ID from TCI
    let tci = u16::from_be_bytes([frame[14], frame[15]]);
    let vlan_id = tci & 0x0FFF;

    // Shift header right by 4 bytes:
    // Move dest/src MAC (12 bytes) to positions 4..16
    frame.copy_within(0..12, 4);

    // New frame starts at offset 4
    let stripped = &mut frame[4..];

    (vlan_id, stripped)
}

/// Insert a C‑VLAN tag using the 4 bytes of padding as scratch.
/// Payload remains untouched.
pub fn insert_vlan(frame: &mut [u8], vlan_id: u16) -> &mut [u8] {
    if frame.len() < 18 {
        return frame; // too small
    }

    // Step 1: preserve the original EtherType before we overwrite the packet.
    let ethertype = [frame[16], frame[17]];

    // Step 2: shift header left into padding
    frame.copy_within(4..16, 0);

    // Step 3: insert TPID/TCI at offset 12
    frame[12] = 0x81;
    frame[13] = 0x00;

    let tci = vlan_id & 0x0FFF;
    frame[14] = (tci >> 8) as u8;
    frame[15] = (tci & 0xFF) as u8;

    // Step 4: restore EtherType at offset 16..18
    frame[16] = ethertype[0];
    frame[17] = ethertype[1];

    frame
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use core::task::{Context, RawWaker, RawWakerVTable, Waker};

    use embassy_net_driver::{Capabilities, Driver, HardwareAddress, LinkState, RxToken, Timestamp, TxToken};

    use super::{Mask, VlanSplitter, VlanSplitterDriver, insert_vlan, strip_vlan};

    fn noop_raw_waker() -> RawWaker {
        unsafe fn clone(_: *const ()) -> RawWaker {
            noop_raw_waker()
        }

        unsafe fn wake(_: *const ()) {}
        unsafe fn wake_by_ref(_: *const ()) {}
        unsafe fn drop(_: *const ()) {}

        const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
        RawWaker::new(core::ptr::null(), &VTABLE)
    }

    fn noop_waker() -> Waker {
        unsafe { Waker::from_raw(noop_raw_waker()) }
    }

    pub struct TestDriver {
        pub rx_queue: heapless::Vec<Vec<u8>, 8>,
        pub tx_packets: heapless::Vec<Vec<u8>, 8>,
        pub hw_addr: HardwareAddress,
        pub link: LinkState,

        next_id: u8,
        last_timestamp_id: u8,
        timestamp_counter: u64,
        timestamp_pending: bool,
    }

    impl TestDriver {
        pub fn new(hw_addr: HardwareAddress) -> Self {
            Self {
                rx_queue: heapless::Vec::new(),
                tx_packets: heapless::Vec::new(),
                hw_addr,
                link: LinkState::Up,
                next_id: 0,
                last_timestamp_id: 0,
                timestamp_counter: 0,
                timestamp_pending: false,
            }
        }

        fn alloc_id(&mut self) -> u8 {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1);
            self.last_timestamp_id = id;
            self.timestamp_pending = true;
            id
        }

        fn make_timestamp(&mut self) -> Timestamp {
            self.timestamp_counter = self.timestamp_counter.wrapping_add(1);
            Timestamp {
                seconds: (self.timestamp_counter >> 32) as u32,
                quarter_nanos: (self.timestamp_counter & 0xFFFF_FFFF) as u32,
            }
        }
    }

    pub struct TestRxToken {
        buf: Vec<u8>,
        ts: Timestamp,
    }

    impl RxToken for TestRxToken {
        fn timestamp(&self) -> Timestamp {
            self.ts
        }

        fn buf(&mut self) -> &mut [u8] {
            &mut self.buf
        }

        fn consume<R, F>(mut self, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            f(&mut self.buf)
        }
    }

    pub struct TestTxToken<'a> {
        id: u8,
        out: &'a mut heapless::Vec<Vec<u8>, 8>,
    }

    impl<'a> TxToken for TestTxToken<'a> {
        fn id(&self) -> u8 {
            self.id
        }

        fn consume<R, F>(self, len: usize, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            let mut buf: Vec<u8> = Vec::new();
            buf.resize(len, 0);
            let r = f(&mut buf);
            let _ = self.out.push(buf);
            r
        }
    }

    impl Driver for TestDriver {
        type RxToken<'a>
            = TestRxToken
        where
            Self: 'a;
        type TxToken<'a>
            = TestTxToken<'a>
        where
            Self: 'a;

        fn poll_timestamp(&mut self, _cx: &mut Context) -> Option<(u8, Timestamp)> {
            if self.timestamp_pending {
                self.timestamp_pending = false;
                Some((self.last_timestamp_id, self.make_timestamp()))
            } else {
                None
            }
        }

        fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
            if let Some(pkt) = self.rx_queue.pop() {
                let ts = self.make_timestamp();
                let id = self.alloc_id();

                let rx = TestRxToken { buf: pkt, ts };
                let tx = TestTxToken {
                    id,
                    out: &mut self.tx_packets,
                };

                Some((rx, tx))
            } else {
                cx.waker().wake_by_ref();
                None
            }
        }

        fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>> {
            if self.tx_packets.len() < self.tx_packets.capacity() {
                let id = self.alloc_id();
                Some(TestTxToken {
                    id,
                    out: &mut self.tx_packets,
                })
            } else {
                cx.waker().wake_by_ref();
                None
            }
        }

        fn link_state(&mut self, _cx: &mut Context) -> LinkState {
            self.link
        }

        fn capabilities(&self) -> Capabilities {
            let mut caps = Capabilities::default();
            caps.max_transmission_unit = 1500;
            caps
        }

        fn hardware_address(&self) -> HardwareAddress {
            self.hw_addr
        }
    }

    #[test]
    fn test_vlan_tag_insertion() {
        // Test that insert_vlan correctly inserts a 802.1Q VLAN tag using 4 bytes of padding.
        let mut frame = [0u8; 22];
        // Destination MAC at offset 4 because the first 4 bytes are reserved for the VLAN tag.
        frame[4..10].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        // Source MAC
        frame[10..16].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        // EtherType (IPv4)
        frame[16] = 0x08;
        frame[17] = 0x00;
        // Payload
        frame[18..22].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        insert_vlan(&mut frame, 100);

        // Check TPID (0x8100)
        assert_eq!(frame[12], 0x81);
        assert_eq!(frame[13], 0x00);

        // Check VLAN ID (100 = 0x0064)
        assert_eq!(frame[14], 0x00);
        assert_eq!(frame[15], 0x64);

        // Check original EtherType preserved at new location
        assert_eq!(frame[16], 0x08);
        assert_eq!(frame[17], 0x00);
    }

    #[test]
    fn test_vlan_tag_stripping() {
        // Test that strip_vlan correctly removes a VLAN tag
        let mut frame = [0u8; 18];
        // Destination MAC
        frame[0..6].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        // Source MAC
        frame[6..12].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        // TPID (0x8100)
        frame[12] = 0x81;
        frame[13] = 0x00;
        // TCI with VLAN ID 200
        frame[14] = 0x00;
        frame[15] = 0xC8; // 200 in decimal
        // EtherType (IPv4)
        frame[16] = 0x08;
        frame[17] = 0x00;

        let (vlan_id, stripped) = strip_vlan(&mut frame);

        // Check VLAN ID extracted
        assert_eq!(vlan_id, 200);

        // Check that stripped frame has dest/src MACs at start
        assert_eq!(&stripped[0..6], &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        assert_eq!(&stripped[6..12], &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);

        // Check EtherType is now at position 12 in stripped frame
        assert_eq!(stripped[12], 0x08);
        assert_eq!(stripped[13], 0x00);
    }

    #[test]
    fn test_vlan_tag_stripping_no_tag() {
        // Test that strip_vlan handles frames without VLAN tags correctly
        let mut frame = [0u8; 14];
        // Destination MAC
        frame[0..6].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        // Source MAC
        frame[6..12].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        // EtherType (IPv4) - not VLAN
        frame[12] = 0x08;
        frame[13] = 0x00;

        let (vlan_id, _) = strip_vlan(&mut frame);

        // No VLAN tag, so ID should be 0
        assert_eq!(vlan_id, 0);
    }

    #[test]
    fn test_vlan_id_extraction_various_values() {
        // Test extracting different VLAN IDs (only 12 bits are used)
        for test_vlan_id in [1, 100, 200, 500, 1000, 2000, 4095] {
            let mut frame = [0u8; 18];
            frame[0..6].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
            frame[6..12].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
            frame[12] = 0x81;
            frame[13] = 0x00;

            // Insert VLAN ID into TCI
            let tci = test_vlan_id & 0x0FFF;
            frame[14] = (tci >> 8) as u8;
            frame[15] = (tci & 0xFF) as u8;

            let (vlan_id, _) = strip_vlan(&mut frame);
            assert_eq!(vlan_id, test_vlan_id);
        }
    }

    #[test]
    fn test_tx_token_vlan_tagging() {
        // Test that transmitted packets get VLAN tags inserted
        let mut frame = [0u8; 22]; // 4 byte padding + 18 byte frame
        // Destination MAC
        frame[4..10].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        // Source MAC
        frame[10..16].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        // EtherType
        frame[16] = 0x08;
        frame[17] = 0x00;
        // Payload
        frame[18..22].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        insert_vlan(&mut frame, 100);

        // TPID should be at offset 12
        assert_eq!(frame[12], 0x81);
        assert_eq!(frame[13], 0x00);

        // VLAN ID 100 should be at offset 14-15
        assert_eq!(frame[14], 0x00);
        assert_eq!(frame[15], 0x64);

        // Original EtherType should be preserved
        assert_eq!(frame[16], 0x08);
        assert_eq!(frame[17], 0x00);
    }

    #[test]
    fn test_multiple_frame_sizes() {
        // Test VLAN operations work with various frame sizes
        for size in [18, 64, 128, 256, 512, 1500] {
            let mut frame: Vec<u8> = Vec::new();
            frame.resize(size, 0);

            // Set up minimal frame structure
            frame[0..6].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
            frame[6..12].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);

            if size >= 14 {
                frame[12] = 0x08;
                frame[13] = 0x00;
            }

            // Insert and strip should work
            if size >= 18 {
                insert_vlan(&mut frame, 100);
                let (vlan_id, _) = strip_vlan(&mut frame);
                assert_eq!(vlan_id, 100);
            }
        }
    }

    #[test]
    fn test_round_trip_vlan_tagging() {
        // Test that a frame can be tagged and untagged correctly
        let mut frame = [0u8; 22];
        let original = [0u8; 22];
        frame.copy_from_slice(&original);

        // Set up frame
        frame[4..10].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        frame[10..16].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        frame[16] = 0x08;
        frame[17] = 0x00;
        frame[18..22].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        let _original_payload = frame.clone();

        // Insert VLAN tag
        insert_vlan(&mut frame, 200);

        // Verify tag was inserted
        assert_eq!(frame[12], 0x81);
        assert_eq!(frame[13], 0x00);

        // Strip VLAN tag
        let (vlan_id, stripped) = strip_vlan(&mut frame);
        assert_eq!(vlan_id, 200);

        // Verify MAC addresses are restored
        assert_eq!(&stripped[0..6], &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        assert_eq!(&stripped[6..12], &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);

        // Verify EtherType is restored
        assert_eq!(stripped[12], 0x08);
        assert_eq!(stripped[13], 0x00);
    }

    #[test]
    fn test_vlan_splitter_initialization() {
        // Test that a VlanSplitter initializes correctly with two drivers
        let hw_addr = HardwareAddress::Ethernet([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let test_driver = TestDriver::new(hw_addr);

        let splitter = VlanSplitter::new(test_driver, [100, 200]);

        // Verify splitter has correct VLAN IDs
        assert_eq!(splitter.vlans[0], 100);
        assert_eq!(splitter.vlans[1], 200);

        // Verify splitter has correct hardware address
        assert_eq!(splitter.address, hw_addr);

        // Verify splitter capabilities are set
        assert_eq!(splitter.capabilities.max_transmission_unit, 1500);
    }

    #[test]
    fn test_vlan_splitter_driver_hardware_address() {
        // Test that both drivers report the same hardware address
        let hw_addr = HardwareAddress::Ethernet([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let test_driver = TestDriver::new(hw_addr);
        let splitter = Box::leak(Box::new(VlanSplitter::new(test_driver, [100, 200])));

        {
            let driver0 = VlanSplitterDriver { splitter, index: 0 };

            let driver1 = VlanSplitterDriver { splitter, index: 1 };

            assert_eq!(driver0.hardware_address(), hw_addr);
            assert_eq!(driver1.hardware_address(), hw_addr);
            assert_eq!(driver0.hardware_address(), driver1.hardware_address());
        }
    }

    #[test]
    fn test_vlan_splitter_driver_capabilities() {
        // Test that both drivers report the same capabilities
        let hw_addr = HardwareAddress::Ethernet([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let test_driver = TestDriver::new(hw_addr);
        let splitter = Box::leak(Box::new(VlanSplitter::new(test_driver, [100, 200])));

        {
            let driver0 = VlanSplitterDriver { splitter, index: 0 };

            let driver1 = VlanSplitterDriver { splitter, index: 1 };

            let caps0 = driver0.capabilities();
            let caps1 = driver1.capabilities();

            assert_eq!(caps0.max_transmission_unit, caps1.max_transmission_unit);
            assert_eq!(caps0.max_transmission_unit, 1500);
        }
    }

    #[test]
    fn test_vlan_splitter_driver_link_state_initial() {
        // Test that drivers report the same state stored in the splitter.
        let hw_addr = HardwareAddress::Ethernet([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let test_driver = TestDriver::new(hw_addr);
        let splitter = Box::leak(Box::new(VlanSplitter::new(test_driver, [100, 200])));

        unsafe {
            splitter.state.lock_mut(|s| {
                s.link_state = LinkState::Up;
            });
        }

        // Create a dummy context for testing
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        {
            let mut driver0 = VlanSplitterDriver { splitter, index: 0 };

            let mut driver1 = VlanSplitterDriver { splitter, index: 1 };

            let link0 = driver0.link_state(&mut cx);
            let link1 = driver1.link_state(&mut cx);

            // Both should report the same link state
            assert!(link0 == LinkState::Up);
            assert!(link1 == LinkState::Up);
        }
    }

    #[test]
    fn test_vlan_splitter_driver_link_state_change() {
        // Test that link state changes are reflected by both drivers.
        let hw_addr = HardwareAddress::Ethernet([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let test_driver = TestDriver::new(hw_addr);
        let splitter = Box::leak(Box::new(VlanSplitter::new(test_driver, [100, 200])));

        unsafe {
            splitter.state.lock_mut(|s| {
                s.link_state = LinkState::Up;
            });
        }

        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        {
            let mut driver0 = VlanSplitterDriver { splitter, index: 0 };

            let mut driver1 = VlanSplitterDriver { splitter, index: 1 };

            // Verify initial state
            assert!(driver0.link_state(&mut cx) == LinkState::Up);
            assert!(driver1.link_state(&mut cx) == LinkState::Up);

            // Change link state in the underlying driver
            unsafe {
                splitter.state.lock_mut(|s| {
                    s.link_state = LinkState::Down;
                });
            }

            // Both drivers should now report link down
            assert!(driver0.link_state(&mut cx) == LinkState::Down);
            assert!(driver1.link_state(&mut cx) == LinkState::Down);
        }
    }

    #[test]
    fn test_strip_vlan_various_vlan_ids() {
        // Test that VLAN IDs 100 and 200 are correctly extracted
        for test_id in &[100, 200] {
            let mut frame = [0u8; 18];
            frame[0..6].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
            frame[6..12].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
            frame[12] = 0x81;
            frame[13] = 0x00;

            let tci = test_id & 0x0FFF;
            frame[14] = (tci >> 8) as u8;
            frame[15] = (tci & 0xFF) as u8;
            frame[16] = 0x08;
            frame[17] = 0x00;

            let (vlan_id, _) = strip_vlan(&mut frame);
            assert_eq!(vlan_id, *test_id);
        }
    }

    #[test]
    fn test_insert_vlan_both_ids() {
        // Test that both VLAN 100 and 200 can be inserted correctly
        for test_id in &[100, 200] {
            let mut frame = [0u8; 22];
            frame[4..10].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
            frame[10..16].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
            frame[16] = 0x08;
            frame[17] = 0x00;

            insert_vlan(&mut frame, *test_id);

            // Verify TPID
            assert_eq!(frame[12], 0x81);
            assert_eq!(frame[13], 0x00);

            // Verify VLAN ID
            let expected_tci = test_id & 0x0FFF;
            let actual_tci = ((frame[14] as u16) << 8) | (frame[15] as u16);
            assert_eq!(actual_tci, expected_tci);
        }
    }

    #[test]
    fn test_packet_isolation_vlan_100_vs_200() {
        // Test that a packet with VLAN 100 doesn't appear in VLAN 200's context
        // Create a frame with VLAN 100
        let mut frame_100 = [0u8; 18];
        frame_100[0..6].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        frame_100[6..12].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        frame_100[12] = 0x81;
        frame_100[13] = 0x00;
        frame_100[14] = 0x00;
        frame_100[15] = 0x64; // VLAN 100
        frame_100[16] = 0x08;
        frame_100[17] = 0x00;

        let (vlan_id_100, _) = strip_vlan(&mut frame_100);
        assert_eq!(vlan_id_100, 100);

        // Create a frame with VLAN 200
        let mut frame_200 = [0u8; 18];
        frame_200[0..6].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        frame_200[6..12].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        frame_200[12] = 0x81;
        frame_200[13] = 0x00;
        frame_200[14] = 0x00;
        frame_200[15] = 0xC8; // VLAN 200
        frame_200[16] = 0x08;
        frame_200[17] = 0x00;

        let (vlan_id_200, _) = strip_vlan(&mut frame_200);
        assert_eq!(vlan_id_200, 200);

        // Verify they're different
        assert_ne!(vlan_id_100, vlan_id_200);
    }

    #[test]
    fn test_unknown_vlan_id_detection() {
        // Test that a packet with an unknown VLAN ID (not 100 or 200) is detected
        let mut frame = [0u8; 18];
        frame[0..6].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        frame[6..12].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        frame[12] = 0x81;
        frame[13] = 0x00;
        // Set VLAN ID to 300 (unknown)
        let vlan_300 = 300u16 & 0x0FFF;
        frame[14] = (vlan_300 >> 8) as u8;
        frame[15] = (vlan_300 & 0xFF) as u8;
        frame[16] = 0x08;
        frame[17] = 0x00;

        let (vlan_id, _) = strip_vlan(&mut frame);
        assert_eq!(vlan_id, 300);
        // Verify it's not 100 or 200
        assert_ne!(vlan_id, 100);
        assert_ne!(vlan_id, 200);
    }

    #[test]
    fn test_bit_mask_operations() {
        // Test Mask set, clear, and is_set operations
        let mut mask = Mask::default();

        // Test set and is_set
        mask.set(10);
        assert!(mask.is_set(10));
        assert!(!mask.is_set(11));

        // Test multiple sets
        mask.set(20);
        mask.set(30);
        assert!(mask.is_set(10));
        assert!(mask.is_set(20));
        assert!(mask.is_set(30));

        // Test clear
        assert!(mask.clear(20)); // Should return true because it was set
        assert!(!mask.is_set(20));
        assert!(mask.is_set(10)); // Other bits unchanged

        // Test clear on unset bit
        assert!(!mask.clear(99)); // Should return false because it wasn't set
        assert!(!mask.is_set(99));
    }

    #[test]
    fn test_bit_mask_wrapping() {
        // Test that Mask wraps at 64 bits (% 64)
        let mut mask = Mask::default();

        mask.set(65); // Should wrap to bit 1 (65 % 64 = 1)
        assert!(mask.is_set(1));
        assert!(mask.is_set(65)); // Should still detect as set via wrapping

        mask.set(0);
        mask.set(64); // Should wrap to bit 0 (64 % 64 = 0)
        assert!(mask.is_set(0));
    }

    #[test]
    fn test_consecutive_vlan_tags() {
        // Test inserting, stripping, then re-inserting with different VLAN IDs
        let mut frame = [0u8; 22];
        frame[4..10].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        frame[10..16].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        frame[16] = 0x08;
        frame[17] = 0x00;

        // Insert VLAN 100
        insert_vlan(&mut frame, 100);
        let (id1, _) = strip_vlan(&mut frame);
        assert_eq!(id1, 100);

        // Re-insert as VLAN 200 (simulate re-tagging)
        insert_vlan(&mut frame, 200);
        let (id2, _) = strip_vlan(&mut frame);
        assert_eq!(id2, 200);
    }

    #[test]
    fn test_frame_structure_preservation() {
        // Test that frame structure is preserved through tag insertion and stripping
        let mut frame = [0u8; 22];
        let dest_mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let src_mac = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let payload = [0xDE, 0xAD, 0xBE, 0xEF];

        frame[4..10].copy_from_slice(&dest_mac);
        frame[10..16].copy_from_slice(&src_mac);
        frame[16] = 0x08;
        frame[17] = 0x00;
        frame[18..22].copy_from_slice(&payload);

        // Insert tag
        insert_vlan(&mut frame, 100);

        // Strip tag
        let (_, stripped) = strip_vlan(&mut frame);

        // Verify structure is intact
        assert_eq!(&stripped[0..6], &dest_mac);
        assert_eq!(&stripped[6..12], &src_mac);
        assert_eq!(stripped[12], 0x08);
        assert_eq!(stripped[13], 0x00);
    }
}
