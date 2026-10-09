//! IP routing.
//!
//! [`Routes`] is the routing table, accessed with [`Stack::routes`].
//!
//! Routes are keyed by a CIDR. On lookup most specific CIDR wins. Each route contains:
//! - via
//! - outgoing interface
//! - optional expiry time.
//!
//! On-link destinations (in the same network as one of the stack's addresses) do
//! not consult the table: the next hop is the destination itself.
//!
//! [`Stack::routes`]: crate::Stack::routes

use embassy_time::Instant;
pub use xarxa::route::{RouteError, RouteOrigin};
use xarxa::wire::{IpAddr, IpCidr};
#[cfg(feature = "ipv4")]
use xarxa::wire::{Ipv4Addr, Ipv4Cidr};
#[cfg(feature = "ipv6")]
use xarxa::wire::{Ipv6Addr, Ipv6Cidr};

use crate::iface::IfaceHandle;
use crate::time::{instant_from_xarxa, instant_to_xarxa};
use crate::{NoWake, Stack};

#[cfg(feature = "ipv4")]
const IPV4_DEFAULT: IpCidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(0, 0, 0, 0), 0));
#[cfg(feature = "ipv6")]
const IPV6_DEFAULT: IpCidr = IpCidr::V6(Ipv6Cidr::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0));

/// A prefix of addresses that should be routed via a router, out of an interface.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct Route {
    /// The prefix this route covers.
    pub cidr: IpCidr,
    /// The router packets for the prefix are sent to.
    pub via_router: IpAddr,
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
    pub fn new_ipv4_gateway(gateway: Ipv4Addr, iface: IfaceHandle) -> Route {
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
    pub fn new_ipv6_gateway(gateway: Ipv6Addr, iface: IfaceHandle) -> Route {
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

/// A routing table, returned by [`Stack::routes`].
#[derive(Copy, Clone)]
pub struct Routes<'d> {
    stack: Stack<'d>,
}

impl<'d> Routes<'d> {
    pub(crate) fn new(stack: Stack<'d>) -> Self {
        Self { stack }
    }

    /// Add a route.
    ///
    /// # Errors
    /// - `NotUnicast`: if `via_router` is not a unicast address.
    /// - `Full`: if the table has no room. Only possible without the `alloc`
    ///   feature, where the limit is [`ROUTE_COUNT`](crate::config::ROUTE_COUNT).
    pub fn add(&self, route: Route) -> Result<(), RouteError> {
        self.stack
            .with(|i| (i.stack.routes_mut().add(route.to_xarxa()), NoWake))
    }

    /// Remove the route at `index` and return it.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn remove(&self, index: usize) -> Route {
        Route::from_xarxa(self.stack.with(|i| (i.stack.routes_mut().remove(index), NoWake)))
    }

    /// Keep only the routes for which `f` returns true.
    pub fn retain(&self, mut f: impl FnMut(&Route) -> bool) {
        self.stack
            .with(|i| (i.stack.routes_mut().retain(|r| f(&Route::from_xarxa(*r))), NoWake))
    }

    /// Remove all routes.
    pub fn clear(&self) {
        self.stack.with(|i| (i.stack.routes_mut().clear(), NoWake))
    }

    /// Iterate over the routes.
    pub fn iter(&self) -> impl Iterator<Item = Route> + 'd {
        let stack = self.stack;
        (0..self.len()).filter_map(move |n| {
            stack.with(|i| (i.stack.routes().iter().nth(n).copied().map(Route::from_xarxa), NoWake))
        })
    }

    /// Number of routes.
    pub fn len(&self) -> usize {
        self.stack.with(|i| (i.stack.routes().len(), NoWake))
    }

    /// Whether there are no routes.
    pub fn is_empty(&self) -> bool {
        self.stack.with(|i| (i.stack.routes().is_empty(), NoWake))
    }

    /// Add a default ipv4 gateway (ie. "ip route add 0.0.0.0/0 via `gateway` dev `iface`").
    ///
    /// Returns the previous default route, if any. On error the previous
    /// default route is kept.
    ///
    /// # Errors
    /// - `NotUnicast`: if `gateway` is not a unicast address.
    /// - `Full`: if the table has no room. Only possible without the `alloc`
    ///   feature, where the limit is [`ROUTE_COUNT`](crate::config::ROUTE_COUNT).
    #[cfg(feature = "ipv4")]
    pub fn add_default_ipv4_route(&self, gateway: Ipv4Addr, iface: IfaceHandle) -> Result<Option<Route>, RouteError> {
        self.stack
            .with(|i| (i.stack.routes_mut().add_default_ipv4_route(gateway, iface), NoWake))
            .map(|r| r.map(Route::from_xarxa))
    }

    /// Add a default ipv6 gateway (ie. "ip -6 route add ::/0 via `gateway` dev `iface`").
    ///
    /// Returns the previous default route, if any. On error the previous
    /// default route is kept.
    ///
    /// # Errors
    /// - `NotUnicast`: if `gateway` is not a unicast address.
    /// - `Full`: if the table has no room. Only possible without the `alloc`
    ///   feature, where the limit is [`ROUTE_COUNT`](crate::config::ROUTE_COUNT).
    #[cfg(feature = "ipv6")]
    pub fn add_default_ipv6_route(&self, gateway: Ipv6Addr, iface: IfaceHandle) -> Result<Option<Route>, RouteError> {
        self.stack
            .with(|i| (i.stack.routes_mut().add_default_ipv6_route(gateway, iface), NoWake))
            .map(|r| r.map(Route::from_xarxa))
    }

    /// Returns the ipv4 default route if there is one in the route table.
    #[cfg(feature = "ipv4")]
    pub fn default_ipv4_route(&self) -> Option<Route> {
        self.stack
            .with(|i| (i.stack.routes().default_ipv4_route(), NoWake))
            .map(Route::from_xarxa)
    }

    /// Returns the ipv6 default route if there is one in the route table.
    #[cfg(feature = "ipv6")]
    pub fn default_ipv6_route(&self) -> Option<Route> {
        self.stack
            .with(|i| (i.stack.routes().default_ipv6_route(), NoWake))
            .map(Route::from_xarxa)
    }

    /// Remove the default ipv4 gateway, returning it if it existed.
    #[cfg(feature = "ipv4")]
    pub fn remove_default_ipv4_route(&self) -> Option<Route> {
        self.stack
            .with(|i| (i.stack.routes_mut().remove_default_ipv4_route(), NoWake))
            .map(Route::from_xarxa)
    }

    /// Remove the default ipv6 gateway, returning it if it existed.
    #[cfg(feature = "ipv6")]
    pub fn remove_default_ipv6_route(&self) -> Option<Route> {
        self.stack
            .with(|i| (i.stack.routes_mut().remove_default_ipv6_route(), NoWake))
            .map(Route::from_xarxa)
    }
}
