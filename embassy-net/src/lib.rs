#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(async_fn_in_trait, impl_trait_projections))]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod device;
#[cfg(feature = "dns")]
pub mod dns;
#[cfg(feature = "tcp")]
pub mod tcp;
mod time;
#[cfg(feature = "udp")]
pub mod udp;

use core::cell::RefCell;
use core::future::{poll_fn, Future};
use core::task::{Context, Poll};

pub use embassy_net_driver as driver;
use embassy_net_driver::{Driver, LinkState, Medium};
use embassy_sync::waitqueue::WakerRegistration;
use embassy_time::{Instant, Timer};
use futures::pin_mut;
use heapless::Vec;
#[cfg(feature = "igmp")]
pub use smoltcp::iface::MulticastError;
use smoltcp::iface::{Interface, SocketHandle, SocketSet, SocketStorage};
#[cfg(feature = "dhcpv4")]
use smoltcp::socket::dhcpv4::{self, RetryConfig};
#[cfg(feature = "udp")]
pub use smoltcp::wire::IpListenEndpoint;
#[cfg(feature = "medium-ethernet")]
pub use smoltcp::wire::{EthernetAddress, HardwareAddress};
pub use smoltcp::wire::{IpAddress, IpCidr};
#[cfg(feature = "proto-ipv4")]
pub use smoltcp::wire::{Ipv4Address, Ipv4Cidr};
#[cfg(feature = "proto-ipv6")]
pub use smoltcp::wire::{Ipv6Address, Ipv6Cidr};

use crate::device::DriverAdapter;
use crate::time::{instant_from_smoltcp, instant_to_smoltcp};

const LOCAL_PORT_MIN: u16 = 1025;
const LOCAL_PORT_MAX: u16 = 65535;
#[cfg(feature = "dns")]
const MAX_QUERIES: usize = 4;

/// Memory resources needed for a network stack.
pub struct StackResources<const SOCK: usize> {
    sockets: [SocketStorage<'static>; SOCK],
    #[cfg(feature = "dns")]
    queries: [Option<dns::DnsQuery>; MAX_QUERIES],
}

impl<const SOCK: usize> StackResources<SOCK> {
    /// Create a new set of stack resources.
    pub fn new() -> Self {
        #[cfg(feature = "dns")]
        const INIT: Option<dns::DnsQuery> = None;
        Self {
            sockets: [SocketStorage::EMPTY; SOCK],
            #[cfg(feature = "dns")]
            queries: [INIT; MAX_QUERIES],
        }
    }
}

/// Static IP address configuration.
#[cfg(feature = "proto-ipv4")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticConfigV4 {
    /// IP address and subnet mask.
    pub address: Ipv4Cidr,
    /// Default gateway.
    pub gateway: Option<Ipv4Address>,
    /// DNS servers.
    pub dns_servers: Vec<Ipv4Address, 3>,
}

/// DHCP configuration.
#[cfg(feature = "dhcpv4")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DhcpConfig {
    /// Maximum lease duration.
    ///
    /// If not set, the lease duration specified by the server will be used.
    /// If set, the lease duration will be capped at this value.
    pub max_lease_duration: Option<embassy_time::Duration>,
    /// Retry configuration.
    pub retry_config: RetryConfig,
    /// Ignore NAKs from DHCP servers.
    ///
    /// This is not compliant with the DHCP RFCs, since theoretically we must stop using the assigned IP when receiving a NAK. This can increase reliability on broken networks with buggy routers or rogue DHCP servers, however.
    pub ignore_naks: bool,
    /// Server port. This is almost always 67. Do not change unless you know what you're doing.
    pub server_port: u16,
    /// Client port. This is almost always 68. Do not change unless you know what you're doing.
    pub client_port: u16,
}

#[cfg(feature = "dhcpv4")]
impl Default for DhcpConfig {
    fn default() -> Self {
        Self {
            max_lease_duration: Default::default(),
            retry_config: Default::default(),
            ignore_naks: Default::default(),
            server_port: smoltcp::wire::DHCP_SERVER_PORT,
            client_port: smoltcp::wire::DHCP_CLIENT_PORT,
        }
    }
}

/// Network stack configuration.
pub enum Config {
    /// Use a static IP address configuration.
    #[cfg(feature = "proto-ipv4")]
    StaticV4(StaticConfigV4),
    /// Use DHCP to obtain an IP address configuration.
    #[cfg(feature = "dhcpv4")]
    Dhcp(DhcpConfig),
}

/// A network stack.
///
/// This is the main entry point for the network stack.
pub struct Stack<D: Driver> {
    pub(crate) socket: RefCell<SocketStack>,
    inner: RefCell<Inner<D>>,
}

struct Inner<D: Driver> {
    device: D,
    link_up: bool,
    #[cfg(feature = "proto-ipv4")]
    static_v4: Option<StaticConfigV4>,
    #[cfg(feature = "dhcpv4")]
    dhcp_socket: Option<SocketHandle>,
    #[cfg(feature = "dns")]
    dns_socket: SocketHandle,
    #[cfg(feature = "dns")]
    dns_waker: WakerRegistration,
}

pub(crate) struct SocketStack {
    pub(crate) sockets: SocketSet<'static>,
    pub(crate) iface: Interface,
    pub(crate) waker: WakerRegistration,
    next_local_port: u16,
}

impl<D: Driver + 'static> Stack<D> {
    /// Create a new network stack.
    pub fn new<const SOCK: usize>(
        mut device: D,
        config: Config,
        resources: &'static mut StackResources<SOCK>,
        random_seed: u64,
    ) -> Self {
        #[cfg(feature = "medium-ethernet")]
        let medium = device.capabilities().medium;

        let mut iface_cfg = smoltcp::iface::Config::new();
        iface_cfg.random_seed = random_seed;
        #[cfg(feature = "medium-ethernet")]
        if medium == Medium::Ethernet {
            iface_cfg.hardware_addr = Some(HardwareAddress::Ethernet(EthernetAddress(device.ethernet_address())));
        }

        let iface = Interface::new(
            iface_cfg,
            &mut DriverAdapter {
                inner: &mut device,
                cx: None,
            },
        );

        let sockets = SocketSet::new(&mut resources.sockets[..]);

        let next_local_port = (random_seed % (LOCAL_PORT_MAX - LOCAL_PORT_MIN) as u64) as u16 + LOCAL_PORT_MIN;

        let mut socket = SocketStack {
            sockets,
            iface,
            waker: WakerRegistration::new(),
            next_local_port,
        };

        let mut inner = Inner {
            device,
            link_up: false,
            #[cfg(feature = "proto-ipv4")]
            static_v4: None,
            #[cfg(feature = "dhcpv4")]
            dhcp_socket: None,
            #[cfg(feature = "dns")]
            dns_socket: socket.sockets.add(dns::Socket::new(
                &[],
                managed::ManagedSlice::Borrowed(&mut resources.queries),
            )),
            #[cfg(feature = "dns")]
            dns_waker: WakerRegistration::new(),
        };

        match config {
            #[cfg(feature = "proto-ipv4")]
            Config::StaticV4(config) => {
                inner.apply_config(&mut socket, config);
            }
            #[cfg(feature = "dhcpv4")]
            Config::Dhcp(config) => {
                let mut dhcp_socket = smoltcp::socket::dhcpv4::Socket::new();
                inner.apply_dhcp_config(&mut dhcp_socket, config);
                let handle = socket.sockets.add(dhcp_socket);
                inner.dhcp_socket = Some(handle);
            }
        }

        Self {
            socket: RefCell::new(socket),
            inner: RefCell::new(inner),
        }
    }

    fn with<R>(&self, f: impl FnOnce(&SocketStack, &Inner<D>) -> R) -> R {
        f(&*self.socket.borrow(), &*self.inner.borrow())
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut SocketStack, &mut Inner<D>) -> R) -> R {
        f(&mut *self.socket.borrow_mut(), &mut *self.inner.borrow_mut())
    }

    /// Get the MAC address of the network interface.
    pub fn ethernet_address(&self) -> [u8; 6] {
        self.with(|_s, i| i.device.ethernet_address())
    }

    /// Get whether the link is up.
    pub fn is_link_up(&self) -> bool {
        self.with(|_s, i| i.link_up)
    }

    /// Get whether the network stack has a valid IP configuration.
    /// This is true if the network stack has a static IP configuration or if DHCP has completed
    pub fn is_config_up(&self) -> bool {
        #[cfg(feature = "proto-ipv4")]
        {
            return self.config_v4().is_some();
        }

        #[cfg(not(any(feature = "proto-ipv4")))]
        {
            false
        }
    }

    /// Get the current IP configuration.
    #[cfg(feature = "proto-ipv4")]
    pub fn config_v4(&self) -> Option<StaticConfigV4> {
        self.with(|_s, i| i.static_v4.clone())
    }

    /// Run the network stack.
    ///
    /// You must call this in a background task, to process network events.
    pub async fn run(&self) -> ! {
        poll_fn(|cx| {
            self.with_mut(|s, i| i.poll(cx, s));
            Poll::<()>::Pending
        })
        .await;
        unreachable!()
    }

    /// Make a query for a given name and return the corresponding IP addresses.
    #[cfg(feature = "dns")]
    pub async fn dns_query(&self, name: &str, qtype: dns::DnsQueryType) -> Result<Vec<IpAddress, 1>, dns::Error> {
        // For A and AAAA queries we try detect whether `name` is just an IP address
        match qtype {
            #[cfg(feature = "proto-ipv4")]
            dns::DnsQueryType::A => {
                if let Ok(ip) = name.parse().map(IpAddress::Ipv4) {
                    return Ok([ip].into_iter().collect());
                }
            }
            #[cfg(feature = "proto-ipv6")]
            dns::DnsQueryType::Aaaa => {
                if let Ok(ip) = name.parse().map(IpAddress::Ipv6) {
                    return Ok([ip].into_iter().collect());
                }
            }
            _ => {}
        }

        let query = poll_fn(|cx| {
            self.with_mut(|s, i| {
                let socket = s.sockets.get_mut::<dns::Socket>(i.dns_socket);
                match socket.start_query(s.iface.context(), name, qtype) {
                    Ok(handle) => Poll::Ready(Ok(handle)),
                    Err(dns::StartQueryError::NoFreeSlot) => {
                        i.dns_waker.register(cx.waker());
                        Poll::Pending
                    }
                    Err(e) => Poll::Ready(Err(e)),
                }
            })
        })
        .await?;

        use embassy_hal_common::drop::OnDrop;
        let drop = OnDrop::new(|| {
            self.with_mut(|s, i| {
                let socket = s.sockets.get_mut::<dns::Socket>(i.dns_socket);
                socket.cancel_query(query);
                s.waker.wake();
                i.dns_waker.wake();
            })
        });

        let res = poll_fn(|cx| {
            self.with_mut(|s, i| {
                let socket = s.sockets.get_mut::<dns::Socket>(i.dns_socket);
                match socket.get_query_result(query) {
                    Ok(addrs) => {
                        i.dns_waker.wake();
                        Poll::Ready(Ok(addrs))
                    }
                    Err(dns::GetQueryResultError::Pending) => {
                        socket.register_query_waker(query, cx.waker());
                        Poll::Pending
                    }
                    Err(e) => {
                        i.dns_waker.wake();
                        Poll::Ready(Err(e.into()))
                    }
                }
            })
        })
        .await;

        drop.defuse();

        res
    }
}

#[cfg(feature = "igmp")]
impl<D: Driver + smoltcp::phy::Device + 'static> Stack<D> {
    /// Join a multicast group.
    pub fn join_multicast_group<T>(&self, addr: T) -> Result<bool, MulticastError>
    where
        T: Into<IpAddress>,
    {
        let addr = addr.into();

        self.with_mut(|s, i| {
            s.iface
                .join_multicast_group(&mut i.device, addr, instant_to_smoltcp(Instant::now()))
        })
    }

    /// Leave a multicast group.
    pub fn leave_multicast_group<T>(&self, addr: T) -> Result<bool, MulticastError>
    where
        T: Into<IpAddress>,
    {
        let addr = addr.into();

        self.with_mut(|s, i| {
            s.iface
                .leave_multicast_group(&mut i.device, addr, instant_to_smoltcp(Instant::now()))
        })
    }

    /// Get whether the network stack has joined the given multicast group.
    pub fn has_multicast_group<T: Into<IpAddress>>(&self, addr: T) -> bool {
        self.socket.borrow().iface.has_multicast_group(addr)
    }
}

impl SocketStack {
    #[allow(clippy::absurd_extreme_comparisons, dead_code)]
    pub fn get_local_port(&mut self) -> u16 {
        let res = self.next_local_port;
        self.next_local_port = if res >= LOCAL_PORT_MAX { LOCAL_PORT_MIN } else { res + 1 };
        res
    }
}

impl<D: Driver + 'static> Inner<D> {
    #[cfg(feature = "proto-ipv4")]
    fn apply_config(&mut self, s: &mut SocketStack, config: StaticConfigV4) {
        #[cfg(feature = "medium-ethernet")]
        let medium = self.device.capabilities().medium;

        debug!("Acquired IP configuration:");

        debug!("   IP address:      {}", config.address);
        s.iface.update_ip_addrs(|addrs| {
            if addrs.is_empty() {
                addrs.push(IpCidr::Ipv4(config.address)).unwrap();
            } else {
                addrs[0] = IpCidr::Ipv4(config.address);
            }
        });

        #[cfg(feature = "medium-ethernet")]
        if medium == Medium::Ethernet {
            if let Some(gateway) = config.gateway {
                debug!("   Default gateway: {}", gateway);
                s.iface.routes_mut().add_default_ipv4_route(gateway).unwrap();
            } else {
                debug!("   Default gateway: None");
                s.iface.routes_mut().remove_default_ipv4_route();
            }
        }
        for (i, s) in config.dns_servers.iter().enumerate() {
            debug!("   DNS server {}:    {}", i, s);
        }

        #[cfg(feature = "dns")]
        {
            let socket = s.sockets.get_mut::<smoltcp::socket::dns::Socket>(self.dns_socket);
            let servers: Vec<IpAddress, 3> = config.dns_servers.iter().map(|c| IpAddress::Ipv4(*c)).collect();
            socket.update_servers(&servers[..]);
        }

        self.static_v4 = Some(config)
    }

    #[cfg(feature = "dhcpv4")]
    fn apply_dhcp_config(&self, socket: &mut smoltcp::socket::dhcpv4::Socket, config: DhcpConfig) {
        socket.set_ignore_naks(config.ignore_naks);
        socket.set_max_lease_duration(config.max_lease_duration.map(crate::time::duration_to_smoltcp));
        socket.set_ports(config.server_port, config.client_port);
        socket.set_retry_config(config.retry_config);
    }

    #[allow(unused)] // used only with dhcp
    fn unapply_config(&mut self, s: &mut SocketStack) {
        #[cfg(feature = "medium-ethernet")]
        let medium = self.device.capabilities().medium;

        debug!("Lost IP configuration");
        s.iface.update_ip_addrs(|ip_addrs| ip_addrs.clear());
        #[cfg(feature = "medium-ethernet")]
        if medium == Medium::Ethernet {
            #[cfg(feature = "proto-ipv4")]
            {
                s.iface.routes_mut().remove_default_ipv4_route();
            }
        }
        #[cfg(feature = "proto-ipv4")]
        {
            self.static_v4 = None
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>, s: &mut SocketStack) {
        s.waker.register(cx.waker());

        #[cfg(feature = "medium-ethernet")]
        if self.device.capabilities().medium == Medium::Ethernet {
            s.iface.set_hardware_addr(HardwareAddress::Ethernet(EthernetAddress(
                self.device.ethernet_address(),
            )));
        }

        let timestamp = instant_to_smoltcp(Instant::now());
        let mut smoldev = DriverAdapter {
            cx: Some(cx),
            inner: &mut self.device,
        };
        s.iface.poll(timestamp, &mut smoldev, &mut s.sockets);

        // Update link up
        let old_link_up = self.link_up;
        self.link_up = self.device.link_state(cx) == LinkState::Up;

        // Print when changed
        if old_link_up != self.link_up {
            info!("link_up = {:?}", self.link_up);
        }

        #[cfg(feature = "dhcpv4")]
        if let Some(dhcp_handle) = self.dhcp_socket {
            let socket = s.sockets.get_mut::<dhcpv4::Socket>(dhcp_handle);

            if self.link_up {
                match socket.poll() {
                    None => {}
                    Some(dhcpv4::Event::Deconfigured) => self.unapply_config(s),
                    Some(dhcpv4::Event::Configured(config)) => {
                        let config = StaticConfigV4 {
                            address: config.address,
                            gateway: config.router,
                            dns_servers: config.dns_servers,
                        };
                        self.apply_config(s, config)
                    }
                }
            } else if old_link_up {
                socket.reset();
                self.unapply_config(s);
            }
        }
        //if old_link_up || self.link_up {
        //    self.poll_configurator(timestamp)
        //}
        //

        if let Some(poll_at) = s.iface.poll_at(timestamp, &mut s.sockets) {
            let t = Timer::at(instant_from_smoltcp(poll_at));
            pin_mut!(t);
            if t.poll(cx).is_ready() {
                cx.waker().wake_by_ref();
            }
        }
    }
}
