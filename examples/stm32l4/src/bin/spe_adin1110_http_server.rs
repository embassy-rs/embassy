#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![no_main]
#![no_std]
// Needed unitl https://github.com/rust-lang/rust/issues/63063 is stablised.
#![feature(type_alias_impl_trait)]
#![feature(associated_type_bounds)]
#![allow(clippy::missing_errors_doc)]

// This example works on a ANALOG DEVICE EVAL-ADIN110EBZ board.
// Settings switch S201 "HW CFG":
//  - Without SPI CRC: OFF-ON-OFF-OFF-OFF
//  -    With SPI CRC: ON -ON-OFF-OFF-OFF
// Settings switch S303 "uC CFG":
// - CFG0: On = static ip, Off = Dhcp
// - CFG1: Ethernet `FCS` on TX path: On, Off
// The webserver shows the actual temperature of the onboard i2c temp sensor.

use core::marker::PhantomData;
use core::sync::atomic::{AtomicI32, Ordering};

use defmt::{error, info, println, unwrap, Format};
use defmt_rtt as _; // global logger
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_futures::yield_now;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StackResources, StaticConfigV4};
use embassy_time::{Delay, Duration, Ticker, Timer};
use embedded_hal_async::i2c::I2c as I2cBus;
use embedded_io::Write as bWrite;
use embedded_io_async::Write;
use hal::gpio::{Input, Level, Output, Speed};
use hal::i2c::{self, I2c};
use hal::rcc::{self};
use hal::rng::{self, Rng};
use hal::{bind_interrupts, exti, pac, peripherals};
use heapless::Vec;
use rand::RngCore;
use static_cell::make_static;
use {embassy_stm32 as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C3_EV => i2c::InterruptHandler<peripherals::I2C3>;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

use embassy_net_adin1110::{self, Device, Runner, ADIN1110};
use embedded_hal_bus::spi::ExclusiveDevice;
use hal::gpio::Pull;
use hal::i2c::Config as I2C_Config;
use hal::rcc::{ClockSrc, PLLClkDiv, PLLMul, PLLSource, PLLSrcDiv};
use hal::spi::{Config as SPI_Config, Spi};
use hal::time::Hertz;

// Basic settings
// MAC-address used by the adin1110
const MAC: [u8; 6] = [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
// Static IP settings
const IP_ADDRESS: Ipv4Cidr = Ipv4Cidr::new(Ipv4Address([192, 168, 1, 5]), 24);
// Listen port for the webserver
const HTTP_LISTEN_PORT: u16 = 80;

pub type SpeSpi = Spi<'static, peripherals::SPI2, peripherals::DMA1_CH1, peripherals::DMA1_CH2>;
pub type SpeSpiCs = ExclusiveDevice<SpeSpi, Output<'static, peripherals::PB12>, Delay>;
pub type SpeInt = exti::ExtiInput<'static, peripherals::PB11>;
pub type SpeRst = Output<'static, peripherals::PC7>;
pub type Adin1110T = ADIN1110<SpeSpiCs>;
pub type TempSensI2c = I2c<'static, peripherals::I2C3, peripherals::DMA1_CH6, peripherals::DMA1_CH7>;

static TEMP: AtomicI32 = AtomicI32::new(0);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::println!("Start main()");

    let mut config = embassy_stm32::Config::default();

    // 80Mhz clock (Source: 8 / SrcDiv: 1 * PLLMul 20 / ClkDiv 2)
    // 80MHz highest frequency for flash 0 wait.
    config.rcc.mux = ClockSrc::PLL(
        PLLSource::HSE(Hertz(8_000_000)),
        PLLClkDiv::Div2,
        PLLSrcDiv::Div1,
        PLLMul::Mul20,
        None,
    );
    config.rcc.hsi48 = true; // needed for rng
    config.rcc.rtc_mux = rcc::RtcClockSource::LSI;

    let dp = embassy_stm32::init(config);

    // RM0432rev9, 5.1.2: Independent I/O supply rail
    // After reset, the I/Os supplied by VDDIO2 are logically and electrically isolated and
    // therefore are not available. The isolation must be removed before using any I/O from
    // PG[15:2], by setting the IOSV bit in the PWR_CR2 register, once the VDDIO2 supply is present
    pac::PWR.cr2().modify(|w| w.set_iosv(true));

    let reset_status = pac::RCC.bdcr().read().0;
    defmt::println!("bdcr before: 0x{:X}", reset_status);

    defmt::println!("Setup IO pins");

    // Setup LEDs
    let _led_uc1_green = Output::new(dp.PC13, Level::Low, Speed::Low);
    let mut led_uc2_red = Output::new(dp.PE2, Level::High, Speed::Low);
    let led_uc3_yellow = Output::new(dp.PE6, Level::High, Speed::Low);
    let led_uc4_blue = Output::new(dp.PG15, Level::High, Speed::Low);

    // Read the uc_cfg switches
    let uc_cfg0 = Input::new(dp.PB2, Pull::None);
    let uc_cfg1 = Input::new(dp.PF11, Pull::None);
    let _uc_cfg2 = Input::new(dp.PG6, Pull::None);
    let _uc_cfg3 = Input::new(dp.PG11, Pull::None);

    // Setup I2C pins
    let temp_sens_i2c = I2c::new(
        dp.I2C3,
        dp.PG7,
        dp.PG8,
        Irqs,
        dp.DMA1_CH6,
        dp.DMA1_CH7,
        Hertz(100_000),
        I2C_Config::default(),
    );

    // Setup IO and SPI for the SPE chip
    let spe_reset_n = Output::new(dp.PC7, Level::Low, Speed::Low);
    let spe_cfg0 = Input::new(dp.PC8, Pull::None);
    let spe_cfg1 = Input::new(dp.PC9, Pull::None);
    let _spe_ts_capt = Output::new(dp.PC6, Level::Low, Speed::Low);

    let spe_int = Input::new(dp.PB11, Pull::None);
    let spe_int = exti::ExtiInput::new(spe_int, dp.EXTI11);

    let spe_spi_cs_n = Output::new(dp.PB12, Level::High, Speed::High);
    let spe_spi_sclk = dp.PB13;
    let spe_spi_miso = dp.PB14;
    let spe_spi_mosi = dp.PB15;

    // Don't turn the clock to high, clock must fit within the system clock as we get a runtime panic.
    let mut spi_config = SPI_Config::default();
    spi_config.frequency = Hertz(25_000_000);

    let spe_spi: SpeSpi = Spi::new(
        dp.SPI2,
        spe_spi_sclk,
        spe_spi_mosi,
        spe_spi_miso,
        dp.DMA1_CH1,
        dp.DMA1_CH2,
        spi_config,
    );
    let spe_spi = SpeSpiCs::new(spe_spi, spe_spi_cs_n, Delay);

    let cfg0_without_crc = spe_cfg0.is_high();
    let cfg1_spi_mode = spe_cfg1.is_high();
    let uc_cfg1_fcs_en = uc_cfg1.is_low();

    defmt::println!(
        "ADIN1110: CFG SPI-MODE 1-{}, CRC-bit 0-{} FCS-{}",
        cfg1_spi_mode,
        cfg0_without_crc,
        uc_cfg1_fcs_en
    );

    // Check the SPI mode selected with the "HW CFG" dip-switch
    if !cfg1_spi_mode {
        error!("Driver doesn´t support SPI Protolcol \"OPEN Alliance\".\nplease use the \"Generic SPI\"! Turn On \"HW CFG\": \"SPI_CFG1\"");
        loop {
            led_uc2_red.toggle();
            Timer::after(Duration::from_hz(10)).await;
        }
    };

    let state = make_static!(embassy_net_adin1110::State::<8, 8>::new());

    let (device, runner) = embassy_net_adin1110::new(
        MAC,
        state,
        spe_spi,
        spe_int,
        spe_reset_n,
        !cfg0_without_crc,
        uc_cfg1_fcs_en,
    )
    .await;

    // Start task blink_led
    unwrap!(spawner.spawn(heartbeat_led(led_uc3_yellow)));
    // Start task temperature measurement
    unwrap!(spawner.spawn(temp_task(temp_sens_i2c, led_uc4_blue)));
    // Start ethernet task
    unwrap!(spawner.spawn(ethernet_task(runner)));

    let mut rng = Rng::new(dp.RNG, Irqs);
    // Generate random seed
    let seed = rng.next_u64();

    let ip_cfg = if uc_cfg0.is_low() {
        println!("Waiting for DHCP...");
        let dhcp4_config = embassy_net::DhcpConfig::default();
        embassy_net::Config::dhcpv4(dhcp4_config)
    } else {
        embassy_net::Config::ipv4_static(StaticConfigV4 {
            address: IP_ADDRESS,
            gateway: None,
            dns_servers: Vec::new(),
        })
    };

    // Init network stack
    let stack = &*make_static!(Stack::new(
        device,
        ip_cfg,
        make_static!(StackResources::<2>::new()),
        seed
    ));

    // Launch network task
    unwrap!(spawner.spawn(net_task(stack)));

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
            led_uc2_red.set_low();

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

            led_uc2_red.set_high();
        }
    }
}

async fn wait_for_config(stack: &'static Stack<Device<'static>>) -> embassy_net::StaticConfigV4 {
    loop {
        if let Some(config) = stack.config_v4() {
            return config;
        }
        yield_now().await;
    }
}

#[embassy_executor::task]
async fn heartbeat_led(mut led: Output<'static, peripherals::PE6>) {
    let mut tmr = Ticker::every(Duration::from_hz(3));
    loop {
        led.toggle();
        tmr.next().await;
    }
}

// ADT7422
#[embassy_executor::task]
async fn temp_task(temp_dev_i2c: TempSensI2c, mut led: Output<'static, peripherals::PG15>) -> ! {
    let mut tmr = Ticker::every(Duration::from_hz(1));
    let mut temp_sens = ADT7422::new(temp_dev_i2c, 0x48).unwrap();

    loop {
        led.set_low();
        match select(temp_sens.read_temp(), Timer::after(Duration::from_millis(500))).await {
            Either::First(i2c_ret) => match i2c_ret {
                Ok(value) => {
                    led.set_high();
                    let temp = i32::from(value);
                    println!("TEMP: {:04x}, {}", temp, temp * 78 / 10);
                    TEMP.store(temp * 78 / 10, Ordering::Relaxed);
                }
                Err(e) => defmt::println!("ADT7422: {}", e),
            },
            Either::Second(_) => println!("Timeout"),
        }

        tmr.next().await;
    }
}

#[embassy_executor::task]
async fn ethernet_task(runner: Runner<'static, SpeSpiCs, SpeInt, SpeRst>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Device<'static>>) -> ! {
    stack.run().await
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

#[derive(Debug, Format)]
pub enum Error<I2cError: Format> {
    I2c(I2cError),
    Address,
}

impl<'d, BUS> ADT7422<'d, BUS>
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
        Timer::after(Duration::from_millis(250)).await;
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
