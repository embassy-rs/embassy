#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use defmt::*;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::TIM2;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::Timer;
use embassy_stm32::{interrupt, pac};
use {defmt_rtt as _, panic_probe as _};

// Define a global button that can be accessed from the interrupt handler
static LED: Mutex<RefCell<Option<Output<'static>>>> = Mutex::new(RefCell::new(None));
static TIMER: Mutex<RefCell<Option<Timer<TIM2>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    info!("Hello World!");
    let p = embassy_stm32::init(Default::default());

    let led = Output::new(p.PB14, Level::High, Speed::Low);

    // Using the low_level timer api to setup a hardware controlled timer interrupt
    let timer = Timer::new(p.TIM2);
    timer.set_frequency(Hertz(10));
    timer.enable_update_interrupt(true);
    timer.set_autoreload_preload(true);
    timer.start();

    cortex_m::interrupt::free(|cs| {
        // saving the configured timer and led in a mutex so they safely can be accessed from our
        // custom interrupt handler.
        LED.borrow(cs).borrow_mut().replace(led);
        TIMER.borrow(cs).borrow_mut().replace(timer);
    });

    unsafe {
        // enable NVIC interrupt for TIM2
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2);
    }

    info!("running main loop...");
    loop {
        cortex_m::asm::wfe();
    }
}

// Setting up custom interrupt handler for the TIM2 interrupt
#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|cs| {
        let mut led = LED.borrow(cs).borrow_mut();
        let led = led.as_mut().unwrap();

        let mut timer = TIMER.borrow(cs).borrow_mut();
        let timer = timer.as_mut().unwrap();
        // clear the interrupt flag for the timer
        timer.clear_update_interrupt();

        led.toggle();
    });
}
