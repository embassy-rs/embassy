//! Reset the bus, enumerate every device behind every hub, and print the tree.
//!
//! Walks the bus breadth-first: enumerate the device on the root port, and whenever one
//! turns out to be a hub, register it and enumerate whatever is on its ports, until the
//! tree is exhausted or the USB tier limit is reached.
//!
//! Device types come from the class drivers in `embassy-usb-host` rather than from
//! hand-rolled class-code checks, so a device is reported the same way here as the driver
//! that would actually claim it sees it. That matters for the awkward ones: smart-card
//! readers are routinely shipped under the vendor-specific class, and a MIDI device is an
//! audio-class device wearing a particular subclass.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select3};
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_time::{Duration, Instant, Timer};
use embassy_usb_driver::Speed;
use embassy_usb_driver::host::UsbHostAllocator;
use embassy_usb_host::class::cdc_acm::find_cdc_acm;
use embassy_usb_host::class::gip::find_gip;
use embassy_usb_host::class::hid::find_hid;
use embassy_usb_host::class::hub::{HubEvent, HubHandler};
use embassy_usb_host::class::msc::find_msc;
use embassy_usb_host::class::uac::descriptors::AudioInterfaceCollection;
use embassy_usb_host::class::vcp::cp210x::{find_cp210x, id};
use embassy_usb_host::descriptor::{ConfigurationDescriptorChain, DeviceDescriptor};
use embassy_usb_host::handler::{EnumerationInfo, HandlerEvent};
use embassy_usb_host::{BusHandle, BusRoute, BusState};
use heapless::Vec;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::host::InterruptHandler<USB>;
});

/// Devices the tree can hold, hubs included.
const MAX_DEVICES: usize = 16;
/// Ports handled per hub.
const MAX_PORTS: usize = 8;
/// Room for one configuration descriptor and everything nested in it.
const CONFIG_BUF: usize = 512;

/// Hub class code (USB 2.0 §11.23.1). A hub declares this at the device level, unlike every
/// other class here, which lives on an interface.
const CLASS_HUB: u8 = 0x09;

/// How many hubs may be chained below the host.
///
/// USB 2.0 §4.1.1 allows seven tiers. The host is tier 1 and a device occupies the last
/// tier, which leaves five for cascaded hubs. A deeper hub is out of spec, so it is
/// reported and left unscanned rather than descended into.
const MAX_HUB_DEPTH: u8 = 5;

/// A hub reports its populated ports one after another once registered. When nothing new
/// has arrived for this long, take the hub as having reported everything it has.
const PORT_QUIET: Duration = Duration::from_millis(1500);
/// Upper bound on one hub's scan, in case events keep trickling in.
const PORT_SCAN_MAX: Duration = Duration::from_secs(6);

/// What a device turned out to be.
#[derive(Clone, Copy, Format)]
enum Kind {
    Hub,
    Hid {
        report_len: u16,
    },
    MassStorage,
    CdcAcm,
    Cp210x,
    Gip,
    Audio,
    /// Nothing claimed it; the device's own class code is shown for triage.
    Unknown {
        class: u8,
    },
}

/// One enumerated device, and where it hangs.
///
/// The position is tracked here rather than read back out of [`EnumerationInfo::route`],
/// which cannot answer it. A `BusRoute` only carries a hub address and port when it is
/// `Translated`, and everything on a full-speed bus behind full-speed hubs is `Direct`.
/// Even when it is `Translated`, the `SplitInfo` names the transaction translator -- the
/// hub doing the speed conversion, which for a device several tiers down is not its
/// parent. The route describes how to talk to a device, not where it sits.
#[derive(Clone, Copy)]
struct Node {
    info: EnumerationInfo,
    /// Index of the hub this sits on, or `None` for the device on the root port.
    parent: Option<u8>,
    /// Port on that hub, as the hub numbers them.
    port: u8,
    kind: Kind,
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = embassy_rp::usb::host::Driver::new(p.USB, Irqs);

    static BUS_STATE: BusState = BusState::new();
    let (mut bus_ctrl, bus) = embassy_usb_host::bus(driver, &BUS_STATE);

    info!("waiting for a device on the root port ...");
    // Drives the bus reset and returns the speed the device settled on.
    let speed = bus_ctrl.wait_for_connection().await;

    let mut nodes: Vec<Node, MAX_DEVICES> = Vec::new();
    let mut config_buf = [0u8; CONFIG_BUF];

    match bus.enumerate(BusRoute::Direct(speed), &mut config_buf).await {
        Ok((info, len)) => {
            let kind = classify(&info.device_desc, &config_buf[..len]);
            // The table is empty here, so this cannot fail.
            nodes
                .push(Node {
                    info,
                    parent: None,
                    port: 0,
                    kind,
                })
                .ok();
        }
        Err(e) => {
            error!("root port enumeration failed: {:?}", e);
            idle().await
        }
    }

    // Breadth-first: children are appended as they are found, so walking `nodes` in order
    // reaches every hub exactly once and needs no separate queue.
    let mut next = 0;
    while next < nodes.len() {
        let node = nodes[next];
        let index = next as u8;
        next += 1;

        if !matches!(node.kind, Kind::Hub) {
            continue;
        }
        let depth = depth_of(&nodes, index);
        if depth >= MAX_HUB_DEPTH {
            warn!(
                "hub at address {} is {} hubs deep, past the {} USB allows - not descending",
                node.info.device_address, depth, MAX_HUB_DEPTH
            );
            continue;
        }

        scan_hub(&bus, &node.info, index, &mut nodes).await;
    }

    info!("");
    info!("=== USB device tree ===");
    print_children(&nodes, None, 0);
    info!("{} devices total", nodes.len());

    idle().await
}

/// How many hubs sit between `index` and the host, by walking up the parent chain.
fn depth_of(nodes: &[Node], index: u8) -> u8 {
    let mut node = &nodes[index as usize];
    let mut depth = 0;
    while let Some(parent) = node.parent {
        depth += 1;
        node = &nodes[parent as usize];
    }
    depth
}

/// Register one hub and enumerate whatever is on its ports, appending each to `nodes`.
async fn scan_hub<'d, A: UsbHostAllocator<'d>>(
    bus: &BusHandle<'d, A>,
    hub_info: &EnumerationInfo,
    parent: u8,
    nodes: &mut Vec<Node, MAX_DEVICES>,
) {
    // Dropped at the end of the scan, which releases the pipes it holds for the next hub.
    let mut hub = match HubHandler::<_, MAX_PORTS>::try_register(bus, hub_info).await {
        Ok(hub) => hub,
        Err(e) => {
            warn!("address {}: hub registration failed: {:?}", hub_info.device_address, e);
            return;
        }
    };

    let mut cap = Timer::at(Instant::now() + PORT_SCAN_MAX);

    loop {
        // `PORT_QUIET` restarts on every iteration, so it measures the gap since the last
        // event rather than the length of the scan; `cap` bounds the scan as a whole.
        let event = match select3(hub.wait_for_event(), Timer::after(PORT_QUIET), &mut cap).await {
            Either3::First(Ok(event)) => event,
            Either3::First(Err(e)) => {
                warn!("hub event failed: {:?}", e);
                return;
            }
            // Nothing new for a while: the hub has reported everything attached to it.
            Either3::Second(_) => return,
            Either3::Third(_) => {
                warn!("hub scan hit its {} s cap", PORT_SCAN_MAX.as_secs());
                return;
            }
        };

        let HandlerEvent::HandlerEvent(HubEvent::DeviceDetected { port, speed }) = event else {
            continue;
        };

        let mut config_buf = [0u8; CONFIG_BUF];
        match hub.enumerate_port(&mut config_buf, port, speed).await {
            Ok((info, len)) => {
                let kind = classify(&info.device_desc, &config_buf[..len]);
                let node = Node {
                    info,
                    parent: Some(parent),
                    port,
                    kind,
                };
                if nodes.push(node).is_err() {
                    warn!("device table full at {} entries - stopping the walk", MAX_DEVICES);
                    return;
                }
            }
            Err(e) => warn!("port {}: enumeration failed: {:?}", port, e),
        }
    }
}

/// Work out what a device is, by asking the class drivers that would claim it.
///
/// Order matters where classes overlap, and the overlaps are called out below.
fn classify(dev: &DeviceDescriptor, config: &[u8]) -> Kind {
    // Hubs are the one class declared on the device descriptor rather than an interface.
    if dev.device_class == CLASS_HUB {
        return Kind::Hub;
    }

    if let Some(hid) = find_hid(config) {
        return Kind::Hid {
            report_len: hid.report_descriptor_len,
        };
    }
    if find_msc(config).is_some() {
        return Kind::MassStorage;
    }
    if find_cdc_acm(config).is_some() {
        return Kind::CdcAcm;
    }
    // Vendor-specific, so it is identified by VID/PID rather than by class.
    if dev.vendor_id == id::VID_SILABS && find_cp210x(config, 0).is_some() {
        return Kind::Cp210x;
    }
    if find_gip(config).is_some() {
        return Kind::Gip;
    }
    if let Ok(cfg) = ConfigurationDescriptorChain::try_from_slice(config) {
        if AudioInterfaceCollection::try_from_configuration(&cfg).is_ok() {
            return Kind::Audio;
        }
    }

    // A composite device declares class 0 at the device level and puts the real class on
    // each interface, so the first interface's class is the one worth showing.
    let class = ConfigurationDescriptorChain::try_from_slice(config)
        .ok()
        .and_then(|cfg| cfg.iter_interface().next().map(|iface| iface.interface_class))
        .unwrap_or(dev.device_class);

    Kind::Unknown { class }
}

/// Print every child of `parent`, then each of their children, indented by depth.
fn print_children(nodes: &[Node], parent: Option<u8>, depth: usize) {
    const INDENT: &str = "                                ";

    for (i, node) in nodes.iter().enumerate() {
        if node.parent != parent {
            continue;
        }

        let pad = &INDENT[..(depth * 2).min(INDENT.len())];
        let desc = &node.info.device_desc;
        if node.parent.is_none() {
            info!(
                "{}root port: {:04x}:{:04x} addr {} {} {}",
                pad,
                desc.vendor_id,
                desc.product_id,
                node.info.device_address,
                Sp(node.info.speed()),
                node.kind
            );
        } else {
            info!(
                "{}port {}: {:04x}:{:04x} addr {} {} {}",
                pad,
                node.port,
                desc.vendor_id,
                desc.product_id,
                node.info.device_address,
                Sp(node.info.speed()),
                node.kind
            );
        }

        print_children(nodes, Some(i as u8), depth + 1);
    }
}

/// [`Speed`] prints as its enum name; this shortens it for the tree.
struct Sp(Speed);

impl Format for Sp {
    fn format(&self, f: defmt::Formatter) {
        match self.0 {
            Speed::Low => defmt::write!(f, "LS"),
            Speed::Full => defmt::write!(f, "FS"),
            Speed::High => defmt::write!(f, "HS"),
        }
    }
}

async fn idle() -> ! {
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
