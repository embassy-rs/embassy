#![no_std]
#![no_main]
teleprobe_meta::target!(b"nrf52840-dk");
teleprobe_meta::timeout!(120);

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_net::{Config, Stack, StackResources};
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::rng::Rng;
use embassy_nrf::spim::{self, Spim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use static_cell::StaticCell;
use {defmt_rtt as _, embassy_net_esp_hosted as hosted, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    RNG => embassy_nrf::rng::InterruptHandler<peripherals::RNG>;
});

// Test-only wifi network, no internet access!
const WIFI_NETWORK: &str = "EmbassyTest";
const WIFI_PASSWORD: &str = "V8YxhKt5CdIAJFud";

#[embassy_executor::task]
async fn wifi_task(
    runner: hosted::Runner<
        'static,
        ExclusiveDevice<Spim<'static, peripherals::SPI3>, Output<'static>, Delay>,
        Input<'static>,
        Output<'static>,
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
    let handshake = Input::new(p.P1_01, Pull::Up);
    let ready = Input::new(p.P1_04, Pull::None);
    let reset = Output::new(p.P1_05, Level::Low, OutputDrive::Standard);

    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M32;
    config.mode = spim::MODE_2; // !!!
    let spi = spim::Spim::new(p.SPI3, Irqs, sck, miso, mosi, config);
    let spi = ExclusiveDevice::new(spi, cs, Delay);

    static STATE: StaticCell<embassy_net_esp_hosted::State> = StaticCell::new();
    let (device, mut control, runner) = embassy_net_esp_hosted::new(
        STATE.init(embassy_net_esp_hosted::State::new()),
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
    static STACK: StaticCell<Stack<MyDriver>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        device,
        Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::<2>::new()),
        seed,
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    perf_client::run(
        stack,
        perf_client::Expected {
            down_kbps: 50,
            up_kbps: 50,
            updown_kbps: 50,
        },
    )
    .await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}
