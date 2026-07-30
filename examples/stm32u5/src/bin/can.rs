#![no_std]
#![no_main]

// CAN example for the STM32U5, in internal loopback mode.
// Tested on a NUCLEO-U545RE-Q; builds under this directory's stm32u5g9zj feature.

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::can::{CanConfigurator, Frame, IT0InterruptHandler, IT1InterruptHandler, filter};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_time::Timer;
use panic_probe as _;

// FDCAN1 has 2 interrupt lines which must be bound

// Manual: rm0456, pg 3023/3651
bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => IT0InterruptHandler<peripherals::FDCAN1>;
    FDCAN1_IT1 => IT1InterruptHandler<peripherals::FDCAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        // The HSE crystal is not populated on the MB1841 board (see UM3062), so the system
        // and FDCAN are clocked from the internal 16 MHz HSI via PLL1
        config.rcc.hsi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div1,  // 16 MHz
            mul: PllMul::Mul10,       // VCO = 16 × 10 = 160 MHz
            divp: None,               // unused
            divq: Some(PllDiv::Div4), // 40 MHz FDCAN kernel clock
            divr: Some(PllDiv::Div1), // 160 MHz for the CPU
        });

        config.rcc.sys = Sysclk::Pll1R;
        config.rcc.mux.fdcan1sel = mux::Fdcansel::Pll1Q;
    }

    let p = embassy_stm32::init(config);
    info!("Device started");

    let mut can = CanConfigurator::new(p.FDCAN1, p.PA11, p.PA12, Irqs);

    // Standard CAN bit rate. Every node on a bus must use the same one —
    // 125k / 250k / 500k / 1M are the common choices.
    can.set_bitrate(250_000);

    can.properties().set_extended_filter(
        filter::ExtendedFilterSlot::_0,
        filter::ExtendedFilter::accept_all_into_fifo1(),
    );

    // Internal loopback: TX is looped to RX inside the chip.
    // External loopback: TX is driven on the pin and looped back through a transceiver
    // Normal mode: needs a transceiver AND a second node to ACK.
    let mut can = can.into_internal_loopback_mode();
    info!("CAN configured !");

    let mut i = 0;
    let mut last_read_ts = embassy_time::Instant::now();

    loop {
        info!("Writing frame...");
        // Arbitrary 29-bit extended ID. On a real bus the ID also sets priority (lower = higher).
        let frame = Frame::new_extended(0x123456F, &[i; 8]).unwrap();
        can.write(&frame).await;

        match can.read().await {
            Ok(envelope) => {
                let (ts, rx_frame) = (envelope.ts, envelope.frame);
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;

                info!(
                    "Rx: {} {:02x} --- {}ms",
                    rx_frame.header().len(),                              // Number of data bytes
                    rx_frame.data()[0..rx_frame.header().len() as usize], // Slice of the valid payload
                    delta,
                )
            }
            Err(err) => error!("Error in frame: {}", err),
        }

        Timer::after_millis(250).await;

        i += 1;
        if i > 2 {
            break;
        }
    }
}
