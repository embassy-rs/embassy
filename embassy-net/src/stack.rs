use core::future::Future;
use core::task::Context;
use core::task::Poll;
use core::{cell::RefCell, future};
use embassy::time::{Instant, Timer};
use embassy::util::ThreadModeMutex;
use embassy::util::{Forever, WakerRegistration};
use futures::pin_mut;
use smoltcp::iface::{InterfaceBuilder, Neighbor, NeighborCache, Route, Routes};
use smoltcp::phy::Device as _;
use smoltcp::phy::Medium;
use smoltcp::socket::SocketSetItem;
use smoltcp::time::Instant as SmolInstant;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};

use crate::device::{Device, DeviceAdapter};
use crate::fmt::*;
use crate::{
    config::{Config, Configurator},
    device::LinkState,
};
use crate::{Interface, SocketSet};

const ADDRESSES_LEN: usize = 1;
const NEIGHBOR_CACHE_LEN: usize = 8;
const SOCKETS_LEN: usize = 2;
const LOCAL_PORT_MIN: u16 = 1025;
const LOCAL_PORT_MAX: u16 = 65535;

struct StackResources {
    addresses: [IpCidr; ADDRESSES_LEN],
    neighbor_cache: [Option<(IpAddress, Neighbor)>; NEIGHBOR_CACHE_LEN],
    sockets: [Option<SocketSetItem<'static>>; SOCKETS_LEN],
    routes: [Option<(IpCidr, Route)>; 1],
}

static STACK_RESOURCES: Forever<StackResources> = Forever::new();
static STACK: ThreadModeMutex<RefCell<Option<Stack>>> = ThreadModeMutex::new(RefCell::new(None));

pub(crate) struct Stack {
    iface: Interface,
    pub sockets: SocketSet,
    link_up: bool,
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
        if let Some(config) = self
            .configurator
            .poll(&mut self.iface, &mut self.sockets, timestamp)
        {
            let medium = self.iface.device().capabilities().medium;

            let (addr, gateway) = match config {
                Config::Up(config) => (config.address.into(), Some(config.gateway)),
                Config::Down => (IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 32), None),
            };

            self.iface.update_ip_addrs(|addrs| {
                let curr_addr = &mut addrs[0];
                if *curr_addr != addr {
                    info!("IPv4 address: {:?} -> {:?}", *curr_addr, addr);
                    *curr_addr = addr;
                }
            });

            if medium == Medium::Ethernet {
                self.iface.routes_mut().update(|r| {
                    let cidr = IpCidr::new(IpAddress::v4(0, 0, 0, 0), 0);
                    let curr_gateway = r.get(&cidr).map(|r| r.via_router);

                    if curr_gateway != gateway.map(|a| a.into()) {
                        info!("IPv4 gateway: {:?} -> {:?}", curr_gateway, gateway);
                        if let Some(gateway) = gateway {
                            r.insert(cidr, Route::new_ipv4_gateway(gateway)).unwrap();
                        } else {
                            r.remove(&cidr);
                        }
                    }
                });
            }
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) {
        self.iface.device_mut().device.register_waker(cx.waker());
        self.waker.register(cx.waker());

        let timestamp = instant_to_smoltcp(Instant::now());
        if let Err(e) = self.iface.poll(&mut self.sockets, timestamp) {
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

/// Initialize embassy_net.
/// This function must be called from thread mode.
pub fn init(device: &'static mut dyn Device, configurator: &'static mut dyn Configurator) {
    const NONE_SOCKET: Option<SocketSetItem<'static>> = None;
    let res = STACK_RESOURCES.put(StackResources {
        addresses: [IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 32)],
        neighbor_cache: [None; NEIGHBOR_CACHE_LEN],
        sockets: [NONE_SOCKET; SOCKETS_LEN],
        routes: [None; 1],
    });

    let ethernet_addr = EthernetAddress([0x02, 0x02, 0x02, 0x02, 0x02, 0x02]);

    let medium = device.capabilities().medium;

    let mut b = InterfaceBuilder::new(DeviceAdapter::new(device));
    b = b.ip_addrs(&mut res.addresses[..]);

    if medium == Medium::Ethernet {
        b = b.ethernet_addr(ethernet_addr);
        b = b.neighbor_cache(NeighborCache::new(&mut res.neighbor_cache[..]));
        b = b.routes(Routes::new(&mut res.routes[..]));
    }

    let iface = b.finalize();

    let sockets = SocketSet::new(&mut res.sockets[..]);

    let local_port = loop {
        let mut res = [0u8; 2];
        embassy::rand::rand(&mut res);
        let port = u16::from_le_bytes(res);
        if port >= LOCAL_PORT_MIN && port <= LOCAL_PORT_MAX {
            break port;
        }
    };

    let stack = Stack {
        iface,
        sockets,
        link_up: false,
        configurator,
        next_local_port: local_port,
        waker: WakerRegistration::new(),
    };

    *STACK.borrow().borrow_mut() = Some(stack);
}

pub fn is_init() -> bool {
    STACK.borrow().borrow().is_some()
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
