#![no_std]
#![no_main]

//! STM32H7 Secondary Core (CM4) Intercore Communication Example
//!
//! This example demonstrates reliable communication between the Cortex-M7 and
//! Cortex-M4 cores. This secondary core monitors shared memory for LED state
//! changes and updates the physical LEDs accordingly.
//!
//! The CM4 core handles:
//! - Responding to state changes from CM7
//! - Controlling the physical green and yellow LEDs
//! - Providing visual feedback via a heartbeat on the red LED
//!
//! Usage:
//! 1. Flash this CM4 (secondary) core binary first
//! 2. Then flash the CM7 (primary) core binary
//! 3. The red LED should blink continuously as a heartbeat
//! 4. Green and yellow LEDs should toggle according to CM7 core signals

/// Module providing shared memory constructs for intercore communication
mod shared {
    use core::sync::atomic::{AtomicU32, Ordering};

    /// State shared between CM7 and CM4 cores for LED control
    #[repr(C, align(4))]
    pub struct SharedLedState {
        pub magic: AtomicU32,
        pub counter: AtomicU32,
        pub led_states: AtomicU32,
    }

    // Bit positions in led_states
    pub const GREEN_LED_BIT: u32 = 0;
    pub const YELLOW_LED_BIT: u32 = 1;

    impl SharedLedState {
        pub const fn new() -> Self {
            Self {
                magic: AtomicU32::new(0xDEADBEEF),
                counter: AtomicU32::new(0),
                led_states: AtomicU32::new(0),
            }
        }

        /// Set LED state by manipulating the appropriate bit in the led_states field
        #[inline(never)]
        #[allow(dead_code)]
        pub fn set_led(&self, is_green: bool, state: bool) {
            let bit = if is_green { GREEN_LED_BIT } else { YELLOW_LED_BIT };
            let current = self.led_states.load(Ordering::SeqCst);

            let new_value = if state {
                current | (1 << bit) // Set bit
            } else {
                current & !(1 << bit) // Clear bit
            };
            self.led_states.store(new_value, Ordering::SeqCst);
        }

        /// Get current LED state
        #[inline(never)]
        pub fn get_led(&self, is_green: bool) -> bool {
            let bit = if is_green { GREEN_LED_BIT } else { YELLOW_LED_BIT };

            let value = self.led_states.load(Ordering::SeqCst);
            (value & (1 << bit)) != 0
        }

        /// Increment counter and return new value
        #[inline(never)]
        #[allow(dead_code)]
        pub fn increment_counter(&self) -> u32 {
            let current = self.counter.load(Ordering::SeqCst);
            let new_value = current.wrapping_add(1);
            self.counter.store(new_value, Ordering::SeqCst);
            new_value
        }

        /// Get current counter value
        #[inline(never)]
        pub fn get_counter(&self) -> u32 {
            let value = self.counter.load(Ordering::SeqCst);
            value
        }
    }

    #[link_section = ".ram_d3"]
    pub static SHARED_LED_STATE: SharedLedState = SharedLedState::new();
}

use core::mem::MaybeUninit;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::SharedData;
use embassy_time::Timer;
use shared::SHARED_LED_STATE;
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".ram_d3"]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

/// Task that continuously blinks the red LED as a heartbeat indicator
#[embassy_executor::task]
async fn blink_heartbeat(mut led: Output<'static>) {
    loop {
        led.toggle();
        info!("CM4 heartbeat");
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize the secondary core
    let p = embassy_stm32::init_secondary(&SHARED_DATA);
    info!("CM4 core initialized!");

    // Verify shared memory is accessible
    let magic = SHARED_LED_STATE.magic.load(core::sync::atomic::Ordering::SeqCst);
    info!("CM4: Magic value = 0x{:X}", magic);

    // Set up LEDs
    let mut green_led = Output::new(p.PB0, Level::Low, Speed::Low); // LD1
    let mut yellow_led = Output::new(p.PE1, Level::Low, Speed::Low); // LD2
    let red_led = Output::new(p.PB14, Level::Low, Speed::Low); // LD3 (heartbeat)

    // Start heartbeat task
    spawner.spawn(unwrap!(blink_heartbeat(red_led)));

    // Track previous values to detect changes
    let mut prev_green = false;
    let mut prev_yellow = false;
    let mut prev_counter = 0;

    info!("CM4: Starting main loop");
    loop {
        // Read current values from shared memory
        let green_state = SHARED_LED_STATE.get_led(true);
        let yellow_state = SHARED_LED_STATE.get_led(false);
        let counter = SHARED_LED_STATE.get_counter();

        // Detect changes
        let green_changed = green_state != prev_green;
        let yellow_changed = yellow_state != prev_yellow;
        let counter_changed = counter != prev_counter;

        // Update LEDs and logs when values change
        if green_changed || yellow_changed || counter_changed {
            if counter_changed {
                info!("CM4: Counter = {}", counter);
                prev_counter = counter;
            }

            if green_changed {
                if green_state {
                    green_led.set_high();
                    info!("CM4: Green LED ON");
                } else {
                    green_led.set_low();
                    info!("CM4: Green LED OFF");
                }
                prev_green = green_state;
            }

            if yellow_changed {
                if yellow_state {
                    yellow_led.set_high();
                    info!("CM4: Yellow LED ON");
                } else {
                    yellow_led.set_low();
                    info!("CM4: Yellow LED OFF");
                }
                prev_yellow = yellow_state;
            }
        }

        Timer::after_millis(10).await;
    }
}
