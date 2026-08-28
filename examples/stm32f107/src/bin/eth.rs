#![no_std]
#![no_main]

use defmt::{info, unwrap, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::StackStorage;
use embassy_net::tcp::{self, TcpListener};
use embassy_net::wire::IpListenEndpoint;
use embassy_stm32::eth::{Ethernet, GenericPhy, PacketQueue, Sma};
use embassy_stm32::peripherals::{ETH, ETH_SMA};
use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, Pll2Mul, Pll2Or3, PllMul, PllPreDiv, PllSource, Sysclk,
};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, eth};
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler<ETH>;
});

type Device = Ethernet<'static, ETH, GenericPhy<Sma<'static, ETH_SMA>>>;

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
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
        prediv: PllPreDiv::Div5,
        mul: PllMul::Mul9,
    });
    config.rcc.prediv2 = PllPreDiv::Div5;
    config.rcc.pll2 = Some(Pll2Or3 { mul: Pll2Mul::Mul8 });
    config.rcc.pll3 = Some(Pll2Or3 { mul: Pll2Mul::Mul10 });
    config.rcc.ahb_pre = AHBPrescaler::Div1;
    config.rcc.apb1_pre = APBPrescaler::Div2;
    config.rcc.apb2_pre = APBPrescaler::Div1;
    config.rcc.sys = Sysclk::Pll1P;

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

    static STACK: StaticCell<StackStorage> = StaticCell::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), 3249);

    // Add the network interface to the stack.
    static DEVICE: StaticCell<Device> = StaticCell::new();
    let iface = unwrap!(stack.add_iface(DEVICE.init(device)).ok());
    iface.set_dhcpv4(Some(Default::default()));
    spawner.spawn(unwrap!(net_task(runner)));
    iface.wait_config_up().await;

    info!("Network task initialized");

    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    let mut listener = TcpListener::new(stack);
    unwrap!(listener.listen(IpListenEndpoint { addr: None, port: 80 }));

    loop {
        let mut socket = unwrap!(listener.accept(&mut rx_buffer, &mut tx_buffer).await);

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
