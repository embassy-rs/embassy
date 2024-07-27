//! This example demonstrates some approaches to communicate between tasks in order to orchestrate the state of the system.
//!
//! We demonstrate how to:
//! - use a channel to send messages between tasks, in this case here in order to have one task control the state of the system.
//! - use a signal to terminate a task.
//! - use command channels to send commands to another task.
//! - use different ways to receive messages, from a straightforwar awaiting on one channel to a more complex awaiting on multiple channels.
//!
//! There are more patterns to orchestrate tasks, this is just one example.
//!
//! We will use these tasks to generate example "state information":
//! - a task that generates random numbers in intervals of 60s
//! - a task that generates random numbers in intervals of 30s
//! - a task that generates random numbers in intervals of 90s
//! - a task that notifies about being attached/disattached from usb power
//! - a task that measures vsys voltage in intervals of 30s

#![no_std]
#![no_main]

use assign_resources::assign_resources;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::adc::{Adc, Channel, Config, InterruptHandler};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::peripherals;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::{channel, signal};
use embassy_time::{Duration, Timer};
use rand::RngCore;
use {defmt_rtt as _, panic_probe as _};

// This is just some preparation, see example `assign_resources.rs` for more information on this. We prep the rresources that we will be using in different tasks.
// **Note**: This will not work with a board that has a wifi chip, because the wifi chip uses pins 24 and 29 for its own purposes. A way around this in software
// is not trivial, at least if you intend to use wifi, too. Workaround is to wire from vsys and vbus pins to appropriate pins on the board through a voltage divider. Then use those pins.
// For this example it will not matter much, the concept of what we are showing remains valid.
assign_resources! {
    vsys: Vsys {
        adc: ADC,
        pin_29: PIN_29,
    },
    vbus: Vbus {
        pin_24: PIN_24,
    },
}

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

/// This is the type of Events that we will send from the worker tasks to the orchestrating task.
enum Events {
    UsbPowered(bool),
    VsysVoltage(f32),
    FirstRandomSeed(u32),
    SecondRandomSeed(u32),
    ThirdRandomSeed(u32),
}

/// This is the type of Commands that we will send from the orchestrating task to the worker tasks.
/// Note that we are lazy here and only have one command, you might want to have more.
enum Commands {
    /// This command will stop the appropriate worker task
    Stop,
}

/// This is the state of the system, we will use this to orchestrate the system. This is a simple example, in a real world application this would be more complex.
#[derive(Default, Debug, Clone, Format)]
struct State {
    usb_powered: bool,
    vsys_voltage: f32,
    first_random_seed: u32,
    second_random_seed: u32,
    third_random_seed: u32,
    times_we_got_first_random_seed: u8,
    maximum_times_we_want_first_random_seed: u8,
}

impl State {
    fn new() -> Self {
        Self {
            usb_powered: false,
            vsys_voltage: 0.0,
            first_random_seed: 0,
            second_random_seed: 0,
            third_random_seed: 0,
            times_we_got_first_random_seed: 0,
            maximum_times_we_want_first_random_seed: 3,
        }
    }
}

/// Channel for the events that we want the orchestrator to react to, all state events are of the type Enum Events.
/// We use a channel with an arbitrary size of 10, the precise size of the queue depends on your use case. This depends on how many events we
/// expect to be generated in a given time frame and how fast the orchestrator can react to them. And then if we rather want the senders to wait for
/// new slots in the queue or if we want the orchestrator to have a backlog of events to process. In this case here we expect to always be enough slots
/// in the queue, so the worker tasks can in all nominal cases send their events and continue with their work without waiting.
/// For the events we - in this case here -  do not want to loose any events, so a channel is a good choice. See embassy_sync docs for other options.
static EVENT_CHANNEL: channel::Channel<CriticalSectionRawMutex, Events, 10> = channel::Channel::new();

/// Signal for stopping the first random signal task. We use a signal here, because we need no queue. It is suffiient to have one signal active.
static STOP_FIRST_RANDOM_SIGNAL: signal::Signal<CriticalSectionRawMutex, Commands> = signal::Signal::new();

// And now we can put all this into use

/// This is the main task, that will not do very much besides spawning the other tasks. This is a design choice, you could do the
/// orchestrating here. This is to show that we do not need a main loop here, the system will run indefinitely as long as at least one task is running.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // initialize the peripherals
    let p = embassy_rp::init(Default::default());
    // split the resources, for convenience - see above
    let r = split_resources! {p};

    // spawn the tasks
    spawner.spawn(orchestrate(spawner)).unwrap();
    spawner.spawn(random_30s(spawner)).unwrap();
    spawner.spawn(random_60s(spawner)).unwrap();
    spawner.spawn(random_90s(spawner)).unwrap();
    spawner.spawn(usb_power(spawner, r.vbus)).unwrap();
    spawner.spawn(vsys_voltage(spawner, r.vsys)).unwrap();
}

/// This is the task handling the system state and orchestrating the other tasks. WEe can regard this as the "main loop" of the system.
#[embassy_executor::task]
async fn orchestrate(_spawner: Spawner) {
    let mut state = State::new();

    // we need to have a receiver for the events
    let receiver = EVENT_CHANNEL.receiver();

    loop {
        // we await on the receiver, this will block until a new event is available
        // as an alternative to this, we could also await on multiple channels, this would block until at least one of the channels has an event
        // see the embassy_futures docs: https://docs.embassy.dev/embassy-futures/git/default/select/index.html
        // The task random_30s does a select, if you want to have a look at that.
        // Another reason to use select may also be that we want to have a timeout, so we can react to the absence of events within a time frame.
        // We keep it simple here.
        let event = receiver.receive().await;

        // react to the events
        match event {
            Events::UsbPowered(usb_powered) => {
                // update the state and/or react to the event here
                state.usb_powered = usb_powered;
                info!("Usb powered: {}", usb_powered);
            }
            Events::VsysVoltage(voltage) => {
                // update the state and/or react to the event here
                state.vsys_voltage = voltage;
                info!("Vsys voltage: {}", voltage);
            }
            Events::FirstRandomSeed(seed) => {
                // update the state and/or react to the event here
                state.first_random_seed = seed;
                // here we change some meta state, we count how many times we got the first random seed
                state.times_we_got_first_random_seed += 1;
                info!(
                    "First random seed: {}, and that was iteration {} of receiving this.",
                    seed, &state.times_we_got_first_random_seed
                );
            }
            Events::SecondRandomSeed(seed) => {
                // update the state and/or react to the event here
                state.second_random_seed = seed;
                info!("Second random seed: {}", seed);
            }
            Events::ThirdRandomSeed(seed) => {
                // update the state and/or react to the event here
                state.third_random_seed = seed;
                info!("Third random seed: {}", seed);
            }
        }
        // we now have an altered state
        // there is a crate for detecting field changes on crates.io (https://crates.io/crates/fieldset) that might be useful here
        // for now we just keep it simple
        info!("State: {:?}", &state);

        // here we react to the state, in this case here we want to stop the first random seed task after we got it a defined number of times
        if state.times_we_got_first_random_seed == state.maximum_times_we_want_first_random_seed {
            info!("Stopping the first random signal task");
            // we send a command to the task
            STOP_FIRST_RANDOM_SIGNAL.signal(Commands::Stop);
        }
    }
}

/// This task will generate random numbers in intervals of 30s
/// The task will terminate after it has received a command signal to stop, see the orchestrate task for that.
#[embassy_executor::task]
async fn random_30s(_spawner: Spawner) {
    let mut rng = RoscRng;
    let sender = EVENT_CHANNEL.sender();
    loop {
        // we either await on the timer or the signal, whichever comes first.
        let futures = select(Timer::after(Duration::from_secs(30)), STOP_FIRST_RANDOM_SIGNAL.wait()).await;
        match futures {
            Either::First(_) => {
                // we received are operating on the timer
                info!("30s are up, generating random number");
                let random_number = rng.next_u32();
                sender.send(Events::FirstRandomSeed(random_number)).await;
            }
            Either::Second(_) => {
                // we received the signal to stop
                info!("Received signal to stop, goodbye!");
                break;
            }
        }
    }
}

/// This task will generate random numbers in intervals of 60s
#[embassy_executor::task]
async fn random_60s(_spawner: Spawner) {
    let mut rng = RoscRng;
    let sender = EVENT_CHANNEL.sender();
    loop {
        Timer::after(Duration::from_secs(60)).await;
        let random_number = rng.next_u32();
        sender.send(Events::SecondRandomSeed(random_number)).await;
    }
}

/// This task will generate random numbers in intervals of 90s
#[embassy_executor::task]
async fn random_90s(_spawner: Spawner) {
    let mut rng = RoscRng;
    let sender = EVENT_CHANNEL.sender();
    loop {
        Timer::after(Duration::from_secs(90)).await;
        let random_number = rng.next_u32();
        sender.send(Events::ThirdRandomSeed(random_number)).await;
    }
}

/// This task will notify if we are connected to usb power
#[embassy_executor::task]
pub async fn usb_power(_spawner: Spawner, r: Vbus) {
    let mut vbus_in = Input::new(r.pin_24, Pull::None);
    let sender = EVENT_CHANNEL.sender();
    loop {
        sender.send(Events::UsbPowered(vbus_in.is_high())).await;
        vbus_in.wait_for_any_edge().await;
    }
}

/// This task will measure the vsys voltage in intervals of 30s
#[embassy_executor::task]
pub async fn vsys_voltage(_spawner: Spawner, r: Vsys) {
    let mut adc = Adc::new(r.adc, Irqs, Config::default());
    let vsys_in = r.pin_29;
    let mut channel = Channel::new_pin(vsys_in, Pull::None);
    let sender = EVENT_CHANNEL.sender();
    loop {
        // read the adc value
        let adc_value = adc.read(&mut channel).await.unwrap();
        // convert the adc value to voltage.
        // 3.3 is the reference voltage, 3.0 is the factor for the inbuilt voltage divider and 4096 is the resolution of the adc
        let voltage = (adc_value as f32) * 3.3 * 3.0 / 4096.0;
        sender.send(Events::VsysVoltage(voltage)).await;
        Timer::after(Duration::from_secs(30)).await;
    }
}
