#![no_std]
#![no_main]

use defmt::{debug, info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{Async, Config, DataBits, InterruptHandler as UARTInterruptHandler, StopBits, Uart, UartRx};
use embassy_time::Timer;
use heapless::Vec;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(pub struct Irqs {
    UART0_IRQ  => UARTInterruptHandler<UART0>;	// Fingerprint scanner (TX)
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

// NOTE: Doesn't work for some reason, it just hangs!
#[embassy_executor::task]
async fn reader(mut rx: UartRx<'static, UART0, Async>) {
    loop {
        let mut buf = [0; 32];
        debug!("Attempting read..");

        //rx.read(&mut buf).await.unwrap();
        match rx.read(&mut buf).await {
            Ok(v) => info!("Read successful: {:?}", v),
            Err(e) => info!("Read error: {:?}", e),
        }
        info!("RX='{:?}'", buf);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");

    let p = embassy_rp::init(Default::default());

    // Initialize the fingerprint scanner.
    let mut config = Config::default();
    config.baudrate = 57600;
    config.stop_bits = StopBits::STOP1;
    config.data_bits = DataBits::DataBits8;

    let (uart, tx_pin, tx_dma, rx_pin, rx_dma) = (p.UART0, p.PIN_16, p.DMA_CH0, p.PIN_17, p.DMA_CH1);
    let uart = Uart::new(uart, tx_pin, rx_pin, Irqs, tx_dma, rx_dma, config);
    let (mut tx, rx) = uart.split();

    unwrap!(spawner.spawn(reader(rx)));
    Timer::after_secs(1).await;

    let mut vec_buf: Vec<u8, 32> = heapless::Vec::new();
    {
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
        let _ = vec_buf.push(0x02); // colour=Blue
        let _ = vec_buf.push(0x00); // times=Infinite

        // SUM
        let chk = compute_checksum(vec_buf.clone());
        let _ = write_cmd_bytes(&mut vec_buf, &chk.to_be_bytes()[..]);

        // =====

        // Send command buffer.
        let data: [u8; 16] = vec_buf.clone().into_array().unwrap();
        debug!("data='{:?}'", data);
        match tx.write(&data).await {
            Ok(..) => info!("Write successful"),
            Err(e) => info!("Write error: {:?}", e),
        }
    }
}
