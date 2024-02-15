#![no_std]
#![no_main]
teleprobe_meta::target!(b"ak-gwe-r7");
teleprobe_meta::timeout!(120);

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_net_enc28j60::Enc28j60;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::rng::Rng;
use embassy_nrf::spim::{self, Spim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    RNG => embassy_nrf::rng::InterruptHandler<peripherals::RNG>;
});

type MyDriver = Enc28j60<ExclusiveDevice<Spim<'static, peripherals::SPI3>, Output<'static>, Delay>, Output<'static>>;

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<MyDriver>) -> ! {
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
    static STACK: StaticCell<Stack<MyDriver>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        device,
        config,
        RESOURCES.init(StackResources::<2>::new()),
        seed,
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    perf_client::run(
        stack,
        perf_client::Expected {
            down_kbps: 200,
            up_kbps: 200,
            updown_kbps: 150,
        },
    )
    .await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}
