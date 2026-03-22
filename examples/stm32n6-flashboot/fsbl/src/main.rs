#![no_std]
#![no_main]

//! FSBL (First Stage Boot Loader) for STM32N6570-DK with embassy-boot

use core::cell::RefCell;

use cortex_m_rt::{entry, exception};
use defmt::{error, info};
use defmt_rtt as _;
use embassy_boot_stm32::*;
use embassy_stm32::pac::{BSEC, RCC, XSPI1, XSPI2, XSPIM};
use embassy_stm32::rcc::XspiClkSrc;
use embassy_stm32::xspi::{ChipSelectHighTime, FIFOThresholdLevel, MemorySize, MemoryType, WrapSize, Xspi};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

mod nor_flash;
mod xspi_flash;

use nor_flash::XspiNorFlash;
use xspi_flash::SpiFlashMemory;

const APP_LOAD_ADDR: u32 = 0x70100400;

/// Enable debug access via BSEC registers.
///
/// On BSEC-open (unfused) devices: these writes are no-ops — debug is
/// always enabled by hardware (ap_unlocked and dbg_unlocked forced to 0xB4).
///
/// On BSEC-closed devices: unlocks the debug access port and enables
/// secure + non-secure debug at HDPL >= 1 (FSBL level).
/// Both registers are write-once per warm reset and persist across warm resets.
fn enable_debug() {
    unsafe {
        // Advance HDPL from 0 (boot ROM) to 1 (FSBL) if needed.
        // On BSEC-closed devices, debug is gated by HDPL — outputs are forced
        // to 0x00 while HDPL = 0xB4 (level 0), regardless of DBGCR contents.
        let hdpl = BSEC.as_ptr().cast::<u32>().byte_add(0xE94).read();
        if hdpl & 0xFF == 0xB4 {
            // Still at HDPL0 (boot ROM level), advance to HDPL1
            BSEC.as_ptr().cast::<u32>().byte_add(0xE98).write(0x60B1_66E7);
        }

        // BSEC_AP_UNLOCK (offset 0xE90): UNLOCK[7:0] = 0xB4
        // Opens the CM55 Debug Access Port so the debugger can access
        // the AHB-AP (AP1) and AXI-AP (AP2).
        BSEC.as_ptr().cast::<u32>().byte_add(0xE90).write(0x0000_00B4);

        // BSEC_DBGCR (offset 0xE8C):
        //   AUTH_SEC[31:24]  = 0xB4 (secure debug authorized)
        //   AUTH_HDPL[23:16] = 0x51 (enable from HDPL1 = FSBL level)
        //   UNLOCK[15:8]     = 0xB4 (non-secure debug authorized)
        //   Reserved[7:0]    = 0x00
        BSEC.as_ptr().cast::<u32>().byte_add(0xE8C).write(0xB451_B400);
    }
}

#[entry]
fn main() -> ! {
    // 1. Enable debug access
    enable_debug();

    // 2. Set vector table
    let core_peri = unsafe { cortex_m::Peripherals::steal() };
    unsafe { core_peri.SCB.vtor.write(0x34180400) };

    // 3. Boot ROM used XSPI1 (mapped to port P2 via XSPIM MODE=1) to load us.
    //    Clean up: disable XSPI1 and XSPI2 before reconfiguring clocks.
    //    We set XSPIM MODE=0 (XSPI2→P2) here since it's a global mux setting.
    RCC.ahb5enr().modify(|w| {
        w.set_xspi1en(true);
        w.set_xspi2en(true);
        w.set_xspimen(true);
    });
    // Abort any residual XSPI1 state (boot ROM controller), then disable
    XSPI1.cr().modify(|w| w.set_abort(true));
    while XSPI1.cr().read().abort() {}
    XSPI1.cr().modify(|w| w.set_en(false));
    // Also clean up XSPI2 just in case
    XSPI2.cr().modify(|w| w.set_abort(true));
    while XSPI2.cr().read().abort() {}
    XSPI2.cr().modify(|w| w.set_en(false));
    // Set XSPIM MODE=0: XSPI1→P1, XSPI2→P2 (boot ROM used MODE=1, we use XSPI2)
    XSPIM.cr().modify(|w| w.set_mode(false));

    // 4. Configure clocks — PER clock (HSI 64 MHz) for XSPI2 kernel clock.
    //    Using PER avoids IC4/PLL routing, so the app's reinit() can use the
    //    same config without touching IC divider registers mid-execution.
    let mut config = embassy_stm32::Config::default();
    config.rcc.mux.xspi2sel = XspiClkSrc::PER;
    config.rcc.vddio3_1v8 = true;
    let p = embassy_stm32::init(config);

    // 5. Enable all RAM early (before XSPI init)
    RCC.memenr().modify(|w| {
        w.set_axisram1en(true);
        w.set_axisram2en(true);
        w.set_axisram3en(true);
        w.set_axisram4en(true);
        w.set_axisram5en(true);
        w.set_axisram6en(true);
        w.set_ahbsram1en(true);
        w.set_ahbsram2en(true);
        w.set_bkpsramen(true);
    });

    info!("Clocks configured, creating XSPI driver...");

    // 6. Configure XSPI2 for external NOR flash — identical to working example
    let spi_config = embassy_stm32::xspi::Config {
        fifo_threshold: FIFOThresholdLevel::_4Bytes,
        memory_type: MemoryType::Macronix,
        delay_hold_quarter_cycle: true,
        #[cfg(feature = "dk")]
        device_size: MemorySize::_128MiB,
        #[cfg(feature = "nucleo")]
        device_size: MemorySize::_64MiB,
        chip_select_high_time: ChipSelectHighTime::_2Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        clock_prescaler: 8,
        sample_shifting: true,
        chip_select_boundary: 0,
        max_transfer: 0,
        refresh: 0,
    };

    let xspi = Xspi::new_blocking_xspi(
        p.XSPI2, p.PN6, p.PN2, p.PN3, p.PN4, p.PN5, p.PN8, p.PN9, p.PN10, p.PN11, p.PN1, spi_config,
    );

    // Dump XSPI2 state via PAC to verify driver init worked
    let ahb5 = RCC.ahb5enr().read();
    info!("AHB5ENR: XSPI2={} XSPIM={}", ahb5.xspi2en(), ahb5.xspimen());
    let cr = XSPI2.cr().read();
    info!(
        "XSPI2 CR: EN={} FMODE={} CSSEL={}",
        cr.en(),
        cr.fmode() as u8,
        cr.cssel() as u8
    );
    let dcr1 = XSPI2.dcr1().read();
    info!(
        "XSPI2 DCR1: MTYP={} DEVSIZE={} CSHT={}",
        dcr1.mtyp() as u8,
        dcr1.devsize() as u8,
        dcr1.csht() as u8
    );
    let dcr2 = XSPI2.dcr2().read();
    info!("XSPI2 DCR2: PRESCALER={}", dcr2.prescaler());
    let ccipr6 = RCC.ccipr6().read();
    info!("CCIPR6: XSPI2SEL={}", ccipr6.xspi2sel() as u8);
    let xspim_cr = XSPIM.cr().read();
    info!(
        "XSPIM CR: MUXEN={} MODE={} CSSEL_OVR_EN={}",
        xspim_cr.muxen(),
        xspim_cr.mode(),
        xspim_cr.cssel_ovr_en()
    );

    // Wait for flash to stabilize (matching reference example)
    cortex_m::asm::delay(6_400_000); // ~100ms

    let mut flash_driver = SpiFlashMemory::new(xspi);

    // 7. Flash ID check
    let flash_id = flash_driver.read_id();
    info!(
        "Flash ID: [{:02x}, {:02x}, {:02x}]",
        flash_id[0], flash_id[1], flash_id[2]
    );

    #[cfg(feature = "dk")]
    const EXPECTED_ID: [u8; 3] = [0xC2, 0x86, 0x1A]; // MX66UW1G45G
    #[cfg(feature = "nucleo")]
    const EXPECTED_ID: [u8; 3] = [0xC2, 0x85, 0x3A]; // MX25UM51245G

    if flash_id == EXPECTED_ID {
        info!("Flash ID OK");
    } else if flash_id == [0, 0, 0] {
        info!("Flash ID all zeros - not responding!");
    } else {
        info!("Unexpected flash ID!");
    }

    // 8. Quick read test
    let mut buf = [0u8; 8];
    flash_driver.read_memory(0x0, &mut buf);
    let val0 = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    info!("Read offset 0x0: 0x{:08x}", val0);

    // 9. Embassy-boot
    info!("Starting embassy-boot...");

    let nor_flash = XspiNorFlash::new(flash_driver);
    let flash = Mutex::<NoopRawMutex, _>::new(RefCell::new(nor_flash));

    let config = BootLoaderConfig::from_linkerfile_blocking(&flash, &flash, &flash);
    let bl = BootLoader::prepare::<_, _, _, 4096>(config);
    info!("embassy-boot prepare done");

    // 10. Get flash driver back for MM mode
    let nor_flash = flash.into_inner().into_inner();
    let mut flash_driver = nor_flash.free();

    // 11. Enable memory-mapped mode and jump
    info!("Enabling memory-mapped mode...");
    flash_driver.enable_mm();

    cortex_m::asm::dsb();
    cortex_m::asm::isb();

    // Verify vector table via memory-mapped reads before jumping
    let vt = APP_LOAD_ADDR as *const u32;
    let initial_sp = unsafe { core::ptr::read_volatile(vt) };
    let reset_vector = unsafe { core::ptr::read_volatile(vt.add(1)) };
    info!(
        "MM read @ 0x{:08x}: SP=0x{:08x} Reset=0x{:08x}",
        APP_LOAD_ADDR, initial_sp, reset_vector
    );

    // Sanity: SP should be in SRAM (0x34xxxxxx), reset vector in ext flash (0x701xxxxx)
    if initial_sp < 0x34000000 || initial_sp > 0x34400000 {
        error!("BAD initial SP! Expected SRAM range 0x34000000-0x34400000");
    }
    if reset_vector < 0x70100000 || reset_vector > 0x70400000 {
        error!("BAD reset vector! Expected ext flash range 0x70100000-0x70400000");
    }

    // Read a few more vector table entries for debugging
    let nmi = unsafe { core::ptr::read_volatile(vt.add(2)) };
    let hardfault = unsafe { core::ptr::read_volatile(vt.add(3)) };
    info!("VT[2] NMI=0x{:08x}  VT[3] HardFault=0x{:08x}", nmi, hardfault);

    info!("Jumping to app at 0x{:08x}", APP_LOAD_ADDR);

    unsafe { bl.load(APP_LOAD_ADDR) }
}

#[unsafe(no_mangle)]
#[cfg_attr(target_os = "none", unsafe(link_section = ".HardFault.user"))]
unsafe extern "C" fn HardFault() {
    loop {
        cortex_m::asm::nop();
    }
}

#[exception]
unsafe fn DefaultHandler(_: i16) -> ! {
    loop {
        cortex_m::asm::nop();
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        error!("PANIC at {}:{}", loc.file(), loc.line());
    } else {
        error!("PANIC (no location)");
    }
    cortex_m::asm::udf();
}
