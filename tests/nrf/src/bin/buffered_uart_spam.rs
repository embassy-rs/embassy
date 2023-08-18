#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
teleprobe_meta::target!(b"nrf52840-dk");

use core::mem;
use core::ptr::NonNull;

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_nrf::buffered_uarte::{self, BufferedUarte};
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::ppi::{Event, Ppi, Task};
use embassy_nrf::uarte::Uarte;
use embassy_nrf::{bind_interrupts, pac, peripherals, uarte};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UARTE0_UART0 => buffered_uarte::InterruptHandler<peripherals::UARTE0>;
    UARTE1 => uarte::InterruptHandler<peripherals::UARTE1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD1M;

    let mut tx_buffer = [0u8; 1024];
    let mut rx_buffer = [0u8; 1024];

    mem::forget(Output::new(&mut p.P1_02, Level::High, OutputDrive::Standard));

    let mut u = BufferedUarte::new(
        p.UARTE0,
        p.TIMER0,
        p.PPI_CH0,
        p.PPI_CH1,
        p.PPI_GROUP0,
        Irqs,
        p.P1_03,
        p.P1_04,
        config.clone(),
        &mut rx_buffer,
        &mut tx_buffer,
    );

    info!("uarte initialized!");

    // uarte needs some quiet time to start rxing properly.
    Timer::after(Duration::from_millis(10)).await;

    // Tx spam in a loop.
    const NSPAM: usize = 17;
    static mut TX_BUF: [u8; NSPAM] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let _spam = Uarte::new(p.UARTE1, Irqs, p.P1_01, p.P1_02, config.clone());
    let spam_peri: pac::UARTE1 = unsafe { mem::transmute(()) };
    let event = unsafe { Event::new_unchecked(NonNull::new_unchecked(&spam_peri.events_endtx as *const _ as _)) };
    let task = unsafe { Task::new_unchecked(NonNull::new_unchecked(&spam_peri.tasks_starttx as *const _ as _)) };
    let mut spam_ppi = Ppi::new_one_to_one(p.PPI_CH2, event, task);
    spam_ppi.enable();
    let p = unsafe { TX_BUF.as_mut_ptr() };
    spam_peri.txd.ptr.write(|w| unsafe { w.ptr().bits(p as u32) });
    spam_peri.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(NSPAM as _) });
    spam_peri.tasks_starttx.write(|w| unsafe { w.bits(1) });

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
