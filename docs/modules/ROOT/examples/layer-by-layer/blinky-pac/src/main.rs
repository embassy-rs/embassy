#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use stm32_metapac as pac;

use pac::gpio::vals;

#[cortex_m_rt::entry]
fn main() -> ! {
    // Enable GPIO clock
    let rcc = pac::RCC;
    unsafe {
        rcc.ahb2enr().modify(|w| {
            w.set_gpioben(true);
            w.set_gpiocen(true);
        });

        rcc.ahb2rstr().modify(|w| {
            w.set_gpiobrst(true);
            w.set_gpiocrst(true);
            w.set_gpiobrst(false);
            w.set_gpiocrst(false);
        });
    }

    // Setup button
    let gpioc = pac::GPIOC;
    const BUTTON_PIN: usize = 13;
    unsafe {
        gpioc
            .pupdr()
            .modify(|w| w.set_pupdr(BUTTON_PIN, vals::Pupdr::PULLUP));
        gpioc
            .otyper()
            .modify(|w| w.set_ot(BUTTON_PIN, vals::Ot::PUSHPULL));
        gpioc
            .moder()
            .modify(|w| w.set_moder(BUTTON_PIN, vals::Moder::INPUT));
    }

    // Setup LED
    let gpiob = pac::GPIOB;
    const LED_PIN: usize = 14;
    unsafe {
        gpiob
            .pupdr()
            .modify(|w| w.set_pupdr(LED_PIN, vals::Pupdr::FLOATING));
        gpiob
            .otyper()
            .modify(|w| w.set_ot(LED_PIN, vals::Ot::PUSHPULL));
        gpiob
            .moder()
            .modify(|w| w.set_moder(LED_PIN, vals::Moder::OUTPUT));
    }

    // Main loop
    loop {
        unsafe {
            if gpioc.idr().read().idr(BUTTON_PIN) == vals::Idr::LOW {
                gpiob.bsrr().write(|w| w.set_bs(LED_PIN, true));
            } else {
                gpiob.bsrr().write(|w| w.set_br(LED_PIN, true));
            }
        }
    }
}
