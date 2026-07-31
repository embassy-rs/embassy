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
use embassy_futures::join::join5;
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

/// Records the host's `SET_CONFIGURATION` into a per-device flag.
struct ConfiguredHandler(&'static AtomicBool);

impl Handler for ConfiguredHandler {
    fn configured(&mut self, configured: bool) {
        if configured {
            self.0.store(true, Ordering::Relaxed);
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut hal_config = embassy_nxp::config::Config::default();
    // The point of this test: drive both controllers from PLL0 at 150 MHz
    // instead of the 96 MHz FRO every other dual-controller binary uses.
    hal_config.main_clock = MainClock::Pll0_150M;
    let p = embassy_nxp::init(hal_config);

    // High speed takes the dedicated USB1 SRAM.
    let hs_driver = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());

    // Full speed takes a region in main SRAM: only one controller can own the
    // dedicated USB1 SRAM, so running both at once forces the other onto a
    // caller-provided region. 4 KiB comfortably fits an FS CDC-ACM device. The
    // local lives in the main task's storage, which is a `static`, not on the
    // stack.
    let mut ep_mem = [0u8; 4096];
    let fs_mem = Memory::buffer(&mut ep_mem);
    let fs_driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, fs_mem);

    // --- High-speed device ---
    //
    // The manufacturer, product and serial strings below are load-bearing: the
    // Linux `/dev/serial/by-id/` symlink name is derived from them, and
    // `host/dual_echo.py` matches on `USB-HS_dual_pll0` / `USB-FS_dual_pll0` to
    // find the two CDC nodes. Changing them breaks the host peer.
    let mut hs_config = embassy_usb::Config::new(0xc0de, 0xcb03);
    hs_config.manufacturer = Some("Embassy");
    hs_config.product = Some("USB-HS dual pll0");
    hs_config.serial_number = Some("hs-dual");
    hs_config.max_power = 100;
    hs_config.max_packet_size_0 = 64;
    hs_config.max_speed = UsbDeviceSpeed::High;

    let mut hs_config_descriptor = [0; 256];
    let mut hs_bos_descriptor = [0; 256];
    let mut hs_control_buf = [0; 64];
    let mut hs_state = State::new();
    let mut hs_handler = ConfiguredHandler(&HS_CONFIGURED);

    let mut hs_builder = embassy_usb::Builder::new(
        hs_driver,
        hs_config,
        &mut hs_config_descriptor,
        &mut hs_bos_descriptor,
        &mut [], // no msos descriptors
        &mut hs_control_buf,
    );
    let mut hs_class = CdcAcmClass::new(&mut hs_builder, &mut hs_state, 512);
    hs_builder.handler(&mut hs_handler);
    let mut hs_usb = hs_builder.build();

    // --- Full-speed device ---
    let mut fs_config = embassy_usb::Config::new(0xc0de, 0xcb04);
    fs_config.manufacturer = Some("Embassy");
    fs_config.product = Some("USB-FS dual pll0");
    fs_config.serial_number = Some("fs-dual");
    fs_config.max_power = 100;
    fs_config.max_packet_size_0 = 64;

    let mut fs_config_descriptor = [0; 256];
    let mut fs_bos_descriptor = [0; 256];
    let mut fs_control_buf = [0; 64];
    let mut fs_state = State::new();
    let mut fs_handler = ConfiguredHandler(&FS_CONFIGURED);

    let mut fs_builder = embassy_usb::Builder::new(
        fs_driver,
        fs_config,
        &mut fs_config_descriptor,
        &mut fs_bos_descriptor,
        &mut [], // no msos descriptors
        &mut fs_control_buf,
    );
    let mut fs_class = CdcAcmClass::new(&mut fs_builder, &mut fs_state, 64);
    fs_builder.handler(&mut fs_handler);
    let mut fs_usb = fs_builder.build();

    let hs_echo = async {
        loop {
            hs_class.wait_connection().await;
            let _ = echo_hs(&mut hs_class).await;
        }
    };

    let fs_echo = async {
        loop {
            fs_class.wait_connection().await;
            let _ = echo_fs(&mut fs_class).await;
        }
    };

    // Both device stacks, both echo loops and the verdict task run concurrently
    // on one executor.
    join5(hs_usb.run(), fs_usb.run(), hs_echo, fs_echo, verdict()).await;
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

    // `bkpt` halts under a debugger, but keep the future's type honest.
    loop {
        Timer::after_millis(1000).await;
    }
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

/// An echo of exactly one max-packet-size packet needs a trailing zero-length
/// packet to end the transfer: without it the host keeps waiting for more and
/// never delivers the reply.
async fn echo_hs<'d>(class: &mut CdcAcmClass<'d, Driver<'d, peripherals::USBHSD>>) -> Result<(), Disconnected> {
    const MPS: usize = 512;
    let mut buf = [0; MPS];
    loop {
        let n = class.read_packet(&mut buf).await?;
        class.write_packet(&buf[..n]).await?;
        if n == MPS {
            class.write_packet(&[]).await?;
        }
        HS_ECHOED.fetch_add(n as u32, Ordering::Relaxed);
    }
}

/// See [`echo_hs`] for the trailing zero-length packet.
async fn echo_fs<'d>(class: &mut CdcAcmClass<'d, Driver<'d, peripherals::USB0>>) -> Result<(), Disconnected> {
    const MPS: usize = 64;
    let mut buf = [0; MPS];
    loop {
        let n = class.read_packet(&mut buf).await?;
        class.write_packet(&buf[..n]).await?;
        if n == MPS {
            class.write_packet(&[]).await?;
        }
        FS_ECHOED.fetch_add(n as u32, Ordering::Relaxed);
    }
}
