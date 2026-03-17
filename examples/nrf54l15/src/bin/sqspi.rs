#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, peripherals, sqspi};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// The firmware binary for the FLPR core (compiled RISC-V code).
// Replace this path with your actual sQSPI firmware binary.

bind_interrupts!(struct Irqs {
    VPR00 => sqspi::InterruptHandler<peripherals::SQSPI>;
});

// RAM buffer for firmware + execution RAM + virtual register interface.
// Total size 0x3D40 per the nRF54L15 porting guide.
static SQSPI_RAM: StaticCell<[u8; 0x3D40]> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut config = sqspi::Config::default();
    config.sck_freq_khz = 8000;
    config.spi_mode = sqspi::MODE_0;
    config.lines = sqspi::SpiLines::Quad1_1_4;

    let mut sqspi = unwrap!(sqspi::Sqspi::new(
        p.SQSPI,
        Irqs,
        SQSPI_FW,
        SQSPI_RAM.init([0; 0x3D40]),
        p.P1_01, // sck
        p.P1_02, // csn
        p.P1_03, // io0
        p.P1_04, // io1
        p.P1_05, // io2
        p.P1_06, // io3
        config,
    ));

    // Read 256 bytes from flash address 0x0000.
    let mut buf = [0u8; 256];
    unwrap!(sqspi.read(0x0000, &mut buf).await);
    info!("read: {:02x}", &buf[..16]);

    // Write 256 bytes to flash address 0x1000.
    unwrap!(sqspi.write(0x1000, &buf).await);
    info!("write done");

    // Erase a 4KB sector.
    unwrap!(sqspi.erase(0x2000).await);
    info!("erase done");

    // Custom instruction: read status register (opcode 0x05).
    let mut status = [0u8; 1];
    unwrap!(sqspi.custom_instruction(0x05, &[], &mut status).await);
    info!("status: {:02x}", status[0]);

    info!("sQSPI example (commented out — needs firmware binary)");
    loop {
        cortex_m::asm::wfe();
    }
}

// TODO: Copy paste from nordic
static SQSPI_FW: &[u8] = &[];
