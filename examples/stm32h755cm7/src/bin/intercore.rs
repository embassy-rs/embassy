#![no_std]
#![no_main]

//! STM32H7 Primary Core (CM7) Intercore Communication Example
//!
//! This example demonstrates reliable communication between the Cortex-M7 and
//! Cortex-M4 cores using a shared memory region configured as non-cacheable
//! via MPU settings.
//!
//! The CM7 core handles:
//! - MPU configuration to make shared memory non-cacheable
//! - Clock initialization
//! - Toggling LED states in shared memory
//!
//! Usage:
//! 1. Flash the CM4 (secondary) core binary first
//! 2. Then flash this CM7 (primary) core binary
//! 3. The system will start with CM7 toggling LED states and CM4 responding by
//!    physically toggling the LEDs

use core::mem::MaybeUninit;

use cortex_m::asm;
use cortex_m::peripheral::MPU;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{Config, SharedData};
use embassy_time::Timer;
use shared::{SHARED_LED_STATE, SRAM4_BASE_ADDRESS, SRAM4_REGION_NUMBER, SRAM4_SIZE_LOG2};
use {defmt_rtt as _, panic_probe as _};

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
        #[allow(dead_code)]
        pub fn get_led(&self, is_green: bool) -> bool {
            let bit = if is_green { GREEN_LED_BIT } else { YELLOW_LED_BIT };

            let value = self.led_states.load(Ordering::SeqCst);
            (value & (1 << bit)) != 0
        }

        /// Increment counter and return new value
        #[inline(never)]
        pub fn increment_counter(&self) -> u32 {
            let current = self.counter.load(Ordering::SeqCst);
            let new_value = current.wrapping_add(1);
            self.counter.store(new_value, Ordering::SeqCst);
            new_value
        }

        /// Get current counter value
        #[inline(never)]
        #[allow(dead_code)]
        pub fn get_counter(&self) -> u32 {
            let value = self.counter.load(Ordering::SeqCst);
            value
        }
    }

    #[unsafe(link_section = ".ram_d3")]
    pub static SHARED_LED_STATE: SharedLedState = SharedLedState::new();

    // Memory region constants for MPU configuration
    pub const SRAM4_BASE_ADDRESS: u32 = 0x38000000;
    pub const SRAM4_SIZE_LOG2: u32 = 15; // 64KB = 2^(15+1)
    pub const SRAM4_REGION_NUMBER: u8 = 0;
}

#[unsafe(link_section = ".ram_d3")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

/// Configure MPU to make SRAM4 region non-cacheable
fn configure_mpu_non_cacheable(mpu: &mut MPU) {
    asm::dmb();
    unsafe {
        // Disable MPU
        mpu.ctrl.write(0);

        // Configure SRAM4 as non-cacheable
        mpu.rnr.write(SRAM4_REGION_NUMBER as u32);

        // Set base address with region number
        mpu.rbar.write(SRAM4_BASE_ADDRESS | (1 << 4));

        // Configure region attributes
        let rasr_value: u32 = (SRAM4_SIZE_LOG2 << 1) | // SIZE=15 (64KB)
            (1 << 0) |                                // ENABLE=1
            (3 << 24) |                               // AP=3 (Full access)
            (1 << 19) |                               // TEX=1
            (1 << 18); // S=1 (Shareable)

        mpu.rasr.write(rasr_value);

        // Enable MPU with default memory map as background
        mpu.ctrl.write(1 | (1 << 2)); // MPU_ENABLE | PRIVDEFENA
    }

    asm::dsb();
    asm::isb();

    info!("MPU configured - SRAM4 set as non-cacheable");
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Set up MPU and cache configuration
    {
        let mut cp = cortex_m::Peripherals::take().unwrap();
        let scb = &mut cp.SCB;

        // First disable caches
        scb.disable_icache();
        scb.disable_dcache(&mut cp.CPUID);

        // Configure MPU
        configure_mpu_non_cacheable(&mut cp.MPU);

        // Re-enable caches
        scb.enable_icache();
        scb.enable_dcache(&mut cp.CPUID);
        asm::dsb();
        asm::isb();
    }

    // Configure the clock system
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Default::default());
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8),
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV2;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.apb3_pre = APBPrescaler::DIV2;
        config.rcc.apb4_pre = APBPrescaler::DIV2;
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
    }

    // Initialize the CM7 core
    let _p = embassy_stm32::init_primary(config, &SHARED_DATA);
    info!("CM7 core initialized with non-cacheable SRAM4!");

    // Verify shared memory is accessible
    let magic = SHARED_LED_STATE.magic.load(core::sync::atomic::Ordering::SeqCst);
    info!("CM7: Magic value = 0x{:X}", magic);

    // Initialize LED states
    SHARED_LED_STATE.set_led(true, false); // Green LED off
    SHARED_LED_STATE.set_led(false, false); // Yellow LED off

    // Main loop - periodically toggle LED states
    let mut green_state = false;
    let mut yellow_state = false;
    let mut loop_count = 0;

    info!("CM7: Starting main loop");
    loop {
        loop_count += 1;
        let counter = SHARED_LED_STATE.increment_counter();

        // Toggle green LED every second
        if loop_count % 10 == 0 {
            green_state = !green_state;
            SHARED_LED_STATE.set_led(true, green_state);
            info!("CM7: Counter = {}, Set green LED to {}", counter, green_state);
        }

        // Toggle yellow LED every 3 seconds
        if loop_count % 30 == 0 {
            yellow_state = !yellow_state;
            SHARED_LED_STATE.set_led(false, yellow_state);
            info!("CM7: Counter = {}, Set yellow LED to {}", counter, yellow_state);
        }

        Timer::after_millis(100).await;
    }
}
