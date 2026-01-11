#![no_std]
#![no_main]

use embassy_neorv32::uart::{self, UartTx};
use embassy_neorv32::{bind_interrupts, dma, peripherals};
use embassy_neorv32_examples::*;

bind_interrupts!(struct Irqs {
    UART0 => uart::InterruptHandler<peripherals::UART0>;
    DMA => dma::InterruptHandler<peripherals::DMA>;
});

const ROWS: usize = 9;
const COLS: usize = 7;
const CHARS: usize = 16;

// Ported to Rust from:
// https://github.com/stnolting/neorv32/blob/main/sw/lib/source/neorv32_aux.c#L605
// This just creates a neat NEORV32 logo :)
const LOGO: [u8; (ROWS * (1 + (COLS * CHARS))) + 1] = {
    const LOGO_RAW: [[u16; COLS]; ROWS] = [
        [0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0300, 0xc630],
        [0x60c7, 0xfc7f, 0x87f8, 0xc0c7, 0xf87f, 0x8303, 0xfffc],
        [0xf0cc, 0x00c0, 0xcc0c, 0xc0cc, 0x0cc0, 0xc30f, 0x000f],
        [0xd8cc, 0x00c0, 0xcc0c, 0xc0c0, 0x0c01, 0x8303, 0x1f8c],
        [0xcccf, 0xf8c0, 0xcff8, 0xc0c0, 0xf806, 0x030f, 0x1f8f],
        [0xc6cc, 0x00c0, 0xcc18, 0x6180, 0x0c18, 0x0303, 0x1f8c],
        [0xc3cc, 0x00c0, 0xcc0c, 0x330c, 0x0c60, 0x030f, 0x000f],
        [0xc187, 0xfc7f, 0x8c06, 0x0c07, 0xf8ff, 0xc303, 0xfffc],
        [0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0300, 0xc630],
    ];

    let mut bytes = [0; (ROWS * (1 + (COLS * CHARS))) + 1];
    let mut i = 0;
    let mut row = 0;

    while row < ROWS {
        bytes[i] = b'\n';
        i += 1;

        let mut col = 0;
        while col < COLS {
            let mut tmp = LOGO_RAW[row][col];

            let mut char = 0;
            while char < CHARS {
                bytes[i] = if (tmp as i16) < 0 { b'#' } else { b' ' };
                i += 1;
                tmp <<= 1;
                char += 1;
            }

            col += 1;
        }

        row += 1;
    }

    bytes[i] = b'\n';
    bytes
};

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_neorv32::init();

    // Setup async UART (TX only) with DMA (since the logo has a lot of data to transfer)
    let mut uart = UartTx::new_async_with_dma(p.UART0, UART_BAUD, UART_IS_SIM, false, p.DMA, Irqs)
        .expect("UART and DMA must be supported");

    uart.write(&LOGO).await.unwrap();

    // Note: '\n' seems necessary for UART writes for sim to flush output
    // Note 2: Now as of v.12.6 UART TX doesn't seem to flush at all until simulation reaches its stop-time :(
    // So if in simulation mode, need to wait until stop time is reached before any output will be visual
    uart.write(b"Hello world! :)\n").await.unwrap();
}
