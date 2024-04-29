#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{Config, DataBits, InterruptHandler as UARTInterruptHandler, Parity, StopBits, Uart};
use embassy_time::Timer;
use heapless::Vec;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(pub struct Irqs {
    UART0_IRQ  => UARTInterruptHandler<UART0>;
});

const ADDRESS: u32 = 0xFFFFFFFF;
const START: u16 = 0xEF01;

// ================================================================================

fn write_cmd_bytes(buf: &mut Vec<u8, 32>, bytes: &[u8]) {
    let _ = buf.extend_from_slice(bytes);
}

fn compute_checksum(buf: Vec<u8, 32>) -> u16 {
    let mut checksum = 0u16;

    let check_end = buf.len();
    let checked_bytes = &buf[6..check_end];
    for byte in checked_bytes {
        checksum += (*byte) as u16;
    }
    return checksum;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start");

    let p = embassy_rp::init(Default::default());

    // Initialize the fingerprint scanner.
    let mut config = Config::default();
    config.baudrate = 57600;
    config.stop_bits = StopBits::STOP1;
    config.data_bits = DataBits::DataBits8;
    config.parity = Parity::ParityNone;

    let (uart, tx_pin, tx_dma, rx_pin, rx_dma) = (p.UART0, p.PIN_16, p.DMA_CH0, p.PIN_17, p.DMA_CH1);
    let uart = Uart::new(uart, tx_pin, rx_pin, Irqs, tx_dma, rx_dma, config);
    let (mut tx, mut rx) = uart.split();

    let mut vec_buf: Vec<u8, 32> = heapless::Vec::new();

    // Cycle through the three colours Red, Blue and Purple.
    for colour in 1..=3 {
        // Clear buffers
        vec_buf.clear();

        // START
        let _ = write_cmd_bytes(&mut vec_buf, &START.to_be_bytes()[..]);

        // ADDRESS
        let _ = write_cmd_bytes(&mut vec_buf, &ADDRESS.to_be_bytes()[..]);

        // PID
        let _ = vec_buf.push(0x01);

        // LENGTH
        let len = <usize as TryInto<u16>>::try_into(vec_buf.len()).unwrap() as u16;
        let _ = write_cmd_bytes(&mut vec_buf, &len.to_be_bytes()[..]);

        // COMMAND
        let _ = vec_buf.push(0x35); // AuraLedConfig

        // DATA
        let _ = vec_buf.push(0x01); // ctrl=Breathing light
        let _ = vec_buf.push(0x50); // speed=80
        let _ = vec_buf.push(colour as u8); // colour=Red, Blue, Purple
        let _ = vec_buf.push(0x00); // times=Infinite

        // SUM
        let chk = compute_checksum(vec_buf.clone());
        let _ = write_cmd_bytes(&mut vec_buf, &chk.to_be_bytes()[..]);

        // =====

        // Send command buffer.
        let data_write: [u8; 16] = vec_buf.clone().into_array().unwrap();
        info!("write ({})='{:?}'", colour, data_write);
        match tx.write(&data_write).await {
            Ok(..) => info!("Write successful."),
            Err(e) => info!("Write error: {:?}", e),
        }

        // =====

        // Read command buffer.
        let mut read_buf: [u8; 1] = [0; 1]; // Can only read one byte at a time!
        let mut data_read: Vec<u8, 32> = heapless::Vec::new(); // Return buffer.
        let mut cnt: u8 = 0; // Keep track of how many packages we've received.

        info!("Attempting read.");
        loop {
            match rx.read(&mut read_buf).await {
                Ok(..) => (),
                Err(e) => info!("  Read error: {:?}", e),
            }

            match cnt {
                _ => data_read.push(read_buf[0]).unwrap(),
            }

            if cnt > 10 {
                info!("read ({})='{:?}'", colour, data_read[..]);
                break;
            }

            cnt = cnt + 1;
        }

        // =====

        if colour != 3 {
            Timer::after_secs(2).await;
            info!("Changing colour.");
        }
    }

    info!("All done..");
}
