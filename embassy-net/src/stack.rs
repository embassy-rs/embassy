use core::cell::RefCell;
use core::future::Future;
use core::task::Context;
use core::task::Poll;
use embassy::time::{Instant, Timer};
use embassy::util::ThreadModeMutex;
use embassy::util::WakerRegistration;
use futures::pin_mut;
use smoltcp::iface::InterfaceBuilder;
#[cfg(feature = "medium-ethernet")]
use smoltcp::iface::{Neighbor, NeighborCache, Route, Routes};
use smoltcp::phy::Device as _;
use smoltcp::phy::Medium;
use smoltcp::socket::SocketSetItem;
use smoltcp::time::Instant as SmolInstant;
#[cfg(feature = "medium-ethernet")]
use smoltcp::wire::EthernetAddress;
use smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr};

use crate::config::Configurator;
use crate::config::Event;
use crate::device::{Device, DeviceAdapter, LinkState};
use crate::{Interface, SocketSet};

const LOCAL_PORT_MIN: u16 = 1025;
const LOCAL_PORT_MAX: u16 = 65535;

pub struct StackResources<const ADDR: usize, const SOCK: usize, const NEIGHBOR: usize> {
    addresses: [IpCidr; ADDR],
    sockets: [Option<SocketSetItem<'static>>; SOCK],

    #[cfg(feature = "medium-ethernet")]
    routes: [Option<(IpCidr, Route)>; 1],
    #[cfg(feature = "medium-ethernet")]
    neighbor_cache: [Option<(IpAddress, Neighbor)>; NEIGHBOR],
}

impl<const ADDR: usize, const SOCK: usize, const NEIGHBOR: usize>
    StackResources<ADDR, SOCK, NEIGHBOR>
{
    pub fn new() -> Self {
        const NONE_SOCKET: Option<SocketSetItem<'static>> = None;

        Self {
            addresses: [IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 32); ADDR],
            sockets: [NONE_SOCKET; SOCK],
            routes: [None; 1],
            neighbor_cache: [None; NEIGHBOR],
        }
    }
}

static STACK: ThreadModeMutex<RefCell<Option<Stack>>> = ThreadModeMutex::new(RefCell::new(None));

pub(crate) struct Stack {
    iface: Interface,
    pub sockets: SocketSet,
    link_up: bool,
    config_up: bool,
    next_local_port: u16,
    configurator: &'static mut dyn Configurator,
    waker: WakerRegistration,
}

impl Stack {
    pub(crate) fn with<R>(f: impl FnOnce(&mut Stack) -> R) -> R {
        let mut stack = STACK.borrow().borrow_mut();
        let stack = stack.as_mut().unwrap();
        f(stack)
    }

    pub fn get_local_port(&mut self) -> u16 {
        let res = self.next_local_port;
        self.next_local_port = if res >= LOCAL_PORT_MAX {
            LOCAL_PORT_MIN
        } else {
            res + 1
        };
        res
    }

    pub(crate) fn wake(&mut self) {
        self.waker.wake()
    }

    fn poll_configurator(&mut self, timestamp: SmolInstant) {
        let medium = self.iface.device().capabilities().medium;

        match self
            .configurator
            .poll(&mut self.iface, &mut self.sockets, timestamp)
        {
            Event::NoChange => {}
            Event::Configured(config) => {
                debug!("Acquired IP configuration:");

                debug!("   IP address:      {}", config.address);
                set_ipv4_addr(&mut self.iface, config.address);

                #[cfg(feature = "medium-ethernet")]
                if medium == Medium::Ethernet {
                    if let Some(gateway) = config.gateway {
                        debug!("   Default gateway: {}", gateway);
                        self.iface
                            .routes_mut()
                            .add_default_ipv4_route(gateway)
                            .unwrap();
                    } else {
                        debug!("   Default gateway: None");
                        self.iface.routes_mut().remove_default_ipv4_route();
                    }
                }
                for (i, s) in config.dns_servers.iter().enumerate() {
                    debug!("   DNS server {}:    {}", i, s);
                }

                self.config_up = true;
            }
            Event::Deconfigured => {
                debug!("Lost IP configuration");
                set_ipv4_addr(&mut self.iface, Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0));
                #[cfg(feature = "medium-ethernet")]
                if medium == Medium::Ethernet {
                    self.iface.routes_mut().remove_default_ipv4_route();
                }
                self.config_up = false;
            }
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) {
        self.iface.device_mut().device.register_waker(cx.waker());
        self.waker.register(cx.waker());

        let timestamp = instant_to_smoltcp(Instant::now());
        if let Err(_) = self.iface.poll(&mut self.sockets, timestamp) {
            // If poll() returns error, it may not be done yet, so poll again later.
            cx.waker().wake_by_ref();
            return;
        }

        // Update link up
        let old_link_up = self.link_up;
        self.link_up = self.iface.device_mut().device.link_state() == LinkState::Up;

        // Print when changed
        if old_link_up != self.link_up {
            if self.link_up {
                info!("Link up!");
            } else {
                info!("Link down!");
            }
        }

        if old_link_up || self.link_up {
            self.poll_configurator(timestamp)
        }

        if let Some(poll_at) = self.iface.poll_at(&mut self.sockets, timestamp) {
            let t = Timer::at(instant_from_smoltcp(poll_at));
            pin_mut!(t);
            if t.poll(cx).is_ready() {
                cx.waker().wake_by_ref();
            }
        }
    }
}

fn set_ipv4_addr(iface: &mut Interface, cidr: Ipv4Cidr) {
    iface.update_ip_addrs(|addrs| {
        let dest = addrs.iter_mut().next().unwrap();
        *dest = IpCidr::Ipv4(cidr);
    });
}

/// Initialize embassy_net.
/// This function must be called from thread mode.
pub fn init<const ADDR: usize, const SOCK: usize, const NEIGH: usize>(
    device: &'static mut dyn Device,
    configurator: &'static mut dyn Configurator,
    resources: &'static mut StackResources<ADDR, SOCK, NEIGH>,
) {
    let medium = device.capabilities().medium;

    #[cfg(feature = "medium-ethernet")]
    let ethernet_addr = if medium == Medium::Ethernet {
        device.ethernet_address()
    } else {
        [0, 0, 0, 0, 0, 0]
    };

    let mut b = InterfaceBuilder::new(DeviceAdapter::new(device));
    b = b.ip_addrs(&mut resources.addresses[..]);

    #[cfg(feature = "medium-ethernet")]
    if medium == Medium::Ethernet {
        b = b.ethernet_addr(EthernetAddress(ethernet_addr));
        b = b.neighbor_cache(NeighborCache::new(&mut resources.neighbor_cache[..]));
        b = b.routes(Routes::new(&mut resources.routes[..]));
    }

    let iface = b.finalize();

    let sockets = SocketSet::new(&mut resources.sockets[..]);

    let local_port = loop {
        let mut res = [0u8; 2];
        rand(&mut res);
        let port = u16::from_le_bytes(res);
        if port >= LOCAL_PORT_MIN && port <= LOCAL_PORT_MAX {
            break port;
        }
    };

    let stack = Stack {
        iface,
        sockets,
        link_up: false,
        config_up: false,
        configurator,
        next_local_port: local_port,
        waker: WakerRegistration::new(),
    };

    *STACK.borrow().borrow_mut() = Some(stack);
}

pub fn is_init() -> bool {
    STACK.borrow().borrow().is_some()
}

pub fn is_link_up() -> bool {
    STACK.borrow().borrow().as_ref().unwrap().link_up
}

pub fn is_config_up() -> bool {
    STACK.borrow().borrow().as_ref().unwrap().config_up
}

pub async fn run() {
    futures::future::poll_fn(|cx| {
        Stack::with(|stack| stack.poll(cx));
        Poll::<()>::Pending
    })
    .await
}

fn instant_to_smoltcp(instant: Instant) -> SmolInstant {
    SmolInstant::from_millis(instant.as_millis() as i64)
}

fn instant_from_smoltcp(instant: SmolInstant) -> Instant {
    Instant::from_millis(instant.total_millis() as u64)
}

extern "Rust" {
    fn _embassy_rand(buf: &mut [u8]);
}

fn rand(buf: &mut [u8]) {
    unsafe { _embassy_rand(buf) }
}
