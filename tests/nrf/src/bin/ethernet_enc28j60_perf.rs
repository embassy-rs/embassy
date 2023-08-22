#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
teleprobe_meta::target!(b"ak-gwe-r7");
teleprobe_meta::timeout!(120);

use defmt::{error, info, unwrap};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Stack, StackResources};
use embassy_net_enc28j60::Enc28j60;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::rng::Rng;
use embassy_nrf::spim::{self, Spim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::{with_timeout, Delay, Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    RNG => embassy_nrf::rng::InterruptHandler<peripherals::RNG>;
});

type MyDriver = Enc28j60<
    ExclusiveDevice<Spim<'static, peripherals::SPI3>, Output<'static, peripherals::P0_15>, Delay>,
    Output<'static, peripherals::P0_13>,
>;

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
    let stack = &*make_static!(Stack::new(
        device,
        config,
        make_static!(StackResources::<2>::new()),
        seed
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    info!("Waiting for DHCP up...");
    while stack.config_v4().is_none() {
        Timer::after(Duration::from_millis(100)).await;
    }
    info!("IP addressing up!");

    let down = test_download(stack).await;
    let up = test_upload(stack).await;
    let updown = test_upload_download(stack).await;

    assert!(down > TEST_EXPECTED_DOWNLOAD_KBPS);
    assert!(up > TEST_EXPECTED_UPLOAD_KBPS);
    assert!(updown > TEST_EXPECTED_UPLOAD_DOWNLOAD_KBPS);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

const TEST_DURATION: usize = 10;
const TEST_EXPECTED_DOWNLOAD_KBPS: usize = 200;
const TEST_EXPECTED_UPLOAD_KBPS: usize = 200;
const TEST_EXPECTED_UPLOAD_DOWNLOAD_KBPS: usize = 150;
const RX_BUFFER_SIZE: usize = 4096;
const TX_BUFFER_SIZE: usize = 4096;
const SERVER_ADDRESS: Ipv4Address = Ipv4Address::new(192, 168, 2, 2);
const DOWNLOAD_PORT: u16 = 4321;
const UPLOAD_PORT: u16 = 4322;
const UPLOAD_DOWNLOAD_PORT: u16 = 4323;

async fn test_download(stack: &'static Stack<MyDriver>) -> usize {
    info!("Testing download...");

    let mut rx_buffer = [0; RX_BUFFER_SIZE];
    let mut tx_buffer = [0; TX_BUFFER_SIZE];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(10)));

    info!("connecting to {:?}:{}...", SERVER_ADDRESS, DOWNLOAD_PORT);
    if let Err(e) = socket.connect((SERVER_ADDRESS, DOWNLOAD_PORT)).await {
        error!("connect error: {:?}", e);
        return 0;
    }
    info!("connected, testing...");

    let mut rx_buf = [0; 4096];
    let mut total: usize = 0;
    with_timeout(Duration::from_secs(TEST_DURATION as _), async {
        loop {
            match socket.read(&mut rx_buf).await {
                Ok(0) => {
                    error!("read EOF");
                    return 0;
                }
                Ok(n) => total += n,
                Err(e) => {
                    error!("read error: {:?}", e);
                    return 0;
                }
            }
        }
    })
    .await
    .ok();

    let kbps = (total + 512) / 1024 / TEST_DURATION;
    info!("download: {} kB/s", kbps);
    kbps
}

async fn test_upload(stack: &'static Stack<MyDriver>) -> usize {
    info!("Testing upload...");

    let mut rx_buffer = [0; RX_BUFFER_SIZE];
    let mut tx_buffer = [0; TX_BUFFER_SIZE];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(10)));

    info!("connecting to {:?}:{}...", SERVER_ADDRESS, UPLOAD_PORT);
    if let Err(e) = socket.connect((SERVER_ADDRESS, UPLOAD_PORT)).await {
        error!("connect error: {:?}", e);
        return 0;
    }
    info!("connected, testing...");

    let buf = [0; 4096];
    let mut total: usize = 0;
    with_timeout(Duration::from_secs(TEST_DURATION as _), async {
        loop {
            match socket.write(&buf).await {
                Ok(0) => {
                    error!("write zero?!??!?!");
                    return 0;
                }
                Ok(n) => total += n,
                Err(e) => {
                    error!("write error: {:?}", e);
                    return 0;
                }
            }
        }
    })
    .await
    .ok();

    let kbps = (total + 512) / 1024 / TEST_DURATION;
    info!("upload: {} kB/s", kbps);
    kbps
}

async fn test_upload_download(stack: &'static Stack<MyDriver>) -> usize {
    info!("Testing upload+download...");

    let mut rx_buffer = [0; RX_BUFFER_SIZE];
    let mut tx_buffer = [0; TX_BUFFER_SIZE];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(10)));

    info!("connecting to {:?}:{}...", SERVER_ADDRESS, UPLOAD_DOWNLOAD_PORT);
    if let Err(e) = socket.connect((SERVER_ADDRESS, UPLOAD_DOWNLOAD_PORT)).await {
        error!("connect error: {:?}", e);
        return 0;
    }
    info!("connected, testing...");

    let (mut reader, mut writer) = socket.split();

    let tx_buf = [0; 4096];
    let mut rx_buf = [0; 4096];
    let mut total: usize = 0;
    let tx_fut = async {
        loop {
            match writer.write(&tx_buf).await {
                Ok(0) => {
                    error!("write zero?!??!?!");
                    return 0;
                }
                Ok(_) => {}
                Err(e) => {
                    error!("write error: {:?}", e);
                    return 0;
                }
            }
        }
    };

    let rx_fut = async {
        loop {
            match reader.read(&mut rx_buf).await {
                Ok(0) => {
                    error!("read EOF");
                    return 0;
                }
                Ok(n) => total += n,
                Err(e) => {
                    error!("read error: {:?}", e);
                    return 0;
                }
            }
        }
    };

    with_timeout(Duration::from_secs(TEST_DURATION as _), join(tx_fut, rx_fut))
        .await
        .ok();

    let kbps = (total + 512) / 1024 / TEST_DURATION;
    info!("upload+download: {} kB/s", kbps);
    kbps
}
