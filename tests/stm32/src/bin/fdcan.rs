#![no_std]
#![no_main]

// required-features: fdcan

#[path = "../common.rs"]
mod common;
use common::*;
use defmt::assert;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::*;
use embassy_stm32::{bind_interrupts, can, Config};
use embassy_time::{Duration, Instant};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<FDCAN1>;
});

struct TestOptions {
    config: Config,
    max_latency: Duration,
    second_fifo_working: bool,
}

#[cfg(any(feature = "stm32h755zi", feature = "stm32h753zi", feature = "stm32h563zi"))]
fn options() -> TestOptions {
    use embassy_stm32::rcc;
    info!("H75 config");
    let mut c = config();
    c.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(25_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    c.rcc.fdcan_clock_source = rcc::FdCanClockSource::HSE;
    TestOptions {
        config: c,
        max_latency: Duration::from_micros(3800),
        second_fifo_working: false,
    }
}

#[cfg(any(feature = "stm32h7a3zi"))]
fn options() -> TestOptions {
    use embassy_stm32::rcc;
    info!("H7a config");
    let mut c = config();
    c.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(25_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    c.rcc.fdcan_clock_source = rcc::FdCanClockSource::HSE;
    TestOptions {
        config: c,
        max_latency: Duration::from_micros(5500),
        second_fifo_working: false,
    }
}

#[cfg(any(feature = "stm32g491re"))]
fn options() -> TestOptions {
    info!("G4 config");
    TestOptions {
        config: config(),
        max_latency: Duration::from_micros(500),
        second_fifo_working: true,
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    //let peripherals = embassy_stm32::init(config());

    let options = options();
    let peripherals = embassy_stm32::init(options.config);

    let mut can = can::Fdcan::new(peripherals.FDCAN1, peripherals.PB8, peripherals.PB9, Irqs);

    // 250k bps
    can.set_bitrate(250_000);

    can.can.set_extended_filter(
        can::filter::ExtendedFilterSlot::_0,
        can::filter::ExtendedFilter::accept_all_into_fifo1(),
    );

    let mut can = can.into_internal_loopback_mode();

    info!("CAN Configured");

    let mut i: u8 = 0;
    loop {
        let tx_frame = can::TxFrame::new(
            can::TxFrameHeader {
                len: 1,
                frame_format: can::FrameFormat::Standard,
                id: can::StandardId::new(0x123).unwrap().into(),
                bit_rate_switching: false,
                marker: None,
            },
            &[i],
        )
        .unwrap();

        info!("Transmitting frame...");
        let tx_ts = Instant::now();
        can.write(&tx_frame).await;

        let envelope = can.read().await.unwrap();
        info!("Frame received!");

        // Check data.
        assert!(i == envelope.data()[0], "{} == {}", i, envelope.data()[0]);

        info!("loopback time {}", envelope.header.time_stamp);
        info!("loopback frame {=u8}", envelope.data()[0]);
        let latency = envelope.timestamp.saturating_duration_since(tx_ts);
        info!("loopback latency {} us", latency.as_micros());

        // Theoretical minimum latency is 55us, actual is usually ~80us
        const MIN_LATENCY: Duration = Duration::from_micros(50);
        // Was failing at 150 but we are not getting a real time stamp. I'm not
        // sure if there are other delays
        assert!(
            MIN_LATENCY <= latency && latency <= options.max_latency,
            "{} <= {} <= {}",
            MIN_LATENCY,
            latency,
            options.max_latency
        );

        i += 1;
        if i > 10 {
            break;
        }
    }

    let max_buffered = if options.second_fifo_working { 6 } else { 3 };

    // Below here, check that we can receive from both FIFO0 and FIFO0
    // Above we configured FIFO1 for extended ID packets. There are only 3 slots
    // in each FIFO so make sure we write enough to fill them both up before reading.
    for i in 0..3 {
        // Try filling up the RX FIFO0 buffers with standard packets
        let tx_frame = can::TxFrame::new(
            can::TxFrameHeader {
                len: 1,
                frame_format: can::FrameFormat::Standard,
                id: can::StandardId::new(0x123).unwrap().into(),
                bit_rate_switching: false,
                marker: None,
            },
            &[i],
        )
        .unwrap();
        info!("Transmitting frame {}", i);
        can.write(&tx_frame).await;
    }
    for i in 3..max_buffered {
        // Try filling up the RX FIFO0 buffers with extended packets
        let tx_frame = can::TxFrame::new(
            can::TxFrameHeader {
                len: 1,
                frame_format: can::FrameFormat::Standard,
                id: can::ExtendedId::new(0x1232344).unwrap().into(),
                bit_rate_switching: false,
                marker: None,
            },
            &[i],
        )
        .unwrap();

        info!("Transmitting frame {}", i);
        can.write(&tx_frame).await;
    }

    // Try and receive all 6 packets
    for i in 0..max_buffered {
        let envelope = can.read().await.unwrap();
        match envelope.header.id {
            can::Id::Extended(id) => {
                info!("Extended received! {:x} {} {}", id.as_raw(), envelope.data()[0], i);
            }
            can::Id::Standard(id) => {
                info!("Standard received! {:x} {} {}", id.as_raw(), envelope.data()[0], i);
            }
        }
    }

    // Test again with a split
    let (mut tx, mut rx) = can.split();
    for i in 0..3 {
        // Try filling up the RX FIFO0 buffers with standard packets
        let tx_frame = can::TxFrame::new(
            can::TxFrameHeader {
                len: 1,
                frame_format: can::FrameFormat::Standard,
                id: can::StandardId::new(0x123).unwrap().into(),
                bit_rate_switching: false,
                marker: None,
            },
            &[i],
        )
        .unwrap();

        info!("Transmitting frame {}", i);
        tx.write(&tx_frame).await;
    }
    for i in 3..max_buffered {
        // Try filling up the RX FIFO0 buffers with extended packets
        let tx_frame = can::TxFrame::new(
            can::TxFrameHeader {
                len: 1,
                frame_format: can::FrameFormat::Standard,
                id: can::ExtendedId::new(0x1232344).unwrap().into(),
                bit_rate_switching: false,
                marker: None,
            },
            &[i],
        )
        .unwrap();

        info!("Transmitting frame {}", i);
        tx.write(&tx_frame).await;
    }

    // Try and receive all 6 packets
    for i in 0..max_buffered {
        let envelope = rx.read().await.unwrap();
        match envelope.header.id {
            can::Id::Extended(id) => {
                info!("Extended received! {:x} {} {}", id.as_raw(), envelope.data()[0], i);
            }
            can::Id::Standard(id) => {
                info!("Standard received! {:x} {} {}", id.as_raw(), envelope.data()[0], i);
            }
        }
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
