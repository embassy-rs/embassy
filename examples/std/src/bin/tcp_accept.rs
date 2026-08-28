use clap::Parser;
use embassy_executor::{Executor, Spawner};
use embassy_net::StackStorage;
use embassy_net::tcp::TcpListener;
use embassy_net::wire::{IpCidr, Ipv4Address};
use embassy_net_tuntap::TunTapDevice;
use embassy_time::{Duration, Timer};
use embedded_io_async::Write as _;
use log::*;
use rand_core::{OsRng, TryRngCore};
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
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    let opts: Opts = Opts::parse();

    // Init network device
    let device = TunTapDevice::new(&opts.tap).unwrap();

    // Generate random seed
    let mut seed = [0; 8];
    OsRng.try_fill_bytes(&mut seed).unwrap();
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    static STACK: StaticCell<StackStorage> = StaticCell::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), seed);

    // Add the TAP interface to the stack.
    static DEVICE: StaticCell<TunTapDevice> = StaticCell::new();
    let iface = stack.add_iface(DEVICE.init(device)).unwrap();

    // Choose between dhcp or static ip
    if opts.static_ip {
        iface
            .add_ip_addr(IpCidr::new(Ipv4Address::new(192, 168, 69, 2).into(), 24))
            .unwrap();
        stack
            .routes()
            .add_default_ipv4_route(Ipv4Address::new(192, 168, 69, 1), iface.handle())
            .unwrap();
    } else {
        iface.set_dhcpv4(Some(Default::default()));
    }

    // Launch network task
    spawner.spawn(net_task(runner).unwrap());

    // Then we can use it!
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut listener = TcpListener::new(stack);
    listener.listen(9999).unwrap();

    loop {
        info!("Listening on TCP:9999...");
        let mut socket = match listener.accept(&mut rx_buffer, &mut tx_buffer).await {
            Ok(socket) => socket,
            Err(_) => {
                warn!("accept error");
                continue;
            }
        };
        socket.set_timeout(Some(Duration::from_secs(10)));

        info!("Accepted a connection");

        // Write some quick output
        for i in 1..=5 {
            let s = format!("{}!  ", i);
            let r = socket.write_all(s.as_bytes()).await;
            if let Err(e) = r {
                warn!("write error: {:?}", e);
                return;
            }

            Timer::after_millis(500).await;
        }
        info!("Closing the connection");
        socket.abort();
        info!("Flushing the RST out...");
        _ = socket.flush().await;
        info!("Finished with the socket");
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
