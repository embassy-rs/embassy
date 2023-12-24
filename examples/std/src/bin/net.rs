use std::default::Default;

use clap::Parser;
use embassy_executor::{Executor, Spawner};
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, Stack, StackResources};
use embassy_net_tuntap::TunTapDevice;
use embassy_time::Duration;
use embedded_io_async::Write;
use heapless::Vec;
use log::*;
use rand_core::{OsRng, RngCore};
use static_cell::StaticCell;

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

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<TunTapDevice>) -> ! {
    stack.run().await
}

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    let opts: Opts = Opts::parse();

    // Init network device
    let device = TunTapDevice::new(&opts.tap).unwrap();

    // Choose between dhcp or static ip
    let config = if opts.static_ip {
        Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24),
            dns_servers: Vec::new(),
            gateway: Some(Ipv4Address::new(192, 168, 69, 1)),
        })
    } else {
        Config::dhcpv4(Default::default())
    };

    // Generate random seed
    let mut seed = [0; 8];
    OsRng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    static STACK: StaticCell<Stack<TunTapDevice>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        device,
        config,
        RESOURCES.init(StackResources::<3>::new()),
        seed,
    ));

    // Launch network task
    spawner.spawn(net_task(stack)).unwrap();

    // Then we can use it!
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

    socket.set_timeout(Some(Duration::from_secs(10)));

    let remote_endpoint = (Ipv4Address::new(192, 168, 69, 100), 8000);
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

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("async_io", log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(main_task(spawner)).unwrap();
    });
}
