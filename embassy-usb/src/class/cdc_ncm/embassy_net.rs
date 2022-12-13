use core::cell::RefCell;
use core::mem::MaybeUninit;
use core::task::Context;

use embassy_futures::select::{select, Either};
use embassy_net::device::{Device as DeviceTrait, DeviceCapabilities, LinkState, Medium};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::waitqueue::WakerRegistration;
use embassy_usb_driver::Driver;

use super::{CdcNcmClass, Receiver, Sender};

pub struct State<'d, const MTU: usize, const N_RX: usize, const N_TX: usize> {
    rx: [PacketBuf<MTU>; N_RX],
    tx: [PacketBuf<MTU>; N_TX],
    inner: MaybeUninit<StateInner<'d, MTU>>,
}

impl<'d, const MTU: usize, const N_RX: usize, const N_TX: usize> State<'d, MTU, N_RX, N_TX> {
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
    link_state: Mutex<NoopRawMutex, RefCell<LinkStateState>>,
}

/// State of the LinkState
struct LinkStateState {
    state: LinkState,
    waker: WakerRegistration,
}

pub struct Runner<'d, D: Driver<'d>, const MTU: usize> {
    tx_usb: Sender<'d, D>,
    tx_chan: zerocopy_channel::Receiver<'d, NoopRawMutex, PacketBuf<MTU>>,
    rx_usb: Receiver<'d, D>,
    rx_chan: zerocopy_channel::Sender<'d, NoopRawMutex, PacketBuf<MTU>>,
    link_state: &'d Mutex<NoopRawMutex, RefCell<LinkStateState>>,
}

impl<'d, D: Driver<'d>, const MTU: usize> Runner<'d, D, MTU> {
    pub async fn run(mut self) -> ! {
        let rx_fut = async move {
            loop {
                trace!("WAITING for connection");
                self.link_state.lock(|s| {
                    let s = &mut *s.borrow_mut();
                    s.state = LinkState::Down;
                    s.waker.wake();
                });

                self.rx_usb.wait_connection().await.unwrap();

                trace!("Connected");
                self.link_state.lock(|s| {
                    let s = &mut *s.borrow_mut();
                    s.state = LinkState::Up;
                    s.waker.wake();
                });

                loop {
                    let p = self.rx_chan.send().await;
                    match self.rx_usb.read_packet(&mut p.buf).await {
                        Ok(n) => {
                            p.len = n;
                            self.rx_chan.send_done();
                        }
                        Err(e) => {
                            warn!("error reading packet: {:?}", e);
                            break;
                        }
                    };
                }
            }
        };
        let tx_fut = async move {
            loop {
                let p = self.tx_chan.recv().await;
                if let Err(e) = self.tx_usb.write_packet(&p.buf[..p.len]).await {
                    warn!("Failed to TX packet: {:?}", e);
                }
                self.tx_chan.recv_done();
            }
        };
        match select(rx_fut, tx_fut).await {
            Either::First(x) => x,
            Either::Second(x) => x,
        }
    }
}

impl<'d, D: Driver<'d>> CdcNcmClass<'d, D> {
    pub fn into_embassy_net_device<const MTU: usize, const N_RX: usize, const N_TX: usize>(
        self,
        state: &'d mut State<'d, MTU, N_RX, N_TX>,
        ethernet_address: [u8; 6],
    ) -> (Runner<'d, D, MTU>, Device<'d, MTU>) {
        let (tx_usb, rx_usb) = self.split();

        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1514; // 1500 IP + 14 ethernet header
        caps.medium = Medium::Ethernet;

        let state = state.inner.write(StateInner {
            rx: zerocopy_channel::Channel::new(&mut state.rx[..]),
            tx: zerocopy_channel::Channel::new(&mut state.tx[..]),
            link_state: Mutex::new(RefCell::new(LinkStateState {
                state: LinkState::Down,
                waker: WakerRegistration::new(),
            })),
        });

        let (rx_sender, rx_receiver) = state.rx.split();
        let (tx_sender, tx_receiver) = state.tx.split();

        (
            Runner {
                tx_usb,
                tx_chan: tx_receiver,
                rx_usb,
                rx_chan: rx_sender,
                link_state: &state.link_state,
            },
            Device {
                caps,
                ethernet_address,
                link_state: &state.link_state,
                rx: rx_receiver,
                tx: tx_sender,
            },
        )
    }
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
    link_state: &'d Mutex<NoopRawMutex, RefCell<LinkStateState>>,
    caps: DeviceCapabilities,
    ethernet_address: [u8; 6],
}

impl<'d, const MTU: usize> DeviceTrait for Device<'d, MTU> {
    type RxToken<'a> = RxToken<'a, MTU> where Self: 'a ;
    type TxToken<'a> = TxToken<'a, MTU> where Self: 'a ;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if self.rx.poll_recv(cx).is_ready() && self.tx.poll_send(cx).is_ready() {
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
    fn capabilities(&self) -> DeviceCapabilities {
        self.caps.clone()
    }

    fn ethernet_address(&self) -> [u8; 6] {
        self.ethernet_address
    }

    fn link_state(&mut self, cx: &mut Context) -> LinkState {
        self.link_state.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.waker.register(cx.waker());
            s.state
        })
    }
}

pub struct RxToken<'a, const MTU: usize> {
    rx: zerocopy_channel::Receiver<'a, NoopRawMutex, PacketBuf<MTU>>,
}

impl<'a, const MTU: usize> embassy_net::device::RxToken for RxToken<'a, MTU> {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // NOTE(unwrap): we checked the queue wasn't full when creating the token.
        let pkt = unwrap!(self.rx.try_recv());
        let r = f(&mut pkt.buf[..pkt.len]);
        self.rx.recv_done();
        r
    }
}

pub struct TxToken<'a, const MTU: usize> {
    tx: zerocopy_channel::Sender<'a, NoopRawMutex, PacketBuf<MTU>>,
}

impl<'a, const MTU: usize> embassy_net::device::TxToken for TxToken<'a, MTU> {
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

mod zerocopy_channel {
    use core::cell::RefCell;
    use core::future::poll_fn;
    use core::marker::PhantomData;
    use core::task::{Context, Poll};

    use embassy_sync::blocking_mutex::raw::RawMutex;
    use embassy_sync::blocking_mutex::Mutex;
    use embassy_sync::waitqueue::WakerRegistration;

    pub struct Channel<'a, M: RawMutex, T> {
        buf: *mut T,
        phantom: PhantomData<&'a mut T>,
        state: Mutex<M, RefCell<State>>,
    }

    impl<'a, M: RawMutex, T> Channel<'a, M, T> {
        pub fn new(buf: &'a mut [T]) -> Self {
            let len = buf.len();
            assert!(len != 0);

            Self {
                buf: buf.as_mut_ptr(),
                phantom: PhantomData,
                state: Mutex::new(RefCell::new(State {
                    len,
                    front: 0,
                    back: 0,
                    full: false,
                    send_waker: WakerRegistration::new(),
                    recv_waker: WakerRegistration::new(),
                })),
            }
        }

        pub fn split(&mut self) -> (Sender<'_, M, T>, Receiver<'_, M, T>) {
            (Sender { channel: self }, Receiver { channel: self })
        }
    }

    pub struct Sender<'a, M: RawMutex, T> {
        channel: &'a Channel<'a, M, T>,
    }

    impl<'a, M: RawMutex, T> Sender<'a, M, T> {
        pub fn borrow(&mut self) -> Sender<'_, M, T> {
            Sender { channel: self.channel }
        }

        pub fn try_send(&mut self) -> Option<&mut T> {
            self.channel.state.lock(|s| {
                let s = &mut *s.borrow_mut();
                match s.push_index() {
                    Some(i) => Some(unsafe { &mut *self.channel.buf.add(i) }),
                    None => None,
                }
            })
        }

        pub fn poll_send(&mut self, cx: &mut Context) -> Poll<&mut T> {
            self.channel.state.lock(|s| {
                let s = &mut *s.borrow_mut();
                match s.push_index() {
                    Some(i) => Poll::Ready(unsafe { &mut *self.channel.buf.add(i) }),
                    None => {
                        s.recv_waker.register(cx.waker());
                        Poll::Pending
                    }
                }
            })
        }

        pub async fn send(&mut self) -> &mut T {
            let i = poll_fn(|cx| {
                self.channel.state.lock(|s| {
                    let s = &mut *s.borrow_mut();
                    match s.push_index() {
                        Some(i) => Poll::Ready(i),
                        None => {
                            s.recv_waker.register(cx.waker());
                            Poll::Pending
                        }
                    }
                })
            })
            .await;
            unsafe { &mut *self.channel.buf.add(i) }
        }

        pub fn send_done(&mut self) {
            self.channel.state.lock(|s| s.borrow_mut().push_done())
        }
    }
    pub struct Receiver<'a, M: RawMutex, T> {
        channel: &'a Channel<'a, M, T>,
    }

    impl<'a, M: RawMutex, T> Receiver<'a, M, T> {
        pub fn borrow(&mut self) -> Receiver<'_, M, T> {
            Receiver { channel: self.channel }
        }

        pub fn try_recv(&mut self) -> Option<&mut T> {
            self.channel.state.lock(|s| {
                let s = &mut *s.borrow_mut();
                match s.pop_index() {
                    Some(i) => Some(unsafe { &mut *self.channel.buf.add(i) }),
                    None => None,
                }
            })
        }

        pub fn poll_recv(&mut self, cx: &mut Context) -> Poll<&mut T> {
            self.channel.state.lock(|s| {
                let s = &mut *s.borrow_mut();
                match s.pop_index() {
                    Some(i) => Poll::Ready(unsafe { &mut *self.channel.buf.add(i) }),
                    None => {
                        s.send_waker.register(cx.waker());
                        Poll::Pending
                    }
                }
            })
        }

        pub async fn recv(&mut self) -> &mut T {
            let i = poll_fn(|cx| {
                self.channel.state.lock(|s| {
                    let s = &mut *s.borrow_mut();
                    match s.pop_index() {
                        Some(i) => Poll::Ready(i),
                        None => {
                            s.send_waker.register(cx.waker());
                            Poll::Pending
                        }
                    }
                })
            })
            .await;
            unsafe { &mut *self.channel.buf.add(i) }
        }

        pub fn recv_done(&mut self) {
            self.channel.state.lock(|s| s.borrow_mut().pop_done())
        }
    }

    struct State {
        len: usize,

        /// Front index. Always 0..=(N-1)
        front: usize,
        /// Back index. Always 0..=(N-1).
        back: usize,

        /// Used to distinguish "empty" and "full" cases when `front == back`.
        /// May only be `true` if `front == back`, always `false` otherwise.
        full: bool,

        send_waker: WakerRegistration,
        recv_waker: WakerRegistration,
    }

    impl State {
        fn increment(&self, i: usize) -> usize {
            if i + 1 == self.len {
                0
            } else {
                i + 1
            }
        }

        fn is_full(&self) -> bool {
            self.full
        }

        fn is_empty(&self) -> bool {
            self.front == self.back && !self.full
        }

        fn push_index(&mut self) -> Option<usize> {
            match self.is_full() {
                true => None,
                false => Some(self.back),
            }
        }

        fn push_done(&mut self) {
            assert!(!self.is_full());
            self.back = self.increment(self.back);
            if self.back == self.front {
                self.full = true;
            }
            self.send_waker.wake();
        }

        fn pop_index(&mut self) -> Option<usize> {
            match self.is_empty() {
                true => None,
                false => Some(self.front),
            }
        }

        fn pop_done(&mut self) {
            assert!(!self.is_empty());
            self.front = self.increment(self.front);
            self.full = false;
            self.recv_waker.wake();
        }
    }
}
