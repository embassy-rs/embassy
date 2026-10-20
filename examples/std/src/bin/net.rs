use core::fmt::Write as _;

use embassy_executor::{Executor, Spawner};
use embassy_net::tcp::TcpSocket;
use embassy_net::wire::Ipv4Addr;
use embassy_net::{StackStorage, StaticPool};
use embassy_net_tuntap::TunTapDevice;
use embassy_time::Duration;
use embedded_io_async::Write;
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

    // Then we can use it!
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer).unwrap();

    socket.set_timeout(Some(Duration::from_secs(10)));

    let remote_addr = (Ipv4Addr::new(192, 168, 69, 100), 8000);
    info!("connecting to {:?}...", remote_addr);
    let r = socket.connect(remote_addr).await;
    if let Err(e) = r {
        warn!("connect error: {:?}", e);
        return;
    }
    info!("connected!");
    for i in 0.. {
        let mut buf = heapless::String::<100>::new();
        write!(buf, "Hello! ({})\r\n", i).unwrap();
        let r = socket.write_all(buf.as_bytes()).await;
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
        spawner.spawn(main_task(spawner).unwrap());
    });
}
