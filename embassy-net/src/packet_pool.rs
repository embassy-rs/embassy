use as_slice::{AsMutSlice, AsSlice};
use core::ops::{Deref, DerefMut, Range};

use atomic_pool::{pool, Box};

pub const MTU: usize = 1516;

#[cfg(feature = "pool-4")]
pub const PACKET_POOL_SIZE: usize = 4;

#[cfg(feature = "pool-8")]
pub const PACKET_POOL_SIZE: usize = 8;

#[cfg(feature = "pool-16")]
pub const PACKET_POOL_SIZE: usize = 16;

#[cfg(feature = "pool-32")]
pub const PACKET_POOL_SIZE: usize = 32;

#[cfg(feature = "pool-64")]
pub const PACKET_POOL_SIZE: usize = 64;

#[cfg(feature = "pool-128")]
pub const PACKET_POOL_SIZE: usize = 128;

pool!(pub PacketPool: [Packet; PACKET_POOL_SIZE]);
pub type PacketBox = Box<PacketPool>;

#[repr(align(4))]
pub struct Packet(pub [u8; MTU]);

impl Packet {
    pub const fn new() -> Self {
        Self([0; MTU])
    }
}

pub trait PacketBoxExt {
    fn slice(self, range: Range<usize>) -> PacketBuf;
}

impl PacketBoxExt for PacketBox {
    fn slice(self, range: Range<usize>) -> PacketBuf {
        PacketBuf {
            packet: self,
            range,
        }
    }
}

impl AsSlice for Packet {
    type Element = u8;

    fn as_slice(&self) -> &[Self::Element] {
        &self.deref()[..]
    }
}

impl AsMutSlice for Packet {
    fn as_mut_slice(&mut self) -> &mut [Self::Element] {
        &mut self.deref_mut()[..]
    }
}

impl Deref for Packet {
    type Target = [u8; MTU];

    fn deref(&self) -> &[u8; MTU] {
        &self.0
    }
}

impl DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut [u8; MTU] {
        &mut self.0
    }
}

pub struct PacketBuf {
    packet: PacketBox,
    range: Range<usize>,
}

impl AsSlice for PacketBuf {
    type Element = u8;

    fn as_slice(&self) -> &[Self::Element] {
        &self.packet[self.range.clone()]
    }
}

impl AsMutSlice for PacketBuf {
    fn as_mut_slice(&mut self) -> &mut [Self::Element] {
        &mut self.packet[self.range.clone()]
    }
}

impl Deref for PacketBuf {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.packet[self.range.clone()]
    }
}

impl DerefMut for PacketBuf {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.packet[self.range.clone()]
    }
}
