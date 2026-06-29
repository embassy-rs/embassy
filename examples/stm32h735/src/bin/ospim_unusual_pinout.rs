#![no_main]
#![no_std]

/// Demonstrates how unusual OCTOSPI pinouts can be used and that OCTOSPIM can be used in unusual ways.
///
/// This was the driving demo for testing changes to the embassy-stm32/src/ospi/mod.rs file that improved OCTOSPIM support.
///
/// Tested on a MakerPnPControl-CORE board (Rev A1) - https://github.com/MakerPnP/makerpnp-control-board
/// Important: Disconnect the CORE board from BASE board before running, as this uses pins on the IO connectors
///
/// Expected output:
/// ```
/// 0.000000 [INFO ] START (ospim_unusual_pinout src/bin/ospim_unusual_pinout.rs:19)
/// 0.000000 [TRACE] rcc: enabled 0x3d:1 (embassy_stm32 src/rcc/mod.rs:354)
/// 0.000000 [DEBUG] flash: latency=3 wrhighfreq=3 (embassy_stm32 src/rcc/h.rs:1258)
/// 0.000000 [TRACE] BDCR ok: 00008200 (embassy_stm32 src/rcc/bd.rs:307)
/// 0.000000 [DEBUG] rcc: Clocks { csi: MaybeHertz(0), hclk1: MaybeHertz(275000000), hclk2: MaybeHertz(275000000), hclk3: MaybeHertz(275000000), hclk4: MaybeHertz(275000000), hse: MaybeHertz(50000000), hsi: MaybeHertz(0), hsi48: MaybeHertz(48000000), i2s_ckin: MaybeHertz(0), lse: MaybeHertz(0), lsi: MaybeHertz(0), pclk1: MaybeHertz(137500000), pclk1_tim: MaybeHertz(275000000), pclk2: MaybeHertz(137500000), pclk2_tim: MaybeHertz(275000000), pclk3: MaybeHertz(137500000), pclk4: MaybeHertz(137500000), pll1_q: MaybeHertz(137500000), pll2_p: MaybeHertz(80000000), pll2_q: MaybeHertz(200000000), pll2_r: MaybeHertz(133333333), pll3_p: MaybeHertz(192000000), pll3_q: MaybeHertz(24000000), pll3_r: MaybeHertz(48000000), rtc: MaybeHertz(32000), sys: MaybeHertz(550000000) } (embassy_stm32 src/rcc/mod.rs:92)
/// 0.000000 [TRACE] rcc: enabled 0x3a:0 (embassy_stm32 src/rcc/mod.rs:354)
/// 0.000000 [INFO ] Hold FPGA in reset (ospim_unusual_pinout src/bin/ospim_unusual_pinout.rs:31)
/// 0.000000 [TRACE] OCTOSPI_IDX: 2 (embassy_stm32 src/ospi/mod.rs:459)
/// 0.000000 [TRACE] IOL_PGROUP: 0b01 (embassy_stm32 src/ospi/mod.rs:463)
/// 0.000030 [TRACE] IOH_PGROUP: N/A (embassy_stm32 src/ospi/mod.rs:467)
/// 0.000030 [TRACE] CLK/NCS/DQS CTRL_PGROUP: 0b10 (embassy_stm32 src/ospi/mod.rs:469)
/// 0.000030 [TRACE] OCTOSPI1_ENABLED: false, OCTOSPI2_ENABLED: false (embassy_stm32 src/ospi/mod.rs:480)
/// 0.000061 [DEBUG] OCTOSPIM_CR: 0x00FF0000 - Cr { muxen: false, req2ack_time: 255 } (embassy_stm32 src/ospi/mod.rs:502)
/// 0.000061 [DEBUG] OCTOSPIM_P1CR: 0x05010111 - P1cr { clken: true, clksrc: false, dqsen: true, dqssrc: false, ncsen: true, ncssrc: false, iolen: true, iolsrc: 0, iohen: true, iohsrc: 2 } (embassy_stm32 src/ospi/mod.rs:503)
/// 0.000091 [DEBUG] OCTOSPIM_P2CR: 0x07050323 - P2cr { clken: true, clksrc: true, dqsen: false, dqssrc: true, ncsen: true, ncssrc: true, iolen: true, iolsrc: 2, iohen: true, iohsrc: 3 } (embassy_stm32 src/ospi/mod.rs:504)
/// 0.000122 [TRACE] rcc: enabled 0x35:19 (embassy_stm32 src/rcc/mod.rs:354)
/// 0.000152 [TRACE] OCTOSPI_IDX: 1 (embassy_stm32 src/ospi/mod.rs:459)
/// 0.000152 [TRACE] IOL_PGROUP: 0b11 (embassy_stm32 src/ospi/mod.rs:463)
/// 0.000183 [TRACE] IOH_PGROUP: 0b10 (embassy_stm32 src/ospi/mod.rs:465)
/// 0.000183 [TRACE] CLK/NCS/DQS CTRL_PGROUP: 0b00 (embassy_stm32 src/ospi/mod.rs:469)
/// 0.000183 [TRACE] OCTOSPI1_ENABLED: false, OCTOSPI2_ENABLED: true (embassy_stm32 src/ospi/mod.rs:480)
/// 0.000213 [DEBUG] OCTOSPIM_CR: 0x00FF0000 - Cr { muxen: false, req2ack_time: 255 } (embassy_stm32 src/ospi/mod.rs:502)
/// 0.000213 [DEBUG] OCTOSPIM_P1CR: 0x05010111 - P1cr { clken: true, clksrc: false, dqsen: true, dqssrc: false, ncsen: true, ncssrc: false, iolen: true, iolsrc: 0, iohen: true, iohsrc: 2 } (embassy_stm32 src/ospi/mod.rs:503)
/// 0.000244 [DEBUG] OCTOSPIM_P2CR: 0x01030323 - P2cr { clken: true, clksrc: true, dqsen: false, dqssrc: true, ncsen: true, ncssrc: true, iolen: true, iolsrc: 1, iohen: true, iohsrc: 0 } (embassy_stm32 src/ospi/mod.rs:504)
/// 0.000274 [TRACE] rcc: enabled 0x35:14 (embassy_stm32 src/rcc/mod.rs:354)
/// 0.010314 [INFO ] FLASH ID: [ef, 40, 15] (ospim_unusual_pinout src/bin/ospim_unusual_pinout.rs:95)
/// 0.010314 [INFO ] Flash ID OK (ospim_unusual_pinout src/bin/ospim_unusual_pinout.rs:103)
/// ...
/// 1.911010 [INFO ] FLASH ID: [ef, 40, 15] (ospim_unusual_pinout src/bin/ospim_unusual_pinout.rs:95)
/// 1.911010 [INFO ] Flash ID OK (ospim_unusual_pinout src/bin/ospim_unusual_pinout.rs:103)
/// 2.011047 [INFO ] Result Attempts: 20, Successes: 20 (ospim_unusual_pinout src/bin/ospim_unusual_pinout.rs:113)
/// 2.011047 [INFO ] END (ospim_unusual_pinout src/bin/ospim_unusual_pinout.rs:114)
/// ```
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::ospi::{
    AddressSize, ChipSelectHighTime, FIFOThresholdLevel, Instance, MemorySize, MemoryType, Ospi, OspiWidth,
    TransferConfig, WrapSize,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START");

    // Initialize peripherals & RCC.
    let p = rcc_setup::stm32h735g_init();

    // Output pin PA8 (also MCO)
    let mut led = Output::new(p.PA8, Level::Low, Speed::Low);

    // on the test board, there is an FPGA connected to the OCTOSPI data lines so that the FPGA
    // can boot from the flash, with an unprogrammed FPGA it will have weak pull-ups on every IO pin
    // and these need to be disabled before the flash can be communicated with.

    info!("Hold FPGA in reset");
    // hold FPGA in RESET mode
    let mut fpga_creset_b = Output::new(p.PF15, Level::Low, Speed::Low);
    fpga_creset_b.set_low();

    let ospi_config = embassy_stm32::ospi::Config {
        fifo_threshold: FIFOThresholdLevel::_16Bytes,
        memory_type: MemoryType::Standard,
        device_size: MemorySize::_2MiB,
        chip_select_high_time: ChipSelectHighTime::_1Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        clock_prescaler: 3, // 133.33Mhz / (3+1) = 33.3Mhz
        sample_shifting: true,
        delay_hold_quarter_cycle: false,
        chip_select_boundary: 0,
        delay_block_bypass: true,
        max_transfer: 0,
        refresh: 0,
    };

    let ospi2 = embassy_stm32::ospi::Ospi::new_blocking_quadspi(
        p.OCTOSPI2,
        p.PF4,  // P2_CLK
        p.PD4,  // P1_IO4
        p.PH3,  // P1_IO5
        p.PC3,  // P1_IO6
        p.PE10, // P1_IO7
        p.PG12, // P2_NCS
        ospi_config,
    );

    // we init `ospi1` after the one we want to use for the flash so that we can be sure it doesn't
    // mess-up the OCTOSPIM configuration needed for the flash.

    #[allow(unused_variables)]
    let ospi1 = embassy_stm32::ospi::Ospi::new_blocking_octospi_with_dqs(
        p.OCTOSPI1,
        p.PF10, // P1_CLK
        p.PG0,  // P2_IO4
        p.PG1,  // P2_IO5
        p.PG10, // P2_IO6
        p.PG11, // P2_IO7
        p.PF0,  // P2_IO0
        p.PF1,  // P2_IO1
        p.PF2,  // P2_IO2
        p.PF3,  // P2_IO3
        p.PC11, // P1_NCS
        p.PA1,  // P1_DQS
        ospi_config,
    );

    // settling time, without this the flash doesn't respond on a warm-boot, it may require reset commands to be sent.
    Timer::after_millis(10).await;

    let mut flash = FlashMemory::new(ospi2).await;

    let mut successes = 0;
    const ATTEMPTS: u32 = 20;

    for _ in 0..ATTEMPTS {
        let flash_id = flash.read_id();
        info!("FLASH ID: {=[u8]:x}", flash_id);

        // Flash chip is a 16MBit W25Q16JV-IQ (W25Q16JVUXIQ)
        // expected output is:
        // 0FLASH ID: [ef, 40, 15] (ospim_unusual_pinout src/bin/ospim_unusual_pinout.rs:86)

        if flash_id == [0xEF, 0x40, 0x15] {
            successes += 1;
            info!("Flash ID OK");
        } else {
            info!("Flash ID mismatch: {=[u8]:x}", flash_id);
        }

        led.toggle();

        Timer::after_millis(100).await;
    }

    info!("Result Attempts: {}, Successes: {}", ATTEMPTS, successes);
    info!("END");
}

const CMD_READ_ID: u8 = 0x9F;
pub struct FlashMemory<I: Instance> {
    ospi: Ospi<'static, I, Blocking>,
}

impl<I: Instance> FlashMemory<I> {
    pub async fn new(ospi: Ospi<'static, I, Blocking>) -> Self {
        let memory = Self { ospi };

        memory
    }

    pub fn read_id(&mut self) -> [u8; 3] {
        let mut buffer = [0; 3];
        let transaction: TransferConfig = TransferConfig {
            iwidth: OspiWidth::SING,
            isize: AddressSize::_8Bit,
            adwidth: OspiWidth::NONE,
            dwidth: OspiWidth::SING,
            instruction: Some(CMD_READ_ID as u32),
            ..Default::default()
        };
        // info!("Reading id: 0x{:X}", transaction.instruction);
        self.ospi.blocking_read(&mut buffer, transaction).unwrap();
        buffer
    }
}

mod rcc_setup {

    use embassy_stm32::rcc::mux::Fmcsel;
    use embassy_stm32::rcc::{Hse, HseMode, *};
    use embassy_stm32::time::Hertz;
    use embassy_stm32::{Config, Peripherals};

    /// Sets up clocks for the stm32h735g mcu
    /// change this if you plan to use a different microcontroller
    pub fn stm32h735g_init() -> Peripherals {
        // setup power and clocks for an stm32h735g-dk run from an external 25 Mhz external oscillator
        let mut config = Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(50),
            mode: HseMode::Oscillator,
        });
        config.rcc.hsi = None;
        config.rcc.csi = false;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div4,  // 12.5Mhz
            mul: PllMul::Mul44,       // 550Mhz
            divp: Some(PllDiv::Div1), // 550Mhz
            divq: Some(PllDiv::Div4), // 110Mhz
            divr: Some(PllDiv::Div2), // 275Mhz
        });
        config.rcc.pll2 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5,  // 10Mhz
            mul: PllMul::Mul40,       // 400Mhz
            divp: Some(PllDiv::Div5), // 80Mhz
            divq: Some(PllDiv::Div2), // 200Mhz
            divr: Some(PllDiv::Div3), // 133.33Mhz (for OSPI)
        });
        // numbers adapted from Drivers/BSP/STM32H735G-DK/stm32h735g_discovery_lcd.c
        // MX_LTDC_ClockConfig
        config.rcc.pll3 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div25, // 2Mhz
            mul: PllMul::Mul96,       // 192Mhz
            divp: Some(PllDiv::Div1), // 192Mhz
            divq: Some(PllDiv::Div8), // 24Mhz
            divr: Some(PllDiv::Div4), // 48Mhz
        });
        config.rcc.voltage_scale = VoltageScale::Scale0;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
        config.rcc.sys = Sysclk::Pll1P; // 550Mhz
        config.rcc.d1c_pre = AHBPrescaler::Div1; // 550Mhz
        config.rcc.ahb_pre = AHBPrescaler::Div2; // 275Mhz
        config.rcc.apb1_pre = APBPrescaler::Div2; // 137.5Mhz
        config.rcc.apb2_pre = APBPrescaler::Div2; // 137.5Mhz
        config.rcc.apb3_pre = APBPrescaler::Div2; // 137.5Mhz
        config.rcc.apb4_pre = APBPrescaler::Div2; // 137.5Mhz

        config.rcc.mux.octospisel = Fmcsel::Pll2R; // 133.33Mhz

        embassy_stm32::init(config)
    }
}
