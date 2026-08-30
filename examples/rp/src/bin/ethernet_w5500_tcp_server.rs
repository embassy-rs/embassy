//! This example implements a TCP echo server on port 1234 and using DHCP.
//! Send it some data, you should see it echoed back and printed in the console.
//!
//! Example written for the [`WIZnet W5500-EVB-Pico`](https://docs.wiznet.io/Product/iEthernet/W5500/w5500-evb-pico) board.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::StackStorage;
use embassy_net::tcp::TcpListener;
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
    let mut led = Output::new(p.PIN_25, Level::Low);

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
    let iface = unwrap!(stack.add_iface(DEVICE.init(device)));
    iface.set_dhcpv4(Some(Default::default()));

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    info!("Waiting for DHCP...");
    iface.wait_config_up().await;
    let lease = unwrap!(iface.dhcpv4_lease());
    let local_addr = lease.address.address();
    info!("IP address: {:?}", local_addr);

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];
    let mut listener = unwrap!(TcpListener::new(stack));
    unwrap!(listener.listen(1234));

    loop {
        led.set_low();
        info!("Listening on TCP:1234...");
        let mut socket = match listener.accept(&mut rx_buffer, &mut tx_buffer).await {
            Ok(socket) => socket,
            Err(e) => {
                warn!("accept error: {:?}", e);
                continue;
            }
        };
        socket.set_timeout(Some(Duration::from_secs(10)));
        info!("Received connection from {:?}", socket.remote_endpoint());
        led.set_high();

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("{:?}", e);
                    break;
                }
            };
            info!("rxd {}", core::str::from_utf8(&buf[..n]).unwrap());

            if let Err(e) = socket.write_all(&buf[..n]).await {
                warn!("write error: {:?}", e);
                break;
            }
        }
    }
}
