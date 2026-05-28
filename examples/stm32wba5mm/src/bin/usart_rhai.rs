#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use embedded_alloc::Heap;

use core::str;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, peripherals,
    usart::{self, Config, Uart},
};
use {defmt_rtt as _, panic_probe as _};

use rhai::{packages::Package, packages::BasicMathPackage, Dynamic, Engine};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 92 * 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    info!("Starting system");

    let mut config1 = Config::default();
    config1.baudrate = 115600;

    //RX/TX connected to USB/UART VCP of ST-Link
    let mut usart = Uart::new_blocking(p.USART1, p.PA8, p.PB12, config1).unwrap();

    let mut engine = Engine::new_raw();
    let package = BasicMathPackage::new();

    // Register the package into the 'Engine'.
    package.register_into_engine(&mut engine);

    // this bit gets commented out to test size without Core
    let std = rhai::packages::CorePackage::new();
    std.register_into_engine(&mut engine);

    let _ = usart.blocking_write(b"Hello Embassy World!\r\n");

    let mut pos = 0;
    let mut buffer = [0u8; 128];
    let mut buf = [0u8; 1];
    loop {
        unwrap!(usart.blocking_read(&mut buf));
        // unwrap!(usart.blocking_write(&buf));
        buffer[pos] = buf[0];

        unwrap!(usart.blocking_write(&buf));

        // Check for newline characters
        if buf[0] == b'\n' || buf[0] == b'\r' {
            // Convert buffer to &str
            if let Ok(line) = str::from_utf8(&buffer[..pos]) {
                // Process the received line
                info!("Received line: {}", line);
                match engine.eval_expression::<Dynamic>(line) {
                    Ok(res) => {
                        unwrap!(usart.blocking_write(format!("{:?},\r\n{:?}\n\r", line, res).as_bytes()));
                    }
                    Err(e) => {
                        let mes = format!("Failed to process line: {:?}\n\rError: {:?}\n\r", line, e);
                        unwrap!(usart.blocking_write(mes.as_bytes()));
                    }
                }
            } else {
                error!("Failed to convert buffer to string");
            }
            pos = 0; // Reset the buffer position
        } else {
            pos += 1;
        }
    }
}
