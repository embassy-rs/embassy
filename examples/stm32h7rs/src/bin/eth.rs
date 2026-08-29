#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::StackStorage;
use embassy_net::udp::UdpSocket;
use embassy_net::wire::{IpCidr, Ipv4Address};
use embassy_stm32::eth::{Ethernet, GenericPhy, PacketQueue, Sma};
use embassy_stm32::peripherals::{ETH, ETH_SMA};
use embassy_stm32::rng::Rng;
use embassy_stm32::{Config, bind_interrupts, eth, peripherals, rng};
use embassy_time::Timer;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler<ETH>;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

type Device = Ethernet<'static, ETH, GenericPhy<Sma<'static, ETH_SMA>>>;

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Default::default()); // needed for RNG
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul50,
            divp: Some(PllDiv::Div2),
            divq: None,
            divr: None,
            divs: None,
            divt: None,
        });
        config.rcc.sys = Sysclk::Pll1P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::Div2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb5_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::High;
    }
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    // Generate random seed.
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    static PACKETS: StaticCell<PacketQueue<4, 4>> = StaticCell::new();
    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<4, 4>::new()),
        p.ETH,
        Irqs,
        p.PB6,
        p.PA7,
        p.PG4,
        p.PG5,
        p.PG13,
        p.PG12,
        p.PG11,
        mac_addr,
        p.ETH_SMA,
        p.PA2,
        p.PG6,
    );

    // Have to use UDP w/ static config to fit in internal flash
    // Init network stack
    static STACK: StaticCell<StackStorage> = StaticCell::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), seed);

    // Add the network interface to the stack.
    static DEVICE: StaticCell<Device> = StaticCell::new();
    let iface = unwrap!(stack.add_iface(DEVICE.init(device)).ok());
    unwrap!(iface.add_ip_addr(IpCidr::new(Ipv4Address::new(10, 42, 0, 61).into(), 24)));
    unwrap!(
        stack
            .routes()
            .add_default_ipv4_route(Ipv4Address::new(10, 42, 0, 1), iface.handle())
    );

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    // Ensure DHCP configuration is up before trying connect
    //iface.wait_config_up().await;

    info!("Network task initialized");

    // Then we can use it!

    let remote_endpoint = (Ipv4Address::new(10, 42, 0, 1), 8000);
    let socket = UdpSocket::new(stack);
    loop {
        // You need to start a server on the host machine, for example: `nc -lu 8000`
        socket
            .send_to(b"Hello, world", remote_endpoint)
            .await
            .expect("Buffer sent");
        Timer::after_secs(1).await;
    }
}
