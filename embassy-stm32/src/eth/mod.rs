//! Ethernet (ETH)
#![macro_use]

#[cfg_attr(any(eth_v1a, eth_v1b, eth_v1c), path = "v1/mod.rs")]
#[cfg_attr(eth_v2, path = "v2/mod.rs")]
mod _version;
mod generic_phy;

use core::mem::MaybeUninit;
use core::task::Context;

use embassy_hal_internal::PeripheralType;
use embassy_net_driver::{Capabilities, HardwareAddress, LinkState};
use embassy_sync::waitqueue::AtomicWaker;

pub use self::_version::{InterruptHandler, *};
pub use self::generic_phy::*;
use crate::rcc::RccPeripheral;

#[allow(unused)]
const MTU: usize = 1514;
const TX_BUFFER_SIZE: usize = 1514;
const RX_BUFFER_SIZE: usize = 1536;

#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub(crate) struct Packet<const N: usize>([u8; N]);

/// Ethernet packet queue.
///
/// This struct owns the memory used for reading and writing packets.
///
/// `TX` is the number of packets in the transmit queue, `RX` in the receive
/// queue. A bigger queue allows the hardware to receive more packets while the
/// CPU is busy doing other things, which may increase performance (especially for RX)
/// at the cost of more RAM usage.
pub struct PacketQueue<const TX: usize, const RX: usize> {
    tx_desc: [TDes; TX],
    rx_desc: [RDes; RX],
    tx_buf: [Packet<TX_BUFFER_SIZE>; TX],
    rx_buf: [Packet<RX_BUFFER_SIZE>; RX],
}

impl<const TX: usize, const RX: usize> PacketQueue<TX, RX> {
    /// Create a new packet queue.
    pub const fn new() -> Self {
        Self {
            tx_desc: [const { TDes::new() }; TX],
            rx_desc: [const { RDes::new() }; RX],
            tx_buf: [Packet([0; TX_BUFFER_SIZE]); TX],
            rx_buf: [Packet([0; RX_BUFFER_SIZE]); RX],
        }
    }

    /// Initialize a packet queue in-place.
    ///
    /// This can be helpful to avoid accidentally stack-allocating the packet queue in the stack. The
    /// Rust compiler can sometimes be a bit dumb when working with large owned values: if you call `new()`
    /// and then store the returned PacketQueue in its final place (like a `static`), the compiler might
    /// place it temporarily on the stack then move it. Since this struct is quite big, it may result
    /// in a stack overflow.
    ///
    /// With this function, you can create an uninitialized `static` with type `MaybeUninit<PacketQueue<...>>`
    /// and initialize it in-place, guaranteeing no stack usage.
    ///
    /// After calling this function, calling `assume_init` on the MaybeUninit is guaranteed safe.
    pub fn init(this: &mut MaybeUninit<Self>) {
        unsafe {
            this.as_mut_ptr().write_bytes(0u8, 1);
        }
    }
}

static WAKER: AtomicWaker = AtomicWaker::new();

impl<'d, T: Instance, P: Phy> embassy_net_driver::Driver for Ethernet<'d, T, P> {
    type RxToken<'a>
        = RxToken<'a, 'd>
    where
        Self: 'a;
    type TxToken<'a>
        = TxToken<'a, 'd>
    where
        Self: 'a;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        WAKER.register(cx.waker());
        if self.rx.available().is_some() && self.tx.available().is_some() {
            Some((RxToken { rx: &mut self.rx }, TxToken { tx: &mut self.tx }))
        } else {
            None
        }
    }

    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>> {
        WAKER.register(cx.waker());
        if self.tx.available().is_some() {
            Some(TxToken { tx: &mut self.tx })
        } else {
            None
        }
    }

    fn capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::default();
        caps.max_transmission_unit = MTU;
        caps.max_burst_size = Some(self.tx.len());
        caps
    }

    fn link_state(&mut self, cx: &mut Context) -> LinkState {
        if self.phy.poll_link(&mut self.station_management, cx) {
            LinkState::Up
        } else {
            LinkState::Down
        }
    }

    fn hardware_address(&self) -> HardwareAddress {
        HardwareAddress::Ethernet(self.mac_addr)
    }
}

/// `embassy-net` RX token.
pub struct RxToken<'a, 'd> {
    rx: &'a mut RDesRing<'d>,
}

impl<'a, 'd> embassy_net_driver::RxToken for RxToken<'a, 'd> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // NOTE(unwrap): we checked the queue wasn't full when creating the token.
        let pkt = unwrap!(self.rx.available());
        let r = f(pkt);
        self.rx.pop_packet();
        r
    }
}

/// `embassy-net` TX token.
pub struct TxToken<'a, 'd> {
    tx: &'a mut TDesRing<'d>,
}

impl<'a, 'd> embassy_net_driver::TxToken for TxToken<'a, 'd> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // NOTE(unwrap): we checked the queue wasn't full when creating the token.
        let pkt = unwrap!(self.tx.available());
        let r = f(&mut pkt[..len]);
        self.tx.transmit(len);
        r
    }
}

/// Station Management Interface (SMI) on an ethernet PHY
pub trait StationManagement {
    /// Read a register over SMI.
    fn smi_read(&mut self, phy_addr: u8, reg: u8) -> u16;
    /// Write a register over SMI.
    fn smi_write(&mut self, phy_addr: u8, reg: u8, val: u16);
}

/// Trait for an Ethernet PHY
pub trait Phy {
    /// Reset PHY and wait for it to come out of reset.
    fn phy_reset<S: StationManagement>(&mut self, sm: &mut S);
    /// PHY initialisation.
    fn phy_init<S: StationManagement>(&mut self, sm: &mut S);
    /// Poll link to see if it is up and FD with 100Mbps
    fn poll_link<S: StationManagement>(&mut self, sm: &mut S, cx: &mut Context) -> bool;
}

impl<'d, T: Instance, P: Phy> Ethernet<'d, T, P> {
    /// Directly expose the SMI interface used by the Ethernet driver.
    ///
    /// This can be used to for example configure special PHY registers for compliance testing.
    pub fn station_management(&mut self) -> &mut impl StationManagement {
        &mut self.station_management
    }

    /// Access the user-supplied `Phy`.
    pub fn phy(&self) -> &P {
        &self.phy
    }

    /// Mutably access the user-supplied `Phy`.
    pub fn phy_mut(&mut self) -> &mut P {
        &mut self.phy
    }
}

trait SealedInstance {
    fn regs() -> crate::pac::eth::Eth;
}

/// Ethernet instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral + Send + 'static {}

impl SealedInstance for crate::peripherals::ETH {
    fn regs() -> crate::pac::eth::Eth {
        crate::pac::ETH
    }
}
impl Instance for crate::peripherals::ETH {}

pin_trait!(RXClkPin, Instance);
pin_trait!(TXClkPin, Instance);
pin_trait!(RefClkPin, Instance);
pin_trait!(MDIOPin, Instance);
pin_trait!(MDCPin, Instance);
pin_trait!(RXDVPin, Instance);
pin_trait!(CRSPin, Instance);
pin_trait!(RXD0Pin, Instance);
pin_trait!(RXD1Pin, Instance);
pin_trait!(RXD2Pin, Instance);
pin_trait!(RXD3Pin, Instance);
pin_trait!(TXD0Pin, Instance);
pin_trait!(TXD1Pin, Instance);
pin_trait!(TXD2Pin, Instance);
pin_trait!(TXD3Pin, Instance);
pin_trait!(TXEnPin, Instance);
