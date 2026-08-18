//! Bluetooth (BLE) HCI transport over the ESP-Hosted `HCI` interface.
//!
//! The ESP coprocessor exposes its Bluetooth controller as an HCI interface multiplexed
//! over the same SPI/SDIO link used for WiFi. This module provides a [`BtDriver`] that
//! implements [`bt_hci::transport::Transport`], so it can be plugged directly into a
//! host stack such as `trouble-host`.

use core::cell::RefCell;
use core::convert::Infallible;
use core::future::Future;
use core::mem::MaybeUninit;

use bt_hci_transport::{PacketToController, ReadHciError};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::zerocopy_channel;
use embedded_io_async::ErrorKind;

/// Maximum size of a single HCI packet (including the 1-byte H4 packet type indicator).
const BT_HCI_MTU: usize = 1024;

/// A buffer holding a single HCI packet.
///
/// `buf[0]` is the H4 packet type indicator (Command / ACL / Sync / Event), the rest is
/// the packet payload. This mirrors how the controller exchanges packets with the host.
pub(crate) struct BtPacketBuf {
    pub(crate) len: usize,
    pub(crate) buf: [u8; BT_HCI_MTU],
}

impl BtPacketBuf {
    /// Create a new, empty packet buffer.
    pub const fn new() -> Self {
        Self {
            len: 0,
            buf: [0; BT_HCI_MTU],
        }
    }
}

/// State backing the Bluetooth driver. Must outlive the driver and runner.
pub(crate) struct BtState {
    rx: [BtPacketBuf; 4],
    tx: [BtPacketBuf; 4],
    inner: MaybeUninit<BtStateInner<'static>>,
}

impl BtState {
    /// Create a new, empty Bluetooth state holder.
    pub const fn new() -> Self {
        Self {
            rx: [const { BtPacketBuf::new() }; 4],
            tx: [const { BtPacketBuf::new() }; 4],
            inner: MaybeUninit::uninit(),
        }
    }
}

impl Default for BtState {
    fn default() -> Self {
        Self::new()
    }
}

struct BtStateInner<'d> {
    rx: zerocopy_channel::Channel<'d, NoopRawMutex, BtPacketBuf>,
    tx: zerocopy_channel::Channel<'d, NoopRawMutex, BtPacketBuf>,
}

/// Bluetooth HCI transport driver.
///
/// Implements [`bt_hci::transport::Transport`]; hand it to a host stack to send and
/// receive HCI packets.
pub struct BtDriver<'d> {
    rx: RefCell<zerocopy_channel::Receiver<'d, NoopRawMutex, BtPacketBuf>>,
    tx: RefCell<zerocopy_channel::Sender<'d, NoopRawMutex, BtPacketBuf>>,
}

/// The runner half of the Bluetooth driver. Driven by the esp-hosted [`Runner`](crate::Runner).
pub(crate) struct BtRunner<'d> {
    /// HCI packets queued by the host stack, to be transmitted to the controller.
    pub(crate) tx_chan: zerocopy_channel::Receiver<'d, NoopRawMutex, BtPacketBuf>,
    /// HCI packets received from the controller, to be delivered to the host stack.
    rx_chan: zerocopy_channel::Sender<'d, NoopRawMutex, BtPacketBuf>,
}

/// Split a [`BtState`] into a runner and a driver.
pub(crate) fn new<'d>(state: &'d mut BtState) -> (BtRunner<'d>, BtDriver<'d>) {
    // safety: this is a self-referential struct, however:
    // - it can't move while the `'d` borrow is active.
    // - when the borrow ends, the dangling references inside the MaybeUninit will never be used again.
    let state_uninit: *mut MaybeUninit<BtStateInner<'d>> =
        (&mut state.inner as *mut MaybeUninit<BtStateInner<'static>>).cast();
    let state = unsafe { &mut *state_uninit }.write(BtStateInner {
        rx: zerocopy_channel::Channel::new(&mut state.rx[..]),
        tx: zerocopy_channel::Channel::new(&mut state.tx[..]),
    });

    let (rx_sender, rx_receiver) = state.rx.split();
    let (tx_sender, tx_receiver) = state.tx.split();

    (
        BtRunner {
            tx_chan: tx_receiver,
            rx_chan: rx_sender,
        },
        BtDriver {
            rx: RefCell::new(rx_receiver),
            tx: RefCell::new(tx_sender),
        },
    )
}

impl<'d> BtRunner<'d> {
    /// Deliver an HCI packet received from the controller to the host stack.
    ///
    /// `payload` is a complete HCI packet whose first byte is the H4 packet type indicator,
    /// exactly as the ESP-Hosted firmware delivers it over the HCI interface.
    pub(crate) fn rx(&mut self, payload: &[u8]) {
        match self.rx_chan.try_send() {
            Some(mut buf) => {
                let n = payload.len().min(BT_HCI_MTU);
                buf.buf[..n].copy_from_slice(&payload[..n]);
                buf.len = n;
                buf.send_done();
            }
            None => warn!("bluetooth rx queue full, dropping packet"),
        }
    }
}

/// HCI transport error.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub enum Error {
    /// I/O error.
    Io(ErrorKind),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl core::error::Error for Error {}

impl<'d> embedded_io_async::ErrorType for BtDriver<'d> {
    type Error = Error;
}

impl embedded_io_async::Error for Error {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Io(e) => *e,
        }
    }
}

// ------------ bt-hci-transport ----------------

impl From<ReadHciError<Infallible>> for Error {
    fn from(e: ReadHciError<Infallible>) -> Self {
        match e {
            ReadHciError::InvalidValue => Error::Io(ErrorKind::InvalidData),
            ReadHciError::BufferTooSmall => Error::Io(ErrorKind::InvalidInput),
            ReadHciError::Read(e) => match e {
                embedded_io_async::ReadExactError::UnexpectedEof => Error::Io(ErrorKind::BrokenPipe),
                embedded_io_async::ReadExactError::Other(_) => unreachable!(),
            },
        }
    }
}

impl<'d> bt_hci_transport::Transport for BtDriver<'d> {
    fn read<'a, P: bt_hci_transport::PacketToHost<'a>>(
        &self,
        rx: &'a mut [u8],
    ) -> impl Future<Output = Result<P, Self::Error>> {
        async {
            let ch = &mut *self.rx.borrow_mut();
            let buf = ch.receive().await;
            assert!(buf.len < rx.len());

            let mut reader = &buf.buf[..buf.len];
            let result =
                bt_hci_transport::PacketKind::read(&mut reader).and_then(|kind| P::read_hci(kind, &mut reader, rx));
            buf.receive_done();
            result.map_err(Error::from)
        }
    }

    fn write<P: PacketToController>(&self, tx: &P) -> impl Future<Output = Result<(), Self::Error>> {
        async {
            let ch = &mut *self.tx.borrow_mut();
            let mut buf = ch.send().await;
            let buf_len = buf.buf.len();

            let mut slice = &mut buf.buf[..];
            bt_hci_transport::WithIndicator::new(tx)
                .write_hci(&mut slice)
                .map_err(|_| Error::Io(ErrorKind::Other))?;

            buf.len = buf_len - slice.len();
            buf.send_done();

            Ok(())
        }
    }
}
