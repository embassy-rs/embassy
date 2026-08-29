#![no_std]
#![no_main]

use core::net::Ipv6Addr;

use defmt::{info, unwrap, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::StackStorage;
use embassy_net::udp::{UdpMetadata, UdpSocket};
use embassy_net::wire::{IpAddress, IpCidr, IpEndpoint, IpListenEndpoint, Ipv6Cidr};
use embassy_nrf::config::{Config, HfclkSource};
use embassy_nrf::rng::Rng;
use embassy_nrf::{bind_interrupts, embassy_net_802154_driver as net, peripherals, radio};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    RADIO => radio::InterruptHandler<peripherals::RADIO>;
    RNG => embassy_nrf::rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::task]
async fn ieee802154_task(runner: net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    // Necessary to run the radio nrf52840 v1.11 5.4.1
    config.hfclk_source = HfclkSource::ExternalXtal;
    let p = embassy_nrf::init(config);

    let mac_addr: [u8; 8] = [2, 3, 4, 5, 6, 7, 8, 9];
    static NRF802154_STATE: StaticCell<net::State<20, 20>> = StaticCell::new();
    let (device, runner) = net::new(mac_addr, p.RADIO, Irqs, NRF802154_STATE.init(net::State::new()))
        .await
        .unwrap();

    spawner.spawn(unwrap!(ieee802154_task(runner)));

    // Swap these when flashing a second board
    let peer = Ipv6Addr::new(0xfe80, 0, 0, 0, 0xd701, 0xda3f, 0x3955, 0x82a4);
    let local = Ipv6Addr::new(0xfe80, 0, 0, 0, 0xd701, 0xda3f, 0x3955, 0x82a5);

    // Generate random seed
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.blocking_fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    static STACK: StaticCell<StackStorage> = StaticCell::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), seed);

    // Add the network interface to the stack.
    static DEVICE: StaticCell<net::Device<'static>> = StaticCell::new();
    let iface = unwrap!(stack.add_iface(DEVICE.init(device)).ok());
    unwrap!(iface.add_ip_addr(IpCidr::Ipv6(Ipv6Cidr::new(local, 64))));

    spawner.spawn(unwrap!(net_task(runner)));

    let mut delay = Delay;
    loop {
        let mut socket = UdpSocket::new(stack);
        socket
            .bind(IpListenEndpoint {
                addr: Some(IpAddress::Ipv6(local)),
                port: 1234,
            })
            .unwrap();
        let rep = UdpMetadata {
            endpoint: IpEndpoint {
                addr: IpAddress::Ipv6(peer),
                port: 1234,
            },
            local_address: Some(IpAddress::Ipv6(local)),
            meta: Default::default(),
        };

        info!("Listening on {:?} UDP:1234...", local);

        let mut recv_buf = [0; 12];
        loop {
            delay.delay_ms(2000).await;
            if socket.may_recv() {
                let n = match socket.recv_from(&mut recv_buf).await {
                    Ok((0, _)) => panic!(),
                    Ok((n, _)) => n,
                    Err(e) => {
                        warn!("read error: {:?}", e);
                        break;
                    }
                };
                info!("Received {:02x}", &recv_buf[..n]);
            }

            info!("Sending");
            socket.send_to(b"Hello World", rep).await.unwrap();
        }
    }
}
