//! Continuous LPUART BBQueue RX HIL stress test for MCXA577.
//!
//! Wire LPUART1 TX (P1_9) to LPUART5 RX (P1_16).

#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a577");

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::clocks::periph_helpers::LpuartClockSel;
use embassy_mcxa::dma::DmaChannel;
use embassy_mcxa::lpuart::{BbqConfig, BbqError, BbqHalfParts, BbqRxMode, Dma, LpuartBbqRx, LpuartTx};
use embassy_mcxa::{bind_interrupts, lpuart};
use embassy_time::{Duration, Instant, Timer, WithTimeout as _};
use panic_probe as _;
use static_cell::ConstStaticCell;

const BAUD_RATE: u32 = 3_000_000;
const DMA_HALF_SIZE: usize = 64;
const RX_BUFFER_SIZE: usize = 4096;
const TX_CHUNK_SIZE: usize = 1024;
const RX_CHUNK_SIZE: usize = 257;

const SUSTAINED_BYTES: usize = 1024 * 1024;
const IDLE_BURST_SIZE: usize = 61;
const IDLE_BURSTS: usize = 4096;
const IDLE_GAP_US: u64 = 20;
const OVERRUN_BYTES: usize = 16 * 1024;
const SLOW_CONSUMER_DELAY_MS: u64 = 20;
const RECOVERY_BYTES: usize = 16 * 1024;

const IO_TIMEOUT: Duration = Duration::from_secs(2);
const PHASE_TIMEOUT: Duration = Duration::from_secs(15);
const DRAIN_TIMEOUT: Duration = Duration::from_millis(2);

const SUSTAINED_SEQUENCE_START: u32 = 0x1111_1111;
const IDLE_SEQUENCE_START: u32 = 0x2222_2222;
const OVERRUN_SEQUENCE_START: u32 = 0x3333_3333;
const RECOVERY_SEQUENCE_START: u32 = 0x4444_4444;

bind_interrupts!(struct Irqs {
    LPUART5 => lpuart::BbqInterruptHandler<hal::peripherals::LPUART5>;
});

static RX_BUFFER: ConstStaticCell<[u8; RX_BUFFER_SIZE]> = ConstStaticCell::new([0; RX_BUFFER_SIZE]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = hal::config::Config::default();
    config.clock_cfg.sirc.fro_12m_enabled = true;
    config.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    config.clock_cfg.firc.as_mut().unwrap().fro_hf_div = Some(Div8::no_div());
    let p = hal::init(config);

    let tx_config = lpuart::Config {
        source: LpuartClockSel::FroHfDiv,
        baudrate_bps: BAUD_RATE,
        tx_fifo_watermark: 0,
        ..Default::default()
    };
    let mut tx = LpuartTx::new_async_with_dma(p.LPUART1, p.P1_9, p.DMA0_CH0, tx_config).unwrap();

    let mut rx_config = BbqConfig::default();
    rx_config.source = LpuartClockSel::FroHfDiv;
    rx_config.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    rx_config.baudrate_bps = BAUD_RATE;

    let rx_parts = BbqHalfParts::new_rx_half(p.LPUART5, Irqs, p.P1_16, RX_BUFFER.take(), DmaChannel::new(p.DMA0_CH5));
    let (mode_name, rx_mode) = if cfg!(feature = "max-frame-baseline") {
        ("max-frame-baseline", BbqRxMode::MaxFrame { size: DMA_HALF_SIZE })
    } else {
        (
            "continuous",
            BbqRxMode::Continuous {
                half_size: DMA_HALF_SIZE,
            },
        )
    };
    let mut rx = LpuartBbqRx::new(rx_parts, rx_config, rx_mode).unwrap();

    Timer::after_millis(10).await;

    info!(
        "BBQueue RX HIL start: mode={=str}, baud={=u32}, transfer_size={=usize}, rx_buffer={=usize}",
        mode_name, BAUD_RATE, DMA_HALF_SIZE, RX_BUFFER_SIZE
    );

    // Run the sustained integrity phase
    run_integrity_phase(
        "sustained",
        &mut tx,
        &mut rx,
        SUSTAINED_BYTES,
        TX_CHUNK_SIZE,
        0,
        SUSTAINED_SEQUENCE_START,
    )
    .await;

    // Run the idle-boundary integrity phase
    run_integrity_phase(
        "idle-boundary",
        &mut tx,
        &mut rx,
        IDLE_BURST_SIZE * IDLE_BURSTS,
        IDLE_BURST_SIZE,
        IDLE_GAP_US,
        IDLE_SEQUENCE_START,
    )
    .await;

    // Run the continuous-mode overrun and recovery phase.
    if !cfg!(feature = "max-frame-baseline") {
        run_overrun_recovery(&mut tx, &mut rx).await;
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

async fn run_integrity_phase<'d>(
    name: &'static str,
    tx: &mut LpuartTx<'d, Dma<'d>>,
    rx: &mut LpuartBbqRx,
    total: usize,
    burst_size: usize,
    idle_gap_us: u64,
    sequence_start: u32,
) {
    let started = Instant::now();
    let transfer = join(
        send_pattern(tx, total, burst_size, idle_gap_us, sequence_start),
        receive_pattern(rx, total, sequence_start),
    );

    let (sent, received) = match transfer.with_timeout(PHASE_TIMEOUT).await {
        Ok(result) => result,
        Err(_) => panic!("BBQueue RX integrity phase timed out"),
    };
    let elapsed_us = started.elapsed().as_micros().max(1);
    let throughput = (received as u64 * 1_000_000) / elapsed_us;

    assert_eq!(sent, total);
    assert_eq!(received, total);
    info!(
        "measurement phase={=str} tx_bytes={=usize} rx_bytes={=usize} sequence_errors=0 elapsed_us={=u64} throughput_Bps={=u64}",
        name, sent, received, elapsed_us, throughput
    );
}

async fn send_pattern<'d>(
    tx: &mut LpuartTx<'d, Dma<'d>>,
    total: usize,
    burst_size: usize,
    idle_gap_us: u64,
    sequence_start: u32,
) -> usize {
    assert!(burst_size > 0 && burst_size <= TX_CHUNK_SIZE);

    let mut buffer = [0u8; TX_CHUNK_SIZE];
    let mut sent = 0;
    while sent < total {
        let len = burst_size.min(total - sent);
        for (index, byte) in buffer[..len].iter_mut().enumerate() {
            *byte = sequence_byte(sent + index, sequence_start);
        }

        let written = tx.write(&buffer[..len]).await.unwrap();
        assert_eq!(written, len);
        sent += written;

        if idle_gap_us != 0 {
            tx.blocking_flush().unwrap();
            Timer::after_micros(idle_gap_us).await;
        }
    }

    tx.blocking_flush().unwrap();
    sent
}

async fn receive_pattern(rx: &mut LpuartBbqRx, total: usize, sequence_start: u32) -> usize {
    let mut buffer = [0u8; RX_CHUNK_SIZE];
    let mut received = 0;

    while received < total {
        let request = buffer.len().min(total - received);
        let read = match rx.read(&mut buffer[..request]).with_timeout(IO_TIMEOUT).await {
            Ok(Ok(read)) => read,
            Ok(Err(_)) => panic!("unexpected BBQueue RX error"),
            Err(_) => panic!("BBQueue RX read timed out"),
        };
        assert!(read != 0);

        for (index, actual) in buffer[..read].iter().copied().enumerate() {
            let offset = received + index;
            let expected = sequence_byte(offset, sequence_start);
            if actual != expected {
                error!(
                    "sequence mismatch offset={=usize} expected={=u8} actual={=u8}",
                    offset, expected, actual
                );
                panic!("BBQueue RX sequence mismatch");
            }
        }
        received += read;
    }

    received
}

async fn run_overrun_recovery<'d>(tx: &mut LpuartTx<'d, Dma<'d>>, rx: &mut LpuartBbqRx) {
    let started = Instant::now();
    let sender = send_pattern(tx, OVERRUN_BYTES, TX_CHUNK_SIZE, 0, OVERRUN_SEQUENCE_START);
    let slow_consumer = async {
        Timer::after_millis(SLOW_CONSUMER_DELAY_MS).await;
        let mut buffer = [0u8; RX_CHUNK_SIZE];
        match rx.read(&mut buffer).await {
            Err(BbqError::Overrun) => {}
            _ => panic!("slow consumer did not report BbqError::Overrun"),
        }
    };

    let (sent, ()) = match join(sender, slow_consumer).with_timeout(PHASE_TIMEOUT).await {
        Ok(result) => result,
        Err(_) => panic!("continuous RX overrun phase timed out"),
    };
    let observed_us = started.elapsed().as_micros();

    assert!(rx.clear_overrun());
    let drained = drain_preserved_queue(rx).await;
    info!(
        "measurement phase=slow-consumer tx_bytes={=usize} consumer_delay_ms={=u64} overrun_reported=true observed_us={=u64} preserved_bytes={=usize}",
        sent, SLOW_CONSUMER_DELAY_MS, observed_us, drained
    );

    run_integrity_phase(
        "overrun-recovery",
        tx,
        rx,
        RECOVERY_BYTES,
        TX_CHUNK_SIZE,
        0,
        RECOVERY_SEQUENCE_START,
    )
    .await;
}

async fn drain_preserved_queue(rx: &mut LpuartBbqRx) -> usize {
    let mut buffer = [0u8; RX_CHUNK_SIZE];
    let mut drained = 0;

    loop {
        match rx.read(&mut buffer).with_timeout(DRAIN_TIMEOUT).await {
            Ok(Ok(read)) => {
                assert!(read != 0);
                drained += read;
            }
            Ok(Err(_)) => panic!("continuous RX failed while draining after recovery"),
            Err(_) => return drained,
        }
    }
}

fn sequence_byte(offset: usize, sequence_start: u32) -> u8 {
    let sequence = sequence_start.wrapping_add((offset / 4) as u32);
    sequence.to_le_bytes()[offset % 4]
}
