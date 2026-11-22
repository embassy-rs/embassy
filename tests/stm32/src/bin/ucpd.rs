// required-features: ucpd
#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use core::sync::atomic::{AtomicBool, Ordering};

use common::*;
use defmt::{assert, assert_eq};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::{poll_once, yield_now};
use embassy_stm32::ucpd::{self, ActivePdPhy, CcPhy, CcPull, CcSel, CcVState, PdPhy, RxBuffers, RxError, Ucpd};
use embassy_stm32::{Peri, bind_interrupts, peripherals};
use embassy_time::{Duration, Timer, block_for, with_timeout};

bind_interrupts!(struct Irqs {
    UCPD1_2 => ucpd::InterruptHandler<peripherals::UCPD1>, ucpd::InterruptHandler<peripherals::UCPD2>;
    DMA1_CHANNEL1 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH1>;
    DMA1_CHANNEL2_3 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH2>,
    embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH3>;
    DMA1_CH4_7_DMAMUX1_OVR => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH4>;
});

static SRC_TO_SNK: [u8; 6] = [0, 1, 2, 3, 4, 5];
static SNK_TO_SRC: [u8; 4] = [9, 8, 7, 6];

/// Messages from the source for the reception tests.
static MESSAGES: [[u8; 4]; 4] = [
    [0x10, 0x11, 0x12, 0x13],
    [0x20, 0x21, 0x22, 0x23],
    [0x30, 0x31, 0x32, 0x33],
    [0x40, 0x41, 0x42, 0x43],
];

/// A message of the largest size, ~9 ms long on the wire.
static LONG_MESSAGE: [u8; 264] = {
    let mut message = [0; 264];
    let mut i = 0;
    while i < message.len() {
        message[i] = i as u8;
        i += 1;
    }
    message
};

/// Number of times reception is stopped in the middle of `LONG_MESSAGE`, at different points.
const STOP_STEPS: u64 = 32;

/// A notification between the source and the sink, which run in the same task.
struct Flag(AtomicBool);

impl Flag {
    const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    fn set(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    async fn wait(&self) {
        while !self.0.load(Ordering::Relaxed) {
            yield_now().await;
        }
        self.0.store(false, Ordering::Relaxed);
    }
}

/// The sink is ready for the next message.
static SINK_READY: Flag = Flag::new();
/// The source has sent the messages of the current test, or started sending `LONG_MESSAGE`.
static SOURCE_SENT: Flag = Flag::new();

async fn wait_for_vstate<T: ucpd::Instance>(cc_phy: &mut CcPhy<'_, T>, vstate: CcVState) {
    let (mut cc1, mut _cc2) = cc_phy.vstate();
    while cc1 != vstate {
        (cc1, _cc2) = cc_phy.wait_for_vstate_change().await;
    }
}

/// Receives the next message, which must be `expected`.
async fn expect_message<T: ucpd::Instance>(pd_phy: &mut ActivePdPhy<'_, '_, T>, expected: &[u8]) {
    let mut buf = [0_u8; 30];
    let n = unwrap!(pd_phy.receive(buf.as_mut()).await);
    assert_eq!(&buf[..n], expected);
}

/// Receives until `MESSAGES[0]` after reactivating in the middle of `LONG_MESSAGE`.
///
/// Before it, the sink may still receive the long message, if it reactivated before its start,
/// or a CRC error, if it reactivated during its start. Never different data, and never `Overrun`,
/// which would mean that a byte was left over from before.
async fn expect_message_after_restart<T: ucpd::Instance>(pd_phy: &mut ActivePdPhy<'_, '_, T>) {
    let mut buf = [0_u8; 264];
    loop {
        match pd_phy.receive(buf.as_mut()).await {
            Ok(n) if buf[..n] == MESSAGES[0] => return,
            Ok(n) => assert_eq!(&buf[..n], LONG_MESSAGE.as_slice()),
            Err(error) => assert!(matches!(error, RxError::Crc)),
        }
    }
}

/// Receives until `MESSAGES[1]` after the sink cancelled a transmission of `LONG_MESSAGE`.
///
/// Before it, the source may receive what the sink sent of the long message: it went on after its
/// DMA stopped, so with its full length but any bytes, or cut short, with a CRC error. The short
/// message after it must arrive intact, and not, for example, shifted by a byte that the first
/// TX DMA request served before the transmission started.
async fn expect_message_after_cancelled_transmission<T: ucpd::Instance>(pd_phy: &mut PdPhy<'_, T>) {
    let mut buf = [0_u8; 264];
    loop {
        match pd_phy.receive(buf.as_mut()).await {
            Ok(n) if buf[..n] == MESSAGES[1] => return,
            Ok(n) => assert_eq!(n, LONG_MESSAGE.len()),
            Err(error) => assert!(matches!(error, RxError::Crc)),
        }
    }
}

async fn source(
    mut ucpd: Ucpd<'static, peripherals::UCPD1>,
    rx_dma: Peri<'static, peripherals::DMA1_CH1>,
    tx_dma: Peri<'static, peripherals::DMA1_CH2>,
) {
    debug!("source: setting default current pull-up");
    ucpd.cc_phy().set_pull(CcPull::SourceDefaultUsb);

    // Wait for default sink.
    debug!("source: wait for sink");
    wait_for_vstate(ucpd.cc_phy(), CcVState::Low).await;

    // Advertise a higher current by changing the pull-up resistor.
    debug!("source: sink detected, setting 3.0A current pull-up");
    ucpd.cc_phy().set_pull(CcPull::Source3_0A);

    let mut rx_buffers = RxBuffers::new();
    let (_, mut pd_phy) = ucpd.split_pd_phy(rx_dma, tx_dma, Irqs, CcSel::Cc1, &mut rx_buffers);

    // Listen for an incoming message
    debug!("source: wait for message from sink");
    let mut snk_to_src_buf = [0_u8; 30];
    let n = unwrap!(pd_phy.receive(snk_to_src_buf.as_mut()).await);
    assert_eq!(n, SNK_TO_SRC.len());
    assert_eq!(&snk_to_src_buf[..n], SNK_TO_SRC.as_slice());

    // Send message
    debug!("source: message received, sending message");
    unwrap!(pd_phy.transmit(SRC_TO_SNK.as_slice()).await);

    // A message is received while the sink isn't waiting for one.
    debug!("source: sending a message to receive in the background");
    SINK_READY.wait().await;
    unwrap!(pd_phy.transmit(&MESSAGES[0]).await);
    SOURCE_SENT.set();

    // A slow reader gets the first of several messages, then `Overrun`.
    debug!("source: sending three messages to a slow reader");
    SINK_READY.wait().await;
    for message in &MESSAGES[..3] {
        unwrap!(pd_phy.transmit(message).await);
    }
    SOURCE_SENT.set();
    SINK_READY.wait().await;
    unwrap!(pd_phy.transmit(&MESSAGES[3]).await);

    // The sink stops reception in the middle of a long message, and receives the next one intact.
    debug!("source: sending long messages to be cut off");
    for _ in 0..STOP_STEPS {
        SINK_READY.wait().await;
        SOURCE_SENT.set();
        unwrap!(pd_phy.transmit(&LONG_MESSAGE).await);
        SINK_READY.wait().await;
        unwrap!(pd_phy.transmit(&MESSAGES[0]).await);
    }

    // The sink cancels transmissions of a long message, and its next message arrives intact.
    debug!("source: receiving after cancelled transmissions");
    for _ in 0..2 * STOP_STEPS {
        expect_message_after_cancelled_transmission(&mut pd_phy).await;
    }
    // The sink also cancels a transmission while it clears up after a cancelled one.
    expect_message_after_cancelled_transmission(&mut pd_phy).await;

    // The sink cancels a Hard Reset, which is sent anyway, and its next message arrives intact.
    debug!("source: receiving after a cancelled hard reset");
    assert!(matches!(
        pd_phy.receive(snk_to_src_buf.as_mut()).await,
        Err(RxError::HardReset)
    ));
    let n = unwrap!(pd_phy.receive(snk_to_src_buf.as_mut()).await);
    assert_eq!(&snk_to_src_buf[..n], MESSAGES[2].as_slice());

    // The sink's Hard Reset aborts a cancelled long message, and its next messages arrive. The
    // first may go out with a bad CRC (see `transmit_with_sop`), the second must arrive intact.
    debug!("source: receiving after a hard reset that aborted a message");
    loop {
        match pd_phy.receive(snk_to_src_buf.as_mut()).await {
            Err(RxError::HardReset) => break,
            r => assert!(matches!(r, Err(RxError::Crc)), "expected the cut long message"),
        }
    }
    loop {
        match pd_phy.receive(snk_to_src_buf.as_mut()).await {
            Ok(n) if snk_to_src_buf[..n] == MESSAGES[1] => break,
            Ok(n) => assert_eq!(&snk_to_src_buf[..n], MESSAGES[0].as_slice()),
            r => assert!(matches!(r, Err(RxError::Crc))),
        }
    }

    // Wait for hard-reset
    debug!("source: message sent, waiting for hard-reset");
    assert!(matches!(
        pd_phy.receive(snk_to_src_buf.as_mut()).await,
        Err(RxError::HardReset)
    ));
}

async fn sink(
    mut ucpd: Ucpd<'static, peripherals::UCPD2>,
    rx_dma: Peri<'static, peripherals::DMA1_CH3>,
    tx_dma: Peri<'static, peripherals::DMA1_CH4>,
) {
    debug!("sink: setting pull down");
    ucpd.cc_phy().set_pull(CcPull::Sink);

    // Wait for default source.
    debug!("sink: waiting for default vstate");
    wait_for_vstate(ucpd.cc_phy(), CcVState::Low).await;

    // Wait higher current pull-up.
    //debug!("sink: source default vstate detected, waiting for 3.0A vstate");
    //wait_for_vstate(ucpd.cc_phy(), CcVState::HIGHEST).await;
    //debug!("sink: source 3.0A vstate detected");
    // TODO: not working yet, why? no idea, replace with timer for now
    Timer::after_millis(100).await;

    let (_, mut idle_pd_phy) = ucpd.split_idle_pd_phy(rx_dma, tx_dma, Irqs);
    let mut rx_buffers = RxBuffers::new();
    let mut pd_phy = idle_pd_phy.activate(CcSel::Cc1, &mut rx_buffers);

    // Send message
    debug!("sink: sending message");
    unwrap!(pd_phy.transmit(SNK_TO_SRC.as_slice()).await);

    // Listen for an incoming message
    debug!("sink: message sent, waiting for message from source");
    let mut src_to_snk_buf = [0_u8; 30];
    let n = unwrap!(pd_phy.receive(src_to_snk_buf.as_mut()).await);
    assert_eq!(n, SRC_TO_SNK.len());
    assert_eq!(&src_to_snk_buf[..n], SRC_TO_SNK.as_slice());

    // A message is received while the sink isn't waiting for one.
    debug!("sink: receiving a message in the background");
    SINK_READY.set();
    SOURCE_SENT.wait().await;
    expect_message(&mut pd_phy, &MESSAGES[0]).await;

    // A slow reader gets the first of several messages, then `Overrun`, and then nothing.
    debug!("sink: receiving three messages as a slow reader");
    SINK_READY.set();
    SOURCE_SENT.wait().await;
    expect_message(&mut pd_phy, &MESSAGES[0]).await;
    let mut buf = [0_u8; 30];
    assert!(matches!(pd_phy.receive(&mut buf).await, Err(RxError::Overrun)));
    assert!(
        with_timeout(Duration::from_millis(10), pd_phy.receive(&mut buf))
            .await
            .is_err()
    );
    SINK_READY.set();
    expect_message(&mut pd_phy, &MESSAGES[3]).await;

    // Stopping reception in the middle of a message leaves nothing behind: the first message
    // after reactivating is received intact.
    debug!("sink: stopping reception in the middle of long messages");
    drop(pd_phy);
    for step in 0..STOP_STEPS {
        let pd_phy = idle_pd_phy.activate(CcSel::Cc1, &mut rx_buffers);
        SINK_READY.set();
        SOURCE_SENT.wait().await;
        block_for(Duration::from_micros(300 * step));
        drop(pd_phy);

        let mut pd_phy = idle_pd_phy.activate(CcSel::Cc1, &mut rx_buffers);
        SINK_READY.set();
        expect_message_after_restart(&mut pd_phy).await;
    }

    // Cancelling a transmission in the middle of a long message doesn't affect the next
    // transmission, whether it comes right away or after going through `IdlePdPhy`. The last
    // steps cancel after the DMA has delivered the whole message, which leaves no underrun.
    debug!("sink: cancelling transmissions of long messages");
    for step in 0..STOP_STEPS {
        let mut pd_phy = idle_pd_phy.activate(CcSel::Cc1, &mut rx_buffers);
        let _ = with_timeout(Duration::from_micros(300 * step), pd_phy.transmit(&LONG_MESSAGE)).await;
        unwrap!(pd_phy.transmit(&MESSAGES[1]).await);

        let _ = with_timeout(Duration::from_micros(300 * step), pd_phy.transmit(&LONG_MESSAGE)).await;
        drop(pd_phy);

        let mut pd_phy = idle_pd_phy.activate(CcSel::Cc1, &mut rx_buffers);
        unwrap!(pd_phy.transmit(&MESSAGES[1]).await);
    }
    let mut pd_phy = idle_pd_phy.activate(CcSel::Cc1, &mut rx_buffers);

    // Cancelling a transmission while it clears up after a cancelled one, before the DMA interrupt
    // could run (as when the task has a higher priority), doesn't affect the next transmission.
    debug!("sink: cancelling a transmission after a cancelled one");
    let _ = with_timeout(Duration::from_millis(2), pd_phy.transmit(&LONG_MESSAGE)).await;
    Timer::after_millis(10).await;
    let first = critical_section::with(|_| poll_once(pd_phy.transmit(&MESSAGES[1])));
    assert!(first.is_pending());
    let sent = with_timeout(Duration::from_millis(10), pd_phy.transmit(&MESSAGES[1])).await;
    assert!(matches!(sent, Ok(Ok(()))));

    // Cancelling a Hard Reset right after it started doesn't block the next transmission.
    debug!("sink: cancelling a hard reset");
    assert!(poll_once(pd_phy.transmit_hardreset()).is_pending());
    let sent = with_timeout(Duration::from_millis(10), pd_phy.transmit(&MESSAGES[2])).await;
    assert!(matches!(sent, Ok(Ok(()))));

    // A Hard Reset that aborts a message doesn't block the next transmissions.
    debug!("sink: aborting a cancelled transmission with a hard reset");
    let _ = with_timeout(Duration::from_millis(2), pd_phy.transmit(&LONG_MESSAGE)).await;
    unwrap!(pd_phy.transmit_hardreset().await);
    for message in &MESSAGES[..2] {
        let sent = with_timeout(Duration::from_millis(10), pd_phy.transmit(message)).await;
        assert!(matches!(sent, Ok(Ok(()))));
    }

    // Send hard reset
    debug!("sink: message received, sending hard-reset");
    unwrap!(pd_phy.transmit_hardreset().await);
}

#[embassy_executor::main(executor = "common::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = init();
    info!("Hello World!");

    // Wire between PD0 and PA8
    let ucpd1 = Ucpd::new(p.UCPD1, p.PA8, p.PB15, Irqs, Default::default());
    let ucpd2 = Ucpd::new(p.UCPD2, p.PD0, p.PD2, Irqs, Default::default());

    join(
        source(ucpd1, p.DMA1_CH1, p.DMA1_CH2),
        sink(ucpd2, p.DMA1_CH3, p.DMA1_CH4),
    )
    .await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}
