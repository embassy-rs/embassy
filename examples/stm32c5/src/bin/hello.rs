#![no_main]
#![no_std]

// Some panic handler needs to be included. This one halts the processor on panic.
use cortex_m_rt::entry;
use {defmt_rtt as _, panic_probe as _};

// Use `main` as the entry point of this application, which may not return.
#[entry]
fn main() -> ! {
    defmt::println!("Hello world! I am an STM32C5");

    loop {}
}
