#![no_main]
#![no_std]

//! For Nucleo STM32H7S3L8 MB1737, has MX25UW25645GXDI00
//!
//! TODO: Currently this only uses single SPI, pending flash chip documentation for octo SPI.

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::time::Hertz;
use embassy_stm32::xspi::{
    AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, Instance, MemorySize, MemoryType, TransferConfig,
    WrapSize, Xspi, XspiWidth,
};
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // RCC config
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(24_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV3,
            mul: PllMul::MUL150,
            divp: Some(PllDiv::DIV2),
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 600 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 300 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 150 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 150 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 150 Mhz
        config.rcc.apb5_pre = APBPrescaler::DIV2; // 150 Mhz
        config.rcc.voltage_scale = VoltageScale::HIGH;
    }

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    let spi_config = embassy_stm32::xspi::Config {
        fifo_threshold: FIFOThresholdLevel::_4Bytes,
        memory_type: MemoryType::Macronix,
        delay_hold_quarter_cycle: true,
        // memory_type: MemoryType::Micron,
        // delay_hold_quarter_cycle: false,
        device_size: MemorySize::_32MiB,
        chip_select_high_time: ChipSelectHighTime::_2Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        // 300mhz / (4+1) = 60mhz. Unsure the limit, need to find a MX25UW25645GXDI00 datasheet.
        clock_prescaler: 3,
        sample_shifting: false,
        chip_select_boundary: 0,
        max_transfer: 0,
        refresh: 0,
    };

    let mut cor = cortex_m::Peripherals::take().unwrap();

    // Not necessary, but recommended if using XIP
    cor.SCB.enable_icache();
    cor.SCB.enable_dcache(&mut cor.CPUID);

    let xspi = embassy_stm32::xspi::Xspi::new_blocking_xspi(
        p.XSPI2, p.PN6, p.PN2, p.PN3, p.PN4, p.PN5, p.PN8, p.PN9, p.PN10, p.PN11, p.PN1, spi_config,
    );

    let mut flash = FlashMemory::new(xspi).await;

    let flash_id = flash.read_id();
    info!("FLASH ID: {=[u8]:x}", flash_id);

    let mut wr_buf = [0u8; 8];
    for i in 0..8 {
        wr_buf[i] = 0x90 + i as u8;
    }
    let mut rd_buf = [0u8; 8];
    flash.erase_sector(0).await;
    flash.write_memory(0, &wr_buf, true).await;
    flash.read_memory(0, &mut rd_buf, true);
    info!("WRITE BUF: {=[u8]:#X}", wr_buf);
    info!("READ BUF: {=[u8]:#X}", rd_buf);
    flash.enable_mm().await;
    info!("Enabled memory mapped mode");

    let first_u32 = unsafe { *(0x70000000 as *const u32) };
    assert_eq!(first_u32, 0x93929190);
    info!("first_u32 {:08x}", first_u32);

    let second_u32 = unsafe { *(0x70000004 as *const u32) };
    assert_eq!(second_u32, 0x97969594);
    info!("second_u32 {:08x}", first_u32);

    flash.disable_mm().await;
    info!("Disabled memory mapped mode");

    info!("DONE");
    // Output pin PE3
    let mut led = Output::new(p.PE3, Level::Low, Speed::Low);

    loop {
        led.toggle();
        Timer::after_millis(1000).await;
    }
}

const MEMORY_PAGE_SIZE: usize = 8;

const CMD_READ: u8 = 0x0B;
const _CMD_QUAD_READ: u8 = 0x6B;

const CMD_WRITE_PG: u8 = 0x02;
const _CMD_QUAD_WRITE_PG: u8 = 0x32;

const CMD_READ_ID: u8 = 0x9F;
const CMD_READ_ID_OCTO: u16 = 0x9F60;

const CMD_ENABLE_RESET: u8 = 0x66;
const CMD_RESET: u8 = 0x99;

const CMD_WRITE_ENABLE: u8 = 0x06;

const CMD_CHIP_ERASE: u8 = 0xC7;
const CMD_SECTOR_ERASE: u8 = 0x20;
const CMD_BLOCK_ERASE_32K: u8 = 0x52;
const CMD_BLOCK_ERASE_64K: u8 = 0xD8;

const CMD_READ_SR: u8 = 0x05;
const CMD_READ_CR: u8 = 0x35;

const CMD_WRITE_SR: u8 = 0x01;
const CMD_WRITE_CR: u8 = 0x31;

/// Implementation of access to flash chip.
///
/// Chip commands are hardcoded as it depends on used chip.
/// This targets a MX25UW25645GXDI00.
pub struct FlashMemory<I: Instance> {
    xspi: Xspi<'static, I, Blocking>,
}

impl<I: Instance> FlashMemory<I> {
    pub async fn new(xspi: Xspi<'static, I, Blocking>) -> Self {
        let mut memory = Self { xspi };

        memory.reset_memory().await;
        memory.enable_octo();
        memory
    }

    async fn qpi_mode(&mut self) {
        // Enter qpi mode
        self.exec_command(0x38).await;

        // Set read param
        let transaction = TransferConfig {
            iwidth: XspiWidth::QUAD,
            dwidth: XspiWidth::QUAD,
            instruction: Some(0xC0),
            ..Default::default()
        };
        self.enable_write().await;
        self.xspi.blocking_write(&[0x30_u8], transaction).unwrap();
        self.wait_write_finish();
    }

    pub async fn disable_mm(&mut self) {
        self.xspi.disable_memory_mapped_mode();
    }

    pub async fn enable_mm(&mut self) {
        self.qpi_mode().await;

        let read_config = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            adwidth: XspiWidth::SING,
            adsize: AddressSize::_24bit,
            dwidth: XspiWidth::SING,
            instruction: Some(CMD_READ as u32),
            dummy: DummyCycles::_8,
            ..Default::default()
        };

        let write_config = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            adwidth: XspiWidth::SING,
            adsize: AddressSize::_24bit,
            dwidth: XspiWidth::SING,
            instruction: Some(CMD_WRITE_PG as u32),
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.xspi.enable_memory_mapped_mode(read_config, write_config).unwrap();
    }

    fn enable_octo(&mut self) {
        let cr = self.read_cr();
        // info!("Read cr: {:x}", cr);
        self.write_cr(cr | 0x02);
        // info!("Read cr after writing: {:x}", cr);
    }

    pub fn disable_octo(&mut self) {
        let cr = self.read_cr();
        self.write_cr(cr & (!(0x02)));
    }

    async fn exec_command_4(&mut self, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::QUAD,
            adwidth: XspiWidth::NONE,
            // adsize: AddressSize::_24bit,
            dwidth: XspiWidth::NONE,
            instruction: Some(cmd as u32),
            address: None,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.xspi.blocking_command(&transaction).unwrap();
    }

    async fn exec_command(&mut self, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            adwidth: XspiWidth::NONE,
            // adsize: AddressSize::_24bit,
            dwidth: XspiWidth::NONE,
            instruction: Some(cmd as u32),
            address: None,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        // info!("Excuting command: {:x}", transaction.instruction);
        self.xspi.blocking_command(&transaction).unwrap();
    }

    pub async fn reset_memory(&mut self) {
        self.exec_command_4(CMD_ENABLE_RESET).await;
        self.exec_command_4(CMD_RESET).await;
        self.exec_command(CMD_ENABLE_RESET).await;
        self.exec_command(CMD_RESET).await;
        self.wait_write_finish();
    }

    pub async fn enable_write(&mut self) {
        self.exec_command(CMD_WRITE_ENABLE).await;
    }

    pub fn read_id(&mut self) -> [u8; 3] {
        let mut buffer = [0; 3];
        let transaction: TransferConfig = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            adwidth: XspiWidth::NONE,
            // adsize: AddressSize::_24bit,
            dwidth: XspiWidth::SING,
            instruction: Some(CMD_READ_ID as u32),
            ..Default::default()
        };
        // info!("Reading id: 0x{:X}", transaction.instruction);
        self.xspi.blocking_read(&mut buffer, transaction).unwrap();
        buffer
    }

    pub fn read_id_8(&mut self) -> [u8; 3] {
        let mut buffer = [0; 3];
        let transaction: TransferConfig = TransferConfig {
            iwidth: XspiWidth::OCTO,
            isize: AddressSize::_16bit,
            adwidth: XspiWidth::OCTO,
            address: Some(0),
            adsize: AddressSize::_32bit,
            dwidth: XspiWidth::OCTO,
            instruction: Some(CMD_READ_ID_OCTO as u32),
            dummy: DummyCycles::_4,
            ..Default::default()
        };
        info!("Reading id: {:#X}", transaction.instruction);
        self.xspi.blocking_read(&mut buffer, transaction).unwrap();
        buffer
    }

    pub fn read_memory(&mut self, addr: u32, buffer: &mut [u8], use_dma: bool) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            adwidth: XspiWidth::SING,
            adsize: AddressSize::_24bit,
            dwidth: XspiWidth::SING,
            instruction: Some(CMD_READ as u32),
            dummy: DummyCycles::_8,
            // dwidth: XspiWidth::QUAD,
            // instruction: Some(CMD_QUAD_READ as u32),
            // dummy: DummyCycles::_8,
            address: Some(addr),
            ..Default::default()
        };
        if use_dma {
            self.xspi.blocking_read(buffer, transaction).unwrap();
        } else {
            self.xspi.blocking_read(buffer, transaction).unwrap();
        }
    }

    fn wait_write_finish(&mut self) {
        while (self.read_sr() & 0x01) != 0 {}
    }

    async fn perform_erase(&mut self, addr: u32, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            adwidth: XspiWidth::SING,
            adsize: AddressSize::_24bit,
            dwidth: XspiWidth::NONE,
            instruction: Some(cmd as u32),
            address: Some(addr),
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.enable_write().await;
        self.xspi.blocking_command(&transaction).unwrap();
        self.wait_write_finish();
    }

    pub async fn erase_sector(&mut self, addr: u32) {
        self.perform_erase(addr, CMD_SECTOR_ERASE).await;
    }

    pub async fn erase_block_32k(&mut self, addr: u32) {
        self.perform_erase(addr, CMD_BLOCK_ERASE_32K).await;
    }

    pub async fn erase_block_64k(&mut self, addr: u32) {
        self.perform_erase(addr, CMD_BLOCK_ERASE_64K).await;
    }

    pub async fn erase_chip(&mut self) {
        self.exec_command(CMD_CHIP_ERASE).await;
    }

    async fn write_page(&mut self, addr: u32, buffer: &[u8], len: usize, use_dma: bool) {
        assert!(
            (len as u32 + (addr & 0x000000ff)) <= MEMORY_PAGE_SIZE as u32,
            "write_page(): page write length exceeds page boundary (len = {}, addr = {:X}",
            len,
            addr
        );

        let transaction = TransferConfig {
            iwidth: XspiWidth::SING,
            adsize: AddressSize::_24bit,
            adwidth: XspiWidth::SING,
            dwidth: XspiWidth::SING,
            instruction: Some(CMD_WRITE_PG as u32),
            // dwidth: XspiWidth::QUAD,
            // instruction: Some(CMD_QUAD_WRITE_PG as u32),
            address: Some(addr),
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.enable_write().await;
        if use_dma {
            self.xspi.blocking_write(buffer, transaction).unwrap();
        } else {
            self.xspi.blocking_write(buffer, transaction).unwrap();
        }
        self.wait_write_finish();
    }

    pub async fn write_memory(&mut self, addr: u32, buffer: &[u8], use_dma: bool) {
        let mut left = buffer.len();
        let mut place = addr;
        let mut chunk_start = 0;

        while left > 0 {
            let max_chunk_size = MEMORY_PAGE_SIZE - (place & 0x000000ff) as usize;
            let chunk_size = if left >= max_chunk_size { max_chunk_size } else { left };
            let chunk = &buffer[chunk_start..(chunk_start + chunk_size)];
            self.write_page(place, chunk, chunk_size, use_dma).await;
            place += chunk_size as u32;
            left -= chunk_size;
            chunk_start += chunk_size;
        }
    }

    fn read_register(&mut self, cmd: u8) -> u8 {
        let mut buffer = [0; 1];
        let transaction: TransferConfig = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            adwidth: XspiWidth::NONE,
            adsize: AddressSize::_24bit,
            dwidth: XspiWidth::SING,
            instruction: Some(cmd as u32),
            address: None,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.xspi.blocking_read(&mut buffer, transaction).unwrap();
        // info!("Read w25q64 register: 0x{:x}", buffer[0]);
        buffer[0]
    }

    fn write_register(&mut self, cmd: u8, value: u8) {
        let buffer = [value; 1];
        let transaction: TransferConfig = TransferConfig {
            iwidth: XspiWidth::SING,
            isize: AddressSize::_8bit,
            instruction: Some(cmd as u32),
            adsize: AddressSize::_24bit,
            adwidth: XspiWidth::NONE,
            dwidth: XspiWidth::SING,
            address: None,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.xspi.blocking_write(&buffer, transaction).unwrap();
    }

    pub fn read_sr(&mut self) -> u8 {
        self.read_register(CMD_READ_SR)
    }

    pub fn read_cr(&mut self) -> u8 {
        self.read_register(CMD_READ_CR)
    }

    pub fn write_sr(&mut self, value: u8) {
        self.write_register(CMD_WRITE_SR, value);
    }

    pub fn write_cr(&mut self, value: u8) {
        self.write_register(CMD_WRITE_CR, value);
    }
}
