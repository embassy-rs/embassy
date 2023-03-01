#![feature(type_alias_impl_trait)]

use std::default::Default;

use clap::Parser;
use embassy_executor::{Executor, Spawner};
use embassy_net::dns::DnsQueryType;
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, Stack, StackResources};
use heapless::Vec;
use log::*;
use rand_core::{OsRng, RngCore};
use static_cell::StaticCell;

#[path = "../tuntap.rs"]
mod tuntap;

use crate::tuntap::TunTapDevice;

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        STATIC_CELL.init_with(move || $val)
    }};
}

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
        Config::Static(embassy_net::StaticConfig {
            address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 1), 24),
            dns_servers: Vec::from_slice(&[Ipv4Address::new(8, 8, 4, 4).into(), Ipv4Address::new(8, 8, 8, 8).into()])
                .unwrap(),
            gateway: Some(Ipv4Address::new(192, 168, 69, 100)),
        })
    } else {
        Config::Dhcp(Default::default())
    };

    // Generate random seed
    let mut seed = [0; 8];
    OsRng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    let stack: &Stack<_> = &*singleton!(Stack::new(device, config, singleton!(StackResources::<3>::new()), seed));

    // Launch network task
    spawner.spawn(net_task(stack)).unwrap();

    let host = "example.com";
    info!("querying host {:?}...", host);
    match stack.dns_query(host, DnsQueryType::A).await {
        Ok(r) => {
            info!("query response: {:?}", r);
        }
        Err(e) => {
            warn!("query error: {:?}", e);
        }
    };
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
