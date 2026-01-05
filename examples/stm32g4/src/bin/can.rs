#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::*;
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, can};
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<FDCAN1>;
});

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

    let use_fd = false;

    // 1M bps
    if use_fd {
        can.set_fd_data_bitrate(1_000_000, false);
    }

    info!("Configured");

    let mut can = can.start(match use_fd {
        true => can::OperatingMode::InternalLoopbackMode,
        false => can::OperatingMode::NormalOperationMode,
    });

    let mut i = 0;
    let mut last_read_ts = embassy_time::Instant::now();

    loop {
        let frame = can::frame::Frame::new_extended(0x123456F, &[i; 8]).unwrap();
        info!("Writing frame");

        _ = can.write(&frame).await;

        match can.read().await {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;
                info!(
                    "Rx: {} {:02x} --- {}ms",
                    rx_frame.header().len(),
                    rx_frame.data()[0..rx_frame.header().len() as usize],
                    delta,
                )
            }
            Err(_err) => error!("Error in frame"),
        }

        Timer::after_millis(250).await;

        i += 1;
        if i > 2 {
            break;
        }
    }

    // Use the FD API's even if we don't get FD packets.

    loop {
        if use_fd {
            let frame = can::frame::FdFrame::new_extended(0x123456F, &[i; 16]).unwrap();
            info!("Writing frame using FD API");
            _ = can.write_fd(&frame).await;
        } else {
            let frame = can::frame::FdFrame::new_extended(0x123456F, &[i; 8]).unwrap();
            info!("Writing frame using FD API");
            _ = can.write_fd(&frame).await;
        }

        match can.read_fd().await {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;
                info!(
                    "Rx: {} {:02x} --- using FD API {}ms",
                    rx_frame.header().len(),
                    rx_frame.data()[0..rx_frame.header().len() as usize],
                    delta,
                )
            }
            Err(_err) => error!("Error in frame"),
        }

        Timer::after_millis(250).await;

        i += 1;
        if i > 4 {
            break;
        }
    }
    i = 0;
    let (mut tx, mut rx, _props) = can.split();
    // With split
    loop {
        let frame = can::frame::Frame::new_extended(0x123456F, &[i; 8]).unwrap();
        info!("Writing frame");
        _ = tx.write(&frame).await;

        match rx.read().await {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;
                info!(
                    "Rx: {} {:02x} --- {}ms",
                    rx_frame.header().len(),
                    rx_frame.data()[0..rx_frame.header().len() as usize],
                    delta,
                )
            }
            Err(_err) => error!("Error in frame"),
        }

        Timer::after_millis(250).await;

        i += 1;

        if i > 2 {
            break;
        }
    }

    let can = can::Can::join(tx, rx);

    info!("\n\n\nBuffered\n");
    if use_fd {
        static TX_BUF: StaticCell<can::TxFdBuf<8>> = StaticCell::new();
        static RX_BUF: StaticCell<can::RxFdBuf<10>> = StaticCell::new();
        let mut can = can.buffered_fd(
            TX_BUF.init(can::TxFdBuf::<8>::new()),
            RX_BUF.init(can::RxFdBuf::<10>::new()),
        );
        loop {
            let frame = can::frame::FdFrame::new_extended(0x123456F, &[i; 16]).unwrap();
            info!("Writing frame");

            _ = can.write(frame).await;

            match can.read().await {
                Ok(envelope) => {
                    let (ts, rx_frame) = (envelope.ts, envelope.frame);
                    let delta = (ts - last_read_ts).as_millis();
                    last_read_ts = ts;
                    info!(
                        "Rx: {} {:02x} --- {}ms",
                        rx_frame.header().len(),
                        rx_frame.data()[0..rx_frame.header().len() as usize],
                        delta,
                    )
                }
                Err(_err) => error!("Error in frame"),
            }

            Timer::after_millis(250).await;

            i = i.wrapping_add(1);
        }
    } else {
        static TX_BUF: StaticCell<can::TxBuf<8>> = StaticCell::new();
        static RX_BUF: StaticCell<can::RxBuf<10>> = StaticCell::new();
        let mut can = can.buffered(
            TX_BUF.init(can::TxBuf::<8>::new()),
            RX_BUF.init(can::RxBuf::<10>::new()),
        );
        loop {
            let frame = can::frame::Frame::new_extended(0x123456F, &[i; 8]).unwrap();
            info!("Writing frame");

            // You can use any of these approaches to send. The writer makes it
            // easy to share sending from multiple tasks.
            //_ = can.write(frame).await;
            //can.writer().try_write(frame).unwrap();
            can.writer().write(frame).await;

            match can.read().await {
                Ok(envelope) => {
                    let (ts, rx_frame) = (envelope.ts, envelope.frame);
                    let delta = (ts - last_read_ts).as_millis();
                    last_read_ts = ts;
                    info!(
                        "Rx: {} {:02x} --- {}ms",
                        rx_frame.header().len(),
                        rx_frame.data()[0..rx_frame.header().len() as usize],
                        delta,
                    )
                }
                Err(_err) => error!("Error in frame"),
            }

            Timer::after_millis(250).await;

            i = i.wrapping_add(1);
        }
    }
}
