//! Both LPC55 USB controllers running at the same time.
//!
//! Brings up USB1 (high-speed, connector **P9**) on the dedicated USB1 SRAM and
//! USB0 (full-speed, connector **P10**) on a buffer in main SRAM, then runs two
//! independent `UsbDevice`s concurrently. Each is a CDC-ACM echo port with its
//! own product string and PID, so the host sees two distinct devices:
//!
//! - `c0de:cafe` "USB-HS dual echo" at 480 Mbps
//! - `c0de:caff` "USB-FS dual echo" at 12 Mbps
//!
//! This is what the two-region [`Memory`] design buys: only one controller can
//! own the dedicated USB1 SRAM, so simultaneous operation requires the other to
//! take a caller-provided region.

#![no_std]
#![no_main]

use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::{join, join3};
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_usb::UsbDeviceSpeed;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
    USB1 => InterruptHandler<peripherals::USBHSD>;
});

/// Endpoint memory for the full-speed controller, in main SRAM.
///
/// 4 KiB comfortably fits an FS CDC-ACM device (command list, SETUP buffer,
/// EP0, one interrupt endpoint and two double-buffered 512-byte bulk slots ~
/// 2.5 KiB). `Memory::buffer` requires at least 512 bytes and 256-byte
/// alignment.
#[repr(C, align(256))]
struct EpMem([u8; 4096]);
static mut EP_MEM: EpMem = EpMem([0; 4096]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nxp::config::Config::default();
    // Required by the high-speed controller; the full-speed one clocks itself.
    config.main_clock = MainClock::FroHf96;
    let p = embassy_nxp::init(config);

    info!("Initialization complete");

    // High speed takes the dedicated USB1 SRAM.
    let hs_driver = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());

    // Full speed takes a region in main SRAM. Edition 2024 forbids references
    // to `static mut`, so go through a raw pointer.
    let fs_mem = Memory::buffer(unsafe { &mut (*(&raw mut EP_MEM)).0 });
    let fs_driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, fs_mem);

    // --- High-speed device ---
    let mut hs_config = embassy_usb::Config::new(0xc0de, 0xcafe);
    hs_config.manufacturer = Some("Embassy");
    hs_config.product = Some("USB-HS dual echo");
    hs_config.serial_number = Some("12345678");
    hs_config.max_power = 100;
    hs_config.max_packet_size_0 = 64;
    hs_config.max_speed = UsbDeviceSpeed::High;

    let mut hs_config_descriptor = [0; 256];
    let mut hs_bos_descriptor = [0; 256];
    let mut hs_control_buf = [0; 64];
    let mut hs_state = State::new();

    let mut hs_builder = embassy_usb::Builder::new(
        hs_driver,
        hs_config,
        &mut hs_config_descriptor,
        &mut hs_bos_descriptor,
        &mut [], // no msos descriptors
        &mut hs_control_buf,
    );
    let mut hs_class = CdcAcmClass::new(&mut hs_builder, &mut hs_state, 512);
    let mut hs_usb = hs_builder.build();

    // --- Full-speed device ---
    let mut fs_config = embassy_usb::Config::new(0xc0de, 0xcaff);
    fs_config.manufacturer = Some("Embassy");
    fs_config.product = Some("USB-FS dual echo");
    fs_config.serial_number = Some("87654321");
    fs_config.max_power = 100;
    fs_config.max_packet_size_0 = 64;

    let mut fs_config_descriptor = [0; 256];
    let mut fs_bos_descriptor = [0; 256];
    let mut fs_control_buf = [0; 64];
    let mut fs_state = State::new();

    let mut fs_builder = embassy_usb::Builder::new(
        fs_driver,
        fs_config,
        &mut fs_config_descriptor,
        &mut fs_bos_descriptor,
        &mut [], // no msos descriptors
        &mut fs_control_buf,
    );
    let mut fs_class = CdcAcmClass::new(&mut fs_builder, &mut fs_state, 64);
    let mut fs_usb = fs_builder.build();

    let hs_echo = async {
        loop {
            hs_class.wait_connection().await;
            info!("HS connected");
            let _ = echo_hs(&mut hs_class).await;
            info!("HS disconnected");
        }
    };

    let fs_echo = async {
        loop {
            fs_class.wait_connection().await;
            info!("FS connected");
            let _ = echo_fs(&mut fs_class).await;
            info!("FS disconnected");
        }
    };

    // Both device stacks and both echo loops run concurrently on one executor.
    join(join3(hs_usb.run(), fs_usb.run(), hs_echo), fs_echo).await;
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
    }
}
