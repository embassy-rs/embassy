//! This example demonstrates some approaches to communicate between tasks in order to orchestrate the state of the system.
//!
//! The system consists of several tasks:
//! - Three tasks that generate random numbers at different intervals (simulating i.e. sensor readings)
//! - A task that monitors USB power connection (hardware event handling)
//! - A task that reads system voltage (ADC sampling)
//! - A consumer task that processes all this information
//!
//! The system maintains state in a single place, wrapped in a Mutex.
//!
//! We demonstrate how to:
//! - use a mutex to maintain shared state between tasks
//! - use a channel to send events between tasks
//! - use an orchestrator task to coordinate tasks and handle state transitions
//! - use signals to notify about state changes and terminate tasks

#![no_std]
#![no_main]

use assign_resources::assign_resources;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::adc::{Adc, Channel, Config, InterruptHandler};
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::{bind_interrupts, peripherals, Peri};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::{channel, signal};
use embassy_time::{Duration, Timer};
use rand::RngCore;
use {defmt_rtt as _, panic_probe as _};

// Hardware resource assignment. See other examples for different ways of doing this.
assign_resources! {
    vsys: Vsys {
        adc: ADC,
        pin_29: PIN_29,
    },
    vbus: Vbus {
        pin_24: PIN_24,
    },
}

// Interrupt binding - required for hardware peripherals like ADC
bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

/// Events that worker tasks send to the orchestrator
enum Events {
    UsbPowered(bool),      // USB connection state changed
    VsysVoltage(f32),      // New voltage reading
    FirstRandomSeed(u32),  // Random number from 30s timer
    SecondRandomSeed(u32), // Random number from 60s timer
    ThirdRandomSeed(u32),  // Random number from 90s timer
    ResetFirstRandomSeed,  // Signal to reset the first counter
}

/// Commands that can control task behavior.
/// Currently only used to stop tasks, but could be extended for other controls.
enum Commands {
    /// Signals a task to stop execution
    Stop,
}

/// The central state of our system, shared between tasks.
#[derive(Clone, Format)]
struct State {
    usb_powered: bool,
    vsys_voltage: f32,
    first_random_seed: u32,
    second_random_seed: u32,
    third_random_seed: u32,
    first_random_seed_task_running: bool,
    times_we_got_first_random_seed: u8,
    maximum_times_we_want_first_random_seed: u8,
}

/// A formatted view of the system status, used for logging. Used for the below `get_system_summary` fn.
#[derive(Format)]
struct SystemStatus {
    power_source: &'static str,
    voltage: f32,
}

impl State {
    const fn new() -> Self {
        Self {
            usb_powered: false,
            vsys_voltage: 0.0,
            first_random_seed: 0,
            second_random_seed: 0,
            third_random_seed: 0,
            first_random_seed_task_running: false,
            times_we_got_first_random_seed: 0,
            maximum_times_we_want_first_random_seed: 3,
        }
    }

    /// Returns a formatted summary of power state and voltage.
    /// Shows how to create methods that work with shared state.
    fn get_system_summary(&self) -> SystemStatus {
        SystemStatus {
            power_source: if self.usb_powered {
                "USB powered"
            } else {
                "Battery powered"
            },
            voltage: self.vsys_voltage,
        }
    }
}

/// The shared state protected by a mutex
static SYSTEM_STATE: Mutex<CriticalSectionRawMutex, State> = Mutex::new(State::new());

/// Channel for events from worker tasks to the orchestrator
static EVENT_CHANNEL: channel::Channel<CriticalSectionRawMutex, Events, 10> = channel::Channel::new();

/// Signal used to stop the first random number task
static STOP_FIRST_RANDOM_SIGNAL: signal::Signal<CriticalSectionRawMutex, Commands> = signal::Signal::new();

/// Signal for notifying about state changes
static STATE_CHANGED: signal::Signal<CriticalSectionRawMutex, ()> = signal::Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources! {p};

    spawner.spawn(orchestrate(spawner)).unwrap();
    spawner.spawn(random_60s(spawner)).unwrap();
    spawner.spawn(random_90s(spawner)).unwrap();
    // `random_30s` is not spawned here, butin the orchestrate task depending on state
    spawner.spawn(usb_power(spawner, r.vbus)).unwrap();
    spawner.spawn(vsys_voltage(spawner, r.vsys)).unwrap();
    spawner.spawn(consumer(spawner)).unwrap();
}

/// Main task that processes all events and updates system state.
#[embassy_executor::task]
async fn orchestrate(spawner: Spawner) {
    let receiver = EVENT_CHANNEL.receiver();

    loop {
        // Do nothing until we receive any event
        let event = receiver.receive().await;

        // Scope in which we want to lock the system state. As an alternative we could also call `drop` on the state
        {
            let mut state = SYSTEM_STATE.lock().await;

            match event {
                Events::UsbPowered(usb_powered) => {
                    state.usb_powered = usb_powered;
                    info!("Usb powered: {}", usb_powered);
                    info!("System summary: {}", state.get_system_summary());
                }
                Events::VsysVoltage(voltage) => {
                    state.vsys_voltage = voltage;
                    info!("Vsys voltage: {}", voltage);
                }
                Events::FirstRandomSeed(seed) => {
                    state.first_random_seed = seed;
                    state.times_we_got_first_random_seed += 1;
                    info!(
                        "First random seed: {}, and that was iteration {} of receiving this.",
                        seed, &state.times_we_got_first_random_seed
                    );
                }
                Events::SecondRandomSeed(seed) => {
                    state.second_random_seed = seed;
                    info!("Second random seed: {}", seed);
                }
                Events::ThirdRandomSeed(seed) => {
                    state.third_random_seed = seed;
                    info!("Third random seed: {}", seed);
                }
                Events::ResetFirstRandomSeed => {
                    state.times_we_got_first_random_seed = 0;
                    state.first_random_seed = 0;
                    info!("Resetting the first random seed counter");
                }
            }

            // Handle task orchestration based on state
            // Just placed as an example here, could be hooked into the event system, puton a timer, ...
            match state.times_we_got_first_random_seed {
                max if max == state.maximum_times_we_want_first_random_seed => {
                    info!("Stopping the first random signal task");
                    STOP_FIRST_RANDOM_SIGNAL.signal(Commands::Stop);
                    EVENT_CHANNEL.sender().send(Events::ResetFirstRandomSeed).await;
                }
                0 => {
                    let respawn_first_random_seed_task = !state.first_random_seed_task_running;
                    // Deliberately dropping the Mutex lock here to release it before a lengthy operation
                    drop(state);
                    if respawn_first_random_seed_task {
                        info!("(Re)-Starting the first random signal task");
                        spawner.spawn(random_30s(spawner)).unwrap();
                    }
                }
                _ => {}
            }
        }

        STATE_CHANGED.signal(());
    }
}

/// Task that monitors state changes and logs system status.
#[embassy_executor::task]
async fn consumer(_spawner: Spawner) {
    loop {
        // Wait for state change notification
        STATE_CHANGED.wait().await;

        let state = SYSTEM_STATE.lock().await;
        info!(
            "State update - {} | Seeds - First: {} (count: {}/{}, running: {}), Second: {}, Third: {}",
            state.get_system_summary(),
            state.first_random_seed,
            state.times_we_got_first_random_seed,
            state.maximum_times_we_want_first_random_seed,
            state.first_random_seed_task_running,
            state.second_random_seed,
            state.third_random_seed
        );
    }
}

/// Task that generates random numbers every 30 seconds until stopped.
/// Shows how to handle both timer events and stop signals.
/// As an example of some routine we want to be on or off depending on other needs.
#[embassy_executor::task]
async fn random_30s(_spawner: Spawner) {
    {
        let mut state = SYSTEM_STATE.lock().await;
        state.first_random_seed_task_running = true;
    }

    let mut rng = RoscRng;
    let sender = EVENT_CHANNEL.sender();

    loop {
        // Wait for either 30s timer or stop signal (like select() in Go)
        match select(Timer::after(Duration::from_secs(30)), STOP_FIRST_RANDOM_SIGNAL.wait()).await {
            Either::First(_) => {
                info!("30s are up, generating random number");
                let random_number = rng.next_u32();
                sender.send(Events::FirstRandomSeed(random_number)).await;
            }
            Either::Second(_) => {
                info!("Received signal to stop, goodbye!");

                let mut state = SYSTEM_STATE.lock().await;
                state.first_random_seed_task_running = false;

                break;
            }
        }
    }
}

/// Task that generates random numbers every 60 seconds. As an example of some routine.
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

/// Task that generates random numbers every 90 seconds. . As an example of some routine.
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

/// Task that monitors USB power connection. As an example of some Interrupt somewhere.
#[embassy_executor::task]
pub async fn usb_power(_spawner: Spawner, r: Vbus) {
    let mut vbus_in = Input::new(r.pin_24, Pull::None);
    let sender = EVENT_CHANNEL.sender();

    loop {
        sender.send(Events::UsbPowered(vbus_in.is_high())).await;
        vbus_in.wait_for_any_edge().await;
    }
}

/// Task that reads system voltage through ADC. As an example of some continuous sensor reading.
#[embassy_executor::task]
pub async fn vsys_voltage(_spawner: Spawner, r: Vsys) {
    let mut adc = Adc::new(r.adc, Irqs, Config::default());
    let vsys_in = r.pin_29;
    let mut channel = Channel::new_pin(vsys_in, Pull::None);
    let sender = EVENT_CHANNEL.sender();

    loop {
        Timer::after(Duration::from_secs(30)).await;
        let adc_value = adc.read(&mut channel).await.unwrap();
        let voltage = (adc_value as f32) * 3.3 * 3.0 / 4096.0;
        sender.send(Events::VsysVoltage(voltage)).await;
    }
}
