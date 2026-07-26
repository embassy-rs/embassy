#![no_std]
#![no_main]

//! Example for the `embassy-nrf` **sQSPI** driver on the nRF54L15-DK.
//!
//! ## Wiring
//! | Flash pin | nRF54L15-DK |
//! |-----------|-------------|
//! | SCLK      | P2.01       |
//! | SI / IO0  | P2.02       |
//! | SO / IO1  | P2.04       |
//! | WP# / IO2 | P2.03       |
//! | HOLD#/IO3 | P2.00       |
//! | CS#       | P2.05       |
//!

use core::mem::MaybeUninit;

use defmt::{assert_eq, info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::sqspi::{self, Config};
use embassy_nrf::{bind_interrupts, peripherals};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    VPR00 => sqspi::InterruptHandler<peripherals::VPR>;
});

/// FLPR firmware (`self_boot == 0`, copied into RAM by the driver).
static FW: &[u8] = include_bytes!("sqspi_firmware.bin");

/// RAM region for the firmware code, working RAM, and register block. The
/// driver aligns the start up to 128 bytes, so size with a little slack.
static FW_RAM: ConstStaticCell<[MaybeUninit<u8>; 0x4000]> = ConstStaticCell::new([MaybeUninit::uninit(); 0x4000]);

const PAGE_SIZE: usize = 4096;

#[repr(C, align(4))]
struct AlignedBuf([u8; PAGE_SIZE]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("sqspi driver example: boot");
    // `embassy_nrf::init` stops and resets the FLPR by default
    // (`config::FlprReset::Reset`), so re-runs start from a clean state.
    let p = embassy_nrf::init(Default::default());

    let ram = FW_RAM.take();

    // Quad I/O (1-4-4) at 8 MHz, 8 MB capacity — same as the nRF52840 example.
    let mut config = Config::default();
    config.capacity = 8 * 1024 * 1024;

    let mut q = unwrap!(sqspi::Sqspi::new(
        p.VPR, Irqs, FW, ram, p.P2_01, p.P2_05, p.P2_02, p.P2_04, p.P2_03, p.P2_00, config,
    ));
    info!("driver ready");

    let mut id = [0; 3];
    unwrap!(q.custom_instruction(0x9F, &[], &mut id).await);
    info!("id: {}", id);

    let mut status = [0; 1];
    unwrap!(q.custom_instruction(0x05, &[], &mut status).await);
    info!("status: {=u8:#04x}", status[0]);

    // Quad I/O needs the QE bit (status bit 6). Writing 0x40 sets QE and clears
    // the block-protection bits, which would otherwise make program/erase fail.
    info!("enabling quad mode (QE)...");
    unwrap!(q.custom_instruction(0x01, &[0x40], &mut []).await);
    unwrap!(q.custom_instruction(0x05, &[], &mut status).await);
    info!("status now: {=u8:#04x} (QE={=u8})", status[0], (status[0] >> 6) & 1);

    let mut buf = AlignedBuf([0u8; PAGE_SIZE]);
    let pattern = |a: u32| (a ^ (a >> 8) ^ (a >> 16) ^ (a >> 24)) as u8;

    for i in 0..8 {
        info!("page {}: erasing...", i);
        unwrap!(q.erase(i * PAGE_SIZE as u32).await);

        for j in 0..PAGE_SIZE {
            buf.0[j] = pattern(j as u32 + i * PAGE_SIZE as u32);
        }
        info!("programming...");
        unwrap!(q.write(i * PAGE_SIZE as u32, &buf.0).await);
    }

    for i in 0..8 {
        info!("page {}: reading...", i);
        unwrap!(q.read(i * PAGE_SIZE as u32, &mut buf.0).await);

        info!("verifying...");
        for j in 0..PAGE_SIZE {
            assert_eq!(buf.0[j], pattern(j as u32 + i * PAGE_SIZE as u32));
        }
    }

    info!("done!");
}
