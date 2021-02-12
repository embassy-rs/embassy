use core::task::{Poll, Waker};
use smoltcp::phy::Device as SmolDevice;
use smoltcp::phy::DeviceCapabilities;
use smoltcp::time::Instant as SmolInstant;

use crate::fmt::*;
use crate::{Packet, PacketBox, PacketBuf};
use crate::Result;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum LinkState {
    Down,
    Up,
}

pub trait Device {
    fn is_transmit_ready(&mut self) -> bool;
    fn transmit(&mut self, pkt: PacketBuf);
    fn receive(&mut self) -> Option<PacketBuf>;

    fn register_waker(&mut self, waker: &Waker);
    fn capabilities(&mut self) -> DeviceCapabilities;
    fn link_state(&mut self) -> LinkState;
}

pub struct DeviceAdapter {
    pub device: &'static mut dyn Device,
    caps: DeviceCapabilities,
}

impl DeviceAdapter {
    pub(crate) fn new(device: &'static mut dyn Device) -> Self {
        Self {
            caps: device.capabilities(),
            device,
        }
    }
}

impl<'a> SmolDevice<'a> for DeviceAdapter {
    type RxToken = RxToken;
    type TxToken = TxToken<'a>;

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let rx_pkt = self.device.receive()?;
        let tx_pkt = PacketBox::new(Packet::new()).unwrap(); // TODO: not sure about unwrap
        let rx_token = RxToken { pkt: rx_pkt };
        let tx_token = TxToken {
            device: self.device,
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
            device: self.device,
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
    fn consume<R, F>(mut self, _timestamp: SmolInstant, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        f(&mut self.pkt)
    }
}

pub struct TxToken<'a> {
    device: &'a mut dyn Device,
    pkt: PacketBox,
}

impl<'a> smoltcp::phy::TxToken for TxToken<'a> {
    fn consume<R, F>(mut self, _timestamp: SmolInstant, len: usize, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        let mut buf = self.pkt.slice(0..len);
        let r = f(&mut buf)?;
        self.device.transmit(buf);
        Ok(r)
    }
}
