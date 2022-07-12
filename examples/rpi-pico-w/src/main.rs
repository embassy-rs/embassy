#![no_std]
#![no_main]
#![feature(type_alias_impl_trait, concat_bytes)]

use core::slice;

use defmt::{assert, assert_eq, panic, *};
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StackResources};
use embassy_rp::gpio::{Flex, Level, Output, Pin};
use embassy_rp::peripherals::{PIN_23, PIN_24, PIN_25, PIN_29};
use embassy_rp::Peripherals;
use embedded_io::asynch::{Read, Write};
use heapless::Vec;
use {defmt_rtt as _, panic_probe as _};

macro_rules! forever {
    ($val:expr) => {{
        type T = impl Sized;
        static FOREVER: Forever<T> = Forever::new();
        FOREVER.put_with(move || $val)
    }};
}

#[embassy::task]
async fn wifi_task(runner: cyw43::Runner<'static, PIN_23, PIN_25, PIN_29, PIN_24>) -> ! {
    runner.run().await
}

#[embassy::task]
async fn net_task(stack: &'static Stack<cyw43::NetDevice<'static>>) -> ! {
    stack.run().await
}

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let (pwr, cs, clk, dio) = (p.PIN_23, p.PIN_25, p.PIN_29, p.PIN_24);
    //let (pwr, cs, clk, dio) = (p.PIN_23, p.PIN_0, p.PIN_1, p.PIN_2);

    let state = forever!(cyw43::State::new());
    let (mut control, runner) = cyw43::new(
        state,
        Output::new(pwr, Level::Low),
        Output::new(cs, Level::High),
        Output::new(clk, Level::Low),
        Flex::new(dio),
    )
    .await;

    spawner.spawn(wifi_task(runner)).unwrap();

    let net_device = control.init().await;

    control.join_open("MikroTik-951589").await;
    //control.join_wpa2("MikroTik-951589", "asdfasdfasdfasdf").await;

    let config = embassy_net::ConfigStrategy::Dhcp;
    //let config = embassy_net::ConfigStrategy::Static(embassy_net::Config {
    //    address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(192, 168, 69, 1)),
    //});

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.

    // Init network stack
    let stack = &*forever!(Stack::new(
        net_device,
        config,
        forever!(StackResources::<1, 2, 8>::new()),
        seed
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    // And now we can use it!

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));

        info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
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

            info!("rxd {:02x}", &buf[..n]);

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
