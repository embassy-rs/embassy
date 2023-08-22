#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
teleprobe_meta::target!(b"rpi-pico");

use cyw43_pio::PioSpi;
use defmt::{assert, panic, *};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Ipv4Address, Stack, StackResources};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::{bind_interrupts, rom_data};
use embassy_time::{with_timeout, Duration, Timer};
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

teleprobe_meta::timeout!(120);

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static, PIN_23>, PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");
    let p = embassy_rp::init(Default::default());

    // needed for reading the firmware from flash via XIP.
    unsafe {
        rom_data::flash_exit_xip();
        rom_data::flash_enter_cmd_xip();
    }

    // cyw43 firmware needs to be flashed manually:
    //     probe-rs download 43439A0.bin     --format bin --chip RP2040 --base-address 0x101b0000
    //     probe-rs download 43439A0_clm.bin --format bin --chip RP2040 --base-address 0x101f8000
    let fw = unsafe { core::slice::from_raw_parts(0x101b0000 as *const u8, 230321) };
    let clm = unsafe { core::slice::from_raw_parts(0x101f8000 as *const u8, 4752) };

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(&mut pio.common, pio.sm0, pio.irq0, cs, p.PIN_24, p.PIN_29, p.DMA_CH0);

    let state = make_static!(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(wifi_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.

    // Init network stack
    let stack = &*make_static!(Stack::new(
        net_device,
        Config::dhcpv4(Default::default()),
        make_static!(StackResources::<2>::new()),
        seed
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    loop {
        match control.join_wpa2(WIFI_NETWORK, WIFI_PASSWORD).await {
            Ok(_) => break,
            Err(err) => {
                panic!("join failed with status={}", err.status);
            }
        }
    }

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
const TEST_EXPECTED_DOWNLOAD_KBPS: usize = 300;
const TEST_EXPECTED_UPLOAD_KBPS: usize = 300;
const TEST_EXPECTED_UPLOAD_DOWNLOAD_KBPS: usize = 300;
const RX_BUFFER_SIZE: usize = 4096;
const TX_BUFFER_SIZE: usize = 4096;
const SERVER_ADDRESS: Ipv4Address = Ipv4Address::new(192, 168, 2, 2);
const DOWNLOAD_PORT: u16 = 4321;
const UPLOAD_PORT: u16 = 4322;
const UPLOAD_DOWNLOAD_PORT: u16 = 4323;

async fn test_download(stack: &'static Stack<cyw43::NetDriver<'static>>) -> usize {
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

async fn test_upload(stack: &'static Stack<cyw43::NetDriver<'static>>) -> usize {
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

async fn test_upload_download(stack: &'static Stack<cyw43::NetDriver<'static>>) -> usize {
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
