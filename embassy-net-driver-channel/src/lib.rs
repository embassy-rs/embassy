#![no_std]
#![doc = include_str!("../README.md")]

// must go first!
mod fmt;

use core::cell::RefCell;
use core::mem::MaybeUninit;
use core::task::{Context, Poll};

use driver::HardwareAddress;
pub use embassy_net_driver as driver;
use embassy_net_driver::{Capabilities, LinkState, Medium};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::waitqueue::WakerRegistration;
use embassy_sync::zerocopy_channel;

pub struct State<const MTU: usize, const N_RX: usize, const N_TX: usize> {
    rx: [PacketBuf<MTU>; N_RX],
    tx: [PacketBuf<MTU>; N_TX],
    inner: MaybeUninit<StateInner<'static, MTU>>,
}

impl<const MTU: usize, const N_RX: usize, const N_TX: usize> State<MTU, N_RX, N_TX> {
    const NEW_PACKET: PacketBuf<MTU> = PacketBuf::new();

    pub const fn new() -> Self {
        Self {
            rx: [Self::NEW_PACKET; N_RX],
            tx: [Self::NEW_PACKET; N_TX],
            inner: MaybeUninit::uninit(),
        }
    }
}

struct StateInner<'d, const MTU: usize> {
    rx: zerocopy_channel::Channel<'d, NoopRawMutex, PacketBuf<MTU>>,
    tx: zerocopy_channel::Channel<'d, NoopRawMutex, PacketBuf<MTU>>,
    shared: Mutex<NoopRawMutex, RefCell<Shared>>,
}

/// State of the LinkState
struct Shared {
    link_state: LinkState,
    waker: WakerRegistration,
    hardware_address: driver::HardwareAddress,
}

pub struct Runner<'d, const MTU: usize> {
    tx_chan: zerocopy_channel::Receiver<'d, NoopRawMutex, PacketBuf<MTU>>,
    rx_chan: zerocopy_channel::Sender<'d, NoopRawMutex, PacketBuf<MTU>>,
    shared: &'d Mutex<NoopRawMutex, RefCell<Shared>>,
}

#[derive(Clone, Copy)]
pub struct StateRunner<'d> {
    shared: &'d Mutex<NoopRawMutex, RefCell<Shared>>,
}

pub struct RxRunner<'d, const MTU: usize> {
    rx_chan: zerocopy_channel::Sender<'d, NoopRawMutex, PacketBuf<MTU>>,
}

pub struct TxRunner<'d, const MTU: usize> {
    tx_chan: zerocopy_channel::Receiver<'d, NoopRawMutex, PacketBuf<MTU>>,
}

impl<'d, const MTU: usize> Runner<'d, MTU> {
    pub fn split(self) -> (StateRunner<'d>, RxRunner<'d, MTU>, TxRunner<'d, MTU>) {
        (
            StateRunner { shared: self.shared },
            RxRunner { rx_chan: self.rx_chan },
            TxRunner { tx_chan: self.tx_chan },
        )
    }

    pub fn borrow_split(&mut self) -> (StateRunner<'_>, RxRunner<'_, MTU>, TxRunner<'_, MTU>) {
        (
            StateRunner { shared: self.shared },
            RxRunner {
                rx_chan: self.rx_chan.borrow(),
            },
            TxRunner {
                tx_chan: self.tx_chan.borrow(),
            },
        )
    }

    pub fn state_runner(&self) -> StateRunner<'d> {
        StateRunner { shared: self.shared }
    }

    pub fn set_link_state(&mut self, state: LinkState) {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.link_state = state;
            s.waker.wake();
        });
    }

    pub fn set_hardware_address(&mut self, address: driver::HardwareAddress) {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.hardware_address = address;
            s.waker.wake();
        });
    }

    pub async fn rx_buf(&mut self) -> &mut [u8] {
        let p = self.rx_chan.send().await;
        &mut p.buf
    }

    pub fn try_rx_buf(&mut self) -> Option<&mut [u8]> {
        let p = self.rx_chan.try_send()?;
        Some(&mut p.buf)
    }

    pub fn poll_rx_buf(&mut self, cx: &mut Context) -> Poll<&mut [u8]> {
        match self.rx_chan.poll_send(cx) {
            Poll::Ready(p) => Poll::Ready(&mut p.buf),
            Poll::Pending => Poll::Pending,
        }
    }

    pub fn rx_done(&mut self, len: usize) {
        let p = self.rx_chan.try_send().unwrap();
        p.len = len;
        self.rx_chan.send_done();
    }

    pub async fn tx_buf(&mut self) -> &mut [u8] {
        let p = self.tx_chan.receive().await;
        &mut p.buf[..p.len]
    }

    pub fn try_tx_buf(&mut self) -> Option<&mut [u8]> {
        let p = self.tx_chan.try_receive()?;
        Some(&mut p.buf[..p.len])
    }

    pub fn poll_tx_buf(&mut self, cx: &mut Context) -> Poll<&mut [u8]> {
        match self.tx_chan.poll_receive(cx) {
            Poll::Ready(p) => Poll::Ready(&mut p.buf[..p.len]),
            Poll::Pending => Poll::Pending,
        }
    }

    pub fn tx_done(&mut self) {
        self.tx_chan.receive_done();
    }
}

impl<'d> StateRunner<'d> {
    pub fn set_link_state(&self, state: LinkState) {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.link_state = state;
            s.waker.wake();
        });
    }

    pub fn set_ethernet_address(&self, address: [u8; 6]) {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.hardware_address = driver::HardwareAddress::Ethernet(address);
            s.waker.wake();
        });
    }

    pub fn set_ieee802154_address(&self, address: [u8; 8]) {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.hardware_address = driver::HardwareAddress::Ieee802154(address);
            s.waker.wake();
        });
    }
}

impl<'d, const MTU: usize> RxRunner<'d, MTU> {
    pub async fn rx_buf(&mut self) -> &mut [u8] {
        let p = self.rx_chan.send().await;
        &mut p.buf
    }

    pub fn try_rx_buf(&mut self) -> Option<&mut [u8]> {
        let p = self.rx_chan.try_send()?;
        Some(&mut p.buf)
    }

    pub fn poll_rx_buf(&mut self, cx: &mut Context) -> Poll<&mut [u8]> {
        match self.rx_chan.poll_send(cx) {
            Poll::Ready(p) => Poll::Ready(&mut p.buf),
            Poll::Pending => Poll::Pending,
        }
    }

    pub fn rx_done(&mut self, len: usize) {
        let p = self.rx_chan.try_send().unwrap();
        p.len = len;
        self.rx_chan.send_done();
    }
}

impl<'d, const MTU: usize> TxRunner<'d, MTU> {
    pub async fn tx_buf(&mut self) -> &mut [u8] {
        let p = self.tx_chan.receive().await;
        &mut p.buf[..p.len]
    }

    pub fn try_tx_buf(&mut self) -> Option<&mut [u8]> {
        let p = self.tx_chan.try_receive()?;
        Some(&mut p.buf[..p.len])
    }

    pub fn poll_tx_buf(&mut self, cx: &mut Context) -> Poll<&mut [u8]> {
        match self.tx_chan.poll_receive(cx) {
            Poll::Ready(p) => Poll::Ready(&mut p.buf[..p.len]),
            Poll::Pending => Poll::Pending,
        }
    }

    pub fn tx_done(&mut self) {
        self.tx_chan.receive_done();
    }
}

pub fn new<'d, const MTU: usize, const N_RX: usize, const N_TX: usize>(
    state: &'d mut State<MTU, N_RX, N_TX>,
    hardware_address: driver::HardwareAddress,
) -> (Runner<'d, MTU>, Device<'d, MTU>) {
    let mut caps = Capabilities::default();
    caps.max_transmission_unit = MTU;
    caps.medium = match &hardware_address {
        HardwareAddress::Ethernet(_) => Medium::Ethernet,
        HardwareAddress::Ieee802154(_) => Medium::Ieee802154,
        HardwareAddress::Ip => Medium::Ip,
    };

    // safety: this is a self-referential struct, however:
    // - it can't move while the `'d` borrow is active.
    // - when the borrow ends, the dangling references inside the MaybeUninit will never be used again.
    let state_uninit: *mut MaybeUninit<StateInner<'d, MTU>> =
        (&mut state.inner as *mut MaybeUninit<StateInner<'static, MTU>>).cast();
    let state = unsafe { &mut *state_uninit }.write(StateInner {
        rx: zerocopy_channel::Channel::new(&mut state.rx[..]),
        tx: zerocopy_channel::Channel::new(&mut state.tx[..]),
        shared: Mutex::new(RefCell::new(Shared {
            link_state: LinkState::Down,
            hardware_address,
            waker: WakerRegistration::new(),
        })),
    });

    let (rx_sender, rx_receiver) = state.rx.split();
    let (tx_sender, tx_receiver) = state.tx.split();

    (
        Runner {
            tx_chan: tx_receiver,
            rx_chan: rx_sender,
            shared: &state.shared,
        },
        Device {
            caps,
            shared: &state.shared,
            rx: rx_receiver,
            tx: tx_sender,
        },
    )
}

pub struct PacketBuf<const MTU: usize> {
    len: usize,
    buf: [u8; MTU],
}

impl<const MTU: usize> PacketBuf<MTU> {
    pub const fn new() -> Self {
        Self { len: 0, buf: [0; MTU] }
    }
}

pub struct Device<'d, const MTU: usize> {
    rx: zerocopy_channel::Receiver<'d, NoopRawMutex, PacketBuf<MTU>>,
    tx: zerocopy_channel::Sender<'d, NoopRawMutex, PacketBuf<MTU>>,
    shared: &'d Mutex<NoopRawMutex, RefCell<Shared>>,
    caps: Capabilities,
}

impl<'d, const MTU: usize> embassy_net_driver::Driver for Device<'d, MTU> {
    type RxToken<'a> = RxToken<'a, MTU> where Self: 'a ;
    type TxToken<'a> = TxToken<'a, MTU> where Self: 'a ;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if self.rx.poll_receive(cx).is_ready() && self.tx.poll_send(cx).is_ready() {
            Some((RxToken { rx: self.rx.borrow() }, TxToken { tx: self.tx.borrow() }))
        } else {
            None
        }
    }

    /// Construct a transmit token.
    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>> {
        if self.tx.poll_send(cx).is_ready() {
            Some(TxToken { tx: self.tx.borrow() })
        } else {
            None
        }
    }

    /// Get a description of device capabilities.
    fn capabilities(&self) -> Capabilities {
        self.caps.clone()
    }

    fn hardware_address(&self) -> driver::HardwareAddress {
        self.shared.lock(|s| s.borrow().hardware_address)
    }

    fn link_state(&mut self, cx: &mut Context) -> LinkState {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.waker.register(cx.waker());
            s.link_state
        })
    }
}

pub struct RxToken<'a, const MTU: usize> {
    rx: zerocopy_channel::Receiver<'a, NoopRawMutex, PacketBuf<MTU>>,
}

impl<'a, const MTU: usize> embassy_net_driver::RxToken for RxToken<'a, MTU> {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // NOTE(unwrap): we checked the queue wasn't full when creating the token.
        let pkt = unwrap!(self.rx.try_receive());
        let r = f(&mut pkt.buf[..pkt.len]);
        self.rx.receive_done();
        r
    }
}

pub struct TxToken<'a, const MTU: usize> {
    tx: zerocopy_channel::Sender<'a, NoopRawMutex, PacketBuf<MTU>>,
}

impl<'a, const MTU: usize> embassy_net_driver::TxToken for TxToken<'a, MTU> {
    fn consume<R, F>(mut self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // NOTE(unwrap): we checked the queue wasn't full when creating the token.
        let pkt = unwrap!(self.tx.try_send());
        let r = f(&mut pkt.buf[..len]);
        pkt.len = len;
        self.tx.send_done();
        r
    }
}
