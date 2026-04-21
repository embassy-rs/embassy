//! TrustZone configuration example for STM32WBA65.
//!
//! This example shows how to use the GTZC and SAU drivers to configure TrustZone
//! security boundaries on the WBA65 family. It must run from the Secure world
//! (i.e., the linker script must place this code in Secure Flash).
//!
//! In a real dual-image firmware:
//! 1. This Secure application starts, configures SAU + GTZC, then jumps to the
//!    Non-Secure application at a known Non-Secure Flash address.
//! 2. The Non-Secure application runs with restricted access as configured below.
//!
//! # WBA65RI Memory Layout
//!
//! Flash (2 MiB total, 0x0800_0000–0x081F_FFFF):
//!   Secure Flash    : 0x0800_0000 – 0x080F_FFFF  (1 MiB)
//!   Non-Secure Flash: 0x0810_0000 – 0x081F_FFFF  (1 MiB)
//!   NSC veneer alias: 0x0C0F_E000 – 0x0C0F_FFFF  (8 KiB, in Secure alias space)
//!   OTP/Info pages  : 0x0BF9_0000 – 0x0BFB_7FFF  (non-secure)
//!
//! SRAM:
//!   SRAM1 (Secure)  : 0x2000_0000 – 0x206F_FFFF  (448 KiB, MPCBB1 28 regs)
//!   SRAM2 (NS)      : 0x2007_0000 – 0x2007_FFFF  (64 KiB,  MPCBB2 4 regs)
//!   SRAM6 (Radio)   : 0x4802_8000 – 0x4802_BFFF  (16 KiB,  MPCBB6 1 reg)
//!
//! Peripherals (Non-Secure): 0x4000_0000 – 0x4FFF_FFFF

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gtzc::{self, Mpcbb, Tzic};
use embassy_stm32::{Config, pac};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// Non-Secure application entry point address (start of NS Flash on WBA65RI).
const NS_VTOR: u32 = 0x0810_0000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let _p = embassy_stm32::init(config);

    info!("TrustZone setup example (Secure world)");

    // ── 0. Enable GTZC clock ────────────────────────────────────────────────
    //
    // GTZC registers are behind the AHB1 bus clock; accessing them without
    // enabling the clock causes a hard fault.
    unsafe { gtzc::enable_clock() };
    info!("GTZC clock enabled");

    // ── 1. Configure the SAU ────────────────────────────────────────────────
    //
    // Define which memory regions are accessible from Non-Secure code.
    // Memory NOT covered by any SAU region is Secure by default (when SAU is
    // enabled). Five regions are configured here following the ST reference
    // partition for WBA65:
    //   0 – Non-Secure Flash
    //   1 – OTP / Information pages (NS)
    //   2 – Non-Secure Callable veneer (Secure alias, NSC)
    //   3 – Non-Secure SRAM2
    //   4 – All peripherals (NS, so the NS app can use them)

    let sau_regions = [
        // Region 0: Non-Secure Flash (1 MiB, upper half of WBA65RI 2 MiB flash)
        embassy_stm32::sau::Region {
            base_address: 0x0810_0000,
            end_address: 0x081F_FFFF,
            attribute: embassy_stm32::sau::Attribute::NonSecure,
        },
        // Region 1: OTP / option bytes in NS — required for NS read of device info
        embassy_stm32::sau::Region {
            base_address: 0x0BF9_0000,
            end_address: 0x0BFB_7FFF,
            attribute: embassy_stm32::sau::Attribute::NonSecure,
        },
        // Region 2: Non-Secure Callable veneer — Secure code callable from NS world
        embassy_stm32::sau::Region {
            base_address: 0x0C0F_E000,
            end_address: 0x0C0F_FFFF,
            attribute: embassy_stm32::sau::Attribute::NonSecureCallable,
        },
        // Region 3: Non-Secure SRAM2 (64 KiB)
        embassy_stm32::sau::Region {
            base_address: 0x2007_0000,
            end_address: 0x2007_FFFF,
            attribute: embassy_stm32::sau::Attribute::NonSecure,
        },
        // Region 4: All peripherals — NS app must access its peripherals
        embassy_stm32::sau::Region {
            base_address: 0x4000_0000,
            end_address: 0x4FFF_FFFF,
            attribute: embassy_stm32::sau::Attribute::NonSecure,
        },
    ];

    unsafe {
        embassy_stm32::sau::init(&sau_regions);
    }
    info!("SAU configured: {} regions", sau_regions.len());

    // ── 2. Configure MPCBB1 (SRAM1 = 448 KiB = 28 × 16 KiB groups) ────────
    //
    // Keep all SRAM1 blocks Secure so NS code cannot read Secure data.
    let mpcbb1 = unsafe { Mpcbb::new(pac::GTZC_MPCBB1) };
    mpcbb1.set_all_secure(/*n_regs=*/ 28, true);
    mpcbb1.set_all_privileged(/*n_regs=*/ 28, false);
    // Suppress TZIC interrupts for Secure-world self-accesses to Secure blocks.
    mpcbb1.set_srwiladis(true);
    info!("MPCBB1 (SRAM1, 448 KiB): all blocks Secure");

    // ── 3. Configure MPCBB2 (SRAM2 = 64 KiB = 4 × 16 KiB groups) ──────────
    //
    // Make SRAM2 fully Non-Secure so the Non-Secure application can use it.
    let mpcbb2 = unsafe { Mpcbb::new(pac::GTZC_MPCBB2) };
    mpcbb2.set_all_secure(/*n_regs=*/ 4, false);
    mpcbb2.set_all_privileged(/*n_regs=*/ 4, false);
    info!("MPCBB2 (SRAM2, 64 KiB): all blocks Non-Secure");

    // ── 4. Configure MPCBB6 (Radio SRAM = 16 KiB = 1 × 16 KiB group) ──────
    //
    // SRAM6 is the 2.4 GHz radio TX/RX buffer at 0x4802_8000.
    // Keep it Secure so NS code cannot tamper with radio payloads.
    let mpcbb6 = unsafe { Mpcbb::new(pac::GTZC_MPCBB6) };
    mpcbb6.set_all_secure(/*n_regs=*/ 1, true);
    mpcbb6.set_all_privileged(/*n_regs=*/ 1, false);
    info!("MPCBB6 (SRAM6 radio, 16 KiB): all blocks Secure");

    // ── 5. Configure TZSC peripheral security ───────────────────────────────
    //
    // Mark specific peripherals as Non-Secure so the NS application can use them.
    // PAC field names are WBA-specific (gtzc_wba variant).
    pac::GTZC_TZSC.tzsc_seccfgr2().modify(|r| r.set_usart1sec(false));
    info!("TZSC: USART1 → Non-Secure");

    // ── 6. Enable TZIC (illegal access interrupts) ──────────────────────────
    //
    // WBA65RI has 4 TZIC register groups. Clear any stale flags first, then
    // enable IRQs so violations are reported via the GTZC interrupt.
    let tzic = unsafe { Tzic::new(pac::GTZC_TZIC) };
    tzic.enable_all(4);
    info!("TZIC: enabled for all 4 register groups");

    // ── 7. Lock the TZSC configuration ──────────────────────────────────────
    //
    // After locking, neither Secure nor Non-Secure code can modify the TZSC
    // security/privilege registers until the next reset.
    unsafe { gtzc::lock() };
    info!("TZSC locked. is_locked = {}", gtzc::is_locked());

    // ── 8. Route selected interrupts to the Non-Secure world ────────────────
    //
    // By default all interrupts target Secure world. Route the interrupts that
    // the NS application handles. IRQ numbers come from the device interrupt
    // vector table (see stm32wba65ri PAC or reference manual).
    //
    // Example: route USART1 global interrupt (IRQ #37 on WBA65) to NS.
    // unsafe { embassy_stm32::sau::route_irq_to_nonsecure(37); }

    // ── 9. Enable FPU access from Non-Secure world ──────────────────────────
    //
    // Allow NS code to use the Cortex-M33 FPU. Without this, any NS floating-
    // point operation raises a UsageFault.
    unsafe { embassy_stm32::sau::enable_nonsecure_fpu() };
    info!("FPU enabled for Non-Secure world");

    // ── 10. Jump to the Non-Secure application ──────────────────────────────
    //
    // In a real dual-image build, uncomment the line below. The NS app must be
    // linked to start at NS_VTOR (0x0810_0000) with a valid vector table.
    //
    // unsafe { embassy_stm32::sau::jump_to_nonsecure(NS_VTOR) };

    info!("TrustZone setup complete — would jump to NS app at {:#010x}.", NS_VTOR);

    loop {
        Timer::after_secs(1).await;
    }
}
