#![no_std]
#![no_main]

use defmt::{info, unwrap, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::StackStorage;
use embassy_net::tcp::TcpListener;
use embassy_net_esp_hosted as hosted;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::rng::Rng;
use embassy_nrf::spim::{self, Spim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io_async::Write;
use hosted::iface::spi::SpiInterface;
use panic_probe as _;
use static_cell::StaticCell;

const WIFI_NETWORK: &str = "EmbassyTest";
const WIFI_PASSWORD: &str = "V8YxhKt5CdIAJFud";

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    RNG => embassy_nrf::rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::task]
async fn wifi_task(
    runner: hosted::Runner<
        'static,
        SpiInterface<ExclusiveDevice<Spim<'static>, Output<'static>, Delay>, Input<'static>>,
        Output<'static>,
    >,
) -> ! {
    runner.run().await
}
#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_nrf::init(Default::default());

    let miso = p.P0_28;
    let sck = p.P0_29;
    let mosi = p.P0_30;
    let cs = Output::new(p.P0_31, Level::High, OutputDrive::HighDrive);
    let handshake = Input::new(p.P1_01, Pull::Up);
    let ready = Input::new(p.P1_04, Pull::None);
    let reset = Output::new(p.P1_05, Level::Low, OutputDrive::Standard);

    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M32;
    config.mode = spim::MODE_2; // !!!
    let spi = spim::Spim::new(p.SPI3, Irqs, sck, miso, mosi, config);
    let spi = ExclusiveDevice::new(spi, cs, Delay);

    let iface = SpiInterface::new(spi, handshake, ready);

    static ESP_STATE: StaticCell<embassy_net_esp_hosted::State> = StaticCell::new();
    let embassy_net_esp_hosted::HostedResources {
        net_device,
        mut control,
        runner,
    } = embassy_net_esp_hosted::new(ESP_STATE.init(embassy_net_esp_hosted::State::new()), iface, reset).await;

    spawner.spawn(unwrap!(wifi_task(runner)));

    unwrap!(control.init().await);
    unwrap!(control.connect(WIFI_NETWORK, WIFI_PASSWORD).await);

    // Generate random seed
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.blocking_fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    // Init network stack
    static STACK: StaticCell<StackStorage> = StaticCell::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), seed);

    // Add the network interface to the stack.
    static DEVICE: StaticCell<hosted::NetDriver<'static>> = StaticCell::new();
    let iface = unwrap!(stack.add_iface(DEVICE.init(net_device)).ok());
    iface.set_dhcpv4(Some(Default::default()));

    spawner.spawn(unwrap!(net_task(runner)));

    // And now we can use it!

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    let mut listener = TcpListener::new(stack);
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
