#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa276 as hal;
use hal::interrupt::typelevel::Handler;
use hal::uart;

mod common;

use embassy_mcxa276::bind_interrupts;
use {defmt_rtt as _, panic_probe as _};

// Bind LPUART2 interrupt to our handler
bind_interrupts!(struct Irqs {
    LPUART2 => hal::uart::UartInterruptHandler;
});

#[used]
#[no_mangle]
static KEEP_LPUART2: unsafe extern "C" fn() = LPUART2;

// Wrapper function for the interrupt handler
unsafe extern "C" fn lpuart2_handler() {
    hal::uart::UartInterruptHandler::on_interrupt();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = hal::init(hal::config::Config::default());

    // Enable/clock UART2 before touching its registers
    unsafe {
        common::init_uart2(hal::pac());
    }
    let src = unsafe { hal::clocks::uart2_src_hz(hal::pac()) };
    let uart = uart::Uart::<uart::Lpuart2>::new(_p.LPUART2, uart::Config::new(src));

    // Configure LPUART2 interrupt for UART operation BEFORE any UART usage
    hal::interrupt::LPUART2.configure_for_uart(hal::interrupt::Priority::from(3));

    // Manually install the interrupt handler and enable RX IRQs in the peripheral
    unsafe {
        hal::interrupt::LPUART2.install_handler(lpuart2_handler);
        // Enable RX interrupts so the handler actually fires on incoming bytes
        uart.enable_rx_interrupts();
    }

    // Print welcome message
    uart.write_str_blocking("UART interrupt echo demo starting...\r\n");
    uart.write_str_blocking("Type characters to echo them back.\r\n");

    // Log using defmt if enabled
    defmt::info!("UART interrupt echo demo starting...");

    loop {
        // Check if we have received any data
        if uart.rx_data_available() {
            if let Some(byte) = uart.try_read_byte() {
                // Echo it back
                uart.write_byte(byte);
                uart.write_str_blocking(" (received)\r\n");
            }
        } else {
            // No data available, wait a bit before checking again
            cortex_m::asm::delay(12_000_000); // ~1 second at 12MHz
        }
    }
}
