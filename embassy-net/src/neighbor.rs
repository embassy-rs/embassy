//! Neighbor cache.

use xarxa::wire::{HardwareAddress, IpAddress};

use crate::Stack;
use crate::iface::IfaceHandle;
use crate::time::instant_to_xarxa;

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
    pub fn get(&self, iface: IfaceHandle, addr: IpAddress) -> Option<xarxa::Neighbor> {
        self.stack.with(|i| i.stack.neighbor_cache().get(iface, addr))
    }

    /// Iterate over the neighbors in the cache.
    pub fn iter(&self) -> impl Iterator<Item = xarxa::Neighbor> + 'd {
        let stack = self.stack;
        (0..self.len()).filter_map(move |n| stack.with(|i| i.stack.neighbor_cache().iter().nth(n)))
    }

    /// Insert a neighbor, replacing any entry for the same interface and address.
    pub fn insert(
        &self,
        iface: IfaceHandle,
        addr: IpAddress,
        hardware_addr: HardwareAddress,
        expires_at: embassy_time::Instant,
    ) {
        self.stack.with_mut(|i| {
            i.stack
                .neighbor_cache_mut()
                .insert(iface, addr, hardware_addr, instant_to_xarxa(expires_at))
        })
    }

    /// Remove a neighbor, returning it.
    pub fn remove(&self, iface: IfaceHandle, addr: IpAddress) -> Option<xarxa::Neighbor> {
        self.stack
            .with_mut(|i| i.stack.neighbor_cache_mut().remove(iface, addr))
    }

    /// Keep only the neighbors `f` returns `true` for.
    pub fn retain(&self, f: impl FnMut(&xarxa::Neighbor) -> bool) {
        self.stack.with_mut(|i| i.stack.neighbor_cache_mut().retain(f))
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
