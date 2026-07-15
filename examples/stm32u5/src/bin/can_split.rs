#![no_std]
#![no_main]

// CAN example for the STM32U5 using split() to drive TX and RX from separate tasks.
// Tested on a NUCLEO-U545RE-Q; builds under this directory's stm32u5g9zj feature.
use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::can::{CanConfigurator, CanRx, CanTx, Frame, IT0InterruptHandler, IT1InterruptHandler, filter};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

// FDCAN1 has 2 interrupt lines which must be bound
// Manual: rm0456, pg 3023/3651
bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => IT0InterruptHandler<peripherals::FDCAN1>;
    FDCAN1_IT1 => IT1InterruptHandler<peripherals::FDCAN1>;
});

#[embassy_executor::task]
async fn writer(mut tx: CanTx<'static>) {
    let mut i = 0;
    let start = Instant::now();
    loop {
        info!("Writing frame...");
        // Arbitrary 29-bit extended ID. On a real bus the ID also sets priority (lower = higher).
        let frame = Frame::new_extended(0x123456F, &[i; 8]).unwrap();
        tx.write(&frame).await;
        let elapsed_ms = start.elapsed().as_millis(); // Time since the task has started
        info!("tx: Sent frame {} at: {} ms", i, elapsed_ms);

        i += 1;
        if i > 3 {
            break;
        }
        // Keep the task from blocking the MCU
        Timer::after_millis(250).await;
    }
}

#[embassy_executor::task]
async fn reader(mut rx: CanRx<'static>) {
    loop {
        match rx.read().await {
            Ok(envelope) => info!("Rx: Received {:02x}", envelope.frame.data()), // Slice of the valid payload
            Err(_err) => error!("rx error"),
        }

        // Keep the task from blocking the MCU
        Timer::after_millis(250).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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

    // Accept every frame into the RX FIFO. Without a filter, the peripheral discards
    // all incoming frames, so read() would block forever.
    can.properties().set_extended_filter(
        filter::ExtendedFilterSlot::_0,
        filter::ExtendedFilter::accept_all_into_fifo1(),
    );

    // Internal loopback: TX is looped to RX inside the chip.
    // External loopback: TX is driven on the pin and looped back through a transceiver
    // Normal mode: needs a transceiver AND a second node to ACK.
    let can = can.into_internal_loopback_mode();
    info!("CAN configured !");

    // if read and write capabilities need to be divided across tasks, use .split() to get owned values that can be passed to tasks
    // split() hands ownership of TX and RX to separate tasks, so neither needs a Mutex to share one Can — the point of the function.

    let (tx, rx, _properties) = can.split();

    spawner.spawn(writer(tx).unwrap());
    spawner.spawn(reader(rx).unwrap());

    // Keep main alive while tasks are polled
    loop {
        Timer::after_millis(100).await;
    }
}
