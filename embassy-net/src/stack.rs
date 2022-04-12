use core::cell::RefCell;
use core::future::Future;
use core::task::Context;
use core::task::Poll;
use embassy::blocking_mutex::ThreadModeMutex;
use embassy::time::{Instant, Timer};
use embassy::waitqueue::WakerRegistration;
use futures::pin_mut;
use smoltcp::iface::InterfaceBuilder;
use smoltcp::iface::SocketStorage;
use smoltcp::time::Instant as SmolInstant;
use smoltcp::wire::{IpCidr, Ipv4Address, Ipv4Cidr};

#[cfg(feature = "medium-ethernet")]
use smoltcp::iface::{Neighbor, NeighborCache, Route, Routes};
#[cfg(feature = "medium-ethernet")]
use smoltcp::phy::{Device as _, Medium};
#[cfg(feature = "medium-ethernet")]
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress};

use crate::config::Configurator;
use crate::config::Event;
use crate::device::{Device, DeviceAdapter, LinkState};
use crate::Interface;

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

impl<const ADDR: usize, const SOCK: usize, const NEIGHBOR: usize>
    StackResources<ADDR, SOCK, NEIGHBOR>
{
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

static STACK: ThreadModeMutex<RefCell<Option<Stack>>> = ThreadModeMutex::new(RefCell::new(None));

pub(crate) struct Stack {
    pub iface: Interface,
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

    #[allow(clippy::absurd_extreme_comparisons)]
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
        #[cfg(feature = "medium-ethernet")]
        let medium = self.iface.device().capabilities().medium;

        match self.configurator.poll(&mut self.iface, timestamp) {
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
        if self.iface.poll(timestamp).is_err() {
            // If poll() returns error, it may not be done yet, so poll again later.
            cx.waker().wake_by_ref();
            return;
        }

        // Update link up
        let old_link_up = self.link_up;
        self.link_up = self.iface.device_mut().device.link_state() == LinkState::Up;

        // Print when changed
        if old_link_up != self.link_up {
            info!("link_up = {:?}", self.link_up);
        }

        if old_link_up || self.link_up {
            self.poll_configurator(timestamp)
        }

        if let Some(poll_at) = self.iface.poll_at(timestamp) {
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
    #[cfg(feature = "medium-ethernet")]
    let medium = device.capabilities().medium;

    #[cfg(feature = "medium-ethernet")]
    let ethernet_addr = if medium == Medium::Ethernet {
        device.ethernet_address()
    } else {
        [0, 0, 0, 0, 0, 0]
    };

    let mut b = InterfaceBuilder::new(DeviceAdapter::new(device), &mut resources.sockets[..]);
    b = b.ip_addrs(&mut resources.addresses[..]);

    #[cfg(feature = "medium-ethernet")]
    if medium == Medium::Ethernet {
        b = b.hardware_addr(HardwareAddress::Ethernet(EthernetAddress(ethernet_addr)));
        b = b.neighbor_cache(NeighborCache::new(&mut resources.neighbor_cache[..]));
        b = b.routes(Routes::new(&mut resources.routes[..]));
    }

    let iface = b.finalize();

    let local_port = loop {
        let mut res = [0u8; 2];
        rand(&mut res);
        let port = u16::from_le_bytes(res);
        if (LOCAL_PORT_MIN..=LOCAL_PORT_MAX).contains(&port) {
            break port;
        }
    };

    let stack = Stack {
        iface,
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

pub async fn run() -> ! {
    futures::future::poll_fn(|cx| {
        Stack::with(|stack| stack.poll(cx));
        Poll::<()>::Pending
    })
    .await;
    unreachable!()
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
