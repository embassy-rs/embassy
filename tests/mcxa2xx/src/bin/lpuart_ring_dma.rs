//! Bounded UART DMA receive regression on MCXA266, using internal loopback.
//!
//! No jumper is needed. This intentionally pauses the consumer and delays DMA
//! interrupt handling; hardware must stop before reusing unread storage.

#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");
teleprobe_meta::timeout!(30);

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::Error;
use embassy_mcxa::lpuart::Lpuart;
use embassy_time::{Duration, Timer, WithTimeout as _};
use panic_probe as _;
use static_cell::ConstStaticCell;

const CAPACITY: usize = 64;
const TIMEOUT: Duration = Duration::from_secs(2);
static RX: ConstStaticCell<[u8; CAPACITY]> = ConstStaticCell::new([0xa5; CAPACITY]);
static RX_ONE: ConstStaticCell<[u8; 1]> = ConstStaticCell::new([0; 1]);
static RX_ODD: ConstStaticCell<[u8; 3]> = ConstStaticCell::new([0; 3]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = hal::config::Config::default();
    config.clock_cfg.sirc.fro_12m_enabled = true;
    config.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(config);
    let config = hal::lpuart::Config {
        baudrate_bps: 115_200,
        rx_fifo_watermark: 0,
        tx_fifo_watermark: 0,
        ..Default::default()
    };
    let uart = Lpuart::new_async_with_dma(p.LPUART2, p.P2_2, p.P2_3, p.DMA0_CH0, p.DMA0_CH1, config).unwrap();
    let (mut tx, mut rx) = uart.split();

    // The initialized UART is exclusively owned here and no DMA is active.
    // A266 RM: LOOPS=1/RSRC=0 routes TX internally into RX with both enabled.
    hal::pac::LPUART2.ctrl().modify(|w| {
        w.set_loops(true);
        w.set_rsrc(false);
    });

    let buffer = RX.take();
    let base = buffer.as_ptr() as u32;
    // SAFETY: static, exclusive DMA-accessible SRAM; the receiver is never
    // forgotten and is dropped before its storage/peripheral can be reused.
    let mut ring = unsafe { rx.into_ring_dma_rx(buffer) }.unwrap();
    let tcd = hal::pac::EDMA_TCD0.tcd(1);
    let nbytes = tcd.tcd_nbytes_mloffno().read().nbytes();
    defmt::info!("ring DMA NBYTES={}, expected=1", nbytes);
    assert_eq!(nbytes, 1, "DMA-14: receive must be paced one byte per request");
    assert!(!tcd.tcd_csr().read().start());
    assert!(tcd.tcd_csr().read().dreq(), "hardware must stop at capacity");
    Timer::after_millis(5).await;
    assert_eq!(tcd.tcd_daddr().read().daddr(), base, "DMA ran without received data");
    defmt::info!("PASS pacing: no unsolicited UART reads");

    let mut sent = [0u8; CAPACITY];
    let mut received = [0u8; CAPACITY];
    for cycle in 0..16u8 {
        for (i, byte) in sent.iter_mut().enumerate() {
            *byte = cycle.wrapping_mul(17).wrapping_add(i as u8);
        }
        // Deliberately delay ALL maskable interrupts during a complete buffer.
        // Correctness cannot depend on counting every wrap interrupt in time.
        critical_section::with(|_| {
            tx.blocking_write(&sent).unwrap();
            tx.blocking_flush().unwrap();
        });
        Timer::after_millis(1).await;
        assert!(!tcd.ch_csr().read().erq());
        assert_eq!(tcd.tcd_daddr().read().daddr(), base + CAPACITY as u32);
        let mut count = 0;
        while count < CAPACITY {
            let end = (count + 7).min(CAPACITY);
            let n = ring
                .read(&mut received[count..end])
                .with_timeout(TIMEOUT)
                .await
                .unwrap()
                .unwrap();
            assert!(n > 0);
            count += n;
        }
        assert_eq!(received, sent);
    }
    defmt::info!("PASS full/chunked/rearm: 16 exact-capacity frames, delayed IRQs");

    // Read before capacity, then fill the remaining physical tail. Rearming
    // must not rewind the producer over bytes that have not been consumed.
    tx.blocking_write(&sent[..17]).unwrap();
    tx.blocking_flush().unwrap();
    Timer::after_millis(1).await;
    assert!(tcd.ch_csr().read().erq());
    assert_eq!(tcd.tcd_daddr().read().daddr(), base + 17);
    assert_eq!(ring.read(&mut received[..5]).await.unwrap(), 5);
    assert_eq!(ring.read(&mut received[5..17]).await.unwrap(), 12);
    tx.blocking_write(&sent[17..]).unwrap();
    tx.blocking_flush().unwrap();
    Timer::after_millis(1).await;
    assert_eq!(ring.read(&mut received[17..]).await.unwrap(), CAPACITY - 17);
    assert_eq!(received, sent);

    // Exercise actual IRQ-driven reads while UART/DMA transmission is ongoing,
    // not only reads after a complete frame has already stopped at capacity.
    let receive = async {
        let mut count = 0;
        while count < CAPACITY {
            let end = (count + 11).min(CAPACITY);
            let n = ring.read(&mut received[count..end]).await.unwrap();
            assert!(n > 0);
            count += n;
        }
    };
    let (written, ()) = join(tx.write(&sent), receive).with_timeout(TIMEOUT).await.unwrap();
    assert_eq!(written.unwrap(), CAPACITY);
    tx.blocking_flush().unwrap();
    assert_eq!(received, sent);
    defmt::info!("PASS partial-window and IRQ-driven in-flight reads");

    // The next byte after a full allocation must stay in the UART/FIFO, not
    // overwrite the first unread word. Excess traffic must report a data gap.
    let overflow = [0x6du8; CAPACITY + 32];
    tx.blocking_write(&overflow).unwrap();
    tx.blocking_flush().unwrap();
    Timer::after_millis(2).await;
    assert!(!tcd.ch_csr().read().erq());
    assert_eq!(tcd.tcd_daddr().read().daddr(), base + CAPACITY as u32);
    assert_eq!(
        ring.read(&mut received).with_timeout(TIMEOUT).await.unwrap(),
        Err(Error::Overrun)
    );
    assert_eq!(
        ring.read(&mut received).with_timeout(TIMEOUT).await.unwrap(),
        Err(Error::Overrun)
    );
    ring.clear();
    defmt::info!("PASS overflow: sticky error, no overwrite, clear/recovery armed");

    // Cancelling a pending read does not relinquish the long-lived receiver.
    assert!(
        ring.read(&mut received)
            .with_timeout(Duration::from_millis(5))
            .await
            .is_err()
    );
    tx.blocking_write(&sent).unwrap();
    tx.blocking_flush().unwrap();
    Timer::after_millis(1).await;
    let n = ring.read(&mut received).with_timeout(TIMEOUT).await.unwrap().unwrap();
    assert_eq!(n, CAPACITY);
    assert_eq!(received, sent);
    drop(ring);
    assert!(!hal::pac::LPUART2.baud().read().rdmae());
    assert!(!tcd.ch_csr().read().erq());
    assert!(!tcd.ch_csr().read().active());
    assert!(!tcd.tcd_csr().read().start());
    defmt::info!("PASS recovery/cancel/drop");

    // BITER=1 must use completion rather than an unsupported half interrupt.
    // SAFETY: both remaining receivers use exclusive static DMA-accessible
    // storage, are never forgotten, and are dropped before reusing this UART.
    let mut one = unsafe { rx.into_ring_dma_rx(RX_ONE.take()) }.unwrap();
    assert!(!tcd.tcd_csr().read().inthalf());
    tx.blocking_write(&[0x91]).unwrap();
    tx.blocking_flush().unwrap();
    let mut byte = [0];
    assert_eq!(one.read(&mut byte).with_timeout(TIMEOUT).await.unwrap().unwrap(), 1);
    assert_eq!(byte, [0x91]);
    drop(one);

    // The allocation need not be a power of two or have an even capacity.
    let mut odd = unsafe { rx.into_ring_dma_rx(RX_ODD.take()) }.unwrap();
    for frame in [[1u8, 2, 3], [0xfe, 0x80, 0x01]] {
        tx.blocking_write(&frame).unwrap();
        tx.blocking_flush().unwrap();
        let mut actual = [0u8; 3];
        let mut n = 0;
        while n < actual.len() {
            n += odd.read(&mut actual[n..]).with_timeout(TIMEOUT).await.unwrap().unwrap();
        }
        assert_eq!(actual, frame);
    }
    drop(odd);
    defmt::info!("PASS one-byte and odd-capacity windows");
    defmt::info!("RING_DMA_ALL_PASS");
    cortex_m::asm::bkpt();
}
