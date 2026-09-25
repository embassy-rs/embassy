//! Neighbor cache.

use embassy_time::Instant;
use xarxa::error::NotUnicast;
use xarxa::wire::{HardwareAddress, IpAddr};

use crate::iface::IfaceHandle;
use crate::time::{instant_from_xarxa, instant_to_xarxa};
use crate::{NoWake, Stack, wake_if_ok};

/// An entry in the [`NeighborCache`].
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Neighbor {
    /// Interface the neighbor is reachable through.
    pub iface: IfaceHandle,
    /// The neighbor's IP address.
    pub addr: IpAddr,
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

/// The neighbor cache: the stack's map of IP addresses to hardware addresses.
///
/// It holds one entry per neighbor, keyed by the interface it is reachable
/// through plus its IP address. Entries are filled in by ARP and neighbor
/// discovery, and expire after 60 s unless traffic from the neighbor refreshes
/// them.
///
/// Access it with [`Stack::neighbor_cache`].
#[derive(Copy, Clone)]
pub struct NeighborCache<'d> {
    stack: Stack<'d>,
}

impl<'d> NeighborCache<'d> {
    pub(crate) fn new(stack: Stack<'d>) -> Self {
        Self { stack }
    }

    /// Get the entry for a neighbor.
    ///
    /// Expired entries are still reported until the stack reuses their slot.
    /// Compare `expires_at` against the current time if that matters.
    pub fn get(&self, iface: IfaceHandle, addr: IpAddr) -> Option<Neighbor> {
        self.stack
            .with(|i| (i.stack.neighbor_cache().get(iface, addr), NoWake))
            .map(Neighbor::from_xarxa)
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> impl Iterator<Item = Neighbor> + 'd {
        let stack = self.stack;
        (0..self.len())
            .filter_map(move |n| stack.with(|i| (i.stack.neighbor_cache().iter().nth(n), NoWake)))
            .map(Neighbor::from_xarxa)
    }

    /// Add or replace an entry, mapping `addr` on `iface` to `hardware_addr`.
    ///
    /// `expires_at` is when the entry stops being used. Pass `Instant::MAX` for
    /// a static entry that never expires. Note that ARP or neighbor discovery
    /// can still replace it if the neighbor answers with a different hardware
    /// address.
    ///
    /// If the cache is full, another entry is evicted to make room.
    ///
    /// # Errors
    /// - `NotUnicast`: if `addr` or `hardware_addr` is not unicast. The cache
    ///   is left unchanged.
    pub fn insert(
        &self,
        iface: IfaceHandle,
        addr: IpAddr,
        hardware_addr: HardwareAddress,
        expires_at: Instant,
    ) -> Result<(), NotUnicast> {
        self.stack.with(|i| {
            wake_if_ok(
                i.stack
                    .neighbor_cache_mut()
                    .insert(iface, addr, hardware_addr, instant_to_xarxa(expires_at)),
            )
        })
    }

    /// Remove the entry for a neighbor, returning it if there was one.
    ///
    /// Removing an entry whose resolution is still in progress leaves the
    /// packets parked on it waiting: they are dropped when their own timeout
    /// expires, a few seconds later.
    pub fn remove(&self, iface: IfaceHandle, addr: IpAddr) -> Option<Neighbor> {
        self.stack
            .with(|i| (i.stack.neighbor_cache_mut().remove(iface, addr), NoWake))
            .map(Neighbor::from_xarxa)
    }

    /// Keep only the entries for which `f` returns true.
    ///
    /// Same caveat as [`NeighborCache::remove`] for entries being resolved.
    pub fn retain(&self, mut f: impl FnMut(&Neighbor) -> bool) {
        self.stack.with(|i| {
            (
                i.stack.neighbor_cache_mut().retain(|n| f(&Neighbor::from_xarxa(*n))),
                NoWake,
            )
        })
    }

    /// Remove all entries for one interface.
    ///
    /// Same caveat as [`NeighborCache::remove`] for entries being resolved.
    pub fn clear_iface(&self, iface: IfaceHandle) {
        self.stack
            .with(|i| (i.stack.neighbor_cache_mut().clear_iface(iface), NoWake))
    }

    /// Remove all entries.
    ///
    /// Same caveat as [`NeighborCache::remove`] for entries being resolved.
    pub fn clear(&self) {
        self.stack.with(|i| (i.stack.neighbor_cache_mut().clear(), NoWake))
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.stack.with(|i| (i.stack.neighbor_cache().len(), NoWake))
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.with(|i| (i.stack.neighbor_cache().is_empty(), NoWake))
    }
}
