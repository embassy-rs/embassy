#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{assert_eq, info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::qspi::Frequency;
use embassy_nrf::{bind_interrupts, peripherals, qspi};
use {defmt_rtt as _, panic_probe as _};

const PAGE_SIZE: usize = 4096;

// Workaround for alignment requirements.
// Nicer API will probably come in the future.
#[repr(C, align(4))]
struct AlignedBuf([u8; 4096]);

bind_interrupts!(struct Irqs {
    QSPI => qspi::InterruptHandler<peripherals::QSPI>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    // Config for the MX25R64 present in the nRF52840 DK
    let mut config = qspi::Config::default();
    config.capacity = 8 * 1024 * 1024; // 8 MB
    config.frequency = Frequency::M32;
    config.read_opcode = qspi::ReadOpcode::READ4IO;
    config.write_opcode = qspi::WriteOpcode::PP4IO;
    config.write_page_size = qspi::WritePageSize::_256BYTES;

    let mut q = qspi::Qspi::new(
        p.QSPI, Irqs, p.P0_19, p.P0_17, p.P0_20, p.P0_21, p.P0_22, p.P0_23, config,
    );

    let mut id = [1; 3];
    unwrap!(q.custom_instruction(0x9F, &[], &mut id).await);
    info!("id: {}", id);

    // Read status register
    let mut status = [4; 1];
    unwrap!(q.custom_instruction(0x05, &[], &mut status).await);

    info!("status: {:?}", status[0]);

    if status[0] & 0x40 == 0 {
        status[0] |= 0x40;

        unwrap!(q.custom_instruction(0x01, &status, &mut []).await);

        info!("enabled quad in status");
    }

    let mut buf = AlignedBuf([0u8; PAGE_SIZE]);

    let pattern = |a: u32| (a ^ (a >> 8) ^ (a >> 16) ^ (a >> 24)) as u8;

    for i in 0..8 {
        info!("page {:?}: erasing... ", i);
        unwrap!(q.erase(i * PAGE_SIZE as u32).await);

        for j in 0..PAGE_SIZE {
            buf.0[j] = pattern((j as u32 + i * PAGE_SIZE as u32) as u32);
        }

        info!("programming...");
        unwrap!(q.write(i * PAGE_SIZE as u32, &buf.0).await);
    }

    for i in 0..8 {
        info!("page {:?}: reading... ", i);
        unwrap!(q.read(i * PAGE_SIZE as u32, &mut buf.0).await);

        info!("verifying...");
        for j in 0..PAGE_SIZE {
            assert_eq!(buf.0[j], pattern((j as u32 + i * PAGE_SIZE as u32) as u32));
        }
    }

    info!("done!")
}
