#![no_std]
#![no_main]

// Drives the on-board MX25LM51245G octal NOR flash (512 Mbit) on the STM32H735G-DK,
// wired to OCTOSPI1 as an octal-DTR (8-8-8, double-transfer-rate) device, in both
// indirect (erase/program/read) and memory-mapped (0x9000_0000) modes.
//
// Use embassy-stm32 feature "stm32h735ig" and probe-rs chip "STM32H735IGKx".
//
// Structure mirrors examples/stm32u5/src/bin/hspi_memory_mapped.rs (same Macronix octa
// flash family, near-identical opcodes) retargeted from the `hspi` module to `ospi`,
// with the blocking driver. Pins / OCTOSPIM routing / RCC follow the board's other
// OCTOSPI examples.
//
// Two octal-DTR TransferConfig requirements, both verified on hardware and both easy to
// miss because nothing in embassy sets them for you:
//
//  1. DQS on reads (`dqse: true`). Octal-DTR read data is sampled against the strobe the
//     flash drives on DQS; without it the controller samples on its own clock and the
//     data comes back bit/byte-misaligned (0xFF reads back as 0x88, the JEDEC id shuffles).
//     Every octal-DTR READ config below sets `dqse: true`; writes/commands leave it off
//     (DQS is a read strobe only).
//
//  2. DDTR on no-data commands (`ddtr: true`). This config enables
//     `delay_hold_quarter_cycle` (DHQC) for read-timing margin, matching ST's BSP. With
//     DHQC on, the OCTOSPI quarter-cycle delay logic is gated by the CCR DDTR bit, so
//     EVERY octal-DTR command must set `ddtr: true` — even instruction-only (WriteEnable)
//     and instruction+address (erase) commands that have no data phase. Omit it and those
//     commands silently no-op while reads keep working. ST's HAL auto-sets DDTR in this
//     case; the embassy driver does not (the logic exists only as dead code in
//     configure_command), so the caller must.

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::mode::Blocking;
use embassy_stm32::ospi::{
    AddressSize, ChipSelectHighTime, Config, DummyCycles, FIFOThresholdLevel, Instance, MemorySize, MemoryType, Ospi,
    OspiWidth, TransferConfig, WrapSize,
};
use {defmt_rtt as _, panic_probe as _};

/// OCTOSPI1 memory-mapped window base (silicon-fixed, RM0468 memory map).
const MEMORY_MAPPED_BASE: u32 = 0x9000_0000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start ospi_memory_mapped (MX25LM51245G octal-DTR NOR on OCTOSPI1)");

    let p = rcc_setup::stm32h735g_init();

    let flash_config = Config {
        fifo_threshold: FIFOThresholdLevel::_4Bytes,
        memory_type: MemoryType::Macronix,
        device_size: MemorySize::_64MiB, // 512 Mbit
        chip_select_high_time: ChipSelectHighTime::_2Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        clock_prescaler: 1, // 200 MHz kernel / 2 = 100 MHz bus
        sample_shifting: false,
        delay_hold_quarter_cycle: true, // see the DHQC/DDTR note at the top of the file
        chip_select_boundary: 0,
        delay_block_bypass: false,
        max_transfer: 0,
        refresh: 0,
    };

    // Pin map: docs/HARDWARE.md "OCTOSPI1 <-> NOR flash pin map". OCTOSPIM Port 1; the
    // pin-group const generics are inferred from the pins.
    let ospi = Ospi::new_blocking_octospi_with_dqs(
        p.OCTOSPI1,
        p.PF10, // CLK
        p.PD11, // IO0
        p.PD12, // IO1
        p.PE2,  // IO2
        p.PD13, // IO3
        p.PD4,  // IO4
        p.PD5,  // IO5
        p.PG9,  // IO6
        p.PD7,  // IO7
        p.PG6,  // NCS
        p.PB2,  // DQS
        flash_config,
    );

    let mut flash = OctaDtrFlashMemory::new(ospi);

    // MX25LM51245G JEDEC ID: C2 (Macronix), 85 (octal 3 V type), 3A (512 Mbit density).
    let flash_id = flash.read_id();
    info!("FLASH ID: {=[u8]:x}", flash_id);
    assert_eq!(flash_id[0], 0xC2, "unexpected manufacturer id (not Macronix)");

    let mut rd_buf = [0u8; 16];

    flash.erase_sector(0);
    flash.read_memory(0, &mut rd_buf);
    info!("READ after erase: {=[u8]:#X}", rd_buf);
    assert_eq!(rd_buf[0], 0xFF);
    assert_eq!(rd_buf[15], 0xFF);

    let mut wr_buf = [0u8; 16];
    for (i, b) in wr_buf.iter_mut().enumerate() {
        *b = i as u8;
    }
    info!("WRITE BUF: {=[u8]:#X}", wr_buf);
    flash.write_memory(0, &wr_buf);
    flash.read_memory(0, &mut rd_buf);
    info!("READ after program: {=[u8]:#X}", rd_buf);
    assert_eq!(rd_buf[0], 0x00);
    assert_eq!(rd_buf[15], 0x0F);

    flash.enable_mm();
    info!("Enabled memory-mapped mode at 0x{=u32:08X}", MEMORY_MAPPED_BASE);

    let first_u32 = unsafe { *(MEMORY_MAPPED_BASE as *const u32) };
    info!("first_u32: 0x{=u32:X}", first_u32);
    assert_eq!(first_u32, 0x03020100);

    let second_u32 = unsafe { *((MEMORY_MAPPED_BASE + 4) as *const u32) };
    info!("second_u32: 0x{=u32:X}", second_u32);
    assert_eq!(second_u32, 0x07060504);

    info!("DONE");
}

/// Blocking octal-DTR driver for the Macronix MX25LM51245G NOR flash. Chip commands are
/// hard-coded as they depend on the chip used; this enables Octal I/O (OPI) + Double
/// Transfer Rate (DTR) and programs the memory-mapped read/write command profiles.
pub struct OctaDtrFlashMemory<'d, I: Instance> {
    ospi: Ospi<'d, I, Blocking>,
}

impl<'d, I: Instance> OctaDtrFlashMemory<'d, I> {
    const MEMORY_PAGE_SIZE: usize = 256;

    // Octal-DTR opcodes are 16-bit (byte, byte ^ 0xFF) Macronix command-integrity pairs.
    const CMD_READ_OCTA_DTR: u16 = 0xEE11;
    const CMD_PAGE_PROGRAM_OCTA_DTR: u16 = 0x12ED;
    const CMD_READ_ID_OCTA_DTR: u16 = 0x9F60;

    const CMD_RESET_ENABLE: u8 = 0x66;
    const CMD_RESET_ENABLE_OCTA_DTR: u16 = 0x6699;
    const CMD_RESET: u8 = 0x99;
    const CMD_RESET_OCTA_DTR: u16 = 0x9966;

    const CMD_WRITE_ENABLE: u8 = 0x06;
    const CMD_WRITE_ENABLE_OCTA_DTR: u16 = 0x06F9;

    const CMD_SECTOR_ERASE_OCTA_DTR: u16 = 0x21DE;
    const CMD_BLOCK_ERASE_OCTA_DTR: u16 = 0xDC23;

    const CMD_READ_SR_OCTA_DTR: u16 = 0x05FA;

    const CMD_WRITE_CR2: u8 = 0x72;

    const CR2_REG1_ADDR: u32 = 0x0000_0000;
    const CR2_OCTA_DTR: u8 = 0x02; // CR2[0]: bit1 = DOPI (octal DTR) enable
    const CR2_REG3_ADDR: u32 = 0x0000_0300;
    const CR2_DC_6_CYCLES: u8 = 0x07; // CR2[3]: array-read dummy-cycle count = 6

    pub fn new(ospi: Ospi<'d, I, Blocking>) -> Self {
        let mut memory = Self { ospi };
        memory.reset_memory();
        memory.enable_octa_dtr();
        memory
    }

    /// Switch the flash from its power-on SPI 1-1-1 mode into octal DTR, by programming
    /// its CR2 registers while still in SPI framing (the chip's current, valid protocol).
    fn enable_octa_dtr(&mut self) {
        self.write_enable_spi();
        self.write_cr2_spi(Self::CR2_REG3_ADDR, Self::CR2_DC_6_CYCLES);
        self.write_enable_spi();
        self.write_cr2_spi(Self::CR2_REG1_ADDR, Self::CR2_OCTA_DTR);
    }

    /// Enter memory-mapped mode. The read profile is the octal-DTR fast read; the write
    /// profile is page program. The embassy driver force-enables DQS on the write
    /// profile (an ES0491 HyperRAM errata workaround) — harmless here since NOR
    /// programming goes through indirect commands, not memory-mapped writes.
    pub fn enable_mm(&mut self) {
        let read_config = TransferConfig {
            iwidth: OspiWidth::OCTO,
            instruction: Some(Self::CMD_READ_OCTA_DTR as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: OspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            addtr: true,
            dwidth: OspiWidth::OCTO,
            ddtr: true,
            dqse: true, // octal-DTR reads sample against the device DQS strobe (see file header)
            dummy: DummyCycles::_6,
            ..Default::default()
        };
        let write_config = TransferConfig {
            iwidth: OspiWidth::OCTO,
            instruction: Some(Self::CMD_PAGE_PROGRAM_OCTA_DTR as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: OspiWidth::OCTO,
            adsize: AddressSize::_32bit,
            addtr: true,
            dwidth: OspiWidth::OCTO,
            ddtr: true,
            ..Default::default()
        };
        self.ospi.enable_memory_mapped_mode(read_config, write_config).unwrap();
    }

    fn exec_command_spi(&mut self, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: OspiWidth::SING,
            instruction: Some(cmd as u32),
            ..Default::default()
        };
        self.ospi.blocking_command(&transaction).unwrap();
    }

    fn exec_command_octa_dtr(&mut self, cmd: u16) {
        let transaction = TransferConfig {
            iwidth: OspiWidth::OCTO,
            instruction: Some(cmd as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            ddtr: true, // mandatory with DHQC even though there is no data phase (see file header)
            ..Default::default()
        };
        self.ospi.blocking_command(&transaction).unwrap();
    }

    fn wait_write_finish_octa_dtr(&mut self) {
        while (self.read_sr_octa_dtr() & 0x01) != 0 {} // poll WIP=0
    }

    /// Defensive reset: the flash has no hardware reset pin, so issue reset-enable +
    /// reset in every possible prior framing (octal-DTR, then SPI). After this the chip
    /// is in its power-on SPI 1-1-1 mode regardless of how a previous session left it.
    pub fn reset_memory(&mut self) {
        self.exec_command_octa_dtr(Self::CMD_RESET_ENABLE_OCTA_DTR);
        self.exec_command_octa_dtr(Self::CMD_RESET_OCTA_DTR);
        self.exec_command_spi(Self::CMD_RESET_ENABLE);
        self.exec_command_spi(Self::CMD_RESET);
    }

    fn write_enable_spi(&mut self) {
        self.exec_command_spi(Self::CMD_WRITE_ENABLE);
    }

    fn write_enable_octa_dtr(&mut self) {
        self.exec_command_octa_dtr(Self::CMD_WRITE_ENABLE_OCTA_DTR);
    }

    pub fn read_id(&mut self) -> [u8; 3] {
        let mut buffer = [0; 6];
        let transaction = TransferConfig {
            iwidth: OspiWidth::OCTO,
            instruction: Some(Self::CMD_READ_ID_OCTA_DTR as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: OspiWidth::OCTO,
            address: Some(0),
            adsize: AddressSize::_32bit,
            addtr: true,
            dwidth: OspiWidth::OCTO,
            ddtr: true,
            dqse: true, // octal-DTR reads sample against the device DQS strobe (see file header)
            dummy: DummyCycles::_5,
            ..Default::default()
        };
        self.ospi.blocking_read(&mut buffer, transaction).unwrap();
        // DTR doubles every byte on the bus; take every other one.
        [buffer[0], buffer[2], buffer[4]]
    }

    pub fn read_memory(&mut self, addr: u32, buffer: &mut [u8]) {
        let transaction = TransferConfig {
            iwidth: OspiWidth::OCTO,
            instruction: Some(Self::CMD_READ_OCTA_DTR as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: OspiWidth::OCTO,
            address: Some(addr),
            adsize: AddressSize::_32bit,
            addtr: true,
            dwidth: OspiWidth::OCTO,
            ddtr: true,
            dqse: true, // octal-DTR reads sample against the device DQS strobe (see file header)
            dummy: DummyCycles::_6,
            ..Default::default()
        };
        self.ospi.blocking_read(buffer, transaction).unwrap();
    }

    fn perform_erase_octa_dtr(&mut self, addr: u32, cmd: u16) {
        let transaction = TransferConfig {
            iwidth: OspiWidth::OCTO,
            instruction: Some(cmd as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: OspiWidth::OCTO,
            address: Some(addr),
            adsize: AddressSize::_32bit,
            addtr: true,
            ddtr: true, // mandatory with DHQC even though there is no data phase (see file header)
            ..Default::default()
        };
        self.write_enable_octa_dtr();
        self.ospi.blocking_command(&transaction).unwrap();
        self.wait_write_finish_octa_dtr();
    }

    pub fn erase_sector(&mut self, addr: u32) {
        info!("Erasing 4K sector at address: 0x{:X}", addr);
        self.perform_erase_octa_dtr(addr, Self::CMD_SECTOR_ERASE_OCTA_DTR);
    }

    pub fn erase_block(&mut self, addr: u32) {
        info!("Erasing 64K block at address: 0x{:X}", addr);
        self.perform_erase_octa_dtr(addr, Self::CMD_BLOCK_ERASE_OCTA_DTR);
    }

    fn write_page_octa_dtr(&mut self, addr: u32, buffer: &[u8], len: usize) {
        assert!(
            (len as u32 + (addr & 0x0000_00ff)) <= Self::MEMORY_PAGE_SIZE as u32,
            "write_page(): page write length exceeds page boundary (len = {}, addr = {:X})",
            len,
            addr
        );
        let transaction = TransferConfig {
            iwidth: OspiWidth::OCTO,
            instruction: Some(Self::CMD_PAGE_PROGRAM_OCTA_DTR as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: OspiWidth::OCTO,
            address: Some(addr),
            adsize: AddressSize::_32bit,
            addtr: true,
            dwidth: OspiWidth::OCTO,
            ddtr: true,
            ..Default::default()
        };
        self.write_enable_octa_dtr();
        self.ospi.blocking_write(buffer, transaction).unwrap();
        self.wait_write_finish_octa_dtr();
    }

    pub fn write_memory(&mut self, addr: u32, buffer: &[u8]) {
        let mut left = buffer.len();
        let mut place = addr;
        let mut chunk_start = 0;

        while left > 0 {
            let max_chunk_size = Self::MEMORY_PAGE_SIZE - (place & 0x0000_00ff) as usize;
            let chunk_size = if left >= max_chunk_size { max_chunk_size } else { left };
            let chunk = &buffer[chunk_start..(chunk_start + chunk_size)];
            self.write_page_octa_dtr(place, chunk, chunk_size);
            place += chunk_size as u32;
            left -= chunk_size;
            chunk_start += chunk_size;
        }
    }

    pub fn read_sr_octa_dtr(&mut self) -> u8 {
        let mut buffer = [0; 2];
        let transaction = TransferConfig {
            iwidth: OspiWidth::OCTO,
            instruction: Some(Self::CMD_READ_SR_OCTA_DTR as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: OspiWidth::OCTO,
            address: Some(0),
            adsize: AddressSize::_32bit,
            addtr: true,
            dwidth: OspiWidth::OCTO,
            ddtr: true,
            dqse: true, // octal-DTR reads sample against the device DQS strobe (see file header)
            dummy: DummyCycles::_5,
            ..Default::default()
        };
        self.ospi.blocking_read(&mut buffer, transaction).unwrap();
        buffer[0]
    }

    pub fn write_cr2_spi(&mut self, addr: u32, value: u8) {
        let buffer = [value; 1];
        let transaction = TransferConfig {
            iwidth: OspiWidth::SING,
            instruction: Some(Self::CMD_WRITE_CR2 as u32),
            adwidth: OspiWidth::SING,
            address: Some(addr),
            adsize: AddressSize::_32bit,
            dwidth: OspiWidth::SING,
            ..Default::default()
        };
        self.ospi.blocking_write(&buffer, transaction).unwrap();
    }
}

mod rcc_setup {
    use embassy_stm32::rcc::mux::Fmcsel;
    use embassy_stm32::rcc::{Hse, HseMode, *};
    use embassy_stm32::time::Hertz;
    use embassy_stm32::{Config, Peripherals};

    /// Clocks for the STM32H735G-DK: SYSCLK 520 MHz (PLL1), OCTOSPI kernel clock 200 MHz
    /// (PLL2_R, routed via `mux.octospisel`) — shared by both OCTOSPI instances. Matches
    /// the board's other OCTOSPI examples.
    pub fn stm32h735g_init() -> Peripherals {
        let mut config = Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(25),
            mode: HseMode::Oscillator,
        });
        config.rcc.hsi = None;
        config.rcc.csi = false;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5,
            mul: PllMul::Mul104,
            divp: Some(PllDiv::Div1),
            divq: Some(PllDiv::Div4),
            divr: Some(PllDiv::Div2),
        });
        // numbers adapted from Drivers/BSP/STM32H735G-DK/stm32h735g_discovery_ospi.c
        // MX_OSPI_ClockConfig
        config.rcc.pll2 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5,
            mul: PllMul::Mul80,
            divp: Some(PllDiv::Div5),
            divq: Some(PllDiv::Div2),
            divr: Some(PllDiv::Div2), // pll2_r = 200 MHz
        });
        config.rcc.voltage_scale = VoltageScale::Scale0;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div2;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.apb3_pre = APBPrescaler::Div2;
        config.rcc.apb4_pre = APBPrescaler::Div2;
        config.rcc.mux.octospisel = Fmcsel::Pll2R;
        embassy_stm32::init(config)
    }
}
