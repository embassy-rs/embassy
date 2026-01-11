#![no_std]

#[cfg(all(feature = "sim", feature = "fpga"))]
compile_error!("Only one of `sim` or `fpga` features must be enabled.");

#[cfg(not(any(feature = "sim", feature = "fpga")))]
compile_error!("At least one of `sim` or `fpga` features must be enabled.");

/// Baud rate UART host expects.
pub const UART_BAUD: u32 = 19200;

/// Represents if the UART peripheral should enter simulation mode.
///
/// Note: In earlier versions of NEORV32 serial output would flush immediately in simulation mode,
/// but as of v1.12.6 output doesn't seem to flush at all until stop-time is reached.
#[cfg(feature = "sim")]
pub const UART_IS_SIM: bool = true;
#[cfg(feature = "fpga")]
pub const UART_IS_SIM: bool = false;

// A helpful custom panic handler for printing panic message over UART
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    use core::fmt::Write;

    let hart = riscv::register::mhartid::read();
    // SAFETY: Don't have a choice if we want to display the panic message,
    // but worst that can happen is the UART output gets corrupted
    let p = unsafe { embassy_neorv32::Peripherals::steal() };
    if let Ok(mut uart) = embassy_neorv32::uart::UartTx::new_blocking(p.UART0, UART_BAUD, UART_IS_SIM, false) {
        writeln!(
            &mut uart,
            "\n\nHART {} PANIC: {} at {}",
            hart,
            info.message(),
            info.location().unwrap()
        )
        .unwrap();
    }

    loop {
        riscv::asm::wfi();
    }
}
