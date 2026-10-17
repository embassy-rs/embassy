#![no_main]
#![no_std]

//! Two XSPI controllers sharing one external memory through the multiplexed
//! XSPI I/O manager (XSPIM).
//!
//! This example is intended for the Nucleo STM32H7S3L8 MB1737, which has a
//! Macronix MX25UW25645GXDI00 flash chip installed on XSPI2 (I/O port 2),
//! powered at 1.8 V. See `xspi_memory_mapped.rs` for the HSLV warnings, which
//! apply here as well.
//!
//! The XSPIM maps XSPI1 and XSPI2 both onto port 2, where the flash is wired:
//! - XSPI2, the *primary*, owns the data, clock and DQS pins and is put in
//!   memory-mapped mode (the flash is then readable at 0x7000_0000).
//! - XSPI1, the *secondary*, owns no pin and issues indirect commands to the
//!   same flash while the primary is memory-mapped.
//!
//! The example is read-only: it never erases or writes the flash. It requires
//! the flash to be in its post-reset SPI mode.

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::mode::Blocking;
use embassy_stm32::rcc::mux::Xspisel;
use embassy_stm32::rcc::{get_corrected_comp_vals, set_and_enable_comp_vals};
use embassy_stm32::time::Hertz;
use embassy_stm32::xspi::xspim::{self, Xspim};
use embassy_stm32::xspi::{
    AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, Instance, MemorySize, MemoryType, TransferConfig,
    WrapSize, Xspi, XspiWidth,
};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // RCC config
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;

        // CSI is required for HSLV and the compensation cells.
        config.rcc.csi = true;

        // Run from the external, 24 MHz crystal oscillator.
        config.rcc.hse = Some(Hse {
            freq: Hertz(24_000_000),
            mode: HseMode::Oscillator,
        });

        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div3,
            mul: PllMul::Mul150,      // 24 MHz / 3 * 150 = 1200 MHz.
            divp: Some(PllDiv::Div2), // 600 MHz PLLCLK for SYSCLK.
            divq: None,
            divr: None,
            divs: None,
            divt: None,
        });

        // PLL2_S will be used as the clock source for XSPI2.
        // A signal with a 50 % duty-cycle is required. The PLL always has a
        // 50% duty-cycle output on post-dividers divs and divt when VCOH is
        // selected (PLLxVCOSEL = 0). This should be so, for the given
        // configuration. However, the uncertainty can be readily avoided by
        // generating double the required frequency, and then post-dividing
        // by 2 in the XSPI peripheral.
        // For bus clk of 133 MHZ: 24 MHz / 4 * 133 / 3 = 266 MHz (and then /2 in XSPI)
        //                200 MHz: 24 MHz / 6 * 200 / 2 = 400 MHz (and then /2 in XSPI)
        config.rcc.pll2 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div6,
            mul: PllMul::Mul200, // 24 MHz / 6 * 200 = 800 MHz
            divp: None,
            divq: None,
            divr: None,
            divs: Some(PllDivSt::Div2), // 400 MHz for XSPI2.
            divt: None,
        });

        // XSPI2 uses PLL2S as its clock source. This allows for more flexible
        // clock rates.
        config.rcc.mux.xspi2sel = Xspisel::Pll2S;
        // XSPI1 also needs a kernel clock, as it is used as the secondary
        // controller, and is run from the same source.
        config.rcc.mux.xspi1sel = Xspisel::Pll2S;

        // Run the core and internal busses at full speed.
        config.rcc.sys = Sysclk::Pll1P; // 600 Mhz
        config.rcc.ahb_pre = AHBPrescaler::Div2; // 300 Mhz
        config.rcc.apb1_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.apb2_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.apb4_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.apb5_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.voltage_scale = VoltageScale::High;

        // Enable HSLV mode to allow high-speed operation at low supply voltage.
        // The STM32H7RS has three power domains where HSLV mode is applicable:
        // - XSPI1 I/Os supplied from VDDXSPI1
        // - XSPI2 I/Os supplied from VDDXSPI2.
        // - General I/Os supplied from VDD.
        //
        // IMPORTANT: Only enable HSLV mode when the specific power domain VDD is
        //            below 2.7 V. Enabling when VDD is > 2.7 V may be destructive!
        //            (RM0477, p.551, Ch. 8.5.10 "SBS I/O compensation cell control
        //            and status register (SBS_CCCSR)")
        //
        // IMPORTANT: JP5 selects all VDD domains EXCEPT XSPI. (Assuming the
        //            factory-default solder bridges are installed.)
        //            Make sure the jumper is in the correct position BEFORE
        //            enabling hslv_io!
        config.rcc.hslv_xspi1 = false; // NUCLEO-H7S3L8: SB20 to VDD (default).
        config.rcc.hslv_xspi2 = true; // NUCLEO-H7S3L8: SB34 to 1.8 V (default).
        config.rcc.hslv_io = false; // NUCLEO-H7S3L8: JP5 (user VDD selection) to 1.8 V or 3.3 V.

        // Selectively enable the compensation cell for the XSPI peripherals.
        // - The XSPI compensation cell auto-tune will fail (and time-out) if no
        //   active device is connected to the selected XSPI port.
        // - The GPIO compensation cell is always enabled, as it can auto-tune
        //   irregardless of connected devices.
        config.rcc.comp_xspi1 = false; // NUCLEO-H7S3L8: not connected.
        config.rcc.comp_xspi2 = true; // NUCLEO-H7S3L8: MX25UW25645GXDI00 flash.
    }

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    let cv = get_corrected_comp_vals();
    info!("Compensation Cells - Corrected Tuning Values:");
    info!("- XSPI1: NMOS {:#04X}   PMOS {:#04X}", cv.octo1_nsrc, cv.octo1_psrc);
    info!("- XSPI2: NNOS {:#04X}   PMOS {:#04X}", cv.octo2_nsrc, cv.octo2_psrc);
    info!("- GPIO:  NMOS {:#04X}   PMOS {:#04X}", cv.io_nsrc, cv.io_psrc);
    set_and_enable_comp_vals(&cv);

    let xspi_config = embassy_stm32::xspi::Config {
        fifo_threshold: FIFOThresholdLevel::_4Bytes,
        memory_type: MemoryType::Macronix,
        delay_hold_quarter_cycle: true,
        device_size: MemorySize::_32MiB,
        chip_select_high_time: ChipSelectHighTime::_2Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        // 400 MHz clock / (3 + 1) = 100 MHz. This is above the maximum for SPI
        // READ instructions so the FAST READ must be used. The nucleo board's
        // flash can run at up to 133 MHz in SPI mode and 200 MHz in OPI mode.
        // This clock prescaler must be even, otherwise the clock will not have
        // symmetric high and low times. (Note: one is internally added to the
        // clock_prescaler value defined here.)
        clock_prescaler: 3,
        sample_shifting: false,
        chip_select_boundary: 0,
        max_transfer: 0,
        refresh: 0,
    };

    // Configure the XSPIM for multiplexed operation on port 2. This must happen
    // before any of the two controllers is created. The NCS pin shared by both
    // controllers is owned by the `Xspim`.
    let xspim = Xspim::new_multiplexed_on_port2(p.PN1, xspim::Config::default());

    // The primary controller is wired to the physical pins of port 2.
    let mut primary = Xspi::new_blocking_xspi_dqs_xspim(
        p.XSPI2,
        p.PN6,
        p.PN2,
        p.PN3,
        p.PN4,
        p.PN5,
        p.PN8,
        p.PN9,
        p.PN10,
        p.PN11,
        p.PN0,
        &xspim,
        xspi_config,
    );

    // The secondary controller needs no pins. Only the XSPI instance that pairs
    // with the primary is accepted (checked at compile time).
    let mut secondary = Xspi::new_blocking_xspi_secondary(p.XSPI1, &xspim, xspi_config);

    // Give the flash time to finish its post-reset initialization.
    Timer::after_micros(50).await;
    if read_id(&mut secondary) == [0, 0, 0] {
        Timer::after_millis(100).await;
    }

    // Both controllers can talk to the flash, one after the other.
    let id_secondary = read_id(&mut secondary);
    let id_primary = read_id(&mut primary);
    info!("FLASH ID via XSPI1 (secondary): {=[u8]:x}", id_secondary);
    info!("FLASH ID via XSPI2 (primary):   {=[u8]:x}", id_primary);
    assert_eq!(id_secondary, [0xc2, 0x81, 0x39], "Flash ID incorrect");
    assert_eq!(id_primary, [0xc2, 0x81, 0x39], "Flash ID incorrect");

    // Put the primary in memory-mapped mode. While it is mapped, its prefetch
    // would keep NCS asserted and starve the secondary, so release NCS after a
    // number of idle clock cycles. The value is a starting point, tune it.
    primary.set_memory_mapped_timeout(Some(100));
    enable_mm(&mut primary);
    info!("Primary (XSPI2) is memory mapped");

    // Meanwhile, the secondary still reaches the flash with indirect commands.
    let id_secondary = read_id(&mut secondary);
    info!("FLASH ID via XSPI1 while XSPI2 is mapped: {=[u8]:x}", id_secondary);
    assert_eq!(id_secondary, [0xc2, 0x81, 0x39], "Flash ID incorrect");

    // Cross-check: the same bytes read by the secondary (indirect) and through
    // the primary's memory map (a plain load from 0x7000_0000).
    let mut indirect = [0u8; 32];
    read_memory(&mut secondary, 0, &mut indirect);
    let mut mapped = [0u8; 32];
    for (i, b) in mapped.iter_mut().enumerate() {
        *b = unsafe { core::ptr::read_volatile((0x7000_0000 + i) as *const u8) };
    }
    info!("indirect via XSPI1: {=[u8]:#X}", indirect[0..16]);
    info!("mapped via XSPI2:   {=[u8]:#X}", mapped[0..16]);
    assert_eq!(indirect, mapped, "Indirect and memory-mapped reads differ");

    primary.disable_memory_mapped_mode();
    info!("DONE");

    loop {
        Timer::after_millis(1000).await;
    }
}

fn read_id<I: Instance>(xspi: &mut Xspi<'_, I, Blocking>) -> [u8; 3] {
    let mut buffer = [0; 3];
    let transaction = TransferConfig {
        iwidth: XspiWidth::SING,
        isize: AddressSize::_8bit,
        adwidth: XspiWidth::NONE,
        dwidth: XspiWidth::SING,
        instruction: Some(0x9F), // Read identification
        ..Default::default()
    };
    xspi.blocking_read(&mut buffer, transaction).unwrap();
    buffer
}

fn read_memory<I: Instance>(xspi: &mut Xspi<'_, I, Blocking>, addr: u32, buffer: &mut [u8]) {
    let transaction = TransferConfig {
        iwidth: XspiWidth::SING,
        adwidth: XspiWidth::SING,
        adsize: AddressSize::_32bit,
        dwidth: XspiWidth::SING,
        instruction: Some(0x0C), // Fast read, 4-byte address
        dummy: DummyCycles::_8,
        address: Some(addr),
        ..Default::default()
    };
    xspi.blocking_read(buffer, transaction).unwrap();
}

fn enable_mm<I: Instance>(xspi: &mut Xspi<'_, I, Blocking>) {
    let read_config = TransferConfig {
        iwidth: XspiWidth::SING,
        isize: AddressSize::_8bit,
        adwidth: XspiWidth::SING,
        adsize: AddressSize::_32bit,
        dwidth: XspiWidth::SING,
        instruction: Some(0x0C), // Fast read, 4-byte address
        dummy: DummyCycles::_8,
        ..Default::default()
    };
    let write_config = TransferConfig {
        iwidth: XspiWidth::SING,
        isize: AddressSize::_8bit,
        adwidth: XspiWidth::SING,
        adsize: AddressSize::_32bit,
        dwidth: XspiWidth::SING,
        instruction: Some(0x12), // Page program, 4-byte address
        dummy: DummyCycles::_0,
        ..Default::default()
    };
    xspi.enable_memory_mapped_mode(read_config, write_config).unwrap();
}
