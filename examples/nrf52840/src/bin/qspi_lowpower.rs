#![no_std]
#![no_main]

use core::mem;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::qspi::Frequency;
use embassy_nrf::{bind_interrupts, peripherals, qspi};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// Workaround for alignment requirements.
// Nicer API will probably come in the future.
#[repr(C, align(4))]
struct AlignedBuf([u8; 64]);

bind_interrupts!(struct Irqs {
    QSPI => qspi::InterruptHandler<peripherals::QSPI>;
});

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let mut p = embassy_nrf::init(Default::default());

    loop {
        // Config for the MX25R64 present in the nRF52840 DK
        let mut config = qspi::Config::default();
        config.capacity = 8 * 1024 * 1024; // 8 MB
        config.frequency = Frequency::M32;
        config.read_opcode = qspi::ReadOpcode::READ4IO;
        config.write_opcode = qspi::WriteOpcode::PP4IO;
        config.write_page_size = qspi::WritePageSize::_256BYTES;
        config.deep_power_down = Some(qspi::DeepPowerDownConfig {
            enter_time: 3, // tDP = 30uS
            exit_time: 3,  // tRDP = 35uS
        });

        let mut q = qspi::Qspi::new(
            &mut p.QSPI,
            Irqs,
            &mut p.P0_19,
            &mut p.P0_17,
            &mut p.P0_20,
            &mut p.P0_21,
            &mut p.P0_22,
            &mut p.P0_23,
            config,
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

        let mut buf = AlignedBuf([0u8; 64]);

        info!("reading...");
        unwrap!(q.read(0, &mut buf.0).await);
        info!("read: {=[u8]:x}", buf.0);

        // Drop the QSPI instance. This disables the peripehral and deconfigures the pins.
        // This clears the borrow on the singletons, so they can now be used again.
        mem::drop(q);

        // Sleep for 1 second. The executor ensures the core sleeps with a WFE when it has nothing to do.
        // During this sleep, the nRF chip should only use ~3uA
        Timer::after_secs(1).await;
    }
}
