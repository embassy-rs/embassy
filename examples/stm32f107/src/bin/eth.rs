#![no_std]
#![no_main]

use defmt::{info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_net::tcp::{self, TcpSocket};
use embassy_net::{IpListenEndpoint, StackResources};
use embassy_stm32::eth::{Ethernet, GenericPhy, PacketQueue, Sma};
use embassy_stm32::peripherals::{ETH, ETH_SMA};
use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, Pll2Mul, Pll2Or3, PllMul, PllPreDiv, PllSource, Sysclk,
};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, eth};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
});

type Device = Ethernet<'static, ETH, GenericPhy<Sma<'static, ETH_SMA>>>;

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let mut config = Config::default();
    config.rcc.hse = Some(Hse {
        freq: Hertz::mhz(25),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll = Some(Pll {
        src: PllSource::PLL2,
        prediv: PllPreDiv::DIV5,
        mul: PllMul::MUL9,
    });
    config.rcc.prediv2 = PllPreDiv::DIV5;
    config.rcc.pll2 = Some(Pll2Or3 { mul: Pll2Mul::MUL8 });
    config.rcc.pll3 = Some(Pll2Or3 { mul: Pll2Mul::MUL10 });
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.sys = Sysclk::PLL1_P;

    let p = embassy_stm32::init(config);
    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];
    static PACKETS: StaticCell<PacketQueue<4, 4>> = StaticCell::new();
    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<4, 4>::new()),
        p.ETH,
        Irqs,
        p.PA1,
        p.PA7,
        p.PC4,
        p.PC5,
        p.PB12,
        p.PB13,
        p.PB11,
        mac_addr,
        p.ETH_SMA,
        p.PA2,
        p.PC1,
    );

    let config = embassy_net::Config::dhcpv4(Default::default());
    // Use the following instead to set a static IPv4 address.
    // let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
    //     address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 62), 24),
    //     dns_servers: Vec::new(),
    //     gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    // });

    static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(device, config, RESOURCES.init(StackResources::new()), 3249);
    spawner.spawn(unwrap!(net_task(runner)));
    stack.wait_config_up().await;

    info!("Network task initialized");

    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        unwrap!(socket.accept(IpListenEndpoint { addr: None, port: 80 }).await);

        let mut read_buffer = [0; 1024];
        loop {
            match socket.read(&mut read_buffer).await {
                Ok(0) => break,
                Ok(bytes) => {
                    info!("Received {} bytes: {:a}", bytes, read_buffer[..bytes]);
                    unwrap!(socket.write(&read_buffer[..bytes]).await);
                }
                Err(tcp::Error::ConnectionReset) => {
                    warn!("Error: connection reset");
                    break;
                }
            }
        }
    }
}
