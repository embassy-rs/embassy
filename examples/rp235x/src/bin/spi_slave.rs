//! This example exchanges data between a blocking SPI master and slave on the RP235x.
//! SPI1 runs on core 0 and SPI0 on core 1 so both blocking drivers can make progress.
//! The example uses SPI mode 1 for a multi-byte transfer under one CS assertion.
//!
//! Connect the following pins:
//! Master (MISO) PIN 12 <- PIN 3 (MISO/TX) Slave
//! Master (MOSI) PIN 11 -> PIN 0 (MOSI/RX) Slave
//! Master (CLK)  PIN 10 -> PIN 2 (CLK) Slave
//! Master (CS)   PIN 13 -> PIN 1 (CS) Slave
//!
//! Each side receives the other side's TX buffer. The slave returns when CS rises.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_rp::time::Hertz;
use embassy_rp::{spi, spi_slave};
use embassy_time::{Duration, block_for};
use panic_probe as _;

static mut CORE1_STACK: Stack<4096> = Stack::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    info!("Starting up!");

    // Master wiring on SPI1.
    let mut master_config = spi::Config::default();
    master_config.phase = spi::Phase::CaptureOnSecondTransition;
    // Leave enough time for the blocking slave to service both FIFOs.
    master_config.frequency = Hertz(100_000);
    let mut master_spi = spi::Spi::new_blocking(p.SPI1, p.PIN_10, p.PIN_11, p.PIN_12, master_config).unwrap();
    let mut master_cs = Output::new(p.PIN_13, Level::High);

    // Slave wiring on SPI0, matching spi_slave_async.
    let mut slave_config = spi_slave::Config::default();
    slave_config.phase = spi::Phase::CaptureOnSecondTransition;
    let mut slave_spi = spi_slave::Spi::new_blocking(p.SPI0, p.PIN_2, p.PIN_0, p.PIN_3, p.PIN_1, slave_config);

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let tx_buf = [0xA_u8, 0xB, 0xC, 0xD, 0xE, 0xF];
            let mut rx_buf = [0_u8; 6];

            loop {
                let (received, sent) = slave_spi.blocking_transfer(&mut rx_buf, &tx_buf).unwrap();
                info!(
                    "Slave: sent {} received {} TX: {:x} RX: {:x}",
                    sent,
                    received,
                    &tx_buf[..sent],
                    &rx_buf[..received],
                );
            }
        },
    );

    let tx_buf = [1_u8, 2, 3, 4, 5, 6];
    let mut rx_buf = [0_u8; 6];

    // Allow core 1 to start and the slave to preload TX before asserting CS.
    block_for(Duration::from_millis(1));
    loop {
        master_cs.set_low();
        master_spi.blocking_transfer(&mut rx_buf, &tx_buf).unwrap();
        master_cs.set_high();

        info!("Master: TX: {:x} RX: {:x}", tx_buf, rx_buf);
        // Allow slave cleanup, logging and rearming before the next assertion.
        block_for(Duration::from_secs(1));
    }
}
