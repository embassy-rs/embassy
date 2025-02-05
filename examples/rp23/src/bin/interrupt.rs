//! This example shows how you can use raw interrupt handlers alongside embassy.
//! The example also showcases some of the options available for sharing resources/data.
//!
//! In the example, an ADC reading is triggered every time the PWM wraps around.
//! The sample data is sent down a channel, to be processed inside a low priority task.
//! The processed data is then used to adjust the PWM duty cycle, once every second.

#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{self, Adc, Blocking};
use embassy_rp::block::ImageDef;
use embassy_rp::gpio::Pull;
use embassy_rp::interrupt;
use embassy_rp::pwm::{Config, Pwm};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Ticker};
use portable_atomic::{AtomicU32, Ordering};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

static COUNTER: AtomicU32 = AtomicU32::new(0);
static PWM: Mutex<CriticalSectionRawMutex, RefCell<Option<Pwm>>> = Mutex::new(RefCell::new(None));
static ADC: Mutex<CriticalSectionRawMutex, RefCell<Option<(Adc<Blocking>, adc::Channel)>>> =
    Mutex::new(RefCell::new(None));
static ADC_VALUES: Channel<CriticalSectionRawMutex, u16, 2048> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let adc = Adc::new_blocking(p.ADC, Default::default());
    let p26 = adc::Channel::new_pin(p.PIN_26, Pull::None);
    ADC.lock(|a| a.borrow_mut().replace((adc, p26)));

    let pwm = Pwm::new_output_b(p.PWM_SLICE4, p.PIN_25, Default::default());
    PWM.lock(|p| p.borrow_mut().replace(pwm));

    // Enable the interrupt for pwm slice 4
    embassy_rp::pac::PWM.irq0_inte().modify(|w| w.set_ch4(true));
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::PWM_IRQ_WRAP_0);
    }

    // Tasks require their resources to have 'static lifetime
    // No Mutex needed when sharing within the same executor/prio level
    static AVG: StaticCell<Cell<u32>> = StaticCell::new();
    let avg = AVG.init(Default::default());
    spawner.must_spawn(processing(avg));

    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        ticker.next().await;
        let freq = COUNTER.swap(0, Ordering::Relaxed);
        info!("pwm freq: {:?} Hz", freq);
        info!("adc average: {:?}", avg.get());

        // Update the pwm duty cycle, based on the averaged adc reading
        let mut config = Config::default();
        config.compare_b = ((avg.get() as f32 / 4095.0) * config.top as f32) as _;
        PWM.lock(|p| p.borrow_mut().as_mut().unwrap().set_config(&config));
    }
}

#[embassy_executor::task]
async fn processing(avg: &'static Cell<u32>) {
    let mut buffer: heapless::HistoryBuffer<u16, 100> = Default::default();
    loop {
        let val = ADC_VALUES.receive().await;
        buffer.write(val);
        let sum: u32 = buffer.iter().map(|x| *x as u32).sum();
        avg.set(sum / buffer.len() as u32);
    }
}

#[interrupt]
fn PWM_IRQ_WRAP_0() {
    critical_section::with(|cs| {
        let mut adc = ADC.borrow(cs).borrow_mut();
        let (adc, p26) = adc.as_mut().unwrap();
        let val = adc.blocking_read(p26).unwrap();
        ADC_VALUES.try_send(val).ok();

        // Clear the interrupt, so we don't immediately re-enter this irq handler
        PWM.borrow(cs).borrow_mut().as_mut().unwrap().clear_wrapped();
    });
    COUNTER.fetch_add(1, Ordering::Relaxed);
}
