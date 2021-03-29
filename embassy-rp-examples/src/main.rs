#![no_std]
#![no_main]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

use core::sync::atomic::{AtomicUsize, Ordering};
use defmt::{panic, *};
use defmt_rtt as _;
use embassy::executor::Spawner;
use embassy::interrupt::InterruptExt;
use embassy_rp::{dma, gpio, interrupt, uart, Peripherals};
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;
use rp2040_pac2 as pac;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER;

defmt::timestamp! {"{=u64}", {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n as u64
}
}

#[embassy::main]
async fn main(spanwer: Spawner) {
    let p = unwrap!(Peripherals::take());

    let mut config = uart::Config::default();
    let mut uart = uart::Uart::new(p.UART0, p.PIN_0, p.PIN_1, p.PIN_2, p.PIN_3, config);
    uart.send("Hello World!\r\n".as_bytes());

    let mut led = gpio::Output::new(p.PIN_25, gpio::Level::Low);

    let irq = interrupt::take!(DMA_IRQ_0);
    unsafe {
        //pac::DMA.inte0().write(|w| w.set_inte0(1 << 0));
    }
    irq.set_handler(dma_irq);
    irq.unpend();
    irq.enable();

    let from: [u32; 4] = [1, 2, 3, 4];
    let mut to: [u32; 4] = [9, 8, 7, 6];
    info!("before dma: from = {:?}, to = {:?}", from, to);
    cortex_m::asm::delay(4_000_000);
    dma::Dma::copy(p.DMA_CH0, &from, &mut to);
    cortex_m::asm::delay(4_000_000);
    info!("after dma: from = {:?}, to = {:?}", from, to);

    loop {
        info!("led on!");
        uart.send("ON!\r".as_bytes());
        led.set_high().unwrap();
        cortex_m::asm::delay(1_000_000);

        info!("led off!");
        uart.send("Off!\r".as_bytes());
        led.set_low().unwrap();
        cortex_m::asm::delay(4_000_000);
    }
}

unsafe fn dma_irq(ctx: *mut ()) {
    info!("DMA IRQ!");
}
