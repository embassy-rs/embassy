#![no_std]
#![no_main]

use defmt::{error, info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::usart::{self, UartRx, UartTx};
use embassy_stm32::{Config, bind_interrupts, dma, mode, peripherals};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    DMA1_STREAM1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA1_STREAM2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

#[unsafe(link_section = ".sram1")]
static GREETING: &str = "Hello Embassy World!\r\n";

#[embassy_executor::task]
async fn tx_task(mut usart: UartTx<'static, mode::Async>) {
    loop {
        match usart.write(GREETING.as_bytes()).await {
            Ok(()) => {
                info!("Sent: {:?}", GREETING);
            }
            Err(err) => {
                error!("Error during send: {}", err);
            }
        }
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn rx_task(mut usart: UartRx<'static, mode::Async>) {
    #[unsafe(link_section = ".sram1")]
    static mut RX_BUF: [u8; GREETING.len()] = [0; GREETING.len()];
    let buf = unsafe { &mut RX_BUF[..] };

    loop {
        if let Err(err) = usart.read(buf).await {
            error!("Error during receive: {}", err);
            continue;
        };
        match str::from_utf8(buf) {
            Ok(str) => {
                info!("Received: {:?}", str);
            }
            Err(_) => {
                info!("Received: {:02X}", buf);
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div16,
            mul: PllMul::Mul200,
            divp: Some(PllDiv::Div2), // 400 MHz
            divq: Some(PllDiv::Div8), // 100 MHz
            divr: None,
        });
    }
    let p = embassy_stm32::init(config);

    let mut config = usart::Config::default();
    config.invert_tx = true;
    config.invert_rx = true;
    // When using USART in slave mode, the master must send 8 clock pulses for 8 bits of data.
    // Therefore, pick exactly one:
    //  - Enable one parity bit for the master only
    //  - Enable clock pulse on the last bit
    config.mode.last_bit = true;
    let usart_master = unwrap!(UartTx::new_master(p.USART3, p.PC12, p.PC10, p.DMA1_CH1, Irqs, config));
    let usart_slave = unwrap!(UartRx::new_slave(p.USART2, p.PD7, p.PD6, p.DMA1_CH2, Irqs, config));

    spawner.spawn(unwrap!(rx_task(usart_slave)));
    spawner.spawn(unwrap!(tx_task(usart_master)));
}
