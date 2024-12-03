#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::fmc::Fmc;
use embassy_stm32::Config;
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // 100mhz
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut core_peri = cortex_m::Peripherals::take().unwrap();

    // taken from stm32h7xx-hal
    core_peri.SCB.enable_icache();
    // See Errata Sheet 2.2.1
    // core_peri.SCB.enable_dcache(&mut core_peri.CPUID);
    core_peri.DWT.enable_cycle_counter();
    // ----------------------------------------------------------
    // Configure MPU for external SDRAM
    // MPU config for SDRAM write-through
    let sdram_size = 32 * 1024 * 1024;

    {
        let mpu = core_peri.MPU;
        let scb = &mut core_peri.SCB;
        let size = sdram_size;
        // Refer to ARM®v7-M Architecture Reference Manual ARM DDI 0403
        // Version E.b Section B3.5
        const MEMFAULTENA: u32 = 1 << 16;

        unsafe {
            /* Make sure outstanding transfers are done */
            cortex_m::asm::dmb();

            scb.shcsr.modify(|r| r & !MEMFAULTENA);

            /* Disable the MPU and clear the control register*/
            mpu.ctrl.write(0);
        }

        const REGION_NUMBER0: u32 = 0x00;
        const REGION_BASE_ADDRESS: u32 = 0xD000_0000;

        const REGION_FULL_ACCESS: u32 = 0x03;
        const REGION_CACHEABLE: u32 = 0x01;
        const REGION_WRITE_BACK: u32 = 0x01;
        const REGION_ENABLE: u32 = 0x01;

        crate::assert_eq!(size & (size - 1), 0, "SDRAM memory region size must be a power of 2");
        crate::assert_eq!(size & 0x1F, 0, "SDRAM memory region size must be 32 bytes or more");
        fn log2minus1(sz: u32) -> u32 {
            for i in 5..=31 {
                if sz == (1 << i) {
                    return i - 1;
                }
            }
            crate::panic!("Unknown SDRAM memory region size!");
        }

        //info!("SDRAM Memory Size 0x{:x}", log2minus1(size as u32));

        // Configure region 0
        //
        // Cacheable, outer and inner write-back, no write allocate. So
        // reads are cached, but writes always write all the way to SDRAM
        unsafe {
            mpu.rnr.write(REGION_NUMBER0);
            mpu.rbar.write(REGION_BASE_ADDRESS);
            mpu.rasr.write(
                (REGION_FULL_ACCESS << 24)
                    | (REGION_CACHEABLE << 17)
                    | (REGION_WRITE_BACK << 16)
                    | (log2minus1(size as u32) << 1)
                    | REGION_ENABLE,
            );
        }

        const MPU_ENABLE: u32 = 0x01;
        const MPU_DEFAULT_MMAP_FOR_PRIVILEGED: u32 = 0x04;

        // Enable
        unsafe {
            mpu.ctrl.modify(|r| r | MPU_DEFAULT_MMAP_FOR_PRIVILEGED | MPU_ENABLE);

            scb.shcsr.modify(|r| r | MEMFAULTENA);

            // Ensure MPU settings take effect
            cortex_m::asm::dsb();
            cortex_m::asm::isb();
        }
    }

    let mut sdram = Fmc::sdram_a12bits_d32bits_4banks_bank2(
        p.FMC,
        // A0-A11
        p.PF0,
        p.PF1,
        p.PF2,
        p.PF3,
        p.PF4,
        p.PF5,
        p.PF12,
        p.PF13,
        p.PF14,
        p.PF15,
        p.PG0,
        p.PG1,
        // BA0-BA1
        p.PG4,
        p.PG5,
        // D0-D31
        p.PD14,
        p.PD15,
        p.PD0,
        p.PD1,
        p.PE7,
        p.PE8,
        p.PE9,
        p.PE10,
        p.PE11,
        p.PE12,
        p.PE13,
        p.PE14,
        p.PE15,
        p.PD8,
        p.PD9,
        p.PD10,
        p.PH8,
        p.PH9,
        p.PH10,
        p.PH11,
        p.PH12,
        p.PH13,
        p.PH14,
        p.PH15,
        p.PI0,
        p.PI1,
        p.PI2,
        p.PI3,
        p.PI6,
        p.PI7,
        p.PI9,
        p.PI10,
        // NBL0 - NBL3
        p.PE0,
        p.PE1,
        p.PI4,
        p.PI5,
        p.PH7,  // SDCKE1
        p.PG8,  // SDCLK
        p.PG15, // SDNCAS
        p.PH6,  // SDNE1 (!CS)
        p.PF11, // SDRAS
        p.PC0,  // SDNWE, change to p.PH5 for EVAL boards
        stm32_fmc::devices::is42s32800g_6::Is42s32800g {},
    );

    let mut delay = Delay;

    let ram_slice = unsafe {
        // Initialise controller and SDRAM
        let ram_ptr: *mut u32 = sdram.init(&mut delay) as *mut _;

        // Convert raw pointer to slice
        core::slice::from_raw_parts_mut(ram_ptr, sdram_size / core::mem::size_of::<u32>())
    };

    // // ----------------------------------------------------------
    // // Use memory in SDRAM
    info!("RAM contents before writing: {:x}", ram_slice[..10]);

    ram_slice[0] = 1;
    ram_slice[1] = 2;
    ram_slice[2] = 3;
    ram_slice[3] = 4;

    info!("RAM contents after writing: {:x}", ram_slice[..10]);

    crate::assert_eq!(ram_slice[0], 1);
    crate::assert_eq!(ram_slice[1], 2);
    crate::assert_eq!(ram_slice[2], 3);
    crate::assert_eq!(ram_slice[3], 4);

    info!("Assertions succeeded.");

    loop {
        Timer::after_millis(1000).await;
    }
}
