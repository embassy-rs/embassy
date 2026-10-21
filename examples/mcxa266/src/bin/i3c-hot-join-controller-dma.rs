//! Two-board I3C Hot-Join loop stress test: MCXA2xx controller.
//!
//! Pair with `i3c-hot-join-target-dma.rs`.

#![no_std]
#![no_main]

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::clocks::periph_helpers::{Div4, I3cClockSel};
use embassy_mcxa::config::Config;
use embassy_mcxa::i3c::controller::{self, BusType, DeviceInfo, I3c, IbiEvent, IbiSlot, InterruptHandler, Payload};
use embassy_mcxa::peripherals::I3C0;
use embassy_time::{Duration, Timer, with_timeout};
use panic_probe as _;

const CYCLES: u32 = 10_000;
const TARGET_DYNAMIC_ADDR: u8 = 0x30;
const TARGET_PART_NUMBER: u32 = 0x484a_0001;
const TRANSFER_LEN: usize = 32;
const WRITE_PATTERN: u8 = 0xaa;
const READ_PATTERN: u8 = 0x55;
const READ_LEN: usize = 16;
const HJ_TIMEOUT: Duration = Duration::from_secs(5);

bind_interrupts! {
    struct Irqs {
        I3C0 => InterruptHandler<I3C0>;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    if let Some(firc) = config.clock_cfg.firc.as_mut() {
        firc.fro_hf_div = Some(Div8::from_divisor(2).unwrap());
    }
    let p = hal::init(config);

    {
        use embassy_mcxa::i3c::PurPin;
        PurPin::<I3C0>::mux(&*p.P0_2);
    }

    let mut cfg = controller::Config::default();
    cfg.clock_config.source = I3cClockSel::FroHfDiv;
    cfg.clock_config.div = Div4::from_divisor(1).unwrap();
    cfg.open_drain_freq = 1_000_000;
    cfg.push_pull_freq = 2_000_000;
    let mut i3c = I3c::new_async_with_dma(p.I3C0, p.P1_9, p.P1_8, p.DMA0_CH0, p.DMA0_CH1, Irqs, cfg).unwrap();

    info!("[ctrl] starting {} Hot-Join cycles", CYCLES);

    let write_buf = [WRITE_PATTERN; TRANSFER_LEN];
    let mut rx = [0u8; TRANSFER_LEN];
    let mut ibi_payload = [0u8; 8];

    i3c.register_ibi(IbiSlot::Slot0, TARGET_DYNAMIC_ADDR, Payload::Yes)
        .unwrap();

    let mut cycle = 0u32;
    while cycle < CYCLES {
        match with_timeout(HJ_TIMEOUT, i3c.async_wait_for_ibi(&mut ibi_payload)).await {
            Ok(Ok(IbiEvent::HotJoin)) => info!("[ctrl] cycle {} Hot-Join received", cycle),
            Ok(Ok(event)) => panic!("unexpected event during Hot-Join: {:?}", event),
            Ok(Err(error)) => panic!("Hot-Join error: {:?}", error),
            Err(_) => panic!("Hot-Join timeout"),
        }

        let mut devices = [DeviceInfo::new(); 8];
        i3c.daa(&mut devices, TARGET_DYNAMIC_ADDR).unwrap();
        let address = devices
            .iter()
            .find(|device| device.partno == TARGET_PART_NUMBER)
            .map(|device| device.addr)
            .unwrap_or_else(|| panic!("Hot-Join target not found after DAA"));

        i3c.async_write(address, &write_buf, BusType::I3cSdr).await.unwrap();
        match i3c.async_wait_for_ibi(&mut ibi_payload).await.unwrap() {
            IbiEvent::Ibi {
                address: ibi_address, ..
            } if ibi_address == address => {}
            event => panic!("unexpected event after write: {:?}", event),
        }

        let n = i3c.async_read(address, &mut rx, BusType::I3cSdr).await.unwrap();
        if n != READ_LEN || rx[..n].iter().any(|&byte| byte != READ_PATTERN) {
            error!("[ctrl] cycle {} read mismatch n={} data={:?}", cycle, n, &rx[..n]);
            panic!("verify read mismatch");
        }

        cycle = cycle.wrapping_add(1);
        if cycle % 1000 == 0 {
            info!("[ctrl] cycle {} OK", cycle);
        }

        i3c.blocking_reset_daa().unwrap();
    }

    info!("[ctrl] {} Hot-Join cycles complete", CYCLES);
    loop {
        Timer::after_millis(5).await;
    }
}
