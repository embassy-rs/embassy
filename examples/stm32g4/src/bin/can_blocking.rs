#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::*;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, can, Config};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<FDCAN1>;
});

// This example demonstrates blocking CAN operations on STM32G4

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(24_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV6,
            mul: PllMul::MUL85,
            divp: None,
            divq: Some(PllQDiv::DIV8), // 42.5 Mhz for fdcan.
            divr: Some(PllRDiv::DIV2), // Main system clock at 170 MHz
        });
        config.rcc.mux.fdcansel = mux::Fdcansel::PLL1_Q;
        config.rcc.sys = Sysclk::PLL1_R;
    }
    let peripherals = embassy_stm32::init(config);

    let mut can = can::CanConfigurator::new(peripherals.FDCAN1, peripherals.PA11, peripherals.PA12, Irqs);

    can.properties().set_extended_filter(
        can::filter::ExtendedFilterSlot::_0,
        can::filter::ExtendedFilter::accept_all_into_fifo1(),
    );

    // 250k bps
    can.set_bitrate(250_000);

    info!("Configured CAN for blocking operations");

    // Start in internal loopback mode for testing
    let mut can = can.start(can::OperatingMode::InternalLoopbackMode);

    info!("CAN Blocking Example Started");

    // Example 1: Basic blocking operations
    for i in 0..3 {
        let frame = can::frame::Frame::new_extended(0x123456F + i as u32, &[i, 0x01, 0x02, 0x03]).unwrap();

        // Blocking write
        info!("Blocking write frame {}", i);
        match can.blocking_write(&frame) {
            Some(_dropped_frame) => {
                warn!("Frame was dropped");
            }
            None => {
                info!("Frame sent successfully");
            }
        }

        // Blocking read
        match can.blocking_read() {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                info!(
                    "Blocking read - Rx: {} {:02x}",
                    rx_frame.header().len(),
                    rx_frame.data()[0..rx_frame.header().len() as usize],
                );
            }
            Err(err) => {
                error!("Blocking read error: {}", err);
            }
        }
    }

    // Example 2: Using FD blocking operations
    for i in 3..6 {
        let frame = can::frame::FdFrame::new_extended(0x200000 + i as u32, &[i, 0x10, 0x20, 0x30, 0x40]).unwrap();

        // Blocking write FD
        info!("Blocking write FD frame {}", i);
        match can.blocking_write_fd(&frame) {
            Some(_dropped_frame) => {
                warn!("FD Frame was dropped");
            }
            None => {
                info!("FD Frame sent successfully");
            }
        }

        // Blocking read FD
        match can.blocking_read_fd() {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                info!(
                    "Blocking read FD - Rx: {} {:02x}",
                    rx_frame.header().len(),
                    rx_frame.data()[0..rx_frame.header().len() as usize],
                );
            }
            Err(err) => {
                error!("Blocking read FD error: {}", err);
            }
        }
    }

    // Example 3: Split CAN blocking operations
    let (mut tx, mut rx, _props) = can.split();

    for i in 6..9 {
        let frame = can::frame::Frame::new_extended(0x300000 + i as u32, &[i, 0xAA, 0xBB, 0xCC]).unwrap();

        // Split blocking write
        info!("Split blocking write frame {}", i);
        match tx.blocking_write(&frame) {
            Some(_dropped_frame) => {
                warn!("Split frame was dropped");
            }
            None => {
                info!("Split frame sent successfully");
            }
        }

        // Split blocking read
        match rx.blocking_read() {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                info!(
                    "Split blocking read - Rx: {} {:02x}",
                    rx_frame.header().len(),
                    rx_frame.data()[0..rx_frame.header().len() as usize],
                );
            }
            Err(err) => {
                error!("Split blocking read error: {}", err);
            }
        }
    }

    // Example 4: Buffered blocking operations
    let can = can::Can::join(tx, rx);

    static TX_BUF: StaticCell<can::TxBuf<8>> = StaticCell::new();
    static RX_BUF: StaticCell<can::RxBuf<10>> = StaticCell::new();

    let mut buffered_can = can.buffered(
        TX_BUF.init(can::TxBuf::<8>::new()),
        RX_BUF.init(can::RxBuf::<10>::new()),
    );

    for i in 9..12 {
        let frame = can::frame::Frame::new_extended(0x400000 + i as u32, &[i, 0xFF, 0xEE, 0xDD]).unwrap();

        // Buffered blocking write
        info!("Buffered blocking write frame {}", i);
        buffered_can.blocking_write(frame);

        // Buffered blocking read
        match buffered_can.blocking_read() {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                info!(
                    "Buffered blocking read - Rx: {} {:02x}",
                    rx_frame.header().len(),
                    rx_frame.data()[0..rx_frame.header().len() as usize],
                );
            }
            Err(err) => {
                error!("Buffered blocking read error: {}", err);
            }
        }
    }

    info!("CAN Blocking Example Completed - all CAN operations were blocking!");

    // Keep the program running (Embassy executor still needed for interrupt handling)
    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}
