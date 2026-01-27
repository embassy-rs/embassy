#![no_main]
#![no_std]

// Tested on an STM32U5G9J-DK2 demo board using the on-board MX66LM1G45G flash memory
// The flash is connected to the HSPI1 port as an OCTA-DTR device
//
// Use embassy-stm32 feature "stm32u5g9zj" and probe-rs chip "STM32U5G9ZJTxQ"

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::hspi::{
    AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, Hspi, HspiWidth, Instance, MemorySize,
    MemoryType, TransferConfig, WrapSize,
};
use embassy_stm32::mode::Async;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, dma, peripherals, rcc};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL7 => dma::InterruptHandler<peripherals::GPDMA1_CH7>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start hspi_memory_mapped");

    // RCC config
    let mut config = embassy_stm32::Config::default();
    config.rcc.hse = Some(rcc::Hse {
        freq: Hertz(16_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    config.rcc.pll1 = Some(rcc::Pll {
        source: rcc::PllSource::HSE,
        prediv: rcc::PllPreDiv::DIV1,
        mul: rcc::PllMul::MUL10,
        divp: None,
        divq: None,
        divr: Some(rcc::PllDiv::DIV1),
    });
    config.rcc.sys = rcc::Sysclk::PLL1_R; // 160 Mhz
    config.rcc.pll2 = Some(rcc::Pll {
        source: rcc::PllSource::HSE,
        prediv: rcc::PllPreDiv::DIV4,
        mul: rcc::PllMul::MUL66,
        divp: None,
        divq: Some(rcc::PllDiv::DIV2),
        divr: None,
    });
    config.rcc.mux.hspi1sel = rcc::mux::Hspisel::PLL2_Q; // 132 MHz

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    let flash_config = embassy_stm32::hspi::Config {
        fifo_threshold: FIFOThresholdLevel::_4Bytes,
        memory_type: MemoryType::Macronix,
        device_size: MemorySize::_1GiB,
        chip_select_high_time: ChipSelectHighTime::_2Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        clock_prescaler: 0,
        sample_shifting: false,
        delay_hold_quarter_cycle: false,
        chip_select_boundary: 0,
        delay_block_bypass: false,
        max_transfer: 0,
        refresh: 0,
    };

    let use_dma = true;

    info!("Testing flash in OCTA DTR mode and memory mapped mode");

    let hspi = Hspi::new_octospi(
        p.HSPI1,
        p.PI3,
        p.PH10,
        p.PH11,
        p.PH12,
        p.PH13,
        p.PH14,
        p.PH15,
        p.PI0,
        p.PI1,
        p.PH9,
        p.PI2,
        p.GPDMA1_CH7,
        Irqs,
        flash_config,
    );

    let mut flash = OctaDtrFlashMemory::new(hspi).await;

    let flash_id = flash.read_id();
    info!("FLASH ID: {=[u8]:x}", flash_id);

    let mut rd_buf = [0u8; 16];
    flash.read_memory(0, &mut rd_buf, use_dma).await;
    info!("READ BUF: {=[u8]:#X}", rd_buf);

    flash.erase_sector(0).await;
    flash.read_memory(0, &mut rd_buf, use_dma).await;
    info!("READ BUF: {=[u8]:#X}", rd_buf);
    assert_eq!(rd_buf[0], 0xFF);
    assert_eq!(rd_buf[15], 0xFF);

    let mut wr_buf = [0u8; 16];
    for i in 0..wr_buf.len() {
        wr_buf[i] = i as u8;
    }
    info!("WRITE BUF: {=[u8]:#X}", wr_buf);
    flash.write_memory(0, &wr_buf, use_dma).await;
    flash.read_memory(0, &mut rd_buf, use_dma).await;
    info!("READ BUF: {=[u8]:#X}", rd_buf);
    assert_eq!(rd_buf[0], 0x00);
    assert_eq!(rd_buf[15], 0x0F);

    flash.enable_mm().await;
    info!("Enabled memory mapped mode");

    let first_u32 = unsafe { *(0xA0000000 as *const u32) };
    info!("first_u32: 0x{=u32:X}", first_u32);
    assert_eq!(first_u32, 0x03020100);

    let second_u32 = unsafe { *(0xA0000004 as *const u32) };
    assert_eq!(second_u32, 0x07060504);
    info!("second_u32: 0x{=u32:X}", second_u32);

    let first_u8 = unsafe { *(0xA0000000 as *const u8) };
    assert_eq!(first_u8, 00);
    info!("first_u8: 0x{=u8:X}", first_u8);

    let second_u8 = unsafe { *(0xA0000001 as *const u8) };
    assert_eq!(second_u8, 0x01);
    info!("second_u8: 0x{=u8:X}", second_u8);

    let third_u8 = unsafe { *(0xA0000002 as *const u8) };
    assert_eq!(third_u8, 0x02);
    info!("third_u8: 0x{=u8:X}", third_u8);

    let fourth_u8 = unsafe { *(0xA0000003 as *const u8) };
    assert_eq!(fourth_u8, 0x03);
    info!("fourth_u8: 0x{=u8:X}", fourth_u8);

    info!("DONE");
}

// Custom implementation for MX66UW1G45G NOR flash memory from Macronix.
// Chip commands are hardcoded as they depend on the chip used.
// This implementation enables Octa I/O (OPI) and Double Transfer Rate (DTR)

pub struct OctaDtrFlashMemory<'d, I: Instance> {
    hspi: Hspi<'d, I, Async>,
}

impl<'d, I: Instance> OctaDtrFlashMemory<'d, I> {
    const MEMORY_PAGE_SIZE: usize = 256;

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

    const CMD_READ_SR: u8 = 0x05;
    const CMD_READ_SR_OCTA_DTR: u16 = 0x05FA;

    const CMD_READ_CR2: u8 = 0x71;
    const CMD_WRITE_CR2: u8 = 0x72;

    const CR2_REG1_ADDR: u32 = 0x00000000;
    const CR2_OCTA_DTR: u8 = 0x02;

    const CR2_REG3_ADDR: u32 = 0x00000300;
    const CR2_DC_6_CYCLES: u8 = 0x07;

    pub async fn new(hspi: Hspi<'d, I, Async>) -> Self {
        let mut memory = Self { hspi };

        memory.reset_memory().await;
        memory.enable_octa_dtr().await;
        memory
    }

    async fn enable_octa_dtr(&mut self) {
        self.write_enable_spi().await;
        self.write_cr2_spi(Self::CR2_REG3_ADDR, Self::CR2_DC_6_CYCLES);
        self.write_enable_spi().await;
        self.write_cr2_spi(Self::CR2_REG1_ADDR, Self::CR2_OCTA_DTR);
    }

    pub async fn enable_mm(&mut self) {
        let read_config = TransferConfig {
            iwidth: HspiWidth::OCTO,
            instruction: Some(Self::CMD_READ_OCTA_DTR as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: HspiWidth::OCTO,
            adsize: AddressSize::_32Bit,
            addtr: true,
            dwidth: HspiWidth::OCTO,
            ddtr: true,
            dummy: DummyCycles::_6,
            ..Default::default()
        };

        let write_config = TransferConfig {
            iwidth: HspiWidth::OCTO,
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: HspiWidth::OCTO,
            adsize: AddressSize::_32Bit,
            addtr: true,
            dwidth: HspiWidth::OCTO,
            ddtr: true,
            ..Default::default()
        };
        self.hspi.enable_memory_mapped_mode(read_config, write_config).unwrap();
    }

    async fn exec_command_spi(&mut self, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: HspiWidth::SING,
            instruction: Some(cmd as u32),
            ..Default::default()
        };
        info!("Excuting command: 0x{:X}", transaction.instruction.unwrap());
        self.hspi.blocking_command(&transaction).unwrap();
    }

    async fn exec_command_octa_dtr(&mut self, cmd: u16) {
        let transaction = TransferConfig {
            iwidth: HspiWidth::OCTO,
            instruction: Some(cmd as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            ..Default::default()
        };
        info!("Excuting command: 0x{:X}", transaction.instruction.unwrap());
        self.hspi.blocking_command(&transaction).unwrap();
    }

    fn wait_write_finish_spi(&mut self) {
        while (self.read_sr_spi() & 0x01) != 0 {}
    }

    fn wait_write_finish_octa_dtr(&mut self) {
        while (self.read_sr_octa_dtr() & 0x01) != 0 {}
    }

    pub async fn reset_memory(&mut self) {
        // servono entrambi i comandi?
        self.exec_command_octa_dtr(Self::CMD_RESET_ENABLE_OCTA_DTR).await;
        self.exec_command_octa_dtr(Self::CMD_RESET_OCTA_DTR).await;
        self.exec_command_spi(Self::CMD_RESET_ENABLE).await;
        self.exec_command_spi(Self::CMD_RESET).await;
        self.wait_write_finish_spi();
    }

    async fn write_enable_spi(&mut self) {
        self.exec_command_spi(Self::CMD_WRITE_ENABLE).await;
    }

    async fn write_enable_octa_dtr(&mut self) {
        self.exec_command_octa_dtr(Self::CMD_WRITE_ENABLE_OCTA_DTR).await;
    }

    pub fn read_id(&mut self) -> [u8; 3] {
        let mut buffer = [0; 6];
        let transaction: TransferConfig = TransferConfig {
            iwidth: HspiWidth::OCTO,
            instruction: Some(Self::CMD_READ_ID_OCTA_DTR as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: HspiWidth::OCTO,
            address: Some(0),
            adsize: AddressSize::_32Bit,
            addtr: true,
            dwidth: HspiWidth::OCTO,
            ddtr: true,
            dummy: DummyCycles::_5,
            ..Default::default()
        };
        info!("Reading flash id: 0x{:X}", transaction.instruction.unwrap());
        self.hspi.blocking_read(&mut buffer, transaction).unwrap();
        [buffer[0], buffer[2], buffer[4]]
    }

    pub async fn read_memory(&mut self, addr: u32, buffer: &mut [u8], use_dma: bool) {
        let transaction = TransferConfig {
            iwidth: HspiWidth::OCTO,
            instruction: Some(Self::CMD_READ_OCTA_DTR as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: HspiWidth::OCTO,
            address: Some(addr),
            adsize: AddressSize::_32Bit,
            addtr: true,
            dwidth: HspiWidth::OCTO,
            ddtr: true,
            dummy: DummyCycles::_6,
            ..Default::default()
        };
        if use_dma {
            self.hspi.read(buffer, transaction).await.unwrap();
        } else {
            self.hspi.blocking_read(buffer, transaction).unwrap();
        }
    }

    async fn perform_erase_octa_dtr(&mut self, addr: u32, cmd: u16) {
        let transaction = TransferConfig {
            iwidth: HspiWidth::OCTO,
            instruction: Some(cmd as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: HspiWidth::OCTO,
            address: Some(addr),
            adsize: AddressSize::_32Bit,
            addtr: true,
            ..Default::default()
        };
        self.write_enable_octa_dtr().await;
        self.hspi.blocking_command(&transaction).unwrap();
        self.wait_write_finish_octa_dtr();
    }

    pub async fn erase_sector(&mut self, addr: u32) {
        info!("Erasing 4K sector at address: 0x{:X}", addr);
        self.perform_erase_octa_dtr(addr, Self::CMD_SECTOR_ERASE_OCTA_DTR).await;
    }

    pub async fn erase_block(&mut self, addr: u32) {
        info!("Erasing 64K block at address: 0x{:X}", addr);
        self.perform_erase_octa_dtr(addr, Self::CMD_BLOCK_ERASE_OCTA_DTR).await;
    }

    async fn write_page_octa_dtr(&mut self, addr: u32, buffer: &[u8], len: usize, use_dma: bool) {
        assert!(
            (len as u32 + (addr & 0x000000ff)) <= Self::MEMORY_PAGE_SIZE as u32,
            "write_page(): page write length exceeds page boundary (len = {}, addr = {:X}",
            len,
            addr
        );

        let transaction = TransferConfig {
            iwidth: HspiWidth::OCTO,
            instruction: Some(Self::CMD_PAGE_PROGRAM_OCTA_DTR as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: HspiWidth::OCTO,
            address: Some(addr),
            adsize: AddressSize::_32Bit,
            addtr: true,
            dwidth: HspiWidth::OCTO,
            ddtr: true,
            ..Default::default()
        };
        self.write_enable_octa_dtr().await;
        if use_dma {
            self.hspi.write(buffer, transaction).await.unwrap();
        } else {
            self.hspi.blocking_write(buffer, transaction).unwrap();
        }
        self.wait_write_finish_octa_dtr();
    }

    pub async fn write_memory(&mut self, addr: u32, buffer: &[u8], use_dma: bool) {
        let mut left = buffer.len();
        let mut place = addr;
        let mut chunk_start = 0;

        while left > 0 {
            let max_chunk_size = Self::MEMORY_PAGE_SIZE - (place & 0x000000ff) as usize;
            let chunk_size = if left >= max_chunk_size { max_chunk_size } else { left };
            let chunk = &buffer[chunk_start..(chunk_start + chunk_size)];
            self.write_page_octa_dtr(place, chunk, chunk_size, use_dma).await;
            place += chunk_size as u32;
            left -= chunk_size;
            chunk_start += chunk_size;
        }
    }

    pub fn read_sr_spi(&mut self) -> u8 {
        let mut buffer = [0; 1];
        let transaction: TransferConfig = TransferConfig {
            iwidth: HspiWidth::SING,
            instruction: Some(Self::CMD_READ_SR as u32),
            dwidth: HspiWidth::SING,
            ..Default::default()
        };
        self.hspi.blocking_read(&mut buffer, transaction).unwrap();
        // info!("Read MX66LM1G45G SR register: 0x{:x}", buffer[0]);
        buffer[0]
    }

    pub fn read_sr_octa_dtr(&mut self) -> u8 {
        let mut buffer = [0; 2];
        let transaction: TransferConfig = TransferConfig {
            iwidth: HspiWidth::OCTO,
            instruction: Some(Self::CMD_READ_SR_OCTA_DTR as u32),
            isize: AddressSize::_16Bit,
            idtr: true,
            adwidth: HspiWidth::OCTO,
            address: Some(0),
            adsize: AddressSize::_32Bit,
            addtr: true,
            dwidth: HspiWidth::OCTO,
            ddtr: true,
            dummy: DummyCycles::_5,
            ..Default::default()
        };
        self.hspi.blocking_read(&mut buffer, transaction).unwrap();
        // info!("Read MX66LM1G45G SR register: 0x{:x}", buffer[0]);
        buffer[0]
    }

    pub fn read_cr2_spi(&mut self, addr: u32) -> u8 {
        let mut buffer = [0; 1];
        let transaction: TransferConfig = TransferConfig {
            iwidth: HspiWidth::SING,
            instruction: Some(Self::CMD_READ_CR2 as u32),
            adwidth: HspiWidth::SING,
            address: Some(addr),
            adsize: AddressSize::_32Bit,
            dwidth: HspiWidth::SING,
            ..Default::default()
        };
        self.hspi.blocking_read(&mut buffer, transaction).unwrap();
        // info!("Read MX66LM1G45G CR2[0x{:X}] register: 0x{:x}", addr, buffer[0]);
        buffer[0]
    }

    pub fn write_cr2_spi(&mut self, addr: u32, value: u8) {
        let buffer = [value; 1];
        let transaction: TransferConfig = TransferConfig {
            iwidth: HspiWidth::SING,
            instruction: Some(Self::CMD_WRITE_CR2 as u32),
            adwidth: HspiWidth::SING,
            address: Some(addr),
            adsize: AddressSize::_32Bit,
            dwidth: HspiWidth::SING,
            ..Default::default()
        };
        self.hspi.blocking_write(&buffer, transaction).unwrap();
    }
}
