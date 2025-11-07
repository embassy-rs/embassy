#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa276 as hal;
use hal::uart;

mod common;

use embassy_mcxa276::bind_interrupts;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

// Bind only OS_EVENT, and retain the symbol explicitly so it can’t be GC’ed.
bind_interrupts!(struct Irqs {
    OS_EVENT => hal::ostimer::time_driver::OsEventHandler;
});

#[used]
#[no_mangle]
static KEEP_OS_EVENT: unsafe extern "C" fn() = OS_EVENT;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = hal::init(hal::config::Config::default());

    // Enable/clock OSTIMER0 and UART2 before touching their registers
    unsafe {
        common::init_ostimer0(hal::pac());
    }
    unsafe {
        common::init_uart2(hal::pac());
    }
    let src = unsafe { hal::clocks::uart2_src_hz(hal::pac()) };
    let uart = uart::Uart::<uart::Lpuart2>::new(_p.LPUART2, uart::Config::new(src));
    uart.write_str_blocking("boot\n");

    // Avoid mass NVIC writes here; DefaultHandler now safely returns.

    // Initialize embassy-time global driver backed by OSTIMER0 (re-enables OS_EVENT with priority)
    // The bind_interrupts! macro handles handler binding automatically

    // Initialize OSTIMER with default 1MHz frequency
    // Adjust this value to match your actual OSTIMER clock frequency
    hal::ostimer::time_driver::init(
        hal::config::Config::default().time_interrupt_priority,
        1_000_000,
    );

    // Removed force-pend; rely on real hardware match to trigger OS_EVENT.

    // Log using defmt if enabled
    defmt::info!("OSTIMER async example starting...");

    loop {
        defmt::info!("tick");
        uart.write_str_blocking("tick\n");
        Timer::after(Duration::from_millis(1000)).await;
    }
}
