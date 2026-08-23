//! Contains a VLAN splitter driver

use core::future::poll_fn;
use core::mem::ManuallyDrop;
use core::task::{Context, Poll};
use core::{array, mem, ptr};

use embassy_net_driver::{Capabilities, Driver, HardwareAddress, LinkState, RxToken, Timestamp, TxTimestamp, TxToken};
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
    timestamp: TxTimestamp,
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
                timestamp: TxTimestamp {
                    id: 0,
                    timestamp: Timestamp {
                        seconds: 0,
                        quarter_nanos: 0,
                    },
                },
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

        #[cfg(feature = "packetmeta-timestamp")]
        // Wait for all drivers to claim their timestamps before getting new tokens.
        loop {
            if !self.capabilities.timestamp {
                break;
            }

            let mut wait_ts = false;
            for (i, m) in s.t_req.iter().enumerate() {
                if m.is_set(s.timestamp.id) {
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
                            #[cfg(feature = "packetmeta-id")]
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
                        #[cfg(feature = "packetmeta-id")]
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

    fn poll_timestamp(&mut self, cx: &mut core::task::Context) -> Option<TxTimestamp> {
        unsafe {
            self.splitter.state.lock_mut(|s| {
                s.wakers[self.index as usize].register(cx.waker());

                if s.t_req[self.index as usize].clear(s.timestamp.id) {
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
    #[cfg(feature = "packetmeta-id")]
    index: u8,
    token: D::TxToken<'d>,
}

impl<'d, D: Driver + 'd, const N: usize> TxToken for VlanTxToken<'d, D, N> {
    fn set_meta(&mut self, meta: embassy_net_driver::PacketMeta) {
        #[cfg(feature = "packetmeta-id")]
        unsafe {
            self.state.lock_mut(|s| {
                // store packet id into the driver tx here.
                s.t_req[self.index as usize].set(meta.id);
                s.tx_out = false;
            });
        }

        self.token.set_meta(meta);
    }

    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let this = ManuallyDrop::new(self);

        unsafe {
            this.state.lock_mut(|s| {
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
    fn meta(&self) -> embassy_net_driver::PacketMeta {
        self.token.meta()
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
    #[cfg(feature = "packetmeta-id")]
    pub fn set(&mut self, n: u32) {
        self.word |= 1u64 << (n % 64);
    }

    /// Clears the nth bit and returns whether it was previously set.
    pub fn clear(&mut self, n: u32) -> bool {
        let mask = 1u64 << (n % 64);
        let was_set = (self.word & mask) != 0;

        self.word &= !mask;
        was_set
    }

    #[cfg(feature = "packetmeta-timestamp")]
    pub fn is_set(&self, n: u32) -> bool {
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
