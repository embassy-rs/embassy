#![no_std]
#![no_main]

use embassy_neorv32::dma::{self, Dma};
use embassy_neorv32::uart::UartTx;
use embassy_neorv32::{bind_interrupts, peripherals};
use embassy_neorv32_examples::*;

bind_interrupts!(struct Irqs {
    DMA => dma::InterruptHandler<peripherals::DMA>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_neorv32::init();

    let mut uart = UartTx::new_blocking(p.UART0, UART_BAUD, UART_IS_SIM, false).expect("UART must be supported");

    // Note: DMA is single-channel only, so only one driver can own it.
    // Typically you would instantiate the DMA driver instance directly like this
    // only when you want to perform memory-to-memory transfers.
    //
    // If you want peripheral drivers (such as UART) to use DMA, you would pass `p.DMA` to their
    // constructors.
    let mut dma = Dma::new(p.DMA, Irqs).expect("DMA must be supported");

    let src = [0xAAu8; 1024];
    let mut dst = [0xFFu8; 1024];

    let res = dma.copy(&src, &mut dst, false).await;
    match res {
        Ok(()) if src == dst => uart.blocking_write(b"DMA transfer succeeded\n"),
        Err(_) => uart.blocking_write(b"DMA transfer encountered an error\n"),
        _ => uart.blocking_write(b"DMA transfer failed\n"),
    }
}
