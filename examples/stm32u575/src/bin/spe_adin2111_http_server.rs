#![no_main]
#![no_std]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_errors_doc)]

// This example works on a Bristlemouth dev kit from Sofar Ocean.
// The webserver shows the actual temperature of the onboard i2c temp sensor.

use core::sync::atomic::{AtomicI32, Ordering};

use defmt::{Format, error, info, println, unwrap};
use defmt_rtt as _; // global logger
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_futures::yield_now;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv6Address, Ipv6Cidr, Stack, StackResources, StaticConfigV6};
use embassy_net_adin1110::{ADIN1110, Device, Runner, Tc6, TxPort};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::i2c::{self, Config as I2C_Config, I2c};
use embassy_stm32::mode::Async;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::spi::mode::Master;
use embassy_stm32::spi::{Config as SPI_Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, exti, interrupt, pac, peripherals};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
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
    EXTI8 => exti::InterruptHandler<interrupt::typelevel::EXTI8>;
    GPDMA1_CHANNEL8 => embassy_stm32::dma::InterruptHandler<peripherals::GPDMA1_CH8>;
    GPDMA1_CHANNEL9 => embassy_stm32::dma::InterruptHandler<peripherals::GPDMA1_CH9>;
    GPDMA1_CHANNEL12 => embassy_stm32::dma::InterruptHandler<peripherals::GPDMA1_CH12>;
    GPDMA1_CHANNEL13 => embassy_stm32::dma::InterruptHandler<peripherals::GPDMA1_CH13>;
});

// Listen port for the webserver
const HTTP_LISTEN_PORT: u16 = 80;

/// Bristlemouth node ID: an FNV-1a 64 hash of the 96-bit factory chip UID, so
/// every board gets a stable identity with nothing to program per unit.
///
/// This mirrors `getNodeId()` in bm_protocol (`src/lib/common/device_info.c`),
/// which is `fnv_64a_buf(UID, 12, 0)`. Note the starting hash value is 0, not
/// the usual FNV-1a offset basis of 0xcbf29ce484222325 that `fnv.h` itself
/// recommends. That is what the C code passes, and matching it is what makes a
/// board running this firmware keep the identity it had under the C firmware.
fn node_id() -> u64 {
    const FNV_64_PRIME: u64 = 0x0000_0100_0000_01b3;

    // `uid()` hands back the three 32-bit UID words as little-endian bytes,
    // which is the same byte order the C code hashes them in.
    let mut hash: u64 = 0;
    for byte in embassy_stm32::uid::uid() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_64_PRIME);
    }
    hash
}

/// The node's IPv6 address: `fd00::` with the node ID as the bottom 64 bits.
/// Matches `bm_ip_init()` in bm_core (`network/bm_lwip.c`).
fn node_ip(id: u64) -> Ipv6Cidr {
    let b = id.to_be_bytes();
    let addr = Ipv6Address::new(
        0xfd00,
        0,
        0,
        0,
        u16::from_be_bytes([b[0], b[1]]),
        u16::from_be_bytes([b[2], b[3]]),
        u16::from_be_bytes([b[4], b[5]]),
        u16::from_be_bytes([b[6], b[7]]),
    );
    Ipv6Cidr::new(addr, 64)
}

/// The node's MAC address: two zero bytes followed by the bottom 32 bits of the
/// node ID. Matches `mac_address()` in bm_core (`common/device.c`); the leading
/// 16 bits are common to all Bristlemouth devices and are zero for now.
fn node_mac(id: u64) -> [u8; 6] {
    let b = id.to_be_bytes();
    [0x00, 0x00, b[4], b[5], b[6], b[7]]
}

pub type SpeSpi = Spi<'static, Async, Master>;
pub type SpeSpiCs = ExclusiveDevice<SpeSpi, Output<'static>, Delay>;
pub type SpeInt = exti::ExtiInput<'static, Async>;
pub type SpeRst = Output<'static>;
pub type Adin1110T = ADIN1110<Tc6<SpeSpiCs>>;

/// The single I2C1 bus, shared between the Bristlefin IO expander and the
/// BME280 environmental sensor.
pub type TempSensI2c = I2c<'static, Async, i2c::Master>;
/// One handle onto the shared I2C bus.
pub type SharedI2c = I2cDevice<'static, NoopRawMutex, TempSensI2c>;
/// The LED driver, shared between the sensor task and the HTTP handler.
pub type SharedLeds = Mutex<NoopRawMutex, Bristlefin<SharedI2c>>;

/// Latest temperature reading, in milli-degrees Celsius.
static TEMP: AtomicI32 = AtomicI32::new(0);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::println!("Start main()");

    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::{MSIRange, Pll, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk};
        // sysclk 160 MHz: MSIS 48 MHz / 3 * 10 / 1
        config.rcc.sys = Sysclk::Pll1R;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Msis,
            prediv: PllPreDiv::Div3,
            mul: PllMul::Mul10,
            divp: Some(PllDiv::Div2),
            divq: Some(PllDiv::Div2),
            divr: Some(PllDiv::Div1),
        });
        config.rcc.msis = Some(MSIRange::Range48mhz);
    }

    let dp = embassy_stm32::init(config);

    let reset_status = pac::RCC.bdcr().read().0;
    defmt::println!("bdcr before: 0x{:X}", reset_status);

    // Derive this board's stable identity from its factory chip UID.
    let node_id = node_id();
    let mac = node_mac(node_id);
    let ip_address = node_ip(node_id);
    info!("Bristlemouth node id: {:016x}", node_id);
    info!("  MAC: {:02x}", mac);
    info!("  IP:  {}", ip_address.address());

    defmt::println!("Setup IO pins");

    // Setup I2C. Both the Bristlefin IO expander (LEDs, power rails) and the
    // BME280 environmental sensor live on this bus, so it is shared.
    let i2c_bus = I2c::new(
        dp.I2C1,
        dp.PB6,
        dp.PB7,
        dp.GPDMA1_CH9,
        dp.GPDMA1_CH8,
        Irqs,
        I2C_Config::default(),
    );
    static I2C_BUS: StaticCell<Mutex<NoopRawMutex, TempSensI2c>> = StaticCell::new();
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c_bus));

    // The IO expander drives the four Bristlefin LEDs and the 3V3 rail that the
    // BME280 hangs off, so it has to come up before the sensor.
    // A missing or unpowered Bristlefin costs us the LEDs and the sensor, but
    // the network half of the example still works, so this is not fatal.
    let mut bristlefin = Bristlefin::new(I2cDevice::new(i2c_bus));
    match bristlefin.init().await {
        // init() brings the 3V3 rail up as part of configuring the outputs.
        Ok(()) => info!("Bristlefin IO expander ready, 3V3 rail on"),
        Err(e) => error!("Bristlefin IO expander not responding: {} (no LEDs/sensor)", e),
    }
    Timer::after_millis(50).await;

    static LEDS: StaticCell<SharedLeds> = StaticCell::new();
    let leds: &'static SharedLeds = LEDS.init(Mutex::new(bristlefin));

    // Setup IO and SPI for the SPE chip
    let spe_reset_n = Output::new(dp.PA0, Level::Low, Speed::Low);
    let spe_int = exti::ExtiInput::new(dp.PB8, dp.EXTI8, Pull::None, Irqs);
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
        Irqs,
        spi_config,
    );
    let spe_spi = SpeSpiCs::new(spe_spi, spe_spi_cs_n, Delay);

    static STATE: StaticCell<embassy_net_adin1110::State<8, 8>> = StaticCell::new();
    let state = STATE.init(embassy_net_adin1110::State::<8, 8>::new());

    // On the Bristlemouth dev kit PH1 controls load switches to the ADIN2111
    let mut adin2111_en = Output::new(dp.PH1, Level::Low, Speed::Low);
    adin2111_en.set_high();
    // From page 22 of the ADIN2111 datasheet
    Timer::after_millis(90).await;

    let (device, runner) =
        embassy_net_adin1110::new_tc6(mac, state, spe_spi, spe_int, spe_reset_n, false, TxPort::Flood).await;

    // Start the environmental sensor task, which doubles as the heartbeat.
    spawner.spawn(unwrap!(temp_task(I2cDevice::new(i2c_bus), leds)));
    // Start ethernet task
    spawner.spawn(unwrap!(ethernet_task(runner)));

    let mut rng = Rng::new(dp.RNG, Irqs);
    // Generate random seed
    let seed = rng.next_u64();

    let ip_cfg = embassy_net::Config::ipv6_static(StaticConfigV6 {
        address: ip_address,
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
            // LED 1 green marks "serving a request".
            if let Err(e) = leds.lock().await.set_led(1, LedState::Green).await {
                error!("LED 1 update failed: {}", e);
            }

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

            info!("Served page to {:?}, temperature {}.{} C", socket.remote_endpoint(), cel, mcel);

            let _ = write!(&mut mb_buf[loc..loc + 7], "{cel}.{mcel}");

            let n = mb_buf.iter().position(|v| *v == 0).unwrap();

            if let Err(e) = socket.write_all(&mb_buf[..n]).await {
                error!("write error: {:?}", e);
                let _ = leds.lock().await.set_led(1, LedState::Red).await;
                break;
            }

            if let Err(e) = leds.lock().await.set_led(1, LedState::Off).await {
                error!("LED 1 update failed: {}", e);
            }
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

/// How long the heartbeat LED stays lit. The Bristlefin LEDs are bright, so
/// this is a brief flash rather than a 50% duty cycle.
const HEARTBEAT_ON: Duration = Duration::from_millis(100);
/// Flash the heartbeat every this many sensor ticks. The sensor runs at 1Hz,
/// so 2 ticks gives 100ms on, 1900ms off.
const HEARTBEAT_EVERY_N_TICKS: u32 = 2;

/// Flash LED 2 green once, briefly.
async fn heartbeat_flash(leds: &'static SharedLeds) {
    if let Err(e) = leds.lock().await.set_led(2, LedState::Green).await {
        error!("LED 2 update failed: {}", e);
        return;
    }
    Timer::after(HEARTBEAT_ON).await;
    if let Err(e) = leds.lock().await.set_led(2, LedState::Off).await {
        error!("LED 2 update failed: {}", e);
    }
}

/// Read the BME280 once a second into [`TEMP`], and use LED 2 as a heartbeat:
/// a short green flash every two seconds while the sensor is healthy, steady
/// red once a read fails.
#[embassy_executor::task]
async fn temp_task(bus: SharedI2c, leds: &'static SharedLeds) -> ! {
    let mut tmr = Ticker::every(Duration::from_hz(1));

    let mut sensor = loop {
        let mut bus = bus.clone();
        // The BME280 is behind the bus multiplexer, so open its channel first.
        let opened = select_mux_channel(&mut bus, BME280_MUX_CHANNEL).await;
        match opened {
            Ok(()) => match Bme280::new(bus, BME280_ADDR).await {
                Ok(s) => break s,
                Err(e) => error!("BME280 init failed: {}, retrying", e),
            },
            Err(e) => error!("I2C mux channel select failed: {}, retrying", e),
        }
        let _ = leds.lock().await.set_led(2, LedState::Red).await;
        Timer::after_millis(1000).await;
    };
    info!("BME280 at 0x{:02x} ready", BME280_ADDR);

    // Nothing else touches the multiplexer, but re-selecting the channel each
    // time costs one byte and keeps this working if anything ever does.
    let mut mux = bus.clone();

    let mut ticks: u32 = 0;
    loop {
        if let Err(e) = select_mux_channel(&mut mux, BME280_MUX_CHANNEL).await {
            error!("I2C mux channel select failed: {}", e);
        }

        match select(sensor.read_temp_millicelsius(), Timer::after_millis(500)).await {
            Either::First(Ok(milli_c)) => {
                println!("TEMP: {} m°C", milli_c);
                TEMP.store(milli_c, Ordering::Relaxed);
                ticks += 1;
                if ticks.is_multiple_of(HEARTBEAT_EVERY_N_TICKS) {
                    heartbeat_flash(leds).await;
                }
            }
            Either::First(Err(e)) => {
                error!("BME280 read failed: {}", e);
                let _ = leds.lock().await.set_led(2, LedState::Red).await;
            }
            Either::Second(()) => {
                error!("BME280 read timed out");
                let _ = leds.lock().await.set_led(2, LedState::Red).await;
            }
        }

        tmr.next().await;
    }
}

#[embassy_executor::task]
async fn ethernet_task(runner: Runner<'static, Tc6<SpeSpiCs>, SpeInt, SpeRst>) -> ! {
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

/// I2C address of the Bristlefin IO expander (PCA9535).
pub const BRISTLEFIN_ADDR: u8 = 0x21;
/// I2C address of the TCA9546A bus multiplexer on the Bristlefin.
pub const TCA9546A_ADDR: u8 = 0x70;
/// I2C address of the BME280, which sits behind the multiplexer.
pub const BME280_ADDR: u8 = 0x76;
/// Multiplexer channel the BME280 is on.
pub const BME280_MUX_CHANNEL: u8 = 0;

/// Open one TCA9546A channel (and close the others).
///
/// The IO expander and the INA232s sit upstream of the multiplexer and stay
/// reachable regardless; the BME280 only answers once its channel is open.
async fn select_mux_channel<BUS>(bus: &mut BUS, channel: u8) -> Result<(), Error<BUS::Error>>
where
    BUS: I2cBus,
    BUS::Error: Format,
{
    bus.write(TCA9546A_ADDR, &[1u8 << channel]).await.map_err(Error::I2c)
}

#[derive(Debug, Format)]
pub enum Error<I2cError: Format> {
    I2c(I2cError),
    /// The device at this address did not identify itself as expected.
    WrongChipId(u8),
}

// ---------------------------------------------------------------------------
// Bristlefin IO expander (PCA9535)
//
// Pin assignment is from the bm_protocol BSP (src/bsp/bm_mote_v1.0/bsp_pins.c):
//   pin 3  BF_3V3_EN   (1 enables the 3V3 rail the BME280 hangs off)
//   pin 9  BF_LED_G1   pin 10 BF_LED_R1
//   pin 11 BF_LED_G2   pin 12 BF_LED_R2
// The LEDs are active low.
// ---------------------------------------------------------------------------

const PCA9535_OUTPUT0: u8 = 0x02;
const PCA9535_CONFIG0: u8 = 0x06;

const BF_PIN_3V3_EN: u8 = 3;
const BF_PIN_LED_G1: u8 = 9;
const BF_PIN_LED_R1: u8 = 10;
const BF_PIN_LED_G2: u8 = 11;
const BF_PIN_LED_R2: u8 = 12;

const BF_LED_ON: bool = false;
const BF_LED_OFF: bool = true;

/// Colour of one of the two bi-colour Bristlefin LEDs.
#[derive(Copy, Clone, Debug, Format, PartialEq, Eq)]
pub enum LedState {
    Off,
    Green,
    Red,
    Yellow,
}

/// The Bristlefin IO expander: LEDs and the 3V3 rail.
pub struct Bristlefin<BUS: I2cBus> {
    bus: BUS,
    /// Cached output port state, so a single pin can be changed without a read.
    out: u16,
}

impl<BUS> Bristlefin<BUS>
where
    BUS: I2cBus,
    BUS::Error: Format,
{
    pub fn new(bus: BUS) -> Self {
        // LEDs off (active low), and the 3V3 rail ENABLED.
        //
        // 3V3 must start high: the I2C pull-ups are on that rail, so if this pin
        // is ever driven low we lose the bus and can no longer talk to the
        // expander to turn it back on. That state survives an MCU reset and
        // needs the board physically power cycled to clear.
        let out = (1 << BF_PIN_3V3_EN)
            | (1 << BF_PIN_LED_G1)
            | (1 << BF_PIN_LED_R1)
            | (1 << BF_PIN_LED_G2)
            | (1 << BF_PIN_LED_R2);
        Self { bus, out }
    }

    /// Load the output latches first, then switch those pins from inputs to
    /// outputs. In this order the pins go straight to their intended level:
    /// the LEDs never glitch on and the 3V3 rail never glitches off.
    pub async fn init(&mut self) -> Result<(), Error<BUS::Error>> {
        self.flush().await?;

        let outputs = (1u16 << BF_PIN_3V3_EN)
            | (1 << BF_PIN_LED_G1)
            | (1 << BF_PIN_LED_R1)
            | (1 << BF_PIN_LED_G2)
            | (1 << BF_PIN_LED_R2);
        // In the PCA9535 config register a 0 bit means "output".
        let config = !outputs;
        self.bus
            .write(BRISTLEFIN_ADDR, &[PCA9535_CONFIG0, config.to_le_bytes()[0], config.to_le_bytes()[1]])
            .await
            .map_err(Error::I2c)
    }

    /// Enable or disable the Bristlefin 3V3 rail. The BME280 needs it.
    pub async fn set_3v3(&mut self, on: bool) -> Result<(), Error<BUS::Error>> {
        self.set_pin(BF_PIN_3V3_EN, on).await
    }

    /// Set LED 1 or 2 to a colour.
    pub async fn set_led(&mut self, led: u8, state: LedState) -> Result<(), Error<BUS::Error>> {
        let (green, red) = if led == 1 {
            (BF_PIN_LED_G1, BF_PIN_LED_R1)
        } else {
            (BF_PIN_LED_G2, BF_PIN_LED_R2)
        };
        let (g, r) = match state {
            LedState::Off => (BF_LED_OFF, BF_LED_OFF),
            LedState::Green => (BF_LED_ON, BF_LED_OFF),
            LedState::Red => (BF_LED_OFF, BF_LED_ON),
            LedState::Yellow => (BF_LED_ON, BF_LED_ON),
        };
        self.write_pin(green, g);
        self.write_pin(red, r);
        self.flush().await
    }

    async fn set_pin(&mut self, pin: u8, high: bool) -> Result<(), Error<BUS::Error>> {
        self.write_pin(pin, high);
        self.flush().await
    }

    fn write_pin(&mut self, pin: u8, high: bool) {
        if high {
            self.out |= 1 << pin;
        } else {
            self.out &= !(1 << pin);
        }
    }

    async fn flush(&mut self) -> Result<(), Error<BUS::Error>> {
        self.bus
            .write(
                BRISTLEFIN_ADDR,
                &[PCA9535_OUTPUT0, self.out.to_le_bytes()[0], self.out.to_le_bytes()[1]],
            )
            .await
            .map_err(Error::I2c)
    }
}

// ---------------------------------------------------------------------------
// BME280 (temperature only)
//
// The dev kit carries a BME280, not the ADT7422 that the ADI eval board this
// example was ported from used. Only the temperature channel is read here.
// ---------------------------------------------------------------------------

const BME280_REG_ID: u8 = 0xD0;
const BME280_REG_CTRL_MEAS: u8 = 0xF4;
const BME280_REG_STATUS: u8 = 0xF3;
const BME280_REG_TEMP: u8 = 0xFA;
const BME280_REG_CALIB: u8 = 0x88;
const BME280_CHIP_ID: u8 = 0x60;
/// Oversampling x1 on temperature (bits 7:5), pressure skipped (bits 4:2),
/// forced mode (bits 1:0).
const BME280_CTRL_MEAS_FORCED: u8 = (0b001 << 5) | 0b01;

pub struct Bme280<BUS: I2cBus> {
    bus: BUS,
    addr: u8,
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
}

impl<BUS> Bme280<BUS>
where
    BUS: I2cBus,
    BUS::Error: Format,
{
    pub async fn new(mut bus: BUS, addr: u8) -> Result<Self, Error<BUS::Error>> {
        let mut id = [0u8; 1];
        bus.write_read(addr, &[BME280_REG_ID], &mut id)
            .await
            .map_err(Error::I2c)?;
        if id[0] != BME280_CHIP_ID {
            return Err(Error::WrongChipId(id[0]));
        }

        // Temperature calibration: dig_T1 (u16), dig_T2, dig_T3 (i16), LSB first.
        let mut calib = [0u8; 6];
        bus.write_read(addr, &[BME280_REG_CALIB], &mut calib)
            .await
            .map_err(Error::I2c)?;

        Ok(Self {
            bus,
            addr,
            dig_t1: u16::from_le_bytes([calib[0], calib[1]]),
            dig_t2: i16::from_le_bytes([calib[2], calib[3]]),
            dig_t3: i16::from_le_bytes([calib[4], calib[5]]),
        })
    }

    /// Trigger a single forced-mode conversion and return milli-degrees Celsius.
    pub async fn read_temp_millicelsius(&mut self) -> Result<i32, Error<BUS::Error>> {
        self.bus
            .write(self.addr, &[BME280_REG_CTRL_MEAS, BME280_CTRL_MEAS_FORCED])
            .await
            .map_err(Error::I2c)?;

        // A x1 temperature conversion takes well under 10ms; poll STATUS.measuring
        // rather than assuming a fixed time.
        for _ in 0..20 {
            Timer::after_millis(2).await;
            let mut status = [0u8; 1];
            self.bus
                .write_read(self.addr, &[BME280_REG_STATUS], &mut status)
                .await
                .map_err(Error::I2c)?;
            if status[0] & 0b0000_1000 == 0 {
                break;
            }
        }

        let mut raw = [0u8; 3];
        self.bus
            .write_read(self.addr, &[BME280_REG_TEMP], &mut raw)
            .await
            .map_err(Error::I2c)?;

        let adc_t = (i32::from(raw[0]) << 12) | (i32::from(raw[1]) << 4) | (i32::from(raw[2]) >> 4);

        Ok(self.compensate(adc_t))
    }

    /// The integer compensation formula from the BME280 datasheet, section 4.2.3.
    /// It yields hundredths of a degree; scale to milli-degrees for [`TEMP`].
    fn compensate(&self, adc_t: i32) -> i32 {
        let dig_t1 = i32::from(self.dig_t1);
        let dig_t2 = i32::from(self.dig_t2);
        let dig_t3 = i32::from(self.dig_t3);

        let var1 = (((adc_t >> 3) - (dig_t1 << 1)) * dig_t2) >> 11;
        let var2 = (((((adc_t >> 4) - dig_t1) * ((adc_t >> 4) - dig_t1)) >> 12) * dig_t3) >> 14;
        let t_fine = var1 + var2;
        let centi_c = (t_fine * 5 + 128) >> 8;
        centi_c * 10
    }
}

// Web page
const PAGE: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta http-equiv="refresh" content="1" >
    <title>ADIN2111 with Rust</title>
  </head>
  <body>
    <p>Bristlemouth dev kit (ADIN2111)</p>
    <table><td>Temp Sensor BME280:</td><td> #00.00  &deg;C</td></table>
  </body>
</html>"#;
