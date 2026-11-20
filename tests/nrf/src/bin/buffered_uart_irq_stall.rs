// required-features: easydma
//! `BufferedUarteRx` must keep receiving after its interrupt was held off while both halves of the
//! RX ring filled up (two ENDRX events seen as one).
//!
//! PIN_A is looped back to PIN_B and bit-banged as a GPIO, so the bytes can be sent with
//! interrupts disabled.
#![no_std]
#![no_main]
// The nRF54L `BufferedUarte` doesn't count RXDRDY events and has no such bug; nothing to run there.
#![cfg_attr(feature = "nrf54l15", allow(unused))]

#[path = "../common.rs"]
mod common;

use defmt::{assert_eq, *};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
#[cfg(not(feature = "nrf54l15"))]
use embassy_nrf::{
    buffered_uarte::{self, BufferedUarteRx},
    peripherals, uarte,
};
use embassy_time::{Duration, Timer, with_timeout};
use panic_probe as _;

teleprobe_meta::timeout!(60);

const CPU_HZ: u32 = 64_000_000;
const BAUD: u32 = 115_200;

/// Send 8N1 frames on `pin`. Call with interrupts disabled.
fn send(pin: &mut Output<'_>, data: impl IntoIterator<Item = u8>) {
    let wait_until = |t: u32| while (cortex_m::peripheral::DWT::cycle_count().wrapping_sub(t) as i32) < 0 {};
    for byte in data {
        let start = cortex_m::peripheral::DWT::cycle_count();
        let edge = |k: u32| start.wrapping_add((k as u64 * CPU_HZ as u64 / BAUD as u64) as u32);
        // start bit, 8 data bits LSB first, stop bit
        for k in 0..10u32 {
            let high = match k {
                0 => false,
                9 => true,
                _ => (byte >> (k - 1)) & 1 != 0,
            };
            wait_until(edge(k));
            pin.set_level(high.into());
        }
        wait_until(edge(10));
    }
}

fn pattern(i: usize) -> u8 {
    b'a' + (i % 26) as u8
}

#[cfg(not(feature = "nrf54l15"))]
async fn expect(u: &mut BufferedUarteRx<'_>, what: &str, expected: impl IntoIterator<Item = u8>) {
    let mut buf = [0u8; 64];
    let mut expected = expected.into_iter().peekable();
    let mut i = 0;
    while expected.peek().is_some() {
        let n = match with_timeout(Duration::from_millis(100), u.read(&mut buf)).await {
            Ok(r) => unwrap!(r),
            Err(_) => defmt::panic!("{}: timed out waiting for byte {}", what, i),
        };
        for &b in &buf[..n] {
            assert_eq!(Some(b), expected.next(), "{}: byte {}", what, i);
            i += 1;
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_nrf::init(Default::default());

    #[cfg(not(feature = "nrf54l15"))]
    {
        let mut cp = unsafe { cortex_m::Peripherals::steal() };
        cp.DCB.enable_trace();
        cp.DWT.enable_cycle_counter();

        let mut config = uarte::Config::default();
        config.parity = uarte::Parity::Excluded;
        config.baudrate = uarte::Baudrate::Baud115200;

        let mut rx_buffer = [0u8; 1024];
        // The driver uses at most two EasyDMA transfers worth of the buffer.
        let ring_len = if cfg!(feature = "nrf52832") {
            2 * 255
        } else {
            rx_buffer.len()
        };
        let half_len = ring_len / 2;

        let mut pin = Output::new(peri!(p, PIN_A).reborrow(), Level::High, OutputDrive::Standard);
        let mut u = buffered_uarte_rx_new!(p, peri!(p, PIN_B).reborrow(), config.clone(), &mut rx_buffer);

        // uarte needs some quiet time to start rxing properly.
        Timer::after_millis(10).await;

        let mut total = 0;
        for round in 0..4 {
            // With the interrupt held off, send enough to fill the running transfer and the queued
            // one, so both end before the interrupt runs. Every other round, send a few bytes more,
            // which wait in the UARTE (it holds 6) until the DMA restarts.
            let extra = if round % 2 == 0 { 0 } else { 3 };
            let n = 2 * half_len - total % half_len + extra;
            cortex_m::interrupt::free(|_| send(&mut pin, (total..total + n).map(pattern)));
            expect(&mut u, "stalled burst", (total..total + n).map(pattern)).await;
            total += n;

            // RX must still work afterwards.
            cortex_m::interrupt::free(|_| send(&mut pin, *b"hello\n"));
            expect(&mut u, "after the stall", *b"hello\n").await;
            total += 6;

            info!("round {} OK", round);
        }
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
