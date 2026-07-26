#![no_std]
#![no_main]

use defmt::{debug, error, info};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, UART0};
use embassy_rp::uart::{Config, DataBits, InterruptHandler as UARTInterruptHandler, Parity, StopBits, Uart};
use embassy_time::{Duration, Timer, with_timeout};
use heapless::Vec;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(pub struct Irqs {
    UART0_IRQ  => UARTInterruptHandler<UART0>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

const START: u16 = 0xEF01;
const ADDRESS: u32 = 0xFFFFFFFF;

// ================================================================================

// Data package format
// Name     Length          Description
// ==========================================================================================================
// Start    2 bytes         Fixed value of 0xEF01; High byte transferred first.
// Address  4 bytes         Default value is 0xFFFFFFFF, which can be modified by command.
//                          High byte transferred first and at wrong adder value, module
//                          will reject to transfer.
// PID      1 byte          01H     Command packet;
//                          02H     Data packet; Data packet shall not appear alone in executing
//                                  processs, must follow command packet or acknowledge packet.
//                          07H     Acknowledge packet;
//                          08H     End of Data packet.
// LENGTH   2 bytes         Refers to the length of package content (command packets and data packets)
//                          plus the length of Checksum (2 bytes). Unit is byte. Max length is 256 bytes.
//                          And high byte is transferred first.
// DATA     -               It can be commands, data, command’s parameters, acknowledge result, etc.
//                          (fingerprint character value, template are all deemed as data);
// SUM      2 bytes         The arithmetic sum of package identifier, package length and all package
//                          contens. Overflowing bits are omitted. high byte is transferred first.

// ================================================================================

// Checksum is calculated on 'length (2 bytes) + data (??)'.
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
    let mut data: Vec<u8, 32> = heapless::Vec::new();

    let mut speeds: Vec<u8, 3> = heapless::Vec::new();
    let _ = speeds.push(0xC8); // Slow
    let _ = speeds.push(0x20); // Medium
    let _ = speeds.push(0x02); // Fast

    // Cycle through the three colours Red, Blue and Purple forever.
    loop {
        for colour in 1..=3 {
            for speed in &speeds {
                // Set the data first, because the length is dependent on that.
                // However, we write the length bits before we do the data.
                data.clear();
                let _ = data.push(0x01); // ctrl=Breathing light
                let _ = data.push(*speed);
                let _ = data.push(colour as u8); // colour=Red, Blue, Purple
                let _ = data.push(0x00); // times=Infinite

                // Clear buffers
                vec_buf.clear();

                // START
                let _ = vec_buf.extend_from_slice(&START.to_be_bytes()[..]);

                // ADDRESS
                let _ = vec_buf.extend_from_slice(&ADDRESS.to_be_bytes()[..]);

                // PID
                let _ = vec_buf.extend_from_slice(&[0x01]);

                // LENGTH
                let len: u16 = (1 + data.len() + 2).try_into().unwrap();
                let _ = vec_buf.extend_from_slice(&len.to_be_bytes()[..]);

                // COMMAND
                let _ = vec_buf.push(0x35); // Command: AuraLedConfig

                // DATA
                let _ = vec_buf.extend_from_slice(&data);

                // SUM
                let chk = compute_checksum(vec_buf.clone());
                let _ = vec_buf.extend_from_slice(&chk.to_be_bytes()[..]);

                // =====

                // Send command buffer.
                let data_write: [u8; 16] = vec_buf.clone().into_array().unwrap();
                debug!("  write='{:?}'", data_write[..]);
                match tx.write(&data_write).await {
                    Ok(..) => info!("Write successful."),
                    Err(e) => error!("Write error: {:?}", e),
                }

                // =====

                // Read command buffer.
                let mut read_buf: [u8; 1] = [0; 1]; // Can only read one byte at a time!
                let mut data_read: Vec<u8, 32> = heapless::Vec::new(); // Save buffer.

                info!("Attempting read.");
                loop {
                    // Some commands, like `Img2Tz()` needs longer, but we hard-code this to 200ms
                    // for this command.
                    match with_timeout(Duration::from_millis(200), rx.read(&mut read_buf)).await {
                        Ok(..) => {
                            // Extract and save read byte.
                            debug!("  r='{=u8:#04x}H' ({:03}D)", read_buf[0], read_buf[0]);
                            let _ = data_read.push(read_buf[0]).unwrap();
                        }
                        Err(..) => break, // TimeoutError -> Ignore.
                    }
                }
                info!("Read successful");
                debug!("  read='{:?}'", data_read[..]);

                Timer::after_secs(3).await;
                info!("Changing speed.");
            }

            info!("Changing colour.");
        }
    }
}
