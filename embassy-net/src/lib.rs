#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    feature = "nightly",
    feature(type_alias_impl_trait, async_fn_in_trait, impl_trait_projections)
)]
#![cfg_attr(feature = "nightly", allow(incomplete_features))]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod device;
#[cfg(feature = "tcp")]
pub mod tcp;
#[cfg(feature = "udp")]
pub mod udp;
#[cfg(feature = "icmp")]
pub mod icmp;

use core::cell::RefCell;
use core::future::{poll_fn, Future};
use core::task::{Context, Poll};

use embassy_sync::waitqueue::WakerRegistration;
use embassy_time::{Instant, Timer};
use futures::pin_mut;
use heapless::Vec;
#[cfg(feature = "dhcpv4")]
use smoltcp::iface::SocketHandle;
use smoltcp::iface::{Interface, InterfaceBuilder, SocketSet, SocketStorage};
#[cfg(feature = "medium-ethernet")]
use smoltcp::iface::{Neighbor, NeighborCache, Route, Routes};
#[cfg(feature = "medium-ethernet")]
use smoltcp::phy::Medium;
#[cfg(feature = "dhcpv4")]
use smoltcp::socket::dhcpv4;
// smoltcp reexports
pub use smoltcp::time::{Duration as SmolDuration, Instant as SmolInstant};
#[cfg(feature = "medium-ethernet")]
pub use smoltcp::wire::{EthernetAddress, HardwareAddress};
pub use smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr};
#[cfg(feature = "proto-ipv6")]
pub use smoltcp::wire::{Ipv6Address, Ipv6Cidr};
#[cfg(feature = "udp")]
pub use smoltcp::{socket::udp::PacketMetadata, wire::IpListenEndpoint};
#[cfg(feature = "icmp")]
pub use smoltcp::{socket::icmp::PacketMetadata, wire::IpListenEndpoint};

use crate::device::{Device, DeviceAdapter, LinkState};

const LOCAL_PORT_MIN: u16 = 1025;
const LOCAL_PORT_MAX: u16 = 65535;

pub struct StackResources<const ADDR: usize, const SOCK: usize, const NEIGHBOR: usize> {
    addresses: [IpCidr; ADDR],
    sockets: [SocketStorage<'static>; SOCK],

    #[cfg(feature = "medium-ethernet")]
    routes: [Option<(IpCidr, Route)>; 1],
    #[cfg(feature = "medium-ethernet")]
    neighbor_cache: [Option<(IpAddress, Neighbor)>; NEIGHBOR],
}

impl<const ADDR: usize, const SOCK: usize, const NEIGHBOR: usize> StackResources<ADDR, SOCK, NEIGHBOR> {
    pub fn new() -> Self {
        Self {
            addresses: [IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 32); ADDR],
            sockets: [SocketStorage::EMPTY; SOCK],
            #[cfg(feature = "medium-ethernet")]
            routes: [None; 1],
            #[cfg(feature = "medium-ethernet")]
            neighbor_cache: [None; NEIGHBOR],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub address: Ipv4Cidr,
    pub gateway: Option<Ipv4Address>,
    pub dns_servers: Vec<Ipv4Address, 3>,
}

pub enum ConfigStrategy {
    Static(Config),
    #[cfg(feature = "dhcpv4")]
    Dhcp,
}

pub struct Stack<D: Device> {
    pub(crate) socket: RefCell<SocketStack>,
    inner: RefCell<Inner<D>>,
}

struct Inner<D: Device> {
    device: D,
    link_up: bool,
    config: Option<Config>,
    #[cfg(feature = "dhcpv4")]
    dhcp_socket: Option<SocketHandle>,
}

pub(crate) struct SocketStack {
    pub(crate) sockets: SocketSet<'static>,
    pub(crate) iface: Interface<'static>,
    pub(crate) waker: WakerRegistration,
    next_local_port: u16,
}

impl<D: Device + 'static> Stack<D> {
    pub fn new<const ADDR: usize, const SOCK: usize, const NEIGH: usize>(
        mut device: D,
        config: ConfigStrategy,
        resources: &'static mut StackResources<ADDR, SOCK, NEIGH>,
        random_seed: u64,
    ) -> Self {
        #[cfg(feature = "medium-ethernet")]
        let medium = device.capabilities().medium;

        #[cfg(feature = "medium-ethernet")]
        let ethernet_addr = if medium == Medium::Ethernet {
            device.ethernet_address()
        } else {
            [0, 0, 0, 0, 0, 0]
        };

        let mut b = InterfaceBuilder::new();
        b = b.ip_addrs(&mut resources.addresses[..]);
        b = b.random_seed(random_seed);

        #[cfg(feature = "medium-ethernet")]
        if medium == Medium::Ethernet {
            b = b.hardware_addr(HardwareAddress::Ethernet(EthernetAddress(ethernet_addr)));
            b = b.neighbor_cache(NeighborCache::new(&mut resources.neighbor_cache[..]));
            b = b.routes(Routes::new(&mut resources.routes[..]));
        }

        let iface = b.finalize(&mut DeviceAdapter {
            inner: &mut device,
            cx: None,
        });

        let sockets = SocketSet::new(&mut resources.sockets[..]);

        let next_local_port = (random_seed % (LOCAL_PORT_MAX - LOCAL_PORT_MIN) as u64) as u16 + LOCAL_PORT_MIN;

        let mut inner = Inner {
            device,
            link_up: false,
            config: None,
            #[cfg(feature = "dhcpv4")]
            dhcp_socket: None,
        };
        let mut socket = SocketStack {
            sockets,
            iface,
            waker: WakerRegistration::new(),
            next_local_port,
        };

        match config {
            ConfigStrategy::Static(config) => inner.apply_config(&mut socket, config),
            #[cfg(feature = "dhcpv4")]
            ConfigStrategy::Dhcp => {
                let handle = socket.sockets.add(smoltcp::socket::dhcpv4::Socket::new());
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

    pub fn ethernet_address(&self) -> [u8; 6] {
        self.with(|_s, i| i.device.ethernet_address())
    }

    pub fn is_link_up(&self) -> bool {
        self.with(|_s, i| i.link_up)
    }

    pub fn is_config_up(&self) -> bool {
        self.with(|_s, i| i.config.is_some())
    }

    pub fn config(&self) -> Option<Config> {
        self.with(|_s, i| i.config.clone())
    }

    pub async fn run(&self) -> ! {
        poll_fn(|cx| {
            self.with_mut(|s, i| i.poll(cx, s));
            Poll::<()>::Pending
        })
        .await;
        unreachable!()
    }
}

impl SocketStack {
    #[allow(clippy::absurd_extreme_comparisons)]
    pub fn get_local_port(&mut self) -> u16 {
        let res = self.next_local_port;
        self.next_local_port = if res >= LOCAL_PORT_MAX { LOCAL_PORT_MIN } else { res + 1 };
        res
    }
}

impl<D: Device + 'static> Inner<D> {
    fn apply_config(&mut self, s: &mut SocketStack, config: Config) {
        #[cfg(feature = "medium-ethernet")]
        let medium = self.device.capabilities().medium;

        debug!("Acquired IP configuration:");

        debug!("   IP address:      {}", config.address);
        self.set_ipv4_addr(s, config.address);

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

        self.config = Some(config)
    }

    #[allow(unused)] // used only with dhcp
    fn unapply_config(&mut self, s: &mut SocketStack) {
        #[cfg(feature = "medium-ethernet")]
        let medium = self.device.capabilities().medium;

        debug!("Lost IP configuration");
        self.set_ipv4_addr(s, Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0));
        #[cfg(feature = "medium-ethernet")]
        if medium == Medium::Ethernet {
            s.iface.routes_mut().remove_default_ipv4_route();
        }
        self.config = None
    }

    fn set_ipv4_addr(&mut self, s: &mut SocketStack, cidr: Ipv4Cidr) {
        s.iface.update_ip_addrs(|addrs| {
            let dest = addrs.iter_mut().next().unwrap();
            *dest = IpCidr::Ipv4(cidr);
        });
    }

    fn poll(&mut self, cx: &mut Context<'_>, s: &mut SocketStack) {
        s.waker.register(cx.waker());

        let timestamp = instant_to_smoltcp(Instant::now());
        let mut smoldev = DeviceAdapter {
            cx: Some(cx),
            inner: &mut self.device,
        };
        if s.iface.poll(timestamp, &mut smoldev, &mut s.sockets).is_err() {
            // If poll() returns error, it may not be done yet, so poll again later.
            cx.waker().wake_by_ref();
            return;
        }

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
                        let config = Config {
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

        if let Some(poll_at) = s.iface.poll_at(timestamp, &mut s.sockets) {
            let t = Timer::at(instant_from_smoltcp(poll_at));
            pin_mut!(t);
            if t.poll(cx).is_ready() {
                cx.waker().wake_by_ref();
            }
        }
    }
}

fn instant_to_smoltcp(instant: Instant) -> SmolInstant {
    SmolInstant::from_millis(instant.as_millis() as i64)
}

fn instant_from_smoltcp(instant: SmolInstant) -> Instant {
    Instant::from_millis(instant.total_millis() as u64)
}
