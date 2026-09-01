//! Neighbor cache.

use embassy_time::Instant;
use xarxa::wire::{HardwareAddress, IpAddress};

use crate::Stack;
use crate::iface::IfaceHandle;
use crate::time::{instant_from_xarxa, instant_to_xarxa};

/// A neighbor: the mapping of an on-link IP address to a hardware address.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Neighbor {
    /// Interface the neighbor is reachable through.
    pub iface: IfaceHandle,
    /// The neighbor's IP address.
    pub addr: IpAddress,
    /// Whether the hardware address is known yet.
    pub state: NeighborState,
}

impl Neighbor {
    fn from_xarxa(neighbor: xarxa::Neighbor) -> Self {
        Self {
            iface: neighbor.iface,
            addr: neighbor.addr,
            state: match neighbor.state {
                xarxa::NeighborState::Incomplete => NeighborState::Incomplete,
                xarxa::NeighborState::Reachable {
                    hardware_addr,
                    expires_at,
                } => NeighborState::Reachable {
                    hardware_addr,
                    expires_at: (expires_at != xarxa::time::Instant::MAX).then(|| instant_from_xarxa(expires_at)),
                },
            },
        }
    }
}

/// State of a [`Neighbor`] entry.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborState {
    /// Address resolution is in progress. Packets for this neighbor are parked
    /// until it resolves or resolution gives up.
    Incomplete,
    /// The neighbor's hardware address is known.
    Reachable {
        /// The neighbor's hardware address.
        hardware_addr: HardwareAddress,
        /// When the entry expires. `None` means never.
        expires_at: Option<Instant>,
    },
}

/// The stack's neighbor cache, returned by [`Stack::neighbor_cache`].
#[derive(Copy, Clone)]
pub struct NeighborCache<'d> {
    stack: Stack<'d>,
}

impl<'d> NeighborCache<'d> {
    pub(crate) fn new(stack: Stack<'d>) -> Self {
        Self { stack }
    }

    /// Look up a neighbor by interface and IP address.
    pub fn get(&self, iface: IfaceHandle, addr: IpAddress) -> Option<Neighbor> {
        self.stack
            .with(|i| i.stack.neighbor_cache().get(iface, addr))
            .map(Neighbor::from_xarxa)
    }

    /// Iterate over the neighbors in the cache.
    pub fn iter(&self) -> impl Iterator<Item = Neighbor> + 'd {
        let stack = self.stack;
        (0..self.len())
            .filter_map(move |n| stack.with(|i| i.stack.neighbor_cache().iter().nth(n)))
            .map(Neighbor::from_xarxa)
    }

    /// Insert a neighbor, replacing any entry for the same interface and address.
    pub fn insert(&self, iface: IfaceHandle, addr: IpAddress, hardware_addr: HardwareAddress, expires_at: Instant) {
        self.stack.with_mut(|i| {
            i.stack
                .neighbor_cache_mut()
                .insert(iface, addr, hardware_addr, instant_to_xarxa(expires_at))
        })
    }

    /// Remove a neighbor, returning it.
    pub fn remove(&self, iface: IfaceHandle, addr: IpAddress) -> Option<Neighbor> {
        self.stack
            .with_mut(|i| i.stack.neighbor_cache_mut().remove(iface, addr))
            .map(Neighbor::from_xarxa)
    }

    /// Keep only the neighbors `f` returns `true` for.
    pub fn retain(&self, mut f: impl FnMut(&Neighbor) -> bool) {
        self.stack
            .with_mut(|i| i.stack.neighbor_cache_mut().retain(|n| f(&Neighbor::from_xarxa(*n))))
    }

    /// Remove every neighbor on one interface.
    pub fn clear_iface(&self, iface: IfaceHandle) {
        self.stack.with_mut(|i| i.stack.neighbor_cache_mut().clear_iface(iface))
    }

    /// Remove every neighbor.
    pub fn clear(&self) {
        self.stack.with_mut(|i| i.stack.neighbor_cache_mut().clear())
    }

    /// The number of neighbors in the cache.
    pub fn len(&self) -> usize {
        self.stack.with(|i| i.stack.neighbor_cache().len())
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.with(|i| i.stack.neighbor_cache().is_empty())
    }
}
