#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Stack, StackResources};
use embassy_net_enc28j60::Enc28j60;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::rng::Rng;
use embassy_nrf::spim::Spim;
use embassy_nrf::{bind_interrupts, peripherals, spim};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io_async::Write;
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    RNG => embassy_nrf::rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::task]
async fn net_task(
    stack: &'static Stack<
        Enc28j60<
            ExclusiveDevice<Spim<'static, peripherals::SPI3>, Output<'static, peripherals::P0_15>, Delay>,
            Output<'static, peripherals::P0_13>,
        >,
    >,
) -> ! {
    stack.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("running!");

    let eth_sck = p.P0_20;
    let eth_mosi = p.P0_22;
    let eth_miso = p.P0_24;
    let eth_cs = p.P0_15;
    let eth_rst = p.P0_13;
    let _eth_irq = p.P0_12;

    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M16;
    let spi = spim::Spim::new(p.SPI3, Irqs, eth_sck, eth_miso, eth_mosi, config);
    let cs = Output::new(eth_cs, Level::High, OutputDrive::Standard);
    let spi = ExclusiveDevice::new(spi, cs, Delay);

    let rst = Output::new(eth_rst, Level::High, OutputDrive::Standard);
    let mac_addr = [2, 3, 4, 5, 6, 7];
    let device = Enc28j60::new(spi, Some(rst), mac_addr);

    let config = embassy_net::Config::dhcpv4(Default::default());
    // let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //    address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    // });

    // Generate random seed
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.blocking_fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    let stack = &*make_static!(Stack::new(
        device,
        config,
        make_static!(StackResources::<2>::new()),
        seed
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    // And now we can use it!

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

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
