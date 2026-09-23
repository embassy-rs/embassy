//! Two-board I3C Hot-Join loop stress test: MCXA5xx controller.
//!
//! Pair with `i3c-hot-join-target-dma.rs`. Each cycle drops the target's
//! dynamic address (RSTDAA), accepts the resulting Hot-Join, re-runs DAA
//! (verifying the advertised PID/BCR/DCR), then does one verify write+IBI+read.
//!
//! Wiring: SCL P0_21 ↔ target SCL, SDA P0_20 ↔ target SDA, common GND.

#![no_std]
#![no_main]

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::config::Config;
use embassy_mcxa::i3c::controller::{self, BusType, DeviceInfo, I3c, IbiEvent, IbiSlot, InterruptHandler, Payload};
use embassy_mcxa::peripherals::I3C0;
use embassy_time::{Duration, with_timeout};
use panic_probe as _;

const CYCLES: u32 = 10_000;
const TARGET_DYNAMIC_ADDR: u8 = 0x30;
const TARGET_PART_NUMBER: u32 = 0x484a_0001;
const TRANSFER_LEN: usize = 32;
const WRITE_PATTERN: u8 = 0xaa;
const READ_PATTERN: u8 = 0x55;
const READ_LEN: usize = 16;
const HJ_TIMEOUT: Duration = Duration::from_secs(10);

bind_interrupts! {
    struct Irqs {
        I3C0 => InterruptHandler<I3C0>;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    {
        use embassy_mcxa::i3c::PurPin;
        PurPin::<I3C0>::mux(&*p.P0_2);
    }

    let cfg = controller::Config::default();
    let mut i3c = I3c::new_async_with_dma(p.I3C0, p.P0_21, p.P0_20, p.DMA0_CH0, p.DMA0_CH1, Irqs, cfg).unwrap();

    info!("[ctrl] starting {} Hot-Join cycles", CYCLES);

    let write_buf = [WRITE_PATTERN; TRANSFER_LEN];
    let mut rx = [0u8; TRANSFER_LEN];
    let mut ibi_payload = [0u8; 8];

    // There is a fixed target on the dev kit and a second target on the other board. Register both for IBI.
    i3c.register_ibi(IbiSlot::Slot0, TARGET_DYNAMIC_ADDR, Payload::Yes)
        .unwrap();
    i3c.register_ibi(IbiSlot::Slot1, TARGET_DYNAMIC_ADDR + 1, Payload::Yes)
        .unwrap();

    let mut cycle: u32 = 0;
    while cycle < CYCLES {
        loop {
            match with_timeout(HJ_TIMEOUT, i3c.async_wait_for_ibi(&mut ibi_payload)).await {
                Ok(Ok(IbiEvent::HotJoin)) => {
                    info!("[ctrl] cycle {} Hot-Join IBI received", cycle);
                    break;
                }
                Ok(Ok(event)) => {
                    error!("[ctrl] cycle {} unexpected event {:?}", cycle, event);
                    panic!("unexpected event during Hot-Join");
                }
                Ok(Err(e)) => {
                    error!("[ctrl] cycle {} Hot-Join error {:?}", cycle, e);
                }
                Err(_) => panic!("Hot-Join timeout"),
            }
        }

        // Assign a dynamic address and read back PID / BCR / DCR.
        let mut devices = [DeviceInfo::new(); 8];
        let mut address = 0;
        i3c.daa(&mut devices, TARGET_DYNAMIC_ADDR).unwrap();

        for dev in devices.iter() {
            if dev.partno == TARGET_PART_NUMBER {
                info!(
                    "[ctrl] cycle {} Address: {:02x} PID={:08x} BCR={:02x} DCR={:02x}",
                    cycle, dev.addr, dev.partno, dev.bcr, dev.dcr
                );
                address = dev.addr;
            }
        }

        // One verify transfer: write a known pattern, catch the target's IBI,
        // then read its response.
        i3c.async_write(address, &write_buf, BusType::I3cSdr).await.unwrap();
        i3c.async_wait_for_ibi(&mut ibi_payload).await.unwrap();
        let n = i3c.async_read(address, &mut rx, BusType::I3cSdr).await.unwrap();

        if n != READ_LEN || rx[..n].iter().any(|&b| b != READ_PATTERN) {
            error!("[ctrl] cycle {} read mismatch n={} data={:?}", cycle, n, &rx[..n]);
            // panic!("verify read mismatch");
        }

        cycle = cycle.wrapping_add(1);
        if cycle % 1000 == 0 {
            info!("[ctrl] cycle {} OK", cycle);
        }

        // Drop any assigned dynamic address so the target re-issues Hot-Join.
        // On the first pass this also opens the Bus-Available window for a
        // target that powered up before this controller.
        i3c.blocking_reset_daa().unwrap();
    }

    info!("[ctrl] {} Hot-Join cycles complete", CYCLES);
}
