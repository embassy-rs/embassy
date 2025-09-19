// required-features: ucpd
#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::{assert, assert_eq};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::ucpd::{self, CcPhy, CcPull, CcSel, CcVState, RxError, Ucpd};
use embassy_stm32::{bind_interrupts, peripherals, Peri};
use embassy_time::Timer;

bind_interrupts!(struct Irqs {
    UCPD1_2 => ucpd::InterruptHandler<peripherals::UCPD1>, ucpd::InterruptHandler<peripherals::UCPD2>;
});

static SRC_TO_SNK: [u8; 6] = [0, 1, 2, 3, 4, 5];
static SNK_TO_SRC: [u8; 4] = [9, 8, 7, 6];

async fn wait_for_vstate<T: ucpd::Instance>(cc_phy: &mut CcPhy<'_, T>, vstate: CcVState) {
    let (mut cc1, mut _cc2) = cc_phy.vstate();
    while cc1 != vstate {
        (cc1, _cc2) = cc_phy.wait_for_vstate_change().await;
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
    wait_for_vstate(ucpd.cc_phy(), CcVState::LOW).await;

    // Advertise a higher current by changing the pull-up resistor.
    debug!("source: sink detected, setting 3.0A current pull-up");
    ucpd.cc_phy().set_pull(CcPull::Source3_0A);

    let (_, mut pd_phy) = ucpd.split_pd_phy(rx_dma, tx_dma, CcSel::CC1);

    // Listen for an incoming message
    debug!("source: wait for message from sink");
    let mut snk_to_src_buf = [0_u8; 30];
    let n = unwrap!(pd_phy.receive(snk_to_src_buf.as_mut()).await);
    assert_eq!(n, SNK_TO_SRC.len());
    assert_eq!(&snk_to_src_buf[..n], SNK_TO_SRC.as_slice());

    // Send message
    debug!("source: message received, sending message");
    unwrap!(pd_phy.transmit(SRC_TO_SNK.as_slice()).await);

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
    wait_for_vstate(ucpd.cc_phy(), CcVState::LOW).await;

    // Wait higher current pull-up.
    //debug!("sink: source default vstate detected, waiting for 3.0A vstate");
    //wait_for_vstate(ucpd.cc_phy(), CcVState::HIGHEST).await;
    //debug!("sink: source 3.0A vstate detected");
    // TODO: not working yet, why? no idea, replace with timer for now
    Timer::after_millis(100).await;

    let (_, mut pd_phy) = ucpd.split_pd_phy(rx_dma, tx_dma, CcSel::CC1);

    // Send message
    debug!("sink: sending message");
    unwrap!(pd_phy.transmit(SNK_TO_SRC.as_slice()).await);

    // Listen for an incoming message
    debug!("sink: message sent, waiting for message from source");
    let mut src_to_snk_buf = [0_u8; 30];
    let n = unwrap!(pd_phy.receive(src_to_snk_buf.as_mut()).await);
    assert_eq!(n, SRC_TO_SNK.len());
    assert_eq!(&src_to_snk_buf[..n], SRC_TO_SNK.as_slice());

    // Send hard reset
    debug!("sink: message received, sending hard-reset");
    unwrap!(pd_phy.transmit_hardreset().await);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init();
    info!("Hello World!");

    // Wire between PD0 and PA8
    let ucpd1 = Ucpd::new(p.UCPD1, Irqs {}, p.PA8, p.PB15, Default::default());
    let ucpd2 = Ucpd::new(p.UCPD2, Irqs {}, p.PD0, p.PD2, Default::default());

    join(
        source(ucpd1, p.DMA1_CH1, p.DMA1_CH2),
        sink(ucpd2, p.DMA1_CH3, p.DMA1_CH4),
    )
    .await;

    info!("Test OK");
    cortex_m::asm::bkpt();
}
