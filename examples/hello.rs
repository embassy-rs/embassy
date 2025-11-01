#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa276 as hal;
use hal::uart;

mod common;

#[cfg(all(feature = "defmt", feature = "defmt-rtt"))]
use defmt_rtt as _;
#[cfg(feature = "defmt")]
use panic_probe as _;
#[cfg(all(feature = "defmt", feature = "defmt-rtt"))]
use rtt_target as _;

/// Simple helper to write a byte as hex to UART
fn write_hex_byte(uart: &hal::uart::Uart<hal::uart::Lpuart2>, byte: u8) {
    const HEX_DIGITS: &[u8] = b"0123456789ABCDEF";
    uart.write_byte(HEX_DIGITS[(byte >> 4) as usize]);
    uart.write_byte(HEX_DIGITS[(byte & 0xF) as usize]);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    #[cfg(feature = "defmt")]
    defmt::info!("boot");

    // Board-level init for UART2 clocks and pins.
    unsafe {
        common::init_uart2(hal::pac());
    }

    // Get UART source frequency from clock configuration
    // Using hardcoded frequency for now - dynamic detection may have issues
    let src = 12_000_000; // FRO_LF_DIV at 12MHz with DIV=0
    let uart = uart::Uart::<uart::Lpuart2>::new(p.LPUART2, uart::Config::new(src));

    // Print welcome message before any async delays to guarantee early console output
    uart.write_str_blocking("\r\n=== MCXA276 UART Echo Demo ===\r\n");
    uart.write_str_blocking("Available commands:\r\n");
    uart.write_str_blocking("  help     - Show this help\r\n");
    uart.write_str_blocking("  echo <text> - Echo back the text\r\n");
    uart.write_str_blocking("  hex <byte> - Display byte in hex (0-255)\r\n");
    uart.write_str_blocking("Type a command: ");

    let mut buffer = [0u8; 64];
    let mut buf_idx = 0;

    loop {
        // Read a byte from UART
        let byte = uart.read_byte_blocking();

        // Echo the character back
        if byte == b'\r' || byte == b'\n' {
            // Enter pressed - process command
            uart.write_str_blocking("\r\n");

            if buf_idx > 0 {
                let command = &buffer[0..buf_idx];

                if command == b"help" {
                    uart.write_str_blocking("Available commands:\r\n");
                    uart.write_str_blocking("  help     - Show this help\r\n");
                    uart.write_str_blocking("  echo <text> - Echo back the text\r\n");
                    uart.write_str_blocking("  hex <byte> - Display byte in hex (0-255)\r\n");
                } else if command.starts_with(b"echo ") && command.len() > 5 {
                    uart.write_str_blocking("Echo: ");
                    uart.write_str_blocking(core::str::from_utf8(&command[5..]).unwrap_or(""));
                    uart.write_str_blocking("\r\n");
                } else if command.starts_with(b"hex ") && command.len() > 4 {
                    // Parse the byte value
                    let num_str = &command[4..];
                    if let Ok(num) = parse_u8(num_str) {
                        uart.write_str_blocking("Hex: 0x");
                        write_hex_byte(&uart, num);
                        uart.write_str_blocking("\r\n");
                    } else {
                        uart.write_str_blocking("Invalid number for hex command\r\n");
                    }
                } else if command.len() > 0 {
                    uart.write_str_blocking("Unknown command: ");
                    uart.write_str_blocking(core::str::from_utf8(command).unwrap_or(""));
                    uart.write_str_blocking("\r\n");
                }
            }

            // Reset buffer and prompt
            buf_idx = 0;
            uart.write_str_blocking("Type a command: ");
        } else if byte == 8 || byte == 127 {
            // Backspace
            if buf_idx > 0 {
                buf_idx -= 1;
                uart.write_str_blocking("\x08 \x08"); // Erase character
            }
        } else if buf_idx < buffer.len() - 1 {
            // Regular character
            buffer[buf_idx] = byte;
            buf_idx += 1;
            uart.write_byte(byte);
        }
    }
}

/// Simple parser for u8 from ASCII bytes
fn parse_u8(bytes: &[u8]) -> Result<u8, ()> {
    let mut result = 0u8;
    for &b in bytes {
        if b >= b'0' && b <= b'9' {
            result = result.checked_mul(10).ok_or(())?;
            result = result.checked_add(b - b'0').ok_or(())?;
        } else {
            return Err(());
        }
    }
    Ok(result)
}

#[cfg(not(feature = "defmt"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
