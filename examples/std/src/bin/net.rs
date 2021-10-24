#![feature(type_alias_impl_trait)]

use clap::Parser;
use embassy::executor::{Executor, Spawner};
use embassy::io::AsyncWriteExt;
use embassy::util::Forever;
use embassy_net::*;
use heapless::Vec;
use log::*;

#[path = "../tuntap.rs"]
mod tuntap;

use crate::tuntap::TunTapDevice;

static DEVICE: Forever<TunTapDevice> = Forever::new();
static CONFIG_STATIC: Forever<StaticConfigurator> = Forever::new();
static CONFIG_DYNAMIC: Forever<DhcpConfigurator> = Forever::new();
static NET_RESOURCES: Forever<StackResources<1, 2, 8>> = Forever::new();

#[derive(Parser)]
#[clap(version = "1.0")]
struct Opts {
    /// TAP device name
    #[clap(long, default_value = "tap0")]
    tap: String,
    /// use a static IP instead of DHCP
    #[clap(long)]
    static_ip: bool,
}

#[embassy::task]
async fn net_task() {
    embassy_net::run().await
}

#[embassy::task]
async fn main_task(spawner: Spawner) {
    let opts: Opts = Opts::parse();

    // Init network device
    let device = TunTapDevice::new(&opts.tap).unwrap();

    // Choose between dhcp or static ip
    let config: &'static mut dyn Configurator = if opts.static_ip {
        CONFIG_STATIC.put(StaticConfigurator::new(Config {
            address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24),
            dns_servers: Vec::new(),
            gateway: Some(Ipv4Address::new(192, 168, 69, 1)),
        }))
    } else {
        CONFIG_DYNAMIC.put(DhcpConfigurator::new())
    };

    let net_resources = StackResources::new();

    // Init network stack
    embassy_net::init(DEVICE.put(device), config, NET_RESOURCES.put(net_resources));

    // Launch network task
    spawner.spawn(net_task()).unwrap();

    // Then we can use it!
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut socket = TcpSocket::new(&mut rx_buffer, &mut tx_buffer);

    socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));

    let remote_endpoint = (Ipv4Address::new(192, 168, 69, 74), 8000);
    info!("connecting to {:?}...", remote_endpoint);
    let r = socket.connect(remote_endpoint).await;
    if let Err(e) = r {
        warn!("connect error: {:?}", e);
        return;
    }
    info!("connected!");
    loop {
        let r = socket.write_all(b"Hello!\n").await;
        if let Err(e) = r {
            warn!("write error: {:?}", e);
            return;
        }
    }
}

#[no_mangle]
fn _embassy_rand(buf: &mut [u8]) {
    use rand_core::{OsRng, RngCore};
    OsRng.fill_bytes(buf);
}

static EXECUTOR: Forever<Executor> = Forever::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("async_io", log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(main_task(spawner)).unwrap();
    });
}
