//! This example shows how to use USB (Universal Serial Bus) in the RP2350 chip.
//!
//! This is a CDC-NCM class implementation, aka Ethernet over USB. It also
//! runs a DHCP server to automatically configure IPv4 on the host machine.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::dhcpd::{DhcpdConfig, DhcpdLease};
use embassy_net::{Ipv4Cidr, StackResources};
use embassy_rp::clocks::RoscRng;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Duration;
use embassy_usb::class::cdc_ncm::embassy_net::{Device, Runner, State as NetState};
use embassy_usb::class::cdc_ncm::{CdcNcmClass, State};
use embassy_usb::{Builder, Config, UsbDevice};
use heapless::Vec;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

type MyDriver = Driver<'static, peripherals::USB>;

const MTU: usize = 1514;

const DHCPD_MAX_LEASES: usize = 32;

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, MyDriver>) -> ! {
    device.run().await
}

#[embassy_executor::task]
async fn usb_ncm_task(class: Runner<'static, MyDriver, MTU>) -> ! {
    class.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static, MTU>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn dhcpcd_task(mut runner: embassy_net::dhcpd::Runner<'static, DHCPD_MAX_LEASES, RoscRng>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut rng = RoscRng;

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs);

    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-Ethernet example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Create embassy-usb DeviceBuilder using the driver and config.
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    let mut builder = Builder::new(
        driver,
        config,
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut [], // no msos descriptors
        &mut CONTROL_BUF.init([0; 128])[..],
    );

    // Our MAC addr.
    let our_mac_addr = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
    // Host's MAC addr. This is the MAC the host "thinks" its USB-to-ethernet adapter has.
    let host_mac_addr = [0x88, 0x88, 0x88, 0x88, 0x88, 0x88];

    // Create classes on the builder.
    static STATE: StaticCell<State> = StaticCell::new();
    let class = CdcNcmClass::new(&mut builder, STATE.init(State::new()), host_mac_addr, 64);

    // Build the builder.
    let usb = builder.build();

    spawner.spawn(unwrap!(usb_task(usb)));

    static NET_STATE: StaticCell<NetState<MTU, 4, 4>> = StaticCell::new();
    let (runner, device) = class.into_embassy_net_device::<MTU, 4, 4>(NET_STATE.init(NetState::new()), our_mac_addr);
    spawner.spawn(unwrap!(usb_ncm_task(runner)));

    // We use a static IPv4 address ourselves
    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(core::net::Ipv4Addr::new(10, 42, 0, 61), 24),
        dns_servers: Default::default(),
        gateway: None,
    });

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    spawner.spawn(unwrap!(net_task(runner)));

    // Confgire and start the DHCPv4 server
    static DHCPD_CONFIG: StaticCell<DhcpdConfig> = StaticCell::new();

    let dhcpd_config: &'static mut DhcpdConfig = DHCPD_CONFIG.init(DhcpdConfig::default());
    dhcpd_config.server_ip = core::net::Ipv4Addr::new(10, 42, 0, 61);
    dhcpd_config.range_start = core::net::Ipv4Addr::new(10, 42, 0, 62);
    dhcpd_config.range_end = core::net::Ipv4Addr::new(10, 42, 0, 64);
    dhcpd_config.subnet_mask = core::net::Ipv4Addr::new(255, 255, 255, 0);
    dhcpd_config.lease_time = Duration::from_secs(300);

    static LEASES: StaticCell<Mutex<NoopRawMutex, Vec<DhcpdLease, DHCPD_MAX_LEASES>>> = StaticCell::new();
    let leases: &'static mut Mutex<NoopRawMutex, Vec<DhcpdLease, DHCPD_MAX_LEASES>> =
        LEASES.init(Mutex::new(Vec::new()));

    let dhcpd_runner = embassy_net::dhcpd::new(stack, rng, dhcpd_config, leases);

    spawner.spawn(unwrap!(dhcpcd_task(dhcpd_runner)));

    // End of init, all other stuff happens in already spawned tasks
}
