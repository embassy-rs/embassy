#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// must go first!
mod fmt;

use core::cell::RefCell;
use core::task::{Context, Poll, Waker};

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, DynamicReceiver, DynamicSender, TrySendError};
use embassy_sync::waitqueue::WakerRegistration;
pub use xarxa_driver as driver;
use xarxa_driver::{Capabilities, HardwareAddress, LinkState, NotSupported, PacketBuf};

/// Channel state.
///
/// Holds the inbound and outbound packet queues, `N_RX` and `N_TX` packets long.
///
/// The queues hold [`PacketBuf`]s, which come from the global packet pool, so their
/// size here is a handful of bytes per slot, not a whole packet each.
pub struct State<const N_RX: usize, const N_TX: usize> {
    rx: Channel<NoopRawMutex, PacketBuf, N_RX>,
    tx: Channel<NoopRawMutex, PacketBuf, N_TX>,
    shared: Mutex<NoopRawMutex, RefCell<Shared>>,
}

impl<const N_RX: usize, const N_TX: usize> State<N_RX, N_TX> {
    /// Create a new channel state.
    pub const fn new() -> Self {
        Self {
            rx: Channel::new(),
            tx: Channel::new(),
            shared: Mutex::new(RefCell::new(Shared {
                link_state: LinkState::Down,
                waker: WakerRegistration::new(),
                hardware_address: HardwareAddress::Ip,
            })),
        }
    }
}

impl<const N_RX: usize, const N_TX: usize> Default for State<N_RX, N_TX> {
    fn default() -> Self {
        Self::new()
    }
}

struct Shared {
    link_state: LinkState,
    waker: WakerRegistration,
    hardware_address: HardwareAddress,
}

/// Channel runner.
///
/// Holds the shared state and the lower end of channels for inbound and outbound packets.
pub struct Runner<'d> {
    tx_chan: DynamicReceiver<'d, PacketBuf>,
    rx_chan: DynamicSender<'d, PacketBuf>,
    shared: &'d Mutex<NoopRawMutex, RefCell<Shared>>,
}

/// State runner.
///
/// Holds the shared state of the channel such as link state.
#[derive(Clone, Copy)]
pub struct StateRunner<'d> {
    shared: &'d Mutex<NoopRawMutex, RefCell<Shared>>,
}

/// RX runner.
///
/// Holds the lower end of the channel for passing inbound packets up the stack.
pub struct RxRunner<'d> {
    rx_chan: DynamicSender<'d, PacketBuf>,
}

/// TX runner.
///
/// Holds the lower end of the channel for passing outbound packets down the stack.
pub struct TxRunner<'d> {
    tx_chan: DynamicReceiver<'d, PacketBuf>,
}

impl<'d> Runner<'d> {
    /// Split the runner into separate runners for controlling state, rx and tx.
    pub fn split(self) -> (StateRunner<'d>, RxRunner<'d>, TxRunner<'d>) {
        (
            StateRunner { shared: self.shared },
            RxRunner { rx_chan: self.rx_chan },
            TxRunner { tx_chan: self.tx_chan },
        )
    }

    /// Split the runner into separate runners for controlling state, rx and tx borrowing the underlying state.
    pub fn borrow_split(&mut self) -> (StateRunner<'_>, RxRunner<'_>, TxRunner<'_>) {
        (
            StateRunner { shared: self.shared },
            RxRunner { rx_chan: self.rx_chan },
            TxRunner { tx_chan: self.tx_chan },
        )
    }

    /// Create a state runner sharing the state channel.
    pub fn state_runner(&self) -> StateRunner<'d> {
        StateRunner { shared: self.shared }
    }

    /// Set the link state.
    pub fn set_link_state(&mut self, state: LinkState) {
        set_link_state(self.shared, state)
    }

    /// Set the hardware address.
    pub fn set_hardware_address(&mut self, address: HardwareAddress) {
        set_hardware_address(self.shared, address)
    }

    /// Wait until there is space in the inbound queue.
    pub async fn rx_ready(&mut self) {
        core::future::poll_fn(|cx| self.rx_chan.poll_ready_to_send(cx)).await
    }

    /// Poll whether there is space in the inbound queue.
    pub fn poll_rx_ready(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.rx_chan.poll_ready_to_send(cx)
    }

    /// Push a received packet to the stack, waiting for space in the inbound queue.
    pub async fn rx(&mut self, buf: PacketBuf) {
        self.rx_chan.send(buf).await
    }

    /// Push a received packet to the stack, if the inbound queue has space right now.
    ///
    /// If it doesn't, the buffer is handed back in the `Err` variant.
    pub fn try_rx(&mut self, buf: PacketBuf) -> Result<(), PacketBuf> {
        self.rx_chan.try_send(buf).map_err(|TrySendError::Full(buf)| buf)
    }

    /// Wait for a packet the stack wants to transmit.
    pub async fn tx(&mut self) -> PacketBuf {
        self.tx_chan.receive().await
    }

    /// Get a packet the stack wants to transmit, if there is one right now.
    pub fn try_tx(&mut self) -> Option<PacketBuf> {
        self.tx_chan.try_receive().ok()
    }

    /// Poll for a packet the stack wants to transmit.
    pub fn poll_tx(&mut self, cx: &mut Context<'_>) -> Poll<PacketBuf> {
        self.tx_chan.poll_receive(cx)
    }
}

impl StateRunner<'_> {
    /// Set link state.
    pub fn set_link_state(&self, state: LinkState) {
        set_link_state(self.shared, state)
    }

    /// Set the hardware address.
    pub fn set_hardware_address(&self, address: HardwareAddress) {
        set_hardware_address(self.shared, address)
    }

    /// Get the hardware address.
    pub fn get_hardware_address(&self) -> HardwareAddress {
        self.shared.lock(|s| s.borrow().hardware_address)
    }
}

impl RxRunner<'_> {
    /// Wait until there is space in the inbound queue.
    pub async fn rx_ready(&mut self) {
        core::future::poll_fn(|cx| self.rx_chan.poll_ready_to_send(cx)).await
    }

    /// Poll whether there is space in the inbound queue.
    pub fn poll_rx_ready(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.rx_chan.poll_ready_to_send(cx)
    }

    /// Push a received packet to the stack, waiting for space in the inbound queue.
    pub async fn rx(&mut self, buf: PacketBuf) {
        self.rx_chan.send(buf).await
    }

    /// Push a received packet to the stack, if the inbound queue has space right now.
    ///
    /// If it doesn't, the buffer is handed back in the `Err` variant.
    pub fn try_rx(&mut self, buf: PacketBuf) -> Result<(), PacketBuf> {
        self.rx_chan.try_send(buf).map_err(|TrySendError::Full(buf)| buf)
    }
}

impl TxRunner<'_> {
    /// Wait for a packet the stack wants to transmit.
    pub async fn tx(&mut self) -> PacketBuf {
        self.tx_chan.receive().await
    }

    /// Get a packet the stack wants to transmit, if there is one right now.
    pub fn try_tx(&mut self) -> Option<PacketBuf> {
        self.tx_chan.try_receive().ok()
    }

    /// Poll for a packet the stack wants to transmit.
    pub fn poll_tx(&mut self, cx: &mut Context<'_>) -> Poll<PacketBuf> {
        self.tx_chan.poll_receive(cx)
    }
}

fn set_link_state(shared: &Mutex<NoopRawMutex, RefCell<Shared>>, state: LinkState) {
    shared.lock(|s| {
        let s = &mut *s.borrow_mut();
        s.link_state = state;
        s.waker.wake();
    })
}

fn set_hardware_address(shared: &Mutex<NoopRawMutex, RefCell<Shared>>, address: HardwareAddress) {
    shared.lock(|s| {
        let s = &mut *s.borrow_mut();
        s.hardware_address = address;
        s.waker.wake();
    })
}

/// Create a channel.
///
/// Returns a pair of handles for interfacing with the peripheral and the networking stack.
///
/// The runner is interfacing with the peripheral at the lower part of the stack.
/// The device is interfacing with the networking stack on the layer above.
pub fn new<'d, const N_RX: usize, const N_TX: usize>(
    state: &'d mut State<N_RX, N_TX>,
    hardware_address: HardwareAddress,
    mtu: usize,
) -> (Runner<'d>, Device<'d>) {
    assert!(
        mtu <= xarxa_driver::config::PACKET_BUF_SIZE,
        "MTU is larger than the packet buffer size. Raise it with xarxa's `packet-buf-size-N` features."
    );

    let state = &*state;
    state
        .shared
        .lock(|s| s.borrow_mut().hardware_address = hardware_address);

    let mut caps = Capabilities::default();
    caps.medium = hardware_address.medium();
    caps.max_transmission_unit = mtu;

    (
        Runner {
            tx_chan: state.tx.dyn_receiver(),
            rx_chan: state.rx.dyn_sender(),
            shared: &state.shared,
        },
        Device {
            caps,
            shared: &state.shared,
            rx: state.rx.dyn_receiver(),
            tx: state.tx.dyn_sender(),
        },
    )
}

/// Channel device.
///
/// Holds the shared state and upper end of channels for inbound and outbound packets.
pub struct Device<'d> {
    rx: DynamicReceiver<'d, PacketBuf>,
    tx: DynamicSender<'d, PacketBuf>,
    shared: &'d Mutex<NoopRawMutex, RefCell<Shared>>,
    caps: Capabilities,
}

impl driver::Driver for Device<'_> {
    fn capabilities(&self) -> Capabilities {
        let mut caps = self.caps.clone();
        caps.medium = self.hardware_address().medium();
        caps
    }

    fn hardware_address(&self) -> HardwareAddress {
        self.shared.lock(|s| s.borrow().hardware_address)
    }

    fn link_state(&mut self) -> LinkState {
        self.shared.lock(|s| s.borrow().link_state)
    }

    fn register_waker(&mut self, waker: &Waker) -> Result<(), NotSupported> {
        let mut cx = Context::from_waker(waker);
        // These register `waker` on the queues unless they're already ready, in which
        // case the stack will see that on its next poll anyway.
        let _ = self.rx.poll_ready_to_receive(&mut cx);
        let _ = self.tx.poll_ready_to_send(&mut cx);
        self.shared.lock(|s| s.borrow_mut().waker.register(waker));
        Ok(())
    }

    fn receive(&mut self) -> Option<PacketBuf> {
        self.rx.try_receive().ok()
    }

    fn can_transmit(&mut self) -> bool {
        !self.tx.is_full()
    }

    fn transmit(&mut self, buf: PacketBuf) -> Result<(), PacketBuf> {
        self.tx.try_send(buf).map_err(|TrySendError::Full(buf)| buf)
    }
}
