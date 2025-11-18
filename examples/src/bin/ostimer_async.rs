#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa_examples::init_uart2_pins;
use embassy_time::{Duration, Timer};
use hal::lpuart::{Config, Lpuart};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Bind only OS_EVENT, and retain the symbol explicitly so it can’t be GC’ed.
bind_interrupts!(struct Irqs {
    OS_EVENT => hal::ostimer::time_driver::OsEventHandler;
});

#[used]
#[no_mangle]
static KEEP_OS_EVENT: unsafe extern "C" fn() = OS_EVENT;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    // Create UART configuration
    let config = Config {
        baudrate_bps: 115_200,
        enable_tx: true,
        enable_rx: true,
        ..Default::default()
    };

    // Create UART instance using LPUART2 with P2_2 as TX and P2_3 as RX
    unsafe {
        init_uart2_pins(hal::pac());
    }
    let mut uart = Lpuart::new_blocking(
        p.LPUART2, // Peripheral
        p.P2_2,    // TX pin
        p.P2_3,    // RX pin
        config,
    )
    .unwrap();
    uart.blocking_write(b"boot\n").unwrap();

    // Avoid mass NVIC writes here; DefaultHandler now safely returns.

    // Initialize embassy-time global driver backed by OSTIMER0 (re-enables OS_EVENT with priority)
    // The bind_interrupts! macro handles handler binding automatically

    // Initialize OSTIMER with default 1MHz frequency
    // Adjust this value to match your actual OSTIMER clock frequency
    hal::ostimer::time_driver::init(hal::config::Config::default().time_interrupt_priority, 1_000_000);

    // Removed force-pend; rely on real hardware match to trigger OS_EVENT.

    // Log using defmt if enabled
    defmt::info!("OSTIMER async example starting...");

    loop {
        defmt::info!("tick");
        uart.write_str_blocking("tick\n");
        Timer::after(Duration::from_millis(1000)).await;
    }
}
