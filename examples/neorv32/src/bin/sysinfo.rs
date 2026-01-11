#![no_std]
#![no_main]

use core::fmt::Write;

use embassy_neorv32::sysinfo::SysInfo;
use embassy_neorv32::uart::UartTx;
use embassy_neorv32_examples::*;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_neorv32::init();

    let mut uart = UartTx::new_blocking(p.UART0, UART_BAUD, UART_IS_SIM, false).expect("UART must be supported");

    // Print clock frequency
    writeln!(&mut uart, "Clock frequency: {} MHz", SysInfo::clock_freq() / 1_000_000).unwrap();

    // Print memory sizes
    writeln!(&mut uart, "IMEM size: {} KiB", SysInfo::imem_size() / 1024).unwrap();
    writeln!(&mut uart, "DMEM size: {} KiB", SysInfo::dmem_size() / 1024).unwrap();

    // Print misc info
    writeln!(&mut uart, "Num harts: {}", SysInfo::num_harts()).unwrap();
    writeln!(&mut uart, "Boot mode: {:?}", SysInfo::boot_mode()).unwrap();
    writeln!(&mut uart, "Internal bus timeout cycles: {}", SysInfo::bus_itmo_cycles()).unwrap();
    writeln!(&mut uart, "External bus timeout cycles: {}", SysInfo::bus_etmo_cycles()).unwrap();

    // Retrieve SoC Config
    let soc_config = SysInfo::soc_config();

    // Print processor features
    uart.blocking_write(b"\nProcessor Features:\n");
    if soc_config.has_imem() {
        uart.blocking_write(b"Internal IMEM\n");
    }
    if soc_config.has_dmem() {
        uart.blocking_write(b"Internal DMEM\n");
    }
    if soc_config.has_icache() {
        uart.blocking_write(b"Internal ICACHE\n");
    }
    if soc_config.has_dcache() {
        uart.blocking_write(b"Internal DCACHE\n");
    }
    if soc_config.has_imem_as_rom() {
        uart.blocking_write(b"Internal IMEM as pre-initialized ROM\n");
    }
    if soc_config.has_bootloader() {
        uart.blocking_write(b"Internal bootloader\n");
    }
    if soc_config.has_xbus() {
        uart.blocking_write(b"External bus interface (XBUS)\n");
    }
    if soc_config.has_ocd() {
        uart.blocking_write(b"On-chip debugger\n");
    }
    if soc_config.has_ocd_auth() {
        uart.blocking_write(b"On-chip debugger authentication\n");
    }

    // Print supported peripherals
    uart.blocking_write(b"\nPeripherals Supported:\n");
    if soc_config.has_uart0() {
        uart.blocking_write(b"UART0\n");
    }
    if soc_config.has_uart1() {
        uart.blocking_write(b"UART1\n");
    }
    if soc_config.has_twi() {
        uart.blocking_write(b"TWI\n");
    }
    if soc_config.has_twd() {
        uart.blocking_write(b"TWD\n");
    }
    if soc_config.has_spi() {
        uart.blocking_write(b"SPI\n");
    }
    if soc_config.has_sdi() {
        uart.blocking_write(b"SDI\n");
    }
    if soc_config.has_gptmr() {
        uart.blocking_write(b"GPTMR\n");
    }
    if soc_config.has_gpio() {
        uart.blocking_write(b"GPIO\n");
    }
    if soc_config.has_pwm() {
        uart.blocking_write(b"PWM\n");
    }
    if soc_config.has_wdt() {
        uart.blocking_write(b"WDT\n");
    }
    if soc_config.has_dma() {
        uart.blocking_write(b"DMA\n");
    }
    if soc_config.has_trng() {
        uart.blocking_write(b"TRNG\n");
    }
    if soc_config.has_onewire() {
        uart.blocking_write(b"ONEWIRE\n");
    }
    if soc_config.has_neoled() {
        uart.blocking_write(b"NEOLED\n");
    }
    if soc_config.has_tracer() {
        uart.blocking_write(b"TRACER\n");
    }
    if soc_config.has_slink() {
        uart.blocking_write(b"SLINK\n");
    }
    if soc_config.has_clint() {
        uart.blocking_write(b"CLINT\n");
    }
    if soc_config.has_cfs() {
        uart.blocking_write(b"CFS\n");
    }

    // Are we in a simulation?
    if soc_config.is_simulation() {
        uart.blocking_write(b"\nThe matrix has you.\n");
    }
}
