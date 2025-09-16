// required-features: two-uarts
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use core::mem;
use core::ptr::NonNull;

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_nrf::buffered_uarte::{self, BufferedUarteRx};
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::ppi::{Event, Ppi, Task};
use embassy_nrf::uarte::UarteTx;
use embassy_nrf::{pac, peripherals, uarte};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD1M;

    let mut rx_buffer = [0u8; 1024];

    mem::forget(Output::new(
        peri!(p, PIN_A).reborrow(),
        Level::High,
        OutputDrive::Standard,
    ));

    let mut u = BufferedUarteRx::new(
        peri!(p, UART0),
        p.TIMER0,
        p.PPI_CH0,
        p.PPI_CH1,
        p.PPI_GROUP0,
        irqs!(UART0_BUFFERED),
        peri!(p, PIN_B),
        config.clone(),
        &mut rx_buffer,
    );

    info!("uarte initialized!");

    // uarte needs some quiet time to start rxing properly.
    Timer::after_millis(10).await;

    // Tx spam in a loop.
    const NSPAM: usize = 17;
    static mut TX_BUF: [u8; NSPAM] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let _spam = UarteTx::new(peri!(p, UART1), irqs!(UART1), peri!(p, PIN_A), config.clone());
    let spam_peri = pac::UARTE1;
    let event = unsafe { Event::new_unchecked(NonNull::new_unchecked(spam_peri.events_endtx().as_ptr())) };
    let task = unsafe { Task::new_unchecked(NonNull::new_unchecked(spam_peri.tasks_starttx().as_ptr())) };
    let mut spam_ppi = Ppi::new_one_to_one(p.PPI_CH2, event, task);
    spam_ppi.enable();
    let p = (&raw mut TX_BUF) as *mut u8;
    spam_peri.txd().ptr().write_value(p as u32);
    spam_peri.txd().maxcnt().write(|w| w.set_maxcnt(NSPAM as _));
    spam_peri.tasks_starttx().write_value(1);

    let mut i = 0;
    let mut total = 0;
    while total < 256 * 1024 {
        let buf = unwrap!(u.fill_buf().await);
        //info!("rx {}", buf);

        for &b in buf {
            assert_eq!(b, unsafe { TX_BUF[i] });

            i = i + 1;
            if i == NSPAM {
                i = 0;
            }
        }

        // Read bytes have to be explicitly consumed, otherwise fill_buf() will return them again
        let n = buf.len();
        u.consume(n);
        total += n;
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
