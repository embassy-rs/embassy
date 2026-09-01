//! IP routing table.
//!
//! The table is stack-wide, shared by every
//! interface; get a handle to it with [`Stack::routes`].

use embassy_time::Instant;
use xarxa::Full;
pub use xarxa::route::RouteOrigin;
use xarxa::wire::{IpAddress, IpCidr};
#[cfg(feature = "ipv4")]
use xarxa::wire::{Ipv4Address, Ipv4Cidr};
#[cfg(feature = "ipv6")]
use xarxa::wire::{Ipv6Address, Ipv6Cidr};

use crate::Stack;
use crate::iface::IfaceHandle;
use crate::time::{instant_from_xarxa, instant_to_xarxa};

#[cfg(feature = "ipv4")]
const IPV4_DEFAULT: IpCidr = IpCidr::Ipv4(Ipv4Cidr::new(Ipv4Address::new(0, 0, 0, 0), 0));
#[cfg(feature = "ipv6")]
const IPV6_DEFAULT: IpCidr = IpCidr::Ipv6(Ipv6Cidr::new(Ipv6Address::new(0, 0, 0, 0, 0, 0, 0, 0), 0));

/// A prefix of addresses that should be routed via a router, out of an interface.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct Route {
    /// The prefix this route covers.
    pub cidr: IpCidr,
    /// The router packets for the prefix are sent to.
    pub via_router: IpAddress,
    /// The interface this route goes out of.
    pub iface: IfaceHandle,
    /// Where the route came from.
    pub origin: RouteOrigin,
    /// `None` means "forever".
    pub preferred_until: Option<Instant>,
    /// `None` means "forever".
    pub expires_at: Option<Instant>,
}

impl Route {
    /// Returns a route to 0.0.0.0/0 via the `gateway`, out of `iface`, with no expiry.
    #[cfg(feature = "ipv4")]
    pub fn new_ipv4_gateway(gateway: Ipv4Address, iface: IfaceHandle) -> Route {
        Route {
            cidr: IPV4_DEFAULT,
            via_router: gateway.into(),
            iface,
            origin: RouteOrigin::Manual,
            preferred_until: None,
            expires_at: None,
        }
    }

    /// Returns a route to ::/0 via the `gateway`, out of `iface`, with no expiry.
    #[cfg(feature = "ipv6")]
    pub fn new_ipv6_gateway(gateway: Ipv6Address, iface: IfaceHandle) -> Route {
        Route {
            cidr: IPV6_DEFAULT,
            via_router: gateway.into(),
            iface,
            origin: RouteOrigin::Manual,
            preferred_until: None,
            expires_at: None,
        }
    }

    /// Returns `true` if the route is a default route for IPv4.
    #[cfg(feature = "ipv4")]
    pub fn is_ipv4_gateway(&self) -> bool {
        self.cidr == IPV4_DEFAULT
    }

    /// Returns `true` if the route is a default route for IPv6.
    #[cfg(feature = "ipv6")]
    pub fn is_ipv6_gateway(&self) -> bool {
        self.cidr == IPV6_DEFAULT
    }

    pub(crate) fn from_xarxa(route: xarxa::route::Route) -> Self {
        Self {
            cidr: route.cidr,
            via_router: route.via_router,
            iface: route.iface,
            origin: route.origin,
            preferred_until: route.preferred_until.map(instant_from_xarxa),
            expires_at: route.expires_at.map(instant_from_xarxa),
        }
    }

    pub(crate) fn to_xarxa(self) -> xarxa::route::Route {
        xarxa::route::Route {
            cidr: self.cidr,
            via_router: self.via_router,
            iface: self.iface,
            origin: self.origin,
            preferred_until: self.preferred_until.map(instant_to_xarxa),
            expires_at: self.expires_at.map(instant_to_xarxa),
        }
    }
}

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
        self.stack.with_mut(|i| i.stack.routes_mut().add(route.to_xarxa()))
    }

    /// Remove the route at `index`, returning it.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn remove(&self, index: usize) -> Route {
        Route::from_xarxa(self.stack.with_mut(|i| i.stack.routes_mut().remove(index)))
    }

    /// Keep only the routes `f` returns `true` for.
    pub fn retain(&self, mut f: impl FnMut(&Route) -> bool) {
        self.stack
            .with_mut(|i| i.stack.routes_mut().retain(|r| f(&Route::from_xarxa(*r))))
    }

    /// Remove every route.
    pub fn clear(&self) {
        self.stack.with_mut(|i| i.stack.routes_mut().clear())
    }

    /// Iterate over the routes in the table.
    pub fn iter(&self) -> impl Iterator<Item = Route> + 'd {
        let stack = self.stack;
        (0..self.len())
            .filter_map(move |n| stack.with(|i| i.stack.routes().iter().nth(n).copied().map(Route::from_xarxa)))
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
            .map(|r| r.map(Route::from_xarxa))
    }

    /// Set the default IPv6 route, via `gateway` out of `iface`.
    ///
    /// Replaces the existing default IPv6 route, which is returned.
    #[cfg(feature = "ipv6")]
    pub fn add_default_ipv6_route(&self, gateway: Ipv6Address, iface: IfaceHandle) -> Result<Option<Route>, Full> {
        self.stack
            .with_mut(|i| i.stack.routes_mut().add_default_ipv6_route(gateway, iface))
            .map(|r| r.map(Route::from_xarxa))
    }

    /// The default IPv4 route, if there is one.
    #[cfg(feature = "ipv4")]
    pub fn get_default_ipv4_route(&self) -> Option<Route> {
        self.stack
            .with(|i| i.stack.routes().get_default_ipv4_route())
            .map(Route::from_xarxa)
    }

    /// The default IPv6 route, if there is one.
    #[cfg(feature = "ipv6")]
    pub fn get_default_ipv6_route(&self) -> Option<Route> {
        self.stack
            .with(|i| i.stack.routes().get_default_ipv6_route())
            .map(Route::from_xarxa)
    }

    /// Remove the default IPv4 route, returning it.
    #[cfg(feature = "ipv4")]
    pub fn remove_default_ipv4_route(&self) -> Option<Route> {
        self.stack
            .with_mut(|i| i.stack.routes_mut().remove_default_ipv4_route())
            .map(Route::from_xarxa)
    }

    /// Remove the default IPv6 route, returning it.
    #[cfg(feature = "ipv6")]
    pub fn remove_default_ipv6_route(&self) -> Option<Route> {
        self.stack
            .with_mut(|i| i.stack.routes_mut().remove_default_ipv6_route())
            .map(Route::from_xarxa)
    }
}
