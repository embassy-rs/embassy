#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::StackStorage;
use embassy_net::tcp::TcpListener;
use embassy_net_enc28j60::Enc28j60;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::rng::Rng;
use embassy_nrf::spim::Spim;
use embassy_nrf::{bind_interrupts, peripherals, spim};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io_async::Write;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    RNG => embassy_nrf::rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
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

    // Generate random seed
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.blocking_fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    static STACK: StaticCell<StackStorage> = StaticCell::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), seed);

    // Add the network interface to the stack.
    static DEVICE: StaticCell<Enc28j60<ExclusiveDevice<Spim<'static>, Output<'static>, Delay>, Output<'static>>> =
        StaticCell::new();
    let iface = unwrap!(stack.add_iface(DEVICE.init(device)));
    iface.set_dhcpv4(Some(Default::default()));

    spawner.spawn(unwrap!(net_task(runner)));

    // And now we can use it!

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    let mut listener = unwrap!(TcpListener::new(stack));
    unwrap!(listener.listen(1234));

    loop {
        info!("Listening on TCP:1234...");
        let mut socket = match listener.accept(&mut rx_buffer, &mut tx_buffer).await {
            Ok(socket) => socket,
            Err(e) => {
                warn!("accept error: {:?}", e);
                continue;
            }
        };
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        info!("Received connection from {:?}", socket.remote_endpoint());

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("read error: {:?}", e);
                    break;
                }
            };

            info!("rxd {:02x}", &buf[..n]);

            match socket.write_all(&buf[..n]).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}
