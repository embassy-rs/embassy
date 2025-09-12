//! This example implements a TCP client that attempts to connect to a host on port 1234 and send it some data once per second.
//!
//! Example written for the [`WIZnet W55RP20-EVB-Pico`](https://docs.wiznet.io/Product/ioNIC/W55RP20/w55rp20-evb-pico) board.
//! Note: the W55RP20 is a single package that contains both a RP2040 and the Wiznet W5500 ethernet
//! controller

#![no_std]
#![no_main]

use core::str::FromStr;

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_net::{Stack, StackResources};
use embassy_net_wiznet::chip::W5500;
use embassy_net_wiznet::*;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio_programs::spi::Spi;
use embassy_rp::spi::{Async, Config as SpiConfig};
use embassy_rp::{bind_interrupts, pio};
use embassy_time::{Delay, Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io_async::Write;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        W5500,
        ExclusiveDevice<Spi<'static, PIO0, 0, Async>, Output<'static>, Delay>,
        Input<'static>,
        Output<'static>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut rng = RoscRng;
    let mut led = Output::new(p.PIN_19, Level::Low);

    // The W55RP20 uses a PIO unit for SPI communication, once the SPI bus has been formed using a
    // PIO statemachine everything else is generally unchanged from the other examples that use the W5500
    let mosi = p.PIN_23;
    let miso = p.PIN_22;
    let clk = p.PIN_21;

    let pio::Pio { mut common, sm0, .. } = pio::Pio::new(p.PIO0, Irqs);

    // Construct an SPI driver backed by a PIO state machine
    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 50_000_000;
    let spi = Spi::new(&mut common, sm0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, spi_cfg);

    // Further control pins
    let cs = Output::new(p.PIN_20, Level::High);
    let w5500_int = Input::new(p.PIN_24, Pull::Up);
    let w5500_reset = Output::new(p.PIN_25, Level::High);

    let mac_addr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00];
    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());
    let (device, runner) = embassy_net_wiznet::new(
        mac_addr,
        state,
        ExclusiveDevice::new(spi, cs, Delay),
        w5500_int,
        w5500_reset,
    )
    .await
    .unwrap();
    spawner.spawn(unwrap!(ethernet_task(runner)));

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        device,
        embassy_net::Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        seed,
    );

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    info!("Waiting for DHCP...");
    let cfg = wait_for_config(stack).await;
    let local_addr = cfg.address.address();
    info!("IP address: {:?}", local_addr);

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    loop {
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        led.set_low();
        info!("Connecting...");
        let host_addr = embassy_net::Ipv4Address::from_str("192.168.1.110").unwrap();
        if let Err(e) = socket.connect((host_addr, 1234)).await {
            warn!("connect error: {:?}", e);
            continue;
        }
        info!("Connected to {:?}", socket.remote_endpoint());
        led.set_high();

        let msg = b"Hello world!\n";
        loop {
            if let Err(e) = socket.write_all(msg).await {
                warn!("write error: {:?}", e);
                break;
            }
            info!("txd: {}", core::str::from_utf8(msg).unwrap());
            Timer::after_secs(1).await;
        }
    }
}

async fn wait_for_config(stack: Stack<'static>) -> embassy_net::StaticConfigV4 {
    loop {
        if let Some(config) = stack.config_v4() {
            return config.clone();
        }
        yield_now().await;
    }
}
