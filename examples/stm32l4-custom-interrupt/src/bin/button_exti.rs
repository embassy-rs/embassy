#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use defmt::*;
use embassy_stm32::exti::blocking::ExtiInput;
use embassy_stm32::exti::TriggerEdge;
use embassy_stm32::gpio::Pull;
use embassy_stm32::interrupt;
use {defmt_rtt as _, panic_probe as _};

// Defining global button references that can be accessed from the interrupt handlers
static BUTTON1: Mutex<RefCell<Option<ExtiInput<'static>>>> = Mutex::new(RefCell::new(None));
static BUTTON2: Mutex<RefCell<Option<ExtiInput<'static>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // setting up the user button on the nucleo board
    let mut button1 = ExtiInput::new(p.PC13, p.EXTI13, Pull::Down, TriggerEdge::Rising);
    button1.rising_edge();

    // setting up an external button connected to the nucleo board
    let mut button2 = ExtiInput::new(p.PC8, p.EXTI8, Pull::Down, TriggerEdge::Rising);
    button2.rising_edge();

    cortex_m::interrupt::free(|cs| {
        BUTTON1.borrow(cs).borrow_mut().replace(button1);
        BUTTON2.borrow(cs).borrow_mut().replace(button2);
    });

    info!("Press the USER button...");
    loop {
        cortex_m::asm::wfe();
    }
}

// Setting up a custom interrupt handler for exti group 15 to 10.
#[interrupt]
fn EXTI15_10() {
    info!("button1 pressed");
    cortex_m::interrupt::free(|cs| {
        let mut button = BUTTON1.borrow(cs).borrow_mut();
        let button = button.as_mut().unwrap();
        button.clear_pending();
    });
}

// Setting up a custom interrupt handler for exti group 9 to 5.
#[interrupt]
fn EXTI9_5() {
    info!("button2 pressed");
    cortex_m::interrupt::free(|cs| {
        let mut button = BUTTON2.borrow(cs).borrow_mut();
        let button = button.as_mut().unwrap();
        button.clear_pending();
    });
}
