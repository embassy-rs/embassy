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
//! IMPORTANT: The addresses and sizes used here are illustrative. Adjust them to
//! match your actual memory layout and linker scripts.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gtzc::{self, Mpcbb, Tzic};
use embassy_stm32::{Config, pac};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let _p = embassy_stm32::init(config);

    info!("TrustZone setup example (Secure world)");

    // ── 1. Configure the SAU ────────────────────────────────────────────────
    //
    // Define which memory regions are accessible from Non-Secure code.
    // Memory NOT covered by any SAU region is Secure by default.
    //
    // STM32WBA65RI flash layout (example; adjust to your linker scripts):
    //   Secure Flash   : 0x0800_0000 – 0x0807_FFFF  (512 KiB)
    //   Non-Secure Flash: 0x0808_0000 – 0x080F_FFFF  (512 KiB)
    //   NSC veneer     : 0x0C07_F000 – 0x0C07_FFFF  (4 KiB, in Secure alias)
    //
    // SRAM layout:
    //   Secure SRAM1   : first 448 KiB → blocks 0-27 of MPCBB1
    //   Non-Secure SRAM2: 0x2007_0000 – 0x2007_FFFF  (64 KiB)

    let sau_regions = [
        embassy_stm32::sau::Region {
            base_address: 0x0808_0000, // Non-Secure Flash
            end_address: 0x080F_FFFF,
            attribute: embassy_stm32::sau::Attribute::NonSecure,
        },
        embassy_stm32::sau::Region {
            base_address: 0x2007_0000, // Non-Secure SRAM2
            end_address: 0x2007_FFFF,
            attribute: embassy_stm32::sau::Attribute::NonSecure,
        },
        embassy_stm32::sau::Region {
            base_address: 0x0C07_F000, // Non-Secure Callable veneer
            end_address: 0x0C07_FFFF,
            attribute: embassy_stm32::sau::Attribute::NonSecureCallable,
        },
    ];

    unsafe {
        embassy_stm32::sau::init(&sau_regions);
    }
    info!("SAU configured: {} regions", sau_regions.len());

    // ── 2. Configure MPCBB1 (SRAM1 = 448 KiB = 28 × 16 KiB groups) ────────
    //
    // WBA65RI SRAM1 is 448 KiB → 28 MPCBB registers (each covers 32 × 512 B = 16 KiB).
    // Keep all SRAM1 blocks Secure.
    let mpcbb1 = unsafe { Mpcbb::new(pac::GTZC_MPCBB1) };
    mpcbb1.set_all_secure(/*n_regs=*/ 14, true);
    mpcbb1.set_all_privileged(/*n_regs=*/ 14, false);
    info!("MPCBB1 (SRAM1): all blocks Secure");

    // ── 3. Configure MPCBB2 (SRAM2 = 64 KiB = 4 × 16 KiB groups) ──────────
    //
    // Make SRAM2 fully Non-Secure so the Non-Secure application can use it.
    let mpcbb2 = unsafe { Mpcbb::new(pac::GTZC_MPCBB2) };
    mpcbb2.set_all_secure(/*n_regs=*/ 4, false);
    mpcbb2.set_all_privileged(/*n_regs=*/ 4, false);
    info!("MPCBB2 (SRAM2): all blocks Non-Secure");

    // ── 4. Configure TZSC peripheral security ───────────────────────────────
    //
    // Mark USART1 as Non-Secure so the NS application can use it for logging.
    // Other peripherals remain Secure (PAC::GTZC_TZSC field names are WBA-specific).
    pac::GTZC_TZSC.tzsc_seccfgr2().modify(|r| r.set_usart1sec(false));
    info!("TZSC: USART1 → Non-Secure");

    // ── 5. Enable TZIC (illegal access interrupts) ──────────────────────────
    //
    // WBA65RI has 4 TZIC register groups.
    let tzic = unsafe { Tzic::new(pac::GTZC_TZIC) };
    tzic.enable_all(4);
    info!("TZIC: enabled for all 4 register groups");

    // ── 6. Lock the TZSC configuration ──────────────────────────────────────
    //
    // After locking, neither Secure nor Non-Secure code can modify the TZSC
    // security/privilege registers until the next reset.
    unsafe { gtzc::lock() };
    info!("TZSC locked. is_locked = {}", gtzc::is_locked());

    // In a real application you would now jump to the Non-Secure application:
    //   let ns_entry = 0x0808_0000 as *const u32;
    //   let ns_vtor  = *ns_entry as *const u32;
    //   let ns_sp    = *ns_vtor;
    //   let ns_reset = *ns_vtor.add(1);
    //   // Set NS VTOR, MSP_NS, then BLX ns_reset

    info!("TrustZone setup complete — would jump to NS app here.");

    loop {
        Timer::after_secs(1).await;
    }
}
