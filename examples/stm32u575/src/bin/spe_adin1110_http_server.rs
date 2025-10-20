#![no_main]
#![no_std]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_errors_doc)]

// This example works on a Bristlemouth dev kit from Sofar Ocean.
// The webserver shows the actual temperature of the onboard i2c temp sensor.

use core::marker::PhantomData;
use core::sync::atomic::{AtomicI32, Ordering};

use defmt::{Format, error, info, println, unwrap};
use defmt_rtt as _; // global logger
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_futures::yield_now;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv6Address, Ipv6Cidr, Stack, StackResources, StaticConfigV6};
use embassy_net_adin1110::{ADIN1110, Device, Runner};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::i2c::{self, Config as I2C_Config, I2c};
use embassy_stm32::mode::Async;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::spi::{Config as SPI_Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, exti, pac, peripherals};
use embassy_time::{Delay, Duration, Ticker, Timer};
use embedded_hal_async::i2c::I2c as I2cBus;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io::Write as bWrite;
use embedded_io_async::Write;
use heapless::Vec;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

// Basic settings
// MAC-address used by the adin1110
const MAC: [u8; 6] = [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
// Static IP settings
const IP_ADDRESS: Ipv6Cidr = Ipv6Cidr::new(Ipv6Address::new(0xfd00, 0, 0, 0, 0xc0ff, 0xeef0, 0xcacc, 0x1a99), 64);
// Listen port for the webserver
const HTTP_LISTEN_PORT: u16 = 80;

pub type SpeSpi = Spi<'static, Async>;
pub type SpeSpiCs = ExclusiveDevice<SpeSpi, Output<'static>, Delay>;
pub type SpeInt = exti::ExtiInput<'static>;
pub type SpeRst = Output<'static>;
pub type Adin1110T = ADIN1110<SpeSpiCs>;
pub type TempSensI2c = I2c<'static, Async, i2c::Master>;

static TEMP: AtomicI32 = AtomicI32::new(0);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::println!("Start main()");

    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::{
            Hse, HseMode, Hsi48Config, Msirange, Pll, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk,
        };
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.hse = Some(Hse {
            freq: Hertz(16_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::MSIS,
            prediv: PllPreDiv::DIV3,
            mul: PllMul::MUL10,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV2),
            divr: None,
        });
        config.rcc.hsi48 = Some(Hsi48Config::default());
        config.rcc.msis = Some(Msirange::RANGE_48MHZ);
    }

    let dp = embassy_stm32::init(config);

    let reset_status = pac::RCC.bdcr().read().0;
    defmt::println!("bdcr before: 0x{:X}", reset_status);

    defmt::println!("Setup IO pins");

    // Setup LEDs
    // TODO add pca9535 crate
    // let _led_uc1_green = Output::new(dp.PC13, Level::Low, Speed::Low); // expander IO1 1
    // let mut led_uc1_red = Output::new(dp.PE2, Level::High, Speed::Low); // expander IO1 2
    // let led_uc2_green = Output::new(dp.PE6, Level::High, Speed::Low); // expander IO1 3
    // let _led_uc2_red = Output::new(dp.PG15, Level::High, Speed::Low); // expander IO1 4

    // Setup I2C pins
    let _temp_sens_i2c = I2c::new(
        dp.I2C1,
        dp.PB6,
        dp.PB7,
        Irqs,
        dp.GPDMA1_CH9,
        dp.GPDMA1_CH8,
        I2C_Config::default(),
    );

    // Setup IO and SPI for the SPE chip
    let spe_reset_n = Output::new(dp.PA0, Level::Low, Speed::Low);
    let spe_int = exti::ExtiInput::new(dp.PB8, dp.EXTI8, Pull::None);
    let spe_spi_cs_n = Output::new(dp.PA15, Level::High, Speed::High);
    let spe_spi_sclk = dp.PB3;
    let spe_spi_miso = dp.PB4;
    let spe_spi_mosi = dp.PB5;

    let mut spi_config = SPI_Config::default();
    spi_config.frequency = Hertz(20_000_000);

    let spe_spi: SpeSpi = Spi::new(
        dp.SPI3,
        spe_spi_sclk,
        spe_spi_mosi,
        spe_spi_miso,
        dp.GPDMA1_CH13,
        dp.GPDMA1_CH12,
        spi_config,
    );
    let spe_spi = SpeSpiCs::new(spe_spi, spe_spi_cs_n, Delay);

    static STATE: StaticCell<embassy_net_adin1110::State<8, 8>> = StaticCell::new();
    let state = STATE.init(embassy_net_adin1110::State::<8, 8>::new());

    let (device, runner) = embassy_net_adin1110::new(MAC, state, spe_spi, spe_int, spe_reset_n, true, false).await;

    // Start task blink_led
    // spawner.spawn(unwrap!(heartbeat_led(led_uc2_green)));
    // Start ethernet task
    spawner.spawn(unwrap!(ethernet_task(runner)));

    let mut rng = Rng::new(dp.RNG, Irqs);
    // Generate random seed
    let seed = rng.next_u64();

    let ip_cfg = embassy_net::Config::ipv6_static(StaticConfigV6 {
        address: IP_ADDRESS,
        gateway: None,
        dns_servers: Vec::new(),
    });

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(device, ip_cfg, RESOURCES.init(StackResources::new()), seed);

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    let cfg = wait_for_config(stack).await;
    let local_addr = cfg.address.address();

    // Then we can use it!
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut mb_buf = [0; 4096];
    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(1)));

        info!("Listening on http://{}:{}...", local_addr, HTTP_LISTEN_PORT);
        if let Err(e) = socket.accept(HTTP_LISTEN_PORT).await {
            defmt::error!("accept error: {:?}", e);
            continue;
        }

        loop {
            let _n = match socket.read(&mut mb_buf).await {
                Ok(0) => {
                    defmt::info!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    defmt::error!("{:?}", e);
                    break;
                }
            };
            // led_uc1_red.set_low();

            let status_line = "HTTP/1.1 200 OK";
            let contents = PAGE;
            let length = contents.len();

            let _ = write!(
                &mut mb_buf[..],
                "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}\r\n\0"
            );
            let loc = mb_buf.iter().position(|v| *v == b'#').unwrap();

            let temp = TEMP.load(Ordering::Relaxed);
            let cel = temp / 1000;
            let mcel = temp % 1000;

            info!("{}.{}", cel, mcel);

            let _ = write!(&mut mb_buf[loc..loc + 7], "{cel}.{mcel}");

            let n = mb_buf.iter().position(|v| *v == 0).unwrap();

            if let Err(e) = socket.write_all(&mb_buf[..n]).await {
                error!("write error: {:?}", e);
                break;
            }

            // led_uc1_red.set_high();
        }
    }
}

async fn wait_for_config(stack: Stack<'static>) -> embassy_net::StaticConfigV6 {
    loop {
        if let Some(config) = stack.config_v6() {
            return config;
        }
        yield_now().await;
    }
}

#[embassy_executor::task]
async fn heartbeat_led(mut led: Output<'static>) {
    let mut tmr = Ticker::every(Duration::from_hz(3));
    loop {
        led.toggle();
        tmr.next().await;
    }
}

// ADT7422
#[embassy_executor::task]
async fn temp_task(temp_dev_i2c: TempSensI2c, mut led: Output<'static>) -> ! {
    let mut tmr = Ticker::every(Duration::from_hz(1));
    let mut temp_sens = ADT7422::new(temp_dev_i2c, 0x48).unwrap();

    loop {
        led.set_low();
        match select(temp_sens.read_temp(), Timer::after_millis(500)).await {
            Either::First(i2c_ret) => match i2c_ret {
                Ok(value) => {
                    led.set_high();
                    let temp = i32::from(value);
                    println!("TEMP: {:04x}, {}", temp, temp * 78 / 10);
                    TEMP.store(temp * 78 / 10, Ordering::Relaxed);
                }
                Err(e) => defmt::println!("ADT7422: {}", e),
            },
            Either::Second(()) => println!("Timeout"),
        }

        tmr.next().await;
    }
}

#[embassy_executor::task]
async fn ethernet_task(runner: Runner<'static, SpeSpiCs, SpeInt, SpeRst>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub enum Registers {
    Temp_MSB = 0x00,
    Temp_LSB,
    Status,
    Cfg,
    T_HIGH_MSB,
    T_HIGH_LSB,
    T_LOW_MSB,
    T_LOW_LSB,
    T_CRIT_MSB,
    T_CRIT_LSB,
    T_HYST,
    ID,
    SW_RESET = 0x2F,
}

pub struct ADT7422<'d, BUS: I2cBus> {
    addr: u8,
    phantom: PhantomData<&'d ()>,
    bus: BUS,
}

#[derive(Debug, Format, PartialEq, Eq)]
pub enum Error<I2cError: Format> {
    I2c(I2cError),
    Address,
}

impl<BUS> ADT7422<'_, BUS>
where
    BUS: I2cBus,
    BUS::Error: Format,
{
    pub fn new(bus: BUS, addr: u8) -> Result<Self, Error<BUS::Error>> {
        if !(0x48..=0x4A).contains(&addr) {
            return Err(Error::Address);
        }

        Ok(Self {
            bus,
            phantom: PhantomData,
            addr,
        })
    }

    pub async fn init(&mut self) -> Result<(), Error<BUS::Error>> {
        let mut cfg = 0b000_0000;
        // if self.int.is_some() {
        //     // Set 1 SPS mode
        //     cfg |= 0b10 << 5;
        // } else {
        // One shot mode
        cfg |= 0b01 << 5;
        // }

        self.write_cfg(cfg).await
    }

    pub async fn read(&mut self, reg: Registers) -> Result<u8, Error<BUS::Error>> {
        let mut buffer = [0u8; 1];
        self.bus
            .write_read(self.addr, &[reg as u8], &mut buffer)
            .await
            .map_err(Error::I2c)?;
        Ok(buffer[0])
    }

    pub async fn write_cfg(&mut self, cfg: u8) -> Result<(), Error<BUS::Error>> {
        let buf = [Registers::Cfg as u8, cfg];
        self.bus.write(self.addr, &buf).await.map_err(Error::I2c)
    }

    pub async fn read_temp(&mut self) -> Result<i16, Error<BUS::Error>> {
        let mut buffer = [0u8; 2];

        // if let Some(int) = &mut self.int {
        //     // Wait for interrupt
        //     int.wait_for_low().await.unwrap();
        // } else {
        // Start: One shot
        let cfg = 0b01 << 5;
        self.write_cfg(cfg).await?;
        Timer::after_millis(250).await;
        self.bus
            .write_read(self.addr, &[Registers::Temp_MSB as u8], &mut buffer)
            .await
            .map_err(Error::I2c)?;
        Ok(i16::from_be_bytes(buffer))
    }
}

// Web page
const PAGE: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta http-equiv="refresh" content="1" >
    <title>ADIN1110 with Rust</title>
  </head>
  <body>
    <p>EVAL-ADIN1110EBZ</p>
    <table><td>Temp Sensor ADT7422:</td><td> #00.00  &deg;C</td></table>
  </body>
</html>"#;
