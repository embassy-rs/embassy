#![no_std]
#![no_main]

// IMPORTANT: This must match EXACTLY the definition in CM7!
mod shared {
    use core::sync::atomic::{AtomicU32, Ordering};

    /// Shared LED state between CM7 and CM4 cores
    #[repr(C, align(4))]
    pub struct SharedLedState {
        // Magic number for validation
        pub magic: AtomicU32,
        // Counter for synchronization testing
        pub counter: AtomicU32,
        // LED states packed into a single atomic
        pub led_states: AtomicU32,
    }

    // Bit positions in led_states
    pub const GREEN_LED_BIT: u32 = 0;
    pub const YELLOW_LED_BIT: u32 = 1;

    impl SharedLedState {
        pub const fn new() -> Self {
            Self {
                magic: AtomicU32::new(0xDEADBEEF), // Magic number
                counter: AtomicU32::new(0),
                led_states: AtomicU32::new(0),
            }
        }

        /// Set LED state using safe bit operations
        #[inline(never)]
        pub fn set_led(&self, is_green: bool, state: bool) {
            let bit = if is_green { GREEN_LED_BIT } else { YELLOW_LED_BIT };

            // Use bit operations to avoid complex atomic operations
            let current = self.led_states.load(Ordering::SeqCst);

            let new_value = if state {
                current | (1 << bit) // Set bit
            } else {
                current & !(1 << bit) // Clear bit
            };

            self.led_states.store(new_value, Ordering::SeqCst);
            core::sync::atomic::compiler_fence(Ordering::SeqCst);
        }

        /// Get LED state using safe bit operations
        #[inline(never)]
        pub fn get_led(&self, is_green: bool) -> bool {
            let bit = if is_green { GREEN_LED_BIT } else { YELLOW_LED_BIT };

            let value = self.led_states.load(Ordering::SeqCst);
            core::sync::atomic::compiler_fence(Ordering::SeqCst);

            (value & (1 << bit)) != 0
        }

        /// Increment counter safely
        #[inline(never)]
        pub fn increment_counter(&self) -> u32 {
            let current = self.counter.load(Ordering::SeqCst);
            let new_value = current.wrapping_add(1);
            self.counter.store(new_value, Ordering::SeqCst);
            core::sync::atomic::compiler_fence(Ordering::SeqCst);
            new_value
        }

        /// Get counter without incrementing
        #[inline(never)]
        pub fn get_counter(&self) -> u32 {
            let value = self.counter.load(Ordering::SeqCst);
            core::sync::atomic::compiler_fence(Ordering::SeqCst);
            value
        }
    }

    #[link_section = ".ram_d3"]
    pub static SHARED_LED_STATE: SharedLedState = SharedLedState::new();

    // SRAM4 memory region constants for MPU configuration
    pub const SRAM4_BASE_ADDRESS: u32 = 0x38000000;
    pub const SRAM4_SIZE_LOG2: u32 = 15; // 64KB = 2^(15+1)
    pub const SRAM4_REGION_NUMBER: u8 = 0; // MPU region number to use
}

use core::mem::MaybeUninit;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::SharedData;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// Use our shared state from the module
use shared::SHARED_LED_STATE;

#[link_section = ".ram_d3"]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

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

    // Read the magic value to ensure shared memory is accessible
    let magic = SHARED_LED_STATE.magic.load(core::sync::atomic::Ordering::SeqCst);
    info!("CM4: Magic value = 0x{:X}", magic);

    // Initialize LEDs
    let mut green_led = Output::new(p.PB0, Level::Low, Speed::Low);   // LD1
    let mut yellow_led = Output::new(p.PE1, Level::Low, Speed::Low);  // LD2
    let red_led = Output::new(p.PB14, Level::Low, Speed::Low);        // LD3 (heartbeat)

    // Start heartbeat task
    unwrap!(spawner.spawn(blink_heartbeat(red_led)));

    // Previous values for detecting changes
    let mut prev_green = false;
    let mut prev_yellow = false;
    let mut prev_counter = 0;

    info!("CM4: Starting main loop");
    loop {
        // Read values from shared memory
        let green_state = SHARED_LED_STATE.get_led(true);
        let yellow_state = SHARED_LED_STATE.get_led(false);
        let counter = SHARED_LED_STATE.get_counter();

        // Check for state changes
        let green_changed = green_state != prev_green;
        let yellow_changed = yellow_state != prev_yellow;
        let counter_changed = counter != prev_counter;

        // If any state changed, log it and update LEDs
        if green_changed || yellow_changed || counter_changed {
            if counter_changed {
                info!("CM4: Counter = {}", counter);
                prev_counter = counter;
            }

            // Update LED states
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

        // Poll at a reasonable rate
        Timer::after_millis(10).await;
    }
}