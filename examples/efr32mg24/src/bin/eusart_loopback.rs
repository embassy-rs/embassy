//! EUSART1 self-test on the Seeed Studio XIAO MG24 with internal loopback. Needs no wiring.
//!
//! TX is on PC06 (D6) and RX is on PC07 (D7).
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_silabs::eusart::{self, Config, ConfigError, DataBits, Error, Parity, Uart};
use embassy_silabs::mode::Async;
use embassy_silabs::{bind_interrupts, peripherals};
use embassy_time::{Duration, with_timeout};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    EUSART1_RX => eusart::RxInterruptHandler<peripherals::EUSART1>;
    EUSART1_TX => eusart::TxInterruptHandler<peripherals::EUSART1>;
});

const PATTERN: [u8; 8] = [0x55, 0xAA, 0x00, 0xFF, 0x12, 0x34, 0x56, 0x78];
const TIMEOUT: Duration = Duration::from_millis(100);

/// Writes `PATTERN` and reads it back at the same time.
async fn round_trip(uart: &mut Uart<'_, Async>, step: &str) -> bool {
    let mut buf = [0u8; PATTERN.len()];
    let (mut tx, mut rx) = uart.split_ref();
    let (written, read) = join(tx.write(&PATTERN), with_timeout(TIMEOUT, rx.read(&mut buf))).await;
    match (written, read) {
        (Ok(()), Ok(Ok(()))) if buf == PATTERN => {
            info!("{}: ok", step);
            true
        }
        (Ok(()), Ok(Ok(()))) => {
            error!("{}: read {=[u8]:x}, expected {=[u8]:x}", step, buf, PATTERN);
            false
        }
        (w, r) => {
            error!("{}: write {}, read {}", step, w, r.map_err(|_| "timeout"));
            false
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init(embassy_silabs::Config::default());

    let mut config = Config::default();
    config.loopback = true;
    let mut uart = unwrap!(Uart::new(p.EUSART1, p.PC06, p.PC07, Irqs, config));

    let mut pass = round_trip(&mut uart, "115200 8N1").await;

    unwrap!(uart.blocking_write(&[0x5A]));
    // `STATUS` lags the `TXDATA` write, but `busy()` must read true at once.
    let busy = uart.busy();
    info!("busy right after a write: {}", busy);
    pass &= busy;
    let mut one = [0u8];
    match with_timeout(TIMEOUT, uart.read(&mut one)).await {
        Ok(Ok(())) if one[0] == 0x5A => info!("one byte: ok"),
        other => {
            error!("one byte: {} {=u8:x}", other.map_err(|_| "timeout"), one[0]);
            pass = false;
        }
    }

    if uart.set_baudrate(0) != Err(ConfigError::BaudrateTooLow) {
        error!("set_baudrate(0) did not return BaudrateTooLow");
        pass = false;
    }
    let mut bad = config;
    bad.data_bits = DataBits::DataBits5;
    if uart.set_config(&bad) != Err(ConfigError::DataBitsNotSupported) {
        error!("set_config(DataBits5) did not return DataBitsNotSupported");
        pass = false;
    }

    unwrap!(uart.set_baudrate(9600));
    pass &= round_trip(&mut uart, "9600 8N1").await;

    config.baudrate = 57_600;
    config.parity = Parity::ParityEven;
    unwrap!(uart.set_config(&config));
    pass &= round_trip(&mut uart, "57600 8E1").await;

    // `TXDATA.TXBREAK` sends data 0 with a low stop bit: a framing error.
    uart.send_break();
    let mut frame = [0xA5u8];
    match with_timeout(TIMEOUT, uart.read(&mut frame)).await {
        Ok(Err(Error::Framing)) => info!("break: framing error"),
        other => {
            error!("break: {} {=u8:x}", other.map_err(|_| "timeout"), frame[0]);
            pass = false;
        }
    }

    pass &= round_trip(&mut uart, "after break").await;

    if pass {
        info!("PASS");
    } else {
        error!("FAIL");
    }
}
