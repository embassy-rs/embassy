//! IP routing table.
//!
//! The table is stack-wide, shared by every
//! interface; get a handle to it with [`Stack::routes`].

use xarxa::Full;
pub use xarxa::route::{Route, RouteOrigin};
#[cfg(feature = "ipv4")]
use xarxa::wire::Ipv4Address;
#[cfg(feature = "ipv6")]
use xarxa::wire::Ipv6Address;

use crate::Stack;
use crate::iface::IfaceHandle;

/// The stack's routing table, returned by [`Stack::routes`].
#[derive(Copy, Clone)]
pub struct Routes<'d> {
    stack: Stack<'d>,
}

impl<'d> Routes<'d> {
    pub(crate) fn new(stack: Stack<'d>) -> Self {
        Self { stack }
    }

    /// Add a route to the table.
    ///
    /// Errors:
    /// - `Full` if the table has no room for another route.
    pub fn add(&self, route: Route) -> Result<(), Full> {
        self.stack.with_mut(|i| i.stack.routes_mut().add(route))
    }

    /// Remove the route at `index`, returning it.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn remove(&self, index: usize) -> Route {
        self.stack.with_mut(|i| i.stack.routes_mut().remove(index))
    }

    /// Keep only the routes `f` returns `true` for.
    pub fn retain(&self, f: impl FnMut(&Route) -> bool) {
        self.stack.with_mut(|i| i.stack.routes_mut().retain(f))
    }

    /// Remove every route.
    pub fn clear(&self) {
        self.stack.with_mut(|i| i.stack.routes_mut().clear())
    }

    /// Iterate over the routes in the table.
    pub fn iter(&self) -> impl Iterator<Item = Route> + 'd {
        let stack = self.stack;
        (0..self.len()).filter_map(move |n| stack.with(|i| i.stack.routes().iter().nth(n).copied()))
    }

    /// The number of routes in the table.
    pub fn len(&self) -> usize {
        self.stack.with(|i| i.stack.routes().len())
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.with(|i| i.stack.routes().is_empty())
    }

    /// Set the default IPv4 route, via `gateway` out of `iface`.
    ///
    /// Replaces the existing default IPv4 route, which is returned.
    #[cfg(feature = "ipv4")]
    pub fn add_default_ipv4_route(&self, gateway: Ipv4Address, iface: IfaceHandle) -> Result<Option<Route>, Full> {
        self.stack
            .with_mut(|i| i.stack.routes_mut().add_default_ipv4_route(gateway, iface))
    }

    /// Set the default IPv6 route, via `gateway` out of `iface`.
    ///
    /// Replaces the existing default IPv6 route, which is returned.
    #[cfg(feature = "ipv6")]
    pub fn add_default_ipv6_route(&self, gateway: Ipv6Address, iface: IfaceHandle) -> Result<Option<Route>, Full> {
        self.stack
            .with_mut(|i| i.stack.routes_mut().add_default_ipv6_route(gateway, iface))
    }

    /// The default IPv4 route, if there is one.
    #[cfg(feature = "ipv4")]
    pub fn get_default_ipv4_route(&self) -> Option<Route> {
        self.stack.with(|i| i.stack.routes().get_default_ipv4_route())
    }

    /// The default IPv6 route, if there is one.
    #[cfg(feature = "ipv6")]
    pub fn get_default_ipv6_route(&self) -> Option<Route> {
        self.stack.with(|i| i.stack.routes().get_default_ipv6_route())
    }

    /// Remove the default IPv4 route, returning it.
    #[cfg(feature = "ipv4")]
    pub fn remove_default_ipv4_route(&self) -> Option<Route> {
        self.stack
            .with_mut(|i| i.stack.routes_mut().remove_default_ipv4_route())
    }

    /// Remove the default IPv6 route, returning it.
    #[cfg(feature = "ipv6")]
    pub fn remove_default_ipv6_route(&self) -> Option<Route> {
        self.stack
            .with_mut(|i| i.stack.routes_mut().remove_default_ipv6_route())
    }
}
