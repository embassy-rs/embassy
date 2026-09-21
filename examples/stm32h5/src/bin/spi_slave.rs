//! SPI slave loopback test on Nucleo-H563ZI.
//!
//! Requires 4 jumper wires (single-board loopback, all on the Arduino/Zio
//! connectors CN7/CN9):
//!   PD14 (D10, master CS, GPIO) -> PA15 (slave NSS / EXTI15)
//!   PA5  (D13, master SCK)      -> PC10 (slave SCK)
//!   PB5  (D11, master MOSI)     -> PC12 (slave MOSI)
//!   PG9  (D12, master MISO)     -> PC11 (slave MISO)
//!
//! SPI1 is the master (software NSS, CS driven manually so it can be
//! deasserted mid-transfer). SPI3 is the slave with hardware NSS + EXTI.
//!
//! Test 1: full-duplex 16-byte transfer, both directions verified.
//! Test 2: early termination — master sends only 5 of 16 bytes and deselects.
//!         The slave read returns promptly with the partial byte count,
//!         like embassy-nrf's Spis.
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rcc::{AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk};
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::time::{Hertz, mhz};
use embassy_stm32::{Config, bind_interrupts, dma, exti, interrupt, peripherals};
use embassy_time::{Duration, Timer, with_timeout};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL0 => dma::InterruptHandler<peripherals::GPDMA1_CH0>;
    GPDMA1_CHANNEL1 => dma::InterruptHandler<peripherals::GPDMA1_CH1>;
    GPDMA1_CHANNEL2 => dma::InterruptHandler<peripherals::GPDMA1_CH2>;
    GPDMA1_CHANNEL3 => dma::InterruptHandler<peripherals::GPDMA1_CH3>;
    EXTI15 => exti::InterruptHandler<interrupt::typelevel::EXTI15>;
});

const N: usize = 16;
const K: usize = 5; // bytes sent in the early-termination test

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    config.rcc.hse = Some(Hse {
        freq: mhz(8),
        mode: HseMode::Oscillator,
    });

    config.rcc.pll1 = Some(Pll {
        source: PllSource::Hse,
        prediv: PllPreDiv::Div4,
        mul: PllMul::Mul250,
        divp: Some(PllDiv::Div2),
        divq: Some(PllDiv::Div2),
        divr: Some(PllDiv::Div2),
    });

    config.rcc.sys = Sysclk::Pll1P;
    config.rcc.ahb_pre = AHBPrescaler::Div1;
    config.rcc.apb1_pre = APBPrescaler::Div1;
    config.rcc.apb2_pre = APBPrescaler::Div1;
    config.rcc.apb3_pre = APBPrescaler::Div1;

    let p = embassy_stm32::init(config);

    let mut config = SpiConfig::default();
    config.frequency = Hertz(25_000_000);

    let mut master = Spi::new(p.SPI1, p.PA5, p.PB5, p.PG9, p.GPDMA1_CH0, p.GPDMA1_CH1, Irqs, config);
    // Active-low NSS: idle high, driven low to select.
    let mut cs = Output::new(p.PD14, Level::High, Speed::VeryHigh);
    let mut slave = Spi::new_slave(
        p.SPI3,
        p.PC10,
        p.PC12,
        p.PC11,
        p.PA15,
        p.GPDMA1_CH2,
        p.GPDMA1_CH3,
        Irqs,
        Some(p.EXTI15),
        config,
    );

    info!("hello world");

    // ---------- Test 1: full-duplex transfer, both directions ----------
    let mut m_tx = [0u8; N];
    let mut m_rx = [0u8; N];
    let mut s_tx = [0u8; N];
    let mut s_rx = [0u8; N];
    for i in 0..N {
        m_tx[i] = 0xA0 + i as u8;
        s_tx[i] = 0x50 + i as u8;
    }

    // The slave must be armed (transfer called) while NSS is still deasserted:
    // the transaction is captured from the NSS falling edge.
    let mut s_res = Ok((0, 0));
    let ((), ()) = join(
        async {
            s_res = slave.transfer(&mut s_rx, &s_tx).await;
        },
        async {
            // Let the slave arm first, then select and clock.
            Timer::after_micros(500).await;
            cs.set_low();
            master.transfer(&mut m_rx, &m_tx).await.unwrap();
            cs.set_high();
        },
    )
    .await;
    let (n_rx, n_tx) = s_res.unwrap();
    defmt::assert!(n_rx == N && n_tx == N, "slave transfer counts wrong: {} {}", n_rx, n_tx);
    Timer::after_millis(1).await;

    info!("test1: master rx {}", m_rx);
    info!("test1: slave rx  {}", s_rx);
    defmt::assert!(m_rx == s_tx, "master received wrong data");
    defmt::assert!(s_rx == m_tx, "slave received wrong data");
    info!("test1 PASSED: full-duplex {} bytes both directions", N);

    // ---------- Test 2: early termination (slave read) ----------
    info!(
        "test2: early termination, master sends {} of {} bytes then deselects",
        K, N
    );
    let mut buf = [0u8; N];
    let early_tx = [0xC0u8, 0xC1, 0xC2, 0xC3, 0xC4];

    cs.set_low();
    let slave_fut = slave.read(&mut buf);
    let master_fut = async {
        Timer::after_micros(500).await;
        master.write::<u8>(&early_tx).await.unwrap();
        // Let the last byte land, then deselect while the slave still expects N bytes.
        Timer::after_micros(200).await;
        cs.set_high();
    };
    let (res, ()) = join(with_timeout(Duration::from_secs(3), slave_fut), master_fut).await;
    Timer::after_millis(1).await;

    match res {
        Ok(Ok(n)) => {
            defmt::assert!(n == K, "slave read returned wrong count: {} (expected {})", n, K);
            defmt::assert!(buf[..K] == early_tx, "slave received wrong data");
            info!("test2 PASSED: slave read returned {} bytes after early deselect", n);
        }
        Ok(Err(e)) => defmt::panic!("test2: slave read returned Err({})", e),
        Err(_) => defmt::panic!("test2: TIMEOUT — slave read hung after deselect"),
    }

    info!("DONE");
    cortex_m::asm::bkpt();
    loop {
        Timer::after_secs(1).await;
    }
}
