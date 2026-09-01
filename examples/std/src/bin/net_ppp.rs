//! Testing against pppd:
//!
//!     echo myuser $(hostname) mypass 192.168.7.10 >> /etc/ppp/pap-secrets
//!     socat -v -x PTY,link=pty1,rawer PTY,link=pty2,rawer
//!     sudo pppd $PWD/pty1 115200 192.168.7.1: ms-dns 8.8.4.4 ms-dns 8.8.8.8 nodetach debug local persist silent noproxyarp
//!     RUST_LOG=trace cargo run --bin net_ppp -- --device pty2
//!     ping 192.168.7.10
//!     nc 192.168.7.10 1234

#![allow(async_fn_in_trait)]

#[path = "../serial_port.rs"]
mod serial_port;

use async_io::Async;
use clap::Parser;
use embassy_executor::{Executor, Spawner};
use embassy_net::iface::Iface;
use embassy_net::tcp::{TcpListener, TcpSocket};
use embassy_net::wire::IpCidr;
use embassy_net::{Stack, StackStorage};
use embassy_net_ppp::Runner;
use embedded_io_async::Write;
use futures::io::BufReader;
use heapless::Vec;
use log::*;
use nix::sys::termios;
use rand_core::{OsRng, TryRngCore};
use static_cell::StaticCell;

use crate::serial_port::SerialPort;

#[derive(Parser)]
#[clap(version = "1.0")]
struct Opts {
    /// Serial port device name
    #[clap(short, long)]
    device: String,
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn ppp_task(iface: Iface<'static>, mut runner: Runner<'static>, port: SerialPort) -> ! {
    let port = Async::new(port).unwrap();
    let port = BufReader::new(port);
    let port = embedded_io_adapters::futures_03::FromFutures::new(port);

    let config = embassy_net_ppp::Config {
        username: b"myuser",
        password: b"mypass",
    };

    let r = runner
        .run(port, config, |ipv4| {
            let Some(addr) = ipv4.address else {
                warn!("PPP did not provide an IP address.");
                return;
            };
            let mut dns_servers = Vec::<_, 3>::new();
            for s in ipv4.dns_servers.iter().flatten() {
                let _ = dns_servers.push(embassy_net::wire::IpAddress::Ipv4(*s));
            }
            iface.set_ip_addrs([IpCidr::new(addr.into(), 0)]).unwrap();
            iface.stack().set_dns_servers(&dns_servers);
        })
        .await;
    match r {
        Err(e) => panic!("{:?}", e),
    }
}

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    let opts: Opts = Opts::parse();

    // Open serial port
    let baudrate = termios::BaudRate::B115200;
    let port = SerialPort::new(opts.device.as_str(), baudrate).unwrap();

    // Init network device
    static STATE: StaticCell<embassy_net_ppp::State<4, 4>> = StaticCell::new();
    let state = STATE.init(embassy_net_ppp::State::<4, 4>::new());
    let (device, runner) = embassy_net_ppp::new(state);

    // Generate random seed
    let mut seed = [0; 8];
    OsRng.try_fill_bytes(&mut seed).unwrap();
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    static STACK: StaticCell<StackStorage> = StaticCell::new();
    let (stack, net_runner) = Stack::new(STACK.init(StackStorage::new()), seed);

    // Add the PPP interface to the stack. It gets its addresses from PPP itself,
    // in `ppp_task`.
    static DEVICE: StaticCell<embassy_net_ppp::Device<'static>> = StaticCell::new();
    let iface = stack.add_iface(DEVICE.init(device)).unwrap();

    // Launch network task
    spawner.spawn(net_task(net_runner).unwrap());
    spawner.spawn(ppp_task(iface, runner, port).unwrap());

    // Then we can use it!
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    let mut listener = TcpListener::new(stack).unwrap();
    listener.listen(1234).unwrap();

    loop {
        info!("Listening on TCP:1234...");
        let token = match listener.accept().await {
            Ok(token) => token,
            Err(e) => {
                warn!("accept error: {:?}", e);
                continue;
            }
        };
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer).unwrap();
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
        if let Err(e) = socket.accept(token).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("read error: {:?}", e);
                    break;
                }
            };

            info!("rxd {:02x?}", &buf[..n]);

            match socket.write_all(&buf[..n]).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .filter_module("polling", log::LevelFilter::Info)
        .filter_module("async_io", log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(main_task(spawner).unwrap());
    });
}
