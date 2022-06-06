#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// global logger
use defmt::{info, unwrap};
use defmt_rtt as _;

use panic_probe as _;

use embassy::channel::Signal;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::Peripherals;

// Expected output (with defmt-print):
//
// 0.000030 INFO  registering sender
// 0.000091 INFO  main task doing other things
// 0.000152 INFO  getting sender
// 0.000213 INFO  waiting for signal
// 1.000152 INFO  main task doing other things
// 1.000213 INFO  signalling, counter: 0
// 1.000335 INFO  signalled, counter: 0
// 2.000213 INFO  main task doing other things
// 2.000274 INFO  signalling, counter: 1         <-- receiver misses this
// 3.000274 INFO  main task doing other things
// 3.000457 INFO  signalling, counter: 2         <-- receiver misses this
// 4.000335 INFO  main task doing other things
// 4.000549 INFO  signalling, counter: 3         <-- receiver misses this
// 5.000396 INFO  main task doing other things
// 5.000640 INFO  signalling, counter: 4
// 6.000427 INFO  waiting for signal
// 6.000457 INFO  signalled, counter: 4
// 6.000579 INFO  main task doing other things
// 6.000762 INFO  signalling, counter: 5
// 7.000640 INFO  main task doing other things
// ...
// 7.000854 INFO  signalling, counter: 6         <-- receiver misses this
// 8.000701 INFO  main task doing other things
// ...
// 10.001129 INFO  signalling, counter: 9
// 11.000579 INFO  waiting for signal
// 11.000610 INFO  signalled, counter: 9
// 11.000885 INFO  main task doing other things

mod sender {
    use super::info;
    use super::Signal;
    use super::{Duration, Timer};
    use core::borrow::Borrow;

    static SIGNAL: Signal<u32> = Signal::new();

    #[embassy::task]
    pub async fn my_sending_task() {
        let mut counter: u32 = 0;

        loop {
            Timer::after(Duration::from_secs(1)).await;

            info!("signalling, counter: {}", counter);
            SIGNAL.signal(counter);

            // receiving task gets a copy of the counter, at it's current value, before it is incremented below

            counter = counter.wrapping_add(1);
        }
    }

    pub fn init_sending_task() {
        info!("registering sender");
        super::receiver::register_sender(SIGNAL.borrow());
    }
}

mod receiver {
    use super::info;
    use super::Signal;
    use super::{Duration, Timer};
    use core::cell::RefCell;
    use embassy::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy::blocking_mutex::Mutex;

    // RefCell because Mutux::lock() give a immutable reference to the closure.
    // Option because a default value needs to be provided for a static value.
    // 'static lifetime for the reference because the signal needs to live forever.
    // Mutex because otherwise you get a 'cannot be shared between threads safely' due to the static lifetime requirement.
    static SIGNAL: Mutex<CriticalSectionRawMutex, RefCell<Option<&'static Signal<u32>>>> =
        Mutex::new(RefCell::new(None));

    #[embassy::task]
    pub async fn my_receiving_task() {
        info!("getting sender");
        let signal = SIGNAL.lock(|s| s.take().unwrap());

        loop {
            info!("waiting for signal");
            let received_counter = signal.wait().await;

            info!("signalled, counter: {}", received_counter);

            // Simulate long complicated process which will cause this task to miss some of the signals
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
        info!("main task doing other things");

        Timer::after(Duration::from_secs(1)).await;
    }
}
