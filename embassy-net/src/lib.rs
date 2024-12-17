#![cfg_attr(not(feature = "std"), no_std)]
#![allow(async_fn_in_trait)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

#[cfg(not(any(feature = "proto-ipv4", feature = "proto-ipv6")))]
compile_error!("You must enable at least one of the following features: proto-ipv4, proto-ipv6");

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[cfg(feature = "dns")]
pub mod dns;
mod driver_util;
#[cfg(feature = "raw")]
pub mod raw;
#[cfg(feature = "tcp")]
pub mod tcp;
mod time;
#[cfg(feature = "udp")]
pub mod udp;

use core::cell::RefCell;
use core::future::{poll_fn, Future};
use core::mem::MaybeUninit;
use core::pin::pin;
use core::task::{Context, Poll};

pub use embassy_net_driver as driver;
use embassy_net_driver::{Driver, LinkState};
use embassy_sync::waitqueue::WakerRegistration;
use embassy_time::{Instant, Timer};
use heapless::Vec;
#[cfg(feature = "dns")]
pub use smoltcp::config::DNS_MAX_SERVER_COUNT;
#[cfg(feature = "multicast")]
pub use smoltcp::iface::MulticastError;
#[cfg(any(feature = "dns", feature = "dhcpv4"))]
use smoltcp::iface::SocketHandle;
use smoltcp::iface::{Interface, SocketSet, SocketStorage};
use smoltcp::phy::Medium;
#[cfg(feature = "dhcpv4")]
use smoltcp::socket::dhcpv4::{self, RetryConfig};
#[cfg(feature = "medium-ethernet")]
pub use smoltcp::wire::EthernetAddress;
#[cfg(any(feature = "medium-ethernet", feature = "medium-ieee802154", feature = "medium-ip"))]
pub use smoltcp::wire::HardwareAddress;
#[cfg(any(feature = "udp", feature = "tcp"))]
pub use smoltcp::wire::IpListenEndpoint;
#[cfg(feature = "medium-ieee802154")]
pub use smoltcp::wire::{Ieee802154Address, Ieee802154Frame};
pub use smoltcp::wire::{IpAddress, IpCidr, IpEndpoint};
#[cfg(feature = "proto-ipv4")]
pub use smoltcp::wire::{Ipv4Address, Ipv4Cidr};
#[cfg(feature = "proto-ipv6")]
pub use smoltcp::wire::{Ipv6Address, Ipv6Cidr};

use crate::driver_util::DriverAdapter;
use crate::time::{instant_from_smoltcp, instant_to_smoltcp};

const LOCAL_PORT_MIN: u16 = 1025;
const LOCAL_PORT_MAX: u16 = 65535;
#[cfg(feature = "dns")]
const MAX_QUERIES: usize = 4;
#[cfg(feature = "dhcpv4-hostname")]
const MAX_HOSTNAME_LEN: usize = 32;

/// Memory resources needed for a network stack.
pub struct StackResources<const SOCK: usize> {
    sockets: MaybeUninit<[SocketStorage<'static>; SOCK]>,
    inner: MaybeUninit<RefCell<Inner>>,
    #[cfg(feature = "dns")]
    queries: MaybeUninit<[Option<dns::DnsQuery>; MAX_QUERIES]>,
    #[cfg(feature = "dhcpv4-hostname")]
    hostname: HostnameResources,
}

#[cfg(feature = "dhcpv4-hostname")]
struct HostnameResources {
    option: MaybeUninit<smoltcp::wire::DhcpOption<'static>>,
    data: MaybeUninit<[u8; MAX_HOSTNAME_LEN]>,
}

impl<const SOCK: usize> StackResources<SOCK> {
    /// Create a new set of stack resources.
    pub const fn new() -> Self {
        Self {
            sockets: MaybeUninit::uninit(),
            inner: MaybeUninit::uninit(),
            #[cfg(feature = "dns")]
            queries: MaybeUninit::uninit(),
            #[cfg(feature = "dhcpv4-hostname")]
            hostname: HostnameResources {
                option: MaybeUninit::uninit(),
                data: MaybeUninit::uninit(),
            },
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

/// Static IPv6 address configuration
#[cfg(feature = "proto-ipv6")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticConfigV6 {
    /// IP address and subnet mask.
    pub address: Ipv6Cidr,
    /// Default gateway.
    pub gateway: Option<Ipv6Address>,
    /// DNS servers.
    pub dns_servers: Vec<Ipv6Address, 3>,
}

/// DHCP configuration.
#[cfg(feature = "dhcpv4")]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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
    /// Our hostname. This will be sent to the DHCP server as Option 12.
    #[cfg(feature = "dhcpv4-hostname")]
    pub hostname: Option<heapless::String<MAX_HOSTNAME_LEN>>,
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
            #[cfg(feature = "dhcpv4-hostname")]
            hostname: None,
        }
    }
}

/// Network stack configuration.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Config {
    /// IPv4 configuration
    #[cfg(feature = "proto-ipv4")]
    pub ipv4: ConfigV4,
    /// IPv6 configuration
    #[cfg(feature = "proto-ipv6")]
    pub ipv6: ConfigV6,
}

impl Config {
    /// IPv4 configuration with static addressing.
    #[cfg(feature = "proto-ipv4")]
    pub const fn ipv4_static(config: StaticConfigV4) -> Self {
        Self {
            ipv4: ConfigV4::Static(config),
            #[cfg(feature = "proto-ipv6")]
            ipv6: ConfigV6::None,
        }
    }

    /// IPv6 configuration with static addressing.
    #[cfg(feature = "proto-ipv6")]
    pub const fn ipv6_static(config: StaticConfigV6) -> Self {
        Self {
            #[cfg(feature = "proto-ipv4")]
            ipv4: ConfigV4::None,
            ipv6: ConfigV6::Static(config),
        }
    }

    /// IPv4 configuration with dynamic addressing.
    ///
    /// # Example
    /// ```rust
    /// # use embassy_net::Config;
    /// let _cfg = Config::dhcpv4(Default::default());
    /// ```
    #[cfg(feature = "dhcpv4")]
    pub const fn dhcpv4(config: DhcpConfig) -> Self {
        Self {
            ipv4: ConfigV4::Dhcp(config),
            #[cfg(feature = "proto-ipv6")]
            ipv6: ConfigV6::None,
        }
    }
}

/// Network stack IPv4 configuration.
#[cfg(feature = "proto-ipv4")]
#[derive(Debug, Clone, Default)]
pub enum ConfigV4 {
    /// Do not configure IPv4.
    #[default]
    None,
    /// Use a static IPv4 address configuration.
    Static(StaticConfigV4),
    /// Use DHCP to obtain an IP address configuration.
    #[cfg(feature = "dhcpv4")]
    Dhcp(DhcpConfig),
}

/// Network stack IPv6 configuration.
#[cfg(feature = "proto-ipv6")]
#[derive(Debug, Clone, Default)]
pub enum ConfigV6 {
    /// Do not configure IPv6.
    #[default]
    None,
    /// Use a static IPv6 address configuration.
    Static(StaticConfigV6),
}

/// Network stack runner.
///
/// You must call [`Runner::run()`] in a background task for the network stack to work.
pub struct Runner<'d, D: Driver> {
    driver: D,
    stack: Stack<'d>,
}

/// Network stack handle
///
/// Use this to create sockets. It's `Copy`, so you can pass
/// it by value instead of by reference.
#[derive(Copy, Clone)]
pub struct Stack<'d> {
    inner: &'d RefCell<Inner>,
}

pub(crate) struct Inner {
    pub(crate) sockets: SocketSet<'static>, // Lifetime type-erased.
    pub(crate) iface: Interface,
    /// Waker used for triggering polls.
    pub(crate) waker: WakerRegistration,
    /// Waker used for waiting for link up or config up.
    state_waker: WakerRegistration,
    hardware_address: HardwareAddress,
    next_local_port: u16,
    link_up: bool,
    #[cfg(feature = "proto-ipv4")]
    static_v4: Option<StaticConfigV4>,
    #[cfg(feature = "proto-ipv6")]
    static_v6: Option<StaticConfigV6>,
    #[cfg(feature = "dhcpv4")]
    dhcp_socket: Option<SocketHandle>,
    #[cfg(feature = "dns")]
    dns_socket: SocketHandle,
    #[cfg(feature = "dns")]
    dns_waker: WakerRegistration,
    #[cfg(feature = "dhcpv4-hostname")]
    hostname: *mut HostnameResources,
}

fn _assert_covariant<'a, 'b: 'a>(x: Stack<'b>) -> Stack<'a> {
    x
}

/// Create a new network stack.
pub fn new<'d, D: Driver, const SOCK: usize>(
    mut driver: D,
    config: Config,
    resources: &'d mut StackResources<SOCK>,
    random_seed: u64,
) -> (Stack<'d>, Runner<'d, D>) {
    let (hardware_address, medium) = to_smoltcp_hardware_address(driver.hardware_address());
    let mut iface_cfg = smoltcp::iface::Config::new(hardware_address);
    iface_cfg.random_seed = random_seed;

    let iface = Interface::new(
        iface_cfg,
        &mut DriverAdapter {
            inner: &mut driver,
            cx: None,
            medium,
        },
        instant_to_smoltcp(Instant::now()),
    );

    unsafe fn transmute_slice<T>(x: &mut [T]) -> &'static mut [T] {
        core::mem::transmute(x)
    }

    let sockets = resources.sockets.write([SocketStorage::EMPTY; SOCK]);
    #[allow(unused_mut)]
    let mut sockets: SocketSet<'static> = SocketSet::new(unsafe { transmute_slice(sockets) });

    let next_local_port = (random_seed % (LOCAL_PORT_MAX - LOCAL_PORT_MIN) as u64) as u16 + LOCAL_PORT_MIN;

    #[cfg(feature = "dns")]
    let dns_socket = sockets.add(dns::Socket::new(
        &[],
        managed::ManagedSlice::Borrowed(unsafe {
            transmute_slice(resources.queries.write([const { None }; MAX_QUERIES]))
        }),
    ));

    let mut inner = Inner {
        sockets,
        iface,
        waker: WakerRegistration::new(),
        state_waker: WakerRegistration::new(),
        next_local_port,
        hardware_address,
        link_up: false,
        #[cfg(feature = "proto-ipv4")]
        static_v4: None,
        #[cfg(feature = "proto-ipv6")]
        static_v6: None,
        #[cfg(feature = "dhcpv4")]
        dhcp_socket: None,
        #[cfg(feature = "dns")]
        dns_socket,
        #[cfg(feature = "dns")]
        dns_waker: WakerRegistration::new(),
        #[cfg(feature = "dhcpv4-hostname")]
        hostname: &mut resources.hostname,
    };

    #[cfg(feature = "proto-ipv4")]
    inner.set_config_v4(config.ipv4);
    #[cfg(feature = "proto-ipv6")]
    inner.set_config_v6(config.ipv6);
    inner.apply_static_config();

    let inner = &*resources.inner.write(RefCell::new(inner));
    let stack = Stack { inner };
    (stack, Runner { driver, stack })
}

fn to_smoltcp_hardware_address(addr: driver::HardwareAddress) -> (HardwareAddress, Medium) {
    match addr {
        #[cfg(feature = "medium-ethernet")]
        driver::HardwareAddress::Ethernet(eth) => (HardwareAddress::Ethernet(EthernetAddress(eth)), Medium::Ethernet),
        #[cfg(feature = "medium-ieee802154")]
        driver::HardwareAddress::Ieee802154(ieee) => (
            HardwareAddress::Ieee802154(Ieee802154Address::Extended(ieee)),
            Medium::Ieee802154,
        ),
        #[cfg(feature = "medium-ip")]
        driver::HardwareAddress::Ip => (HardwareAddress::Ip, Medium::Ip),

        #[allow(unreachable_patterns)]
        _ => panic!(
            "Unsupported medium {:?}. Make sure to enable the right medium feature in embassy-net's Cargo features.",
            addr
        ),
    }
}

impl<'d> Stack<'d> {
    fn with<R>(&self, f: impl FnOnce(&Inner) -> R) -> R {
        f(&self.inner.borrow())
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut Inner) -> R) -> R {
        f(&mut self.inner.borrow_mut())
    }

    /// Get the hardware address of the network interface.
    pub fn hardware_address(&self) -> HardwareAddress {
        self.with(|i| i.hardware_address)
    }

    /// Check whether the link is up.
    pub fn is_link_up(&self) -> bool {
        self.with(|i| i.link_up)
    }

    /// Check whether the network stack has a valid IP configuration.
    /// This is true if the network stack has a static IP configuration or if DHCP has completed
    pub fn is_config_up(&self) -> bool {
        let v4_up;
        let v6_up;

        #[cfg(feature = "proto-ipv4")]
        {
            v4_up = self.config_v4().is_some();
        }
        #[cfg(not(feature = "proto-ipv4"))]
        {
            v4_up = false;
        }

        #[cfg(feature = "proto-ipv6")]
        {
            v6_up = self.config_v6().is_some();
        }
        #[cfg(not(feature = "proto-ipv6"))]
        {
            v6_up = false;
        }

        v4_up || v6_up
    }

    /// Wait for the network device to obtain a link signal.
    pub async fn wait_link_up(&self) {
        self.wait(|| self.is_link_up()).await
    }

    /// Wait for the network device to lose link signal.
    pub async fn wait_link_down(&self) {
        self.wait(|| !self.is_link_up()).await
    }

    /// Wait for the network stack to obtain a valid IP configuration.
    ///
    /// ## Notes:
    /// - Ensure [`Runner::run`] has been started before using this function.
    ///
    /// - This function may never return (e.g. if no configuration is obtained through DHCP).
    /// The caller is supposed to handle a timeout for this case.
    ///
    /// ## Example
    /// ```ignore
    /// let config = embassy_net::Config::dhcpv4(Default::default());
    /// // Init network stack
    /// // NOTE: DHCP and DNS need one socket slot if enabled. This is why we're
    /// // provisioning space for 3 sockets here: one for DHCP, one for DNS, and one for your code (e.g. TCP).
    /// // If you use more sockets you must increase this. If you don't enable DHCP or DNS you can decrease it.
    /// static RESOURCES: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();
    /// let (stack, runner) = embassy_net::new(
    ///    driver,
    ///    config,
    ///    RESOURCES.init(embassy_net::StackResources::new()),
    ///    seed
    /// );
    /// // Launch network task that runs `runner.run().await`
    /// spawner.spawn(net_task(runner)).unwrap();
    /// // Wait for DHCP config
    /// stack.wait_config_up().await;
    /// // use the network stack
    /// // ...
    /// ```
    pub async fn wait_config_up(&self) {
        self.wait(|| self.is_config_up()).await
    }

    /// Wait for the network stack to lose a valid IP configuration.
    pub async fn wait_config_down(&self) {
        self.wait(|| !self.is_config_up()).await
    }

    fn wait<'a>(&'a self, mut predicate: impl FnMut() -> bool + 'a) -> impl Future<Output = ()> + 'a {
        poll_fn(move |cx| {
            if predicate() {
                Poll::Ready(())
            } else {
                // If the config is not up, we register a waker that is woken up
                // when a config is applied (static or DHCP).
                trace!("Waiting for config up");

                self.with_mut(|i| {
                    i.state_waker.register(cx.waker());
                });

                Poll::Pending
            }
        })
    }

    /// Get the current IPv4 configuration.
    ///
    /// If using DHCP, this will be None if DHCP hasn't been able to
    /// acquire an IP address, or Some if it has.
    #[cfg(feature = "proto-ipv4")]
    pub fn config_v4(&self) -> Option<StaticConfigV4> {
        self.with(|i| i.static_v4.clone())
    }

    /// Get the current IPv6 configuration.
    #[cfg(feature = "proto-ipv6")]
    pub fn config_v6(&self) -> Option<StaticConfigV6> {
        self.with(|i| i.static_v6.clone())
    }

    /// Set the IPv4 configuration.
    #[cfg(feature = "proto-ipv4")]
    pub fn set_config_v4(&self, config: ConfigV4) {
        self.with_mut(|i| {
            i.set_config_v4(config);
            i.apply_static_config();
        })
    }

    /// Set the IPv6 configuration.
    #[cfg(feature = "proto-ipv6")]
    pub fn set_config_v6(&self, config: ConfigV6) {
        self.with_mut(|i| {
            i.set_config_v6(config);
            i.apply_static_config();
        })
    }

    /// Make a query for a given name and return the corresponding IP addresses.
    #[cfg(feature = "dns")]
    pub async fn dns_query(
        &self,
        name: &str,
        qtype: dns::DnsQueryType,
    ) -> Result<Vec<IpAddress, { smoltcp::config::DNS_MAX_RESULT_COUNT }>, dns::Error> {
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
            self.with_mut(|i| {
                let socket = i.sockets.get_mut::<dns::Socket>(i.dns_socket);
                match socket.start_query(i.iface.context(), name, qtype) {
                    Ok(handle) => {
                        i.waker.wake();
                        Poll::Ready(Ok(handle))
                    }
                    Err(dns::StartQueryError::NoFreeSlot) => {
                        i.dns_waker.register(cx.waker());
                        Poll::Pending
                    }
                    Err(e) => Poll::Ready(Err(e)),
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
                let socket = i.sockets.get_mut::<dns::Socket>(i.dns_socket);
                socket.cancel_query(query);
                i.waker.wake();
                i.dns_waker.wake();
            })
        });

        let res = poll_fn(|cx| {
            self.with_mut(|i| {
                let socket = i.sockets.get_mut::<dns::Socket>(i.dns_socket);
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

#[cfg(feature = "multicast")]
impl<'d> Stack<'d> {
    /// Join a multicast group.
    pub fn join_multicast_group(&self, addr: impl Into<IpAddress>) -> Result<(), MulticastError> {
        self.with_mut(|i| i.iface.join_multicast_group(addr))
    }

    /// Leave a multicast group.
    pub fn leave_multicast_group(&self, addr: impl Into<IpAddress>) -> Result<(), MulticastError> {
        self.with_mut(|i| i.iface.leave_multicast_group(addr))
    }

    /// Get whether the network stack has joined the given multicast group.
    pub fn has_multicast_group(&self, addr: impl Into<IpAddress>) -> bool {
        self.with(|i| i.iface.has_multicast_group(addr))
    }
}

impl Inner {
    #[allow(clippy::absurd_extreme_comparisons)]
    pub fn get_local_port(&mut self) -> u16 {
        let res = self.next_local_port;
        self.next_local_port = if res >= LOCAL_PORT_MAX { LOCAL_PORT_MIN } else { res + 1 };
        res
    }

    #[cfg(feature = "proto-ipv4")]
    pub fn set_config_v4(&mut self, config: ConfigV4) {
        // Handle static config.
        self.static_v4 = match config.clone() {
            ConfigV4::None => None,
            #[cfg(feature = "dhcpv4")]
            ConfigV4::Dhcp(_) => None,
            ConfigV4::Static(c) => Some(c),
        };

        // Handle DHCP config.
        #[cfg(feature = "dhcpv4")]
        match config {
            ConfigV4::Dhcp(c) => {
                // Create the socket if it doesn't exist.
                if self.dhcp_socket.is_none() {
                    let socket = smoltcp::socket::dhcpv4::Socket::new();
                    let handle = self.sockets.add(socket);
                    self.dhcp_socket = Some(handle);
                }

                // Configure it
                let socket = self.sockets.get_mut::<dhcpv4::Socket>(unwrap!(self.dhcp_socket));
                socket.set_ignore_naks(c.ignore_naks);
                socket.set_max_lease_duration(c.max_lease_duration.map(crate::time::duration_to_smoltcp));
                socket.set_ports(c.server_port, c.client_port);
                socket.set_retry_config(c.retry_config);

                socket.set_outgoing_options(&[]);
                #[cfg(feature = "dhcpv4-hostname")]
                if let Some(h) = c.hostname {
                    // safety:
                    // - we just did set_outgoing_options([]) so we know the socket is no longer holding a reference.
                    // - we know this pointer lives for as long as the stack exists, because `new()` borrows
                    //   the resources for `'d`. Therefore it's OK to pass a reference to this to smoltcp.
                    let hostname = unsafe { &mut *self.hostname };

                    // create data
                    let data = hostname.data.write([0; MAX_HOSTNAME_LEN]);
                    data[..h.len()].copy_from_slice(h.as_bytes());
                    let data: &[u8] = &data[..h.len()];

                    // set the option.
                    let option = hostname.option.write(smoltcp::wire::DhcpOption { data, kind: 12 });
                    socket.set_outgoing_options(core::slice::from_ref(option));
                }

                socket.reset();
            }
            _ => {
                // Remove DHCP socket if any.
                if let Some(socket) = self.dhcp_socket {
                    self.sockets.remove(socket);
                    self.dhcp_socket = None;
                }
            }
        }
    }

    #[cfg(feature = "proto-ipv6")]
    pub fn set_config_v6(&mut self, config: ConfigV6) {
        self.static_v6 = match config {
            ConfigV6::None => None,
            ConfigV6::Static(c) => Some(c),
        };
    }

    fn apply_static_config(&mut self) {
        let mut addrs = Vec::new();
        #[cfg(feature = "dns")]
        let mut dns_servers: Vec<_, 6> = Vec::new();
        #[cfg(feature = "proto-ipv4")]
        let mut gateway_v4 = None;
        #[cfg(feature = "proto-ipv6")]
        let mut gateway_v6 = None;

        #[cfg(feature = "proto-ipv4")]
        if let Some(config) = &self.static_v4 {
            debug!("IPv4: UP");
            debug!("   IP address:      {:?}", config.address);
            debug!("   Default gateway: {:?}", config.gateway);

            unwrap!(addrs.push(IpCidr::Ipv4(config.address)).ok());
            gateway_v4 = config.gateway;
            #[cfg(feature = "dns")]
            for s in &config.dns_servers {
                debug!("   DNS server:      {:?}", s);
                unwrap!(dns_servers.push(s.clone().into()).ok());
            }
        } else {
            info!("IPv4: DOWN");
        }

        #[cfg(feature = "proto-ipv6")]
        if let Some(config) = &self.static_v6 {
            debug!("IPv6: UP");
            debug!("   IP address:      {:?}", config.address);
            debug!("   Default gateway: {:?}", config.gateway);

            unwrap!(addrs.push(IpCidr::Ipv6(config.address)).ok());
            gateway_v6 = config.gateway.into();
            #[cfg(feature = "dns")]
            for s in &config.dns_servers {
                debug!("   DNS server:      {:?}", s);
                unwrap!(dns_servers.push(s.clone().into()).ok());
            }
        } else {
            info!("IPv6: DOWN");
        }

        // Apply addresses
        self.iface.update_ip_addrs(|a| *a = addrs);

        // Apply gateways
        #[cfg(feature = "proto-ipv4")]
        if let Some(gateway) = gateway_v4 {
            unwrap!(self.iface.routes_mut().add_default_ipv4_route(gateway));
        } else {
            self.iface.routes_mut().remove_default_ipv4_route();
        }
        #[cfg(feature = "proto-ipv6")]
        if let Some(gateway) = gateway_v6 {
            unwrap!(self.iface.routes_mut().add_default_ipv6_route(gateway));
        } else {
            self.iface.routes_mut().remove_default_ipv6_route();
        }

        // Apply DNS servers
        #[cfg(feature = "dns")]
        if !dns_servers.is_empty() {
            let count = if dns_servers.len() > DNS_MAX_SERVER_COUNT {
                warn!("Number of DNS servers exceeds DNS_MAX_SERVER_COUNT, truncating list.");
                DNS_MAX_SERVER_COUNT
            } else {
                dns_servers.len()
            };
            self.sockets
                .get_mut::<smoltcp::socket::dns::Socket>(self.dns_socket)
                .update_servers(&dns_servers[..count]);
        }

        self.state_waker.wake();
    }

    fn poll<D: Driver>(&mut self, cx: &mut Context<'_>, driver: &mut D) {
        self.waker.register(cx.waker());

        let (_hardware_addr, medium) = to_smoltcp_hardware_address(driver.hardware_address());

        #[cfg(any(feature = "medium-ethernet", feature = "medium-ieee802154"))]
        {
            let do_set = match medium {
                #[cfg(feature = "medium-ethernet")]
                Medium::Ethernet => true,
                #[cfg(feature = "medium-ieee802154")]
                Medium::Ieee802154 => true,
                #[allow(unreachable_patterns)]
                _ => false,
            };
            if do_set {
                self.iface.set_hardware_addr(_hardware_addr);
            }
        }

        let timestamp = instant_to_smoltcp(Instant::now());
        let mut smoldev = DriverAdapter {
            cx: Some(cx),
            inner: driver,
            medium,
        };
        self.iface.poll(timestamp, &mut smoldev, &mut self.sockets);

        // Update link up
        let old_link_up = self.link_up;
        self.link_up = driver.link_state(cx) == LinkState::Up;

        // Print when changed
        if old_link_up != self.link_up {
            info!("link_up = {:?}", self.link_up);
            self.state_waker.wake();
        }

        #[cfg(feature = "dhcpv4")]
        if let Some(dhcp_handle) = self.dhcp_socket {
            let socket = self.sockets.get_mut::<dhcpv4::Socket>(dhcp_handle);

            let configure = if self.link_up {
                if old_link_up != self.link_up {
                    socket.reset();
                }
                match socket.poll() {
                    None => false,
                    Some(dhcpv4::Event::Deconfigured) => {
                        self.static_v4 = None;
                        true
                    }
                    Some(dhcpv4::Event::Configured(config)) => {
                        self.static_v4 = Some(StaticConfigV4 {
                            address: config.address,
                            gateway: config.router,
                            dns_servers: config.dns_servers,
                        });
                        true
                    }
                }
            } else if old_link_up {
                socket.reset();
                self.static_v4 = None;
                true
            } else {
                false
            };
            if configure {
                self.apply_static_config()
            }
        }

        if let Some(poll_at) = self.iface.poll_at(timestamp, &mut self.sockets) {
            let t = pin!(Timer::at(instant_from_smoltcp(poll_at)));
            if t.poll(cx).is_ready() {
                cx.waker().wake_by_ref();
            }
        }
    }
}

impl<'d, D: Driver> Runner<'d, D> {
    /// Run the network stack.
    ///
    /// You must call this in a background task, to process network events.
    pub async fn run(&mut self) -> ! {
        poll_fn(|cx| {
            self.stack.with_mut(|i| i.poll(cx, &mut self.driver));
            Poll::<()>::Pending
        })
        .await;
        unreachable!()
    }
}
