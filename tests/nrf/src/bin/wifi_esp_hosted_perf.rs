#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
teleprobe_meta::target!(b"nrf52840-dk");
teleprobe_meta::timeout!(120);

use defmt::{error, info, unwrap};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Ipv4Address, Stack, StackResources};
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::rng::Rng;
use embassy_nrf::spim::{self, Spim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::{with_timeout, Delay, Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use static_cell::make_static;
use {defmt_rtt as _, embassy_net_esp_hosted as hosted, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    RNG => embassy_nrf::rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::task]
async fn wifi_task(
    runner: hosted::Runner<
        'static,
        ExclusiveDevice<Spim<'static, peripherals::SPI3>, Output<'static, peripherals::P0_31>, Delay>,
        Input<'static, AnyPin>,
        Output<'static, peripherals::P1_05>,
    >,
) -> ! {
    runner.run().await
}

type MyDriver = hosted::NetDriver<'static>;

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<MyDriver>) -> ! {
    stack.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_nrf::init(Default::default());

    let miso = p.P0_28;
    let sck = p.P0_29;
    let mosi = p.P0_30;
    let cs = Output::new(p.P0_31, Level::High, OutputDrive::HighDrive);
    let handshake = Input::new(p.P1_01.degrade(), Pull::Up);
    let ready = Input::new(p.P1_04.degrade(), Pull::None);
    let reset = Output::new(p.P1_05, Level::Low, OutputDrive::Standard);

    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M32;
    config.mode = spim::MODE_2; // !!!
    let spi = spim::Spim::new(p.SPI3, Irqs, sck, miso, mosi, config);
    let spi = ExclusiveDevice::new(spi, cs, Delay);

    let (device, mut control, runner) = embassy_net_esp_hosted::new(
        make_static!(embassy_net_esp_hosted::State::new()),
        spi,
        handshake,
        ready,
        reset,
    )
    .await;

    unwrap!(spawner.spawn(wifi_task(runner)));

    unwrap!(control.init().await);
    unwrap!(control.connect(WIFI_NETWORK, WIFI_PASSWORD).await);

    // Generate random seed
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.blocking_fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    let stack = &*make_static!(Stack::new(
        device,
        Config::dhcpv4(Default::default()),
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

// Test-only wifi network, no internet access!
const WIFI_NETWORK: &str = "EmbassyTest";
const WIFI_PASSWORD: &str = "V8YxhKt5CdIAJFud";

const TEST_DURATION: usize = 10;
const TEST_EXPECTED_DOWNLOAD_KBPS: usize = 50;
const TEST_EXPECTED_UPLOAD_KBPS: usize = 50;
const TEST_EXPECTED_UPLOAD_DOWNLOAD_KBPS: usize = 50;
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
