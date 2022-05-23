use core::task::Waker;
use smoltcp::phy::Device as SmolDevice;
use smoltcp::phy::DeviceCapabilities;
use smoltcp::time::Instant as SmolInstant;

use crate::packet_pool::PacketBoxExt;
use crate::{Packet, PacketBox, PacketBuf};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum LinkState {
    Down,
    Up,
}

// 'static required due to the "fake GAT" in smoltcp::phy::Device.
// https://github.com/smoltcp-rs/smoltcp/pull/572
pub trait Device {
    fn is_transmit_ready(&mut self) -> bool;
    fn transmit(&mut self, pkt: PacketBuf);
    fn receive(&mut self) -> Option<PacketBuf>;

    fn register_waker(&mut self, waker: &Waker);
    fn capabilities(&self) -> DeviceCapabilities;
    fn link_state(&mut self) -> LinkState;
    fn ethernet_address(&self) -> [u8; 6];
}

impl<T: ?Sized + Device> Device for &'static mut T {
    fn is_transmit_ready(&mut self) -> bool {
        T::is_transmit_ready(self)
    }
    fn transmit(&mut self, pkt: PacketBuf) {
        T::transmit(self, pkt)
    }
    fn receive(&mut self) -> Option<PacketBuf> {
        T::receive(self)
    }
    fn register_waker(&mut self, waker: &Waker) {
        T::register_waker(self, waker)
    }
    fn capabilities(&self) -> DeviceCapabilities {
        T::capabilities(self)
    }
    fn link_state(&mut self) -> LinkState {
        T::link_state(self)
    }
    fn ethernet_address(&self) -> [u8; 6] {
        T::ethernet_address(self)
    }
}

pub struct DeviceAdapter<D: Device> {
    pub device: D,
    caps: DeviceCapabilities,
}

impl<D: Device> DeviceAdapter<D> {
    pub(crate) fn new(device: D) -> Self {
        Self {
            caps: device.capabilities(),
            device,
        }
    }
}

impl<'a, D: Device + 'static> SmolDevice<'a> for DeviceAdapter<D> {
    type RxToken = RxToken;
    type TxToken = TxToken<'a, D>;

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let tx_pkt = PacketBox::new(Packet::new())?;
        let rx_pkt = self.device.receive()?;
        let rx_token = RxToken { pkt: rx_pkt };
        let tx_token = TxToken {
            device: &mut self.device,
            pkt: tx_pkt,
        };

        Some((rx_token, tx_token))
    }

    /// Construct a transmit token.
    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        if !self.device.is_transmit_ready() {
            return None;
        }

        let tx_pkt = PacketBox::new(Packet::new())?;
        Some(TxToken {
            device: &mut self.device,
            pkt: tx_pkt,
        })
    }

    /// Get a description of device capabilities.
    fn capabilities(&self) -> DeviceCapabilities {
        self.caps.clone()
    }
}

pub struct RxToken {
    pkt: PacketBuf,
}

impl smoltcp::phy::RxToken for RxToken {
    fn consume<R, F>(mut self, _timestamp: SmolInstant, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        f(&mut self.pkt)
    }
}

pub struct TxToken<'a, D: Device> {
    device: &'a mut D,
    pkt: PacketBox,
}

impl<'a, D: Device> smoltcp::phy::TxToken for TxToken<'a, D> {
    fn consume<R, F>(self, _timestamp: SmolInstant, len: usize, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        let mut buf = self.pkt.slice(0..len);
        let r = f(&mut buf)?;
        self.device.transmit(buf);
        Ok(r)
    }
}
