#![no_std]
#![allow(async_fn_in_trait)]
#![allow(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(any(feature = "ipv4", feature = "ipv6")))]
compile_error!("You must enable at least one of the following features: ipv4, ipv6");

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[cfg(feature = "dns")]
pub mod dns;
pub mod iface;
#[cfg(any(feature = "medium-ethernet", feature = "medium-ieee802154"))]
mod neighbor;
#[cfg(feature = "raw")]
pub mod raw;
pub mod route;
#[cfg(feature = "tcp")]
pub mod tcp;
mod time;
#[cfg(feature = "udp")]
pub mod udp;

use core::cell::RefCell;
use core::future::{Future, poll_fn};
use core::mem::MaybeUninit;
use core::pin::pin;
use core::task::{Context, Poll};

use embassy_sync::waitqueue::WakerRegistration;
use embassy_time::{Instant, Timer};
#[cfg(feature = "icmp-errors")]
pub use xarxa::IcmpError;
use xarxa::driver::{Driver, LinkState};
use xarxa::iface::IfaceHandle;
pub use xarxa::{Full, config, driver, wire};
#[cfg(any(feature = "medium-ethernet", feature = "medium-ieee802154"))]
pub use xarxa::{Neighbor, NeighborState};

use crate::iface::Iface;
#[cfg(any(feature = "medium-ethernet", feature = "medium-ieee802154"))]
pub use crate::neighbor::NeighborCache;
use crate::route::Routes;
use crate::time::{instant_from_xarxa, instant_to_xarxa};

/// Error returned by `try_*` socket methods.
///
/// `WouldBlock` indicates the operation would block (e.g. no data available,
/// send buffer full). `Other` wraps the socket-specific error type for any
/// other failure.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryError<T> {
    /// The operation would block; try again later.
    WouldBlock,
    /// A socket-specific error occurred.
    Other(T),
}

/// Memory storage needed for a network stack.
///
/// The stack holds this for the rest of the program: put it in a `static`
/// (with `StaticCell`), or declare it before the stack.
///
/// This holds only the stack-wide state. The drivers live wherever the caller
/// puts them, and are handed to [`Stack::add_iface`].
///
/// Socket storage is not here either: the stack has a fixed number of socket
/// slots per type, set by the `*-socket-count-N` features of `xarxa`, and the
/// packet buffers come from a global pool sized by `packet-buf-count-N`.
pub struct StackStorage<'d> {
    inner: MaybeUninit<RefCell<Inner<'d>>>,
}

impl<'d> StackStorage<'d> {
    /// Create the storage for a stack.
    pub const fn new() -> Self {
        Self {
            inner: MaybeUninit::uninit(),
        }
    }
}

impl Default for StackStorage<'_> {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct Inner<'d> {
    pub(crate) stack: xarxa::Stack<'d>,
    /// Waker used for triggering polls.
    pub(crate) waker: WakerRegistration,
    /// Sum of every interface's configuration generation at the last poll.
    pub(crate) config_generation: u32,
    #[cfg(feature = "dns")]
    pub(crate) dns: xarxa::dns::DnsClient,
    #[cfg(feature = "dns")]
    pub(crate) dns_waker: WakerRegistration,
    /// DNS servers set by hand, used on top of the ones DHCPv4 learns.
    #[cfg(feature = "dns")]
    pub(crate) static_dns_servers: heapless::Vec<wire::IpAddress, { config::DNS_MAX_SERVER_COUNT }>,
}

/// A network stack.
///
/// This is a handle to the stack created by [`Stack::new`]. It's `Copy`, so
/// you can pass it by value instead of by reference.
#[derive(Copy, Clone)]
pub struct Stack<'d> {
    pub(crate) inner: &'d core::cell::RefCell<Inner<'d>>,
}

impl<'d> Stack<'d> {
    /// Create a new network stack.
    ///
    /// The stack starts out with no interfaces: add them with
    /// [`add_iface`](Self::add_iface).
    pub fn new(storage: &'d mut StackStorage<'d>, random_seed: u64) -> (Self, Runner<'d>) {
        #[allow(unused_mut)]
        let mut stack = xarxa::Stack::new(random_seed);

        #[cfg(feature = "dns")]
        let dns = unwrap!(xarxa::dns::DnsClient::new(&mut stack, &[]).ok());

        let inner = Inner {
            stack,
            waker: WakerRegistration::new(),
            config_generation: 0,
            #[cfg(feature = "dns")]
            dns,
            #[cfg(feature = "dns")]
            dns_waker: WakerRegistration::new(),
            #[cfg(feature = "dns")]
            static_dns_servers: heapless::Vec::new(),
        };

        let inner = &*storage.inner.write(core::cell::RefCell::new(inner));
        let stack = Stack { inner };
        (stack, Runner { stack })
    }

    /// Borrow the stack, without waking the runner.
    pub(crate) fn with<R>(&self, f: impl FnOnce(&mut Inner<'d>) -> R) -> R {
        f(&mut self.inner.borrow_mut())
    }

    /// Borrow the stack, and wake the runner afterwards so it processes what
    /// changed.
    pub(crate) fn with_mut<R>(&self, f: impl FnOnce(&mut Inner<'d>) -> R) -> R {
        let mut inner = self.inner.borrow_mut();
        let r = f(&mut inner);
        inner.waker.wake();
        r
    }

    /// Add an interface to the stack, taking ownership of the driver.
    ///
    /// See [`add_iface`](Self::add_iface) for the no-alloc version.
    #[cfg(feature = "alloc")]
    pub fn add_iface(&self, driver: alloc::boxed::Box<dyn Driver + 'd>) -> Result<Iface<'d>, Full> {
        let handle = self.with_mut(|i| i.stack.add_iface(driver))?;
        Ok(self.iface(handle))
    }

    /// Add an interface to the stack, borrowing the driver.
    ///
    /// The driver is borrowed for as long as the stack lives. With a `StaticCell`
    /// that is `'static`; with a local, the enclosing scope.
    ///
    /// # Example
    /// ```ignore
    /// static ETH: StaticCell<Device> = StaticCell::new();
    /// let eth = stack.add_iface(ETH.init(device)).unwrap();
    /// ```
    pub fn add_iface(&self, driver: &'d mut dyn Driver) -> Result<Iface<'d>, Full> {
        let handle = self.with_mut(|i| i.stack.add_iface_borrowed(driver))?;
        Ok(self.iface(handle))
    }

    /// Get an interface by its handle.
    ///
    /// # Panics
    /// Panics if the handle does not belong to an interface on this stack.
    pub fn iface(&self, handle: IfaceHandle) -> Iface<'d> {
        self.with(|i| {
            // Check the handle is live, so a bad one panics here instead of somewhere
            // deeper the first time the interface is used.
            let _ = i.stack.iface(handle).capabilities();
        });
        Iface::new(*self, handle)
    }

    /// Remove an interface from the stack.
    ///
    /// Its addresses, routes and neighbor cache entries go with it. Sockets bound
    /// to one of its addresses are not closed, they just stop receiving.
    ///
    /// # Panics
    /// Panics if the handle does not belong to an interface on this stack.
    pub fn remove_iface(&self, handle: IfaceHandle) {
        self.with_mut(|i| i.stack.remove_iface(handle))
    }

    /// The stack's neighbor cache, shared by all interfaces.
    #[cfg(any(feature = "medium-ethernet", feature = "medium-ieee802154"))]
    pub fn neighbor_cache(&self) -> NeighborCache<'d> {
        NeighborCache::new(*self)
    }

    /// The stack's routing table, shared by all interfaces.
    pub fn routes(&self) -> Routes<'d> {
        Routes::new(*self)
    }

    /// Set the DNS servers to use, on top of the ones learned from DHCPv4.
    ///
    /// The runner keeps the DNS client's server list in step with the DHCPv4
    /// leases of every interface. The servers set here are used in addition to
    /// those, and come first.
    #[cfg(feature = "dns")]
    pub fn set_dns_servers(&self, servers: &[crate::wire::IpAddress]) {
        self.with_mut(|i| {
            i.static_dns_servers.clear();
            for s in servers {
                if i.static_dns_servers.push(*s).is_err() {
                    warn!("too many DNS servers, dropping the rest");
                    break;
                }
            }
            i.update_dns_servers();
        })
    }

    /// Make a query for a given name and return the corresponding IP addresses.
    #[cfg(feature = "dns")]
    pub async fn dns_query(
        &self,
        name: &str,
        qtype: dns::DnsQueryType,
    ) -> Result<heapless::Vec<crate::wire::IpAddress, { xarxa::config::DNS_MAX_RESULT_COUNT }>, dns::Error> {
        use crate::wire::IpAddress;

        // For A and AAAA queries we try detect whether `name` is just an IP address
        match qtype {
            #[cfg(feature = "ipv4")]
            dns::DnsQueryType::A => {
                if let Ok(ip) = name.parse().map(IpAddress::Ipv4) {
                    return Ok([ip].into_iter().collect());
                }
            }
            #[cfg(feature = "ipv6")]
            dns::DnsQueryType::Aaaa => {
                if let Ok(ip) = name.parse().map(IpAddress::Ipv6) {
                    return Ok([ip].into_iter().collect());
                }
            }
            _ => {}
        }

        let query = poll_fn(|cx| {
            self.with_mut(|i| {
                let Inner {
                    stack, dns, dns_waker, ..
                } = i;
                match dns.start_query(stack, name, qtype) {
                    Ok(handle) => Poll::Ready(Ok::<_, dns::Error>(handle)),
                    Err(xarxa::dns::StartQueryError::NoFreeSlot) => {
                        dns_waker.register(cx.waker());
                        Poll::Pending
                    }
                    Err(e) => Poll::Ready(Err(e.into())),
                }
            })
        })
        .await?;

        #[must_use = "to delay the drop handler invocation to the end of the scope"]
        struct OnDrop<F: FnOnce()> {
            f: core::mem::MaybeUninit<F>,
        }

        impl<F: FnOnce()> OnDrop<F> {
            fn new(f: F) -> Self {
                Self {
                    f: core::mem::MaybeUninit::new(f),
                }
            }

            fn defuse(self) {
                core::mem::forget(self)
            }
        }

        impl<F: FnOnce()> Drop for OnDrop<F> {
            fn drop(&mut self) {
                unsafe { self.f.as_ptr().read()() }
            }
        }

        let drop = OnDrop::new(|| {
            self.with_mut(|i| {
                i.dns.cancel_query(query);
                i.dns_waker.wake();
            })
        });

        let res = poll_fn(|cx| {
            self.with_mut(|i| match i.dns.get_query_result(query) {
                Ok(addrs) => {
                    i.dns_waker.wake();
                    Poll::Ready(Ok(addrs))
                }
                Err(xarxa::dns::GetQueryResultError::Pending) => {
                    i.dns.register_query_waker(query, cx.waker());
                    Poll::Pending
                }
                Err(e) => {
                    i.dns_waker.wake();
                    Poll::Ready(Err(e.into()))
                }
            })
        })
        .await;

        drop.defuse();

        res
    }

    /// Whether any interface has a non-link-local IPv6 address.
    #[cfg(all(feature = "ipv6", feature = "dns"))]
    pub(crate) fn any_ipv6(&self) -> bool {
        self.with(|i| {
            let mut iter = i.stack.ifaces();
            while let Some((_, iface)) = iter.next() {
                if iface
                    .ip_addrs()
                    .iter()
                    .any(|a| matches!(a.cidr, xarxa::wire::IpCidr::Ipv6(_)) && !is_link_local(a))
                {
                    return true;
                }
            }
            false
        })
    }
}

/// Network stack runner.
///
/// You must call [`Runner::run()`] in a background task for the network stack to work.
pub struct Runner<'d> {
    stack: Stack<'d>,
}

impl<'d> Runner<'d> {
    /// Run the network stack.
    ///
    /// You must call this in a background task, to process network events.
    pub async fn run(&mut self) -> ! {
        poll_fn(|cx| {
            self.stack.with(|i| i.poll(cx));
            Poll::<()>::Pending
        })
        .await;
        unreachable!()
    }
}

impl Inner<'_> {
    /// The sum of every interface's configuration generation.
    ///
    /// Any interface's generation changing changes this, which is all the runner
    /// needs to know that it should look again. One counter instead of one per
    /// interface, since nothing here acts on *which* interface changed.
    fn config_generation(&mut self) -> u32 {
        let mut sum = 0u32;
        let mut iter = self.stack.ifaces();
        while let Some((_, iface)) = iter.next() {
            sum = sum.wrapping_add(iface.config_generation());
        }
        sum
    }

    /// Hand the DNS client the static servers, then those every interface learned
    /// over DHCPv4.
    #[cfg(feature = "dns")]
    pub(crate) fn update_dns_servers(&mut self) {
        let mut servers: heapless::Vec<crate::wire::IpAddress, { xarxa::config::DNS_MAX_SERVER_COUNT }> =
            heapless::Vec::new();
        let mut truncated = false;

        for s in &self.static_dns_servers {
            truncated |= servers.push(*s).is_err();
        }

        #[cfg(feature = "dhcpv4")]
        {
            let mut iter = self.stack.ifaces();
            while let Some((_, iface)) = iter.next() {
                let Some(lease) = iface.dhcpv4_lease() else { continue };
                for s in &lease.dns_servers {
                    truncated |= servers.push((*s).into()).is_err();
                }
            }
        }

        if truncated {
            warn!("Number of DNS servers exceeds DNS_MAX_SERVER_COUNT, truncating list.");
        }

        self.dns.update_servers(&servers);
    }

    /// Log an interface's addresses, after something changed them.
    fn log_config(&mut self) {
        let mut iter = self.stack.ifaces();
        while let Some((handle, iface)) = iter.next() {
            info!("iface {:?}: config changed", handle);
            for addr in iface.ip_addrs() {
                info!("   addr: {:?} ({:?})", addr.cidr, addr.origin);
            }
        }
        for route in self.stack.routes().iter() {
            info!("   route: {:?}", route);
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) {
        self.waker.register(cx.waker());

        let mut iter = self.stack.ifaces();
        while let Some((_, mut iface)) = iter.next() {
            // embassy-net sleeps until the driver wakes it, so a driver that cannot
            // register a waker would stall the stack forever. Fail loudly instead.
            unwrap!(
                iface.driver_mut().register_waker(cx.waker()),
                "the driver does not support register_waker, which embassy-net requires"
            );
        }

        let now = instant_to_xarxa(Instant::now());
        #[allow(unused_mut)]
        let mut deadline = self.stack.poll(now);

        #[cfg(feature = "dns")]
        {
            deadline = deadline.min(self.dns.poll(&mut self.stack));
        }

        // An interface's generation is bumped whenever its addresses or routes
        // change, whoever changed them, so this catches DHCPv4 and SLAAC too.
        let generation = self.config_generation();
        if generation != self.config_generation {
            self.config_generation = generation;
            self.log_config();
            #[cfg(feature = "dns")]
            self.update_dns_servers();
        }

        if deadline <= now {
            cx.waker().wake_by_ref();
        } else if deadline != xarxa::time::Instant::MAX {
            let t = pin!(Timer::at(instant_from_xarxa(deadline)));
            if t.poll(cx).is_ready() {
                cx.waker().wake_by_ref();
            }
        }
    }
}

/// Wait until `predicate` holds, re-checking whenever `iface` changes state.
pub(crate) fn wait_iface<'a>(
    stack: Stack<'a>,
    handle: IfaceHandle,
    mut predicate: impl FnMut(&mut xarxa::iface::Iface<'_, 'a>) -> bool + 'a,
) -> impl Future<Output = ()> + 'a {
    poll_fn(move |cx| {
        stack.with(|i| {
            let mut iface = i.stack.iface(handle);
            if predicate(&mut iface) {
                Poll::Ready(())
            } else {
                iface.register_waker(cx.waker());
                Poll::Pending
            }
        })
    })
}

/// Whether an address is one the stack derived by itself, rather than one that
/// counts as the interface being configured.
pub(crate) fn is_link_local(addr: &xarxa::iface::IfaceAddr) -> bool {
    #[cfg(all(any(feature = "medium-ethernet", feature = "medium-ieee802154"), feature = "ipv6"))]
    {
        addr.origin == xarxa::iface::AddrOrigin::LinkLocal
    }
    #[cfg(not(all(any(feature = "medium-ethernet", feature = "medium-ieee802154"), feature = "ipv6")))]
    {
        let _ = addr;
        false
    }
}

/// Whether an interface counts as configured: it has an address that something
/// other than IPv6 link-local autoconfiguration put there.
pub(crate) fn is_config_up(iface: &xarxa::iface::Iface<'_, '_>) -> bool {
    iface.ip_addrs().iter().any(|a| !is_link_local(a))
}

/// Whether an interface's link is up.
pub(crate) fn is_link_up(iface: &mut xarxa::iface::Iface<'_, '_>) -> bool {
    iface.link_state() == LinkState::Up
}
