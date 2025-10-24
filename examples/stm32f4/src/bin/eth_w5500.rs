#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, StackResources};
use embassy_net_wiznet::chip::W5500;
use embassy_net_wiznet::{Device, Runner, State};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::rng::Rng;
use embassy_stm32::spi::Spi;
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, peripherals, rng, spi};
use embassy_time::{Delay, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io_async::Write;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    HASH_RNG => rng::InterruptHandler<peripherals::RNG>;
});

type EthernetSPI = ExclusiveDevice<Spi<'static, Async>, Output<'static>, Delay>;
#[embassy_executor::task]
async fn ethernet_task(runner: Runner<'static, W5500, EthernetSPI, ExtiInput<'static>, Output<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL180,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 180 / 2 = 180Mhz.
            divq: None,
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    // Generate random seed
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    unwrap!(rng.async_fill_bytes(&mut seed).await);
    let seed = u64::from_le_bytes(seed);

    let mut spi_cfg = spi::Config::default();
    spi_cfg.frequency = Hertz(50_000_000); // up to 50m works
    let (miso, mosi, clk) = (p.PA6, p.PA7, p.PA5);
    let spi = Spi::new(p.SPI1, clk, mosi, miso, p.DMA2_CH3, p.DMA2_CH0, spi_cfg);
    let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);
    let spi = unwrap!(ExclusiveDevice::new(spi, cs, Delay));

    let w5500_int = ExtiInput::new(p.PB0, p.EXTI0, Pull::Up);
    let w5500_reset = Output::new(p.PB1, Level::High, Speed::VeryHigh);

    let mac_addr = [0x02, 234, 3, 4, 82, 231];
    static STATE: StaticCell<State<2, 2>> = StaticCell::new();
    let state = STATE.init(State::<2, 2>::new());
    let (device, runner) = embassy_net_wiznet::new(mac_addr, state, spi, w5500_int, w5500_reset)
        .await
        .unwrap();
    spawner.spawn(unwrap!(ethernet_task(runner)));

    let config = embassy_net::Config::dhcpv4(Default::default());
    //let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //    address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    //});

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    // Ensure DHCP configuration is up before trying connect
    stack.wait_config_up().await;

    info!("Network task initialized");

    // Then we can use it!
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        let remote_endpoint = (Ipv4Address::new(10, 42, 0, 1), 8000);
        info!("connecting...");
        let r = socket.connect(remote_endpoint).await;
        if let Err(e) = r {
            info!("connect error: {:?}", e);
            Timer::after_secs(1).await;
            continue;
        }
        info!("connected!");
        let buf = [0; 1024];
        loop {
            let r = socket.write_all(&buf).await;
            if let Err(e) = r {
                info!("write error: {:?}", e);
                break;
            }
            Timer::after_secs(1).await;
        }
    }
}
