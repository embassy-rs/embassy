//! This example shows how you can allow multiple simultaneous TCP connections, by having multiple sockets listening on the same port.
//!
//! Example written for the [`WIZnet W5500-EVB-Pico2`](https://docs.wiznet.io/Product/iEthernet/W5500/w5500-evb-pico2) board.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpListener;
use embassy_net::{Stack, StackStorage};
use embassy_net_wiznet::chip::W5500;
use embassy_net_wiznet::*;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1};
use embassy_rp::spi::{Async, Config as SpiConfig, Spi};
use embassy_rp::{bind_interrupts, dma};
use embassy_time::{Delay, Duration};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io_async::Write;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        W5500,
        ExclusiveDevice<Spi<'static, Async>, Output<'static>, Delay>,
        Input<'static>,
        Output<'static>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut rng = RoscRng;

    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 50_000_000;
    let (miso, mosi, clk) = (p.PIN_16, p.PIN_19, p.PIN_18);
    let spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Irqs, spi_cfg);
    let cs = Output::new(p.PIN_17, Level::High);
    let w5500_int = Input::new(p.PIN_21, Pull::Up);
    let w5500_reset = Output::new(p.PIN_20, Level::High);

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
    static STACK: StaticCell<StackStorage> = StaticCell::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), seed);

    // Add the network interface to the stack.
    static DEVICE: StaticCell<Device<'static>> = StaticCell::new();
    let iface = unwrap!(stack.add_iface(DEVICE.init(device)).ok());
    iface.set_dhcpv4(Some(Default::default()));

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    info!("Waiting for DHCP...");
    iface.wait_config_up().await;
    let lease = unwrap!(iface.dhcpv4_lease());
    let local_addr = lease.address.address();
    info!("IP address: {:?}", local_addr);

    // Create two sockets listening to the same port, to handle simultaneous connections
    spawner.spawn(unwrap!(listen_task(stack, 0, 1234)));
    spawner.spawn(unwrap!(listen_task(stack, 1, 1234)));
}

#[embassy_executor::task(pool_size = 2)]
async fn listen_task(stack: Stack<'static>, id: u8, port: u16) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];
    let mut listener = TcpListener::new(stack);
    unwrap!(listener.listen(port));

    loop {
        info!("SOCKET {}: Listening on TCP:{}...", id, port);
        let mut socket = match listener.accept(&mut rx_buffer, &mut tx_buffer).await {
            Ok(socket) => socket,
            Err(e) => {
                warn!("accept error: {:?}", e);
                continue;
            }
        };
        socket.set_timeout(Some(Duration::from_secs(10)));
        info!("SOCKET {}: Received connection from {:?}", id, socket.remote_endpoint());

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("SOCKET {}: {:?}", id, e);
                    break;
                }
            };
            info!("SOCKET {}: rxd {}", id, core::str::from_utf8(&buf[..n]).unwrap());

            if let Err(e) = socket.write_all(&buf[..n]).await {
                warn!("write error: {:?}", e);
                break;
            }
        }
    }
}
