//! Both LPC55 USB controllers running concurrently off `MainClock::Pll0_150M`.
//!
//! Requires host cables on **both** P9 (high speed) and P10 (full speed).
//!
//! The other dual-controller code in this repo runs on `MainClock::FroHf96`, so
//! the PLL0 main-clock path was never exercised with USB live. PLL0 at 150 MHz
//! satisfies the USB-HS >= 96 MHz system-clock requirement while changing the
//! divider arithmetic behind both controllers' 48/480 MHz clocks, which is what
//! this test covers: full enumeration of both devices, an asserted high-speed
//! negotiation on USB1, and a bulk echo round-trip on each port.
//!
//! Host peer: `tests/lpc55/host/dual_echo.py`, started once the ready banner
//! `dual pll0 ready` appears.

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::{join, join3};
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, pac, peripherals};
use embassy_time::{Duration, Instant, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Handler, UsbDeviceSpeed};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
    USB1 => InterruptHandler<peripherals::USBHSD>;
});

/// Bytes the host echoes per port before the test is considered passed. The
/// host script sends 8 x 512 bytes, so this is reached exactly once per port.
const ECHO_TARGET: u32 = 4096;

/// Budget for both echo streams, measured from the ready banner.
const ECHO_TIMEOUT: Duration = Duration::from_secs(30);

static HS_CONFIGURED: AtomicBool = AtomicBool::new(false);
static FS_CONFIGURED: AtomicBool = AtomicBool::new(false);
static HS_ECHOED: AtomicU32 = AtomicU32::new(0);
static FS_ECHOED: AtomicU32 = AtomicU32::new(0);

struct ConfiguredHandler(&'static AtomicBool);

impl Handler for ConfiguredHandler {
    fn configured(&mut self, configured: bool) {
        if configured {
            self.0.store(true, Ordering::Relaxed);
        }
    }
}

struct Resources<'d> {
    config_descriptor: [u8; 256],
    bos_descriptor: [u8; 256],
    control_buf: [u8; 64],
    state: State<'d>,
    handler: ConfiguredHandler,
    echoed: &'static AtomicU32,
}

impl<'d> Resources<'d> {
    const fn new(configured: &'static AtomicBool, echoed: &'static AtomicU32) -> Self {
        Self {
            config_descriptor: [0; 256],
            bos_descriptor: [0; 256],
            control_buf: [0; 64],
            state: State::new(),
            handler: ConfiguredHandler(configured),
            echoed,
        }
    }
}

struct Params {
    pid: u16,
    product: &'static str,
    serial: &'static str,
    max_speed: UsbDeviceSpeed,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut hal_config = embassy_nxp::config::Config::default();
    // The point of this test: drive both controllers from PLL0 at 150 MHz
    // instead of the 96 MHz FRO every other dual-controller binary uses.
    hal_config.main_clock = MainClock::Pll0_150M;
    let p = embassy_nxp::init(hal_config);
    let revision = pac::SYSCON.dieid().read().rev_id();
    let pll_locked = pac::SYSCON.pll0stat().read().lock();
    let main_clock_source = pac::SYSCON.mainclkselb().read().sel();
    defmt::assert_eq!(revision, 1, "PLL0 test requires LPC55 revision 1B");
    defmt::assert!(pll_locked, "PLL0 lost lock after clock initialization");
    defmt::assert!(
        main_clock_source == pac::syscon::vals::MainclkselbSel::Enum0x1,
        "main clock did not select PLL0"
    );
    info!("REV_ID={} PLL0STAT.LOCK={} MAINCLKSELB=PLL0", revision, pll_locked);

    // USBHSD can write its command list only in the dedicated USB1 SRAM.
    // USB0 uses main SRAM so both controllers can run at the same time.
    let hs_driver = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());
    let mut ep_mem = [0u8; 4096];
    let fs_driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, Memory::buffer(&mut ep_mem));
    let mut hs_resources = Resources::new(&HS_CONFIGURED, &HS_ECHOED);
    let mut fs_resources = Resources::new(&FS_CONFIGURED, &FS_ECHOED);

    // These strings are load-bearing: Linux derives `/dev/serial/by-id/` names
    // from them, and `host/dual_echo.py` uses the product names to find both ports.
    let hs = device::<_, 512>(
        hs_driver,
        &mut hs_resources,
        Params {
            pid: 0xcb03,
            product: "USB-HS dual pll0",
            serial: "hs-dual",
            max_speed: UsbDeviceSpeed::High,
        },
    );
    let fs = device::<_, 64>(
        fs_driver,
        &mut fs_resources,
        Params {
            pid: 0xcb04,
            product: "USB-FS dual pll0",
            serial: "fs-dual",
            max_speed: UsbDeviceSpeed::Full,
        },
    );

    join3(hs, fs, verdict()).await;
}

async fn device<'d, D: embassy_usb::driver::Driver<'d>, const MPS: usize>(
    driver: D,
    resources: &'d mut Resources<'d>,
    params: Params,
) {
    let mut config = embassy_usb::Config::new(0xc0de, params.pid);
    config.manufacturer = Some("Embassy");
    config.product = Some(params.product);
    config.serial_number = Some(params.serial);
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = params.max_speed;

    let echoed = resources.echoed;
    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        &mut resources.config_descriptor,
        &mut resources.bos_descriptor,
        &mut [],
        &mut resources.control_buf,
    );
    let mut class = CdcAcmClass::new(&mut builder, &mut resources.state, MPS as u16);
    builder.handler(&mut resources.handler);
    let mut usb = builder.build();

    let echo = async {
        loop {
            class.wait_connection().await;
            let _ = echo::<D, MPS>(&mut class, echoed).await;
        }
    };
    join(usb.run(), echo).await;
}

/// Waits for both devices to enumerate, checks what only the device can see,
/// prints the host's ready banner and then waits for both echo streams.
async fn verdict() -> ! {
    while !(HS_CONFIGURED.load(Ordering::Relaxed) && FS_CONFIGURED.load(Ordering::Relaxed)) {
        Timer::after_millis(50).await;
    }

    // 10b = high speed, per the DEVCMDSTAT.SPEED field docs. A silent fallback
    // to full speed on P9 would otherwise pass unnoticed.
    defmt::assert_eq!(
        pac::USBHSD.devcmdstat().read().speed(),
        0b10,
        "USB1 did not negotiate high speed under PLL0"
    );
    defmt::assert!(
        pac::USB0.devcmdstat().read().dev_addr() != 0,
        "USB0 was configured but DEVCMDSTAT.DEV_ADDR is still 0"
    );

    info!("dual pll0 ready");

    let deadline = Instant::now() + ECHO_TIMEOUT;
    loop {
        let hs = HS_ECHOED.load(Ordering::Relaxed);
        let fs = FS_ECHOED.load(Ordering::Relaxed);
        if hs >= ECHO_TARGET && fs >= ECHO_TARGET {
            break;
        }
        if Instant::now() >= deadline {
            panic!(
                "echo did not complete within 30 s: hs={} bytes, fs={} bytes (target {} each)",
                hs, fs, ECHO_TARGET
            );
        }
        Timer::after_millis(50).await;
    }

    info!("Test OK");
    cortex_m::asm::bkpt();

    loop {
        Timer::after_millis(1000).await;
    }
}

struct Disconnected;

impl From<EndpointError> for Disconnected {
    fn from(error: EndpointError) -> Self {
        match error {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected,
        }
    }
}

/// A full packet needs a trailing zero-length packet to terminate the transfer.
async fn echo<'d, D: embassy_usb::driver::Driver<'d>, const MPS: usize>(
    class: &mut CdcAcmClass<'d, D>,
    echoed: &AtomicU32,
) -> Result<(), Disconnected> {
    let mut buf = [0; MPS];
    loop {
        let n = class.read_packet(&mut buf).await?;
        class.write_packet(&buf[..n]).await?;
        if n == MPS {
            class.write_packet(&[]).await?;
        }
        echoed.fetch_add(n as u32, Ordering::Relaxed);
    }
}
