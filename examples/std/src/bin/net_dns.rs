use embassy_executor::{Executor, Spawner};
use embassy_net::dns::DnsQueryType;
use embassy_net::{StackStorage, StaticPool};
use embassy_net_tuntap::TunTapDevice;
use getrandom::SysRng;
use log::*;
use rand_core::TryRng;
use static_cell::StaticCell;

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    // Init network device
    let device = TunTapDevice::new("tap99").unwrap();

    // Generate random seed
    let mut seed = [0; 8];
    SysRng.try_fill_bytes(&mut seed).unwrap();
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    static STACK: StaticCell<StackStorage> = StaticCell::new();
    static POOL: StaticPool = StaticPool::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), &POOL, seed);

    // Add the TAP interface to the stack.
    static DEVICE: StaticCell<TunTapDevice> = StaticCell::new();
    let iface = stack.add_iface_borrowed(DEVICE.init(device)).unwrap();

    // Get an IP address over DHCP
    iface.set_dhcpv4(Some(Default::default())).unwrap();

    // Launch network task
    spawner.spawn(net_task(runner).unwrap());

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
        spawner.spawn(main_task(spawner).unwrap());
    });
}
