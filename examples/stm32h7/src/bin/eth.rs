#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Duration, Timer};
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Stack, StackResources};
use embassy_stm32::eth::generic_smi::GenericSMI;
use embassy_stm32::eth::{Ethernet, State};
use embassy_stm32::peripherals::ETH;
use embassy_stm32::rng::Rng;
use embassy_stm32::time::mhz;
use embassy_stm32::{interrupt, Config};
use embassy_util::Forever;
use embedded_io::asynch::Write;
use rand_core::RngCore;
use {defmt_rtt as _, panic_probe as _};

macro_rules! forever {
    ($val:expr) => {{
        type T = impl Sized;
        static FOREVER: Forever<T> = Forever::new();
        FOREVER.put_with(move || $val)
    }};
}

type Device = Ethernet<'static, ETH, GenericSMI, 4, 4>;

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Device>) -> ! {
    stack.run().await
}

pub fn config() -> Config {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(mhz(400));
    config.rcc.hclk = Some(mhz(200));
    config.rcc.pll1.q_ck = Some(mhz(100));
    config
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    // Generate random seed.
    let mut rng = Rng::new(p.RNG);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    let eth_int = interrupt::take!(ETH);
    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    let device = unsafe {
        Ethernet::new(
            forever!(State::new()),
            p.ETH,
            eth_int,
            p.PA1,
            p.PA2,
            p.PC1,
            p.PA7,
            p.PC4,
            p.PC5,
            p.PG13,
            p.PB13,
            p.PG11,
            GenericSMI,
            mac_addr,
            0,
        )
    };

    let config = embassy_net::ConfigStrategy::Dhcp;
    //let config = embassy_net::ConfigStrategy::Static(embassy_net::Config {
    //    address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    //});

    // Init network stack
    let stack = &*forever!(Stack::new(
        device,
        config,
        forever!(StackResources::<1, 2, 8>::new()),
        seed
    ));

    // Launch network task
    unwrap!(spawner.spawn(net_task(&stack)));

    info!("Network task initialized");

    // Then we can use it!
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    loop {
        let mut socket = TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);

        socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));

        let remote_endpoint = (Ipv4Address::new(10, 42, 0, 1), 8000);
        info!("connecting...");
        let r = socket.connect(remote_endpoint).await;
        if let Err(e) = r {
            info!("connect error: {:?}", e);
            continue;
        }
        info!("connected!");
        loop {
            let r = socket.write_all(b"Hello\n").await;
            if let Err(e) = r {
                info!("write error: {:?}", e);
                return;
            }
            Timer::after(Duration::from_secs(1)).await;
        }
    }
}
