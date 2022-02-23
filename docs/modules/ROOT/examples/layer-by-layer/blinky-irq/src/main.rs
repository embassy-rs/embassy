#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use embassy_stm32::{
    gpio::{Input, Level, Output, Pin, Pull, Speed},
    interrupt, pac,
    peripherals::{PB14, PC13},
};

static BUTTON: Mutex<RefCell<Option<Input<'static, PC13>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<Output<'static, PB14>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let p = embassy_stm32::init(Default::default());
    let led = Output::new(p.PB14, Level::Low, Speed::Low);
    let mut button = Input::new(p.PC13, Pull::Up);

    cortex_m::interrupt::free(|cs| unsafe {
        enable_interrupt(&mut button);

        LED.borrow(cs).borrow_mut().replace(led);
        BUTTON.borrow(cs).borrow_mut().replace(button);

        NVIC::unmask(pac::Interrupt::EXTI15_10);
    });

    loop {
        cortex_m::asm::wfe();
    }
}

#[interrupt]
fn EXTI15_10() {
    cortex_m::interrupt::free(|cs| {
        let mut button = BUTTON.borrow(cs).borrow_mut();
        let button = button.as_mut().unwrap();

        let mut led = LED.borrow(cs).borrow_mut();
        let led = led.as_mut().unwrap();
        if check_interrupt(button) {
            if button.is_low() {
                led.set_high();
            } else {
                led.set_low();
            }
        }
        clear_interrupt(button);
    });
}
//
//
//
//
//
//
// "Hidden" HAL-like methods for doing interrupts with embassy. Hardcode pin just to give audience an idea of what it looks like

const PORT: u8 = 2;
const PIN: usize = 13;
fn check_interrupt<P: Pin>(_pin: &mut Input<'static, P>) -> bool {
    let exti = pac::EXTI;
    unsafe {
        let pin = PIN;
        let lines = exti.pr(0).read();
        lines.line(pin)
    }
}

fn clear_interrupt<P: Pin>(_pin: &mut Input<'static, P>) {
    let exti = pac::EXTI;
    unsafe {
        let pin = PIN;
        let mut lines = exti.pr(0).read();
        lines.set_line(pin, true);
        exti.pr(0).write_value(lines);
    }
}

fn enable_interrupt<P: Pin>(_pin: &mut Input<'static, P>) {
    cortex_m::interrupt::free(|_| unsafe {
        let rcc = pac::RCC;
        rcc.apb2enr().modify(|w| w.set_syscfgen(true));

        let port = PORT;
        let pin = PIN;
        let syscfg = pac::SYSCFG;
        let exti = pac::EXTI;
        syscfg.exticr(pin / 4).modify(|w| w.set_exti(pin % 4, port));
        exti.imr(0).modify(|w| w.set_line(pin, true));
        exti.rtsr(0).modify(|w| w.set_line(pin, true));
        exti.ftsr(0).modify(|w| w.set_line(pin, true));
    });
}
