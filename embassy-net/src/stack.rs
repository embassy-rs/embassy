use core::cell::UnsafeCell;
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
use smoltcp::phy::{Device as _, Medium};
#[cfg(feature = "dhcpv4")]
use smoltcp::socket::dhcpv4;
use smoltcp::time::Instant as SmolInstant;
#[cfg(feature = "medium-ethernet")]
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress};
use smoltcp::wire::{IpCidr, Ipv4Address, Ipv4Cidr};

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
    pub(crate) socket: UnsafeCell<SocketStack>,
    inner: UnsafeCell<Inner<D>>,
}

struct Inner<D: Device> {
    device: DeviceAdapter<D>,
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

unsafe impl<D: Device> Send for Stack<D> {}

impl<D: Device + 'static> Stack<D> {
    pub fn new<const ADDR: usize, const SOCK: usize, const NEIGH: usize>(
        device: D,
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

        let mut device = DeviceAdapter::new(device);

        let mut b = InterfaceBuilder::new();
        b = b.ip_addrs(&mut resources.addresses[..]);
        b = b.random_seed(random_seed);

        #[cfg(feature = "medium-ethernet")]
        if medium == Medium::Ethernet {
            b = b.hardware_addr(HardwareAddress::Ethernet(EthernetAddress(ethernet_addr)));
            b = b.neighbor_cache(NeighborCache::new(&mut resources.neighbor_cache[..]));
            b = b.routes(Routes::new(&mut resources.routes[..]));
        }

        let iface = b.finalize(&mut device);

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
            socket: UnsafeCell::new(socket),
            inner: UnsafeCell::new(inner),
        }
    }

    /// SAFETY: must not call reentrantly.
    unsafe fn with<R>(&self, f: impl FnOnce(&SocketStack, &Inner<D>) -> R) -> R {
        f(&*self.socket.get(), &*self.inner.get())
    }

    /// SAFETY: must not call reentrantly.
    unsafe fn with_mut<R>(&self, f: impl FnOnce(&mut SocketStack, &mut Inner<D>) -> R) -> R {
        f(&mut *self.socket.get(), &mut *self.inner.get())
    }

    pub fn ethernet_address(&self) -> [u8; 6] {
        unsafe { self.with(|_s, i| i.device.device.ethernet_address()) }
    }

    pub fn is_link_up(&self) -> bool {
        unsafe { self.with(|_s, i| i.link_up) }
    }

    pub fn is_config_up(&self) -> bool {
        unsafe { self.with(|_s, i| i.config.is_some()) }
    }

    pub fn config(&self) -> Option<Config> {
        unsafe { self.with(|_s, i| i.config.clone()) }
    }

    pub async fn run(&self) -> ! {
        poll_fn(|cx| {
            unsafe { self.with_mut(|s, i| i.poll(cx, s)) }
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
        self.device.device.register_waker(cx.waker());
        s.waker.register(cx.waker());

        let timestamp = instant_to_smoltcp(Instant::now());
        if s.iface.poll(timestamp, &mut self.device, &mut s.sockets).is_err() {
            // If poll() returns error, it may not be done yet, so poll again later.
            cx.waker().wake_by_ref();
            return;
        }

        // Update link up
        let old_link_up = self.link_up;
        self.link_up = self.device.device.link_state() == LinkState::Up;

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
                        let mut dns_servers = Vec::new();
                        for s in &config.dns_servers {
                            if let Some(addr) = s {
                                dns_servers.push(addr.clone()).unwrap();
                            }
                        }

                        self.apply_config(
                            s,
                            Config {
                                address: config.address,
                                gateway: config.router,
                                dns_servers,
                            },
                        )
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
