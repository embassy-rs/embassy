// required-features: eth
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
use common::*;
use embassy_executor::Spawner;
use embassy_net::StackResources;
use embassy_stm32::eth::{Ethernet, GenericPhy, PacketQueue, Sma};
use embassy_stm32::peripherals::{ETH, ETH_SMA};
use embassy_stm32::rng::Rng;
use embassy_stm32::{bind_interrupts, eth, peripherals, rng};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

teleprobe_meta::timeout!(120);

#[cfg(not(any(feature = "stm32h563zi", feature = "stm32f767zi", feature = "stm32f207zg")))]
bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    HASH_RNG => rng::InterruptHandler<peripherals::RNG>;
});
#[cfg(any(feature = "stm32h563zi", feature = "stm32f767zi", feature = "stm32f207zg"))]
bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

type Device = Ethernet<'static, ETH, GenericPhy<Sma<'static, ETH_SMA>>>;

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = init();
    info!("Hello World!");

    // Generate random seed.
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    // Ensure different boards get different MAC
    // so running tests concurrently doesn't break (they're all in the same LAN)
    #[cfg(feature = "stm32f429zi")]
    let n = 1;
    #[cfg(feature = "stm32h755zi")]
    let n = 2;
    #[cfg(feature = "stm32h563zi")]
    let n = 3;
    #[cfg(feature = "stm32f767zi")]
    let n = 4;
    #[cfg(feature = "stm32f207zg")]
    let n = 5;
    #[cfg(feature = "stm32h753zi")]
    let n = 6;

    let mac_addr = [0x00, n, 0xDE, 0xAD, 0xBE, 0xEF];

    // F2 runs out of RAM
    #[cfg(feature = "stm32f207zg")]
    const PACKET_QUEUE_SIZE: usize = 2;
    #[cfg(not(feature = "stm32f207zg"))]
    const PACKET_QUEUE_SIZE: usize = 4;

    static PACKETS: StaticCell<PacketQueue<PACKET_QUEUE_SIZE, PACKET_QUEUE_SIZE>> = StaticCell::new();

    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<PACKET_QUEUE_SIZE, PACKET_QUEUE_SIZE>::new()),
        p.ETH,
        Irqs,
        p.PA1,
        p.PA7,
        p.PC4,
        p.PC5,
        p.PG13,
        #[cfg(not(feature = "stm32h563zi"))]
        p.PB13,
        #[cfg(feature = "stm32h563zi")]
        p.PB15,
        p.PG11,
        mac_addr,
        p.ETH_SMA,
        p.PA2,
        p.PC1,
    );

    let config = embassy_net::Config::dhcpv4(Default::default());
    //let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //    address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    //});

    // Init network stack
    static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    perf_client::run(
        stack,
        perf_client::Expected {
            down_kbps: 1000,
            up_kbps: 1000,
            updown_kbps: 1000,
        },
    )
    .await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}
