#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use cortex_m::asm;
use cortex_m::peripheral::MPU;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{Config, SharedData};
use embassy_time::Timer;
use shared::{SHARED_LED_STATE, SRAM4_BASE_ADDRESS, SRAM4_REGION_NUMBER, SRAM4_SIZE_LOG2};
use {defmt_rtt as _, panic_probe as _};

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
        #[allow(dead_code)]
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
        #[allow(dead_code)]
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

#[link_section = ".ram_d3"]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

// Function to configure MPU with your provided settings
fn configure_mpu_non_cacheable(mpu: &mut MPU) {
    // Ensure all operations complete before reconfiguring MPU/caches
    asm::dmb();
    unsafe {
        // Disable MPU
        mpu.ctrl.write(0);

        // Configure SRAM4 as non-cacheable
        // Set region number (0)
        mpu.rnr.write(SRAM4_REGION_NUMBER as u32);

        // Set base address (SRAM4 = 0x38000000) with VALID bit and region number
        mpu.rbar.write(
            SRAM4_BASE_ADDRESS | (1 << 4), // Region number = 0 (explicit in RBAR)
        );

        // Configure region attributes:
        // SIZE=15 (64KB = 2^(15+1))
        // ENABLE=1
        // AP=3 (Full access)
        // TEX=1, S=1, C=0, B=0 (Normal memory, Non-cacheable, Shareable)
        let rasr_value: u32 = (SRAM4_SIZE_LOG2 << 1) | // SIZE=15 (64KB)
            (1 << 0) |                                // ENABLE=1
            (3 << 24) |                               // AP=3 (Full access)
            (1 << 19) |                               // TEX=1
            (1 << 18); // S=1 (Shareable)

        mpu.rasr.write(rasr_value);

        // Enable MPU with default memory map as background
        mpu.ctrl.write(1 | (1 << 2)); // MPU_ENABLE | PRIVDEFENA
    }

    // Ensure changes are committed
    asm::dsb();
    asm::isb();

    info!("MPU configured - SRAM4 set as non-cacheable");
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Configure MPU to make SRAM4 non-cacheable
    {
        let mut cp = cortex_m::Peripherals::take().unwrap();
        let scb = &mut cp.SCB;

        scb.disable_icache();
        scb.disable_dcache(&mut cp.CPUID);

        // 2. MPU setup
        configure_mpu_non_cacheable(&mut cp.MPU);

        // 3. re-enable caches
        scb.enable_icache();
        scb.enable_dcache(&mut cp.CPUID);
        asm::dsb();
        asm::isb();
    }

    // Configure the clocks
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

    // Read the magic value to ensure shared memory is accessible
    let magic = SHARED_LED_STATE.magic.load(core::sync::atomic::Ordering::SeqCst);
    info!("CM7: Magic value = 0x{:X}", magic);

    // Initialize shared memory state
    SHARED_LED_STATE.set_led(true, false); // Green LED off
    SHARED_LED_STATE.set_led(false, false); // Yellow LED off

    // Main loop - update shared memory values
    let mut green_state = false;
    let mut yellow_state = false;
    let mut loop_count = 0;

    info!("CM7: Starting main loop");
    loop {
        // Update loop counter
        loop_count += 1;

        // Update shared counter
        let counter = SHARED_LED_STATE.increment_counter();

        // Every second, toggle green LED state
        if loop_count % 10 == 0 {
            green_state = !green_state;
            SHARED_LED_STATE.set_led(true, green_state);
            info!("CM7: Counter = {}, Set green LED to {}", counter, green_state);
        }

        // Every 3 seconds, toggle yellow LED state
        if loop_count % 30 == 0 {
            yellow_state = !yellow_state;
            SHARED_LED_STATE.set_led(false, yellow_state);
            info!("CM7: Counter = {}, Set yellow LED to {}", counter, yellow_state);
        }

        // Wait 100ms before next cycle
        Timer::after_millis(100).await;
    }
}
