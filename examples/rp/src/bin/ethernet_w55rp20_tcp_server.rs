//! This example implements a TCP echo server on port 1234 and using DHCP.
//! Send it some data, you should see it echoed back and printed in the console.
//!
//! Example written for the [`WIZnet W55RP20-EVB-Pico`](https://docs.wiznet.io/Product/ioNIC/W55RP20/w55rp20-evb-pico) board.
//! Note: the W55RP20 is a single package that contains both a RP2040 and the Wiznet W5500 ethernet
//! controller

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
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIO0};
use embassy_rp::pio_programs::spi::Spi;
use embassy_rp::spi::{Async, Config as SpiConfig};
use embassy_rp::{bind_interrupts, pio};
use embassy_time::{Delay, Duration};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io_async::Write;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
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
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
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
    spi_cfg.frequency = 12_500_000; // The PIO SPI program is much less stable than the actual SPI
    // peripheral, use higher speeds at your peril
    let spi = Spi::new(&mut common, sm0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Irqs, spi_cfg);

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
