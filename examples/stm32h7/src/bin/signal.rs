#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::Peripherals;

use defmt_rtt as _;
// global logger
use panic_probe as _;


mod sender {
    use core::borrow::Borrow;
    use defmt::*;
    use embassy::channel::Signal;
    use embassy::time::{Duration, Timer};

    static SIGNAL: Signal<u32> = Signal::new();

    #[embassy::task]
    pub async fn my_sending_task() {

        let mut counter: u32 = 0;

        loop {
            Timer::after(Duration::from_secs(1)).await;

            info!("signalling, counter: {}", counter);
            SIGNAL.signal(counter);

            // receiving task gets a copy of the counter, before it is incremented

            counter = counter.wrapping_add(1);
        }
    }

    pub fn init_sending_task() {
        info!("registering sender");
        super::receiver::register_sender(SIGNAL.borrow());
    }
}

mod receiver {
    use core::cell::RefCell;
    use defmt::*;
    use embassy::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy::channel::Signal;
    use embassy::blocking_mutex::Mutex;
    use embassy::time::{Duration, Timer};

    // RefCell because Mutux::lock() give a immutable reference to the closure.
    // Option because a default value needs to be provided for a static value.
    // 'static lifetime for the reference because the signal needs to live forever.
    // Mutex because otherwise you get a 'cannot be shared between threads safely' due to the static lifetime requirement.
    static SIGNAL: Mutex<CriticalSectionRawMutex, RefCell<Option<&'static Signal<u32>>>> = Mutex::new(RefCell::new(None));

    #[embassy::task]
    pub async fn my_receiving_task() {

        info!("getting sender");
        let signal = SIGNAL.lock(|s| s.take().unwrap() );

        loop {
            info!("waiting for signal");
            let received_counter = signal.wait().await;

            info!("signalled, counter: {}", received_counter);

            // Simulate long complicated process
            Timer::after(Duration::from_secs(5)).await;
        }
    }

    pub fn register_sender(signal: &'static Signal<u32>) {
        SIGNAL.lock(|s| s.replace(Some(signal)));
    }
}

#[embassy::main]
async fn main(spawner: Spawner, _p: Peripherals) {

    sender::init_sending_task();

    unwrap!(spawner.spawn(receiver::my_receiving_task()));
    unwrap!(spawner.spawn(sender::my_sending_task()));

    loop {
        Timer::after(Duration::from_secs(1)).await;
        info!(".");
    }
}
