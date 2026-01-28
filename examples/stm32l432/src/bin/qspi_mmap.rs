#![no_std]
#![no_main]
#![allow(dead_code)]
/// This example demonstrates how to use the QSPI peripheral in both indirect-mode and memory-mapped mode.
/// If you want to test this example, please pay attention to flash pins and check flash device datasheet
/// to make sure operations in this example are compatible with your device, especially registers I/O operations.
use defmt::info;
use embassy_stm32::qspi::enums::{
    AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, MemorySize, QspiWidth, SampleShifting,
};
use embassy_stm32::qspi::{self, Instance, TransferConfig};
use embassy_stm32::{bind_interrupts, dma, mode, peripherals};
pub struct FlashMemory<I: Instance> {
    qspi: qspi::Qspi<'static, I, mode::Async>,
}
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const MEMORY_PAGE_SIZE: usize = 256;
const CMD_READ_SR: u8 = 0x05;
const CMD_READ_CR: u8 = 0x35;
const CMD_QUAD_READ: u8 = 0x6B;
const CMD_QUAD_WRITE_PG: u8 = 0x32;
const CMD_READ_ID: u8 = 0x9F;
const CMD_READ_MID: u8 = 0x90;
const CMD_READ_UUID: u8 = 0x4B;
const CMD_ENABLE_RESET: u8 = 0x66;
const CMD_RESET: u8 = 0x99;
const CMD_WRITE_ENABLE: u8 = 0x06;
const CMD_SECTOR_ERASE: u8 = 0x20;

const CMD_WRITE_SR: u8 = 0x01;

impl<I: Instance> FlashMemory<I> {
    pub fn new(qspi: qspi::Qspi<'static, I, mode::Async>) -> Self {
        let mut memory = Self { qspi };

        memory.reset_memory();
        memory.enable_quad();

        memory
    }
    fn enable_quad(&mut self) {
        let sr = self.read_sr_lsb();
        let cr = self.read_sr_msb();

        self.write_sr(sr, cr | 0x02);
    }
    fn read_register(&mut self, cmd: u8) -> u8 {
        let mut buffer = [0; 1];
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: cmd,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_read(&mut buffer, transaction);
        buffer[0]
    }

    fn write_register(&mut self, cmd: u8, value: u8) {
        let buffer = [value; 1];
        let transaction: TransferConfig = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: cmd,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_write(&buffer, transaction);
    }
    pub fn write_sr(&mut self, lsb: u8, msb: u8) {
        let buffer = [lsb, msb];
        let transaction: TransferConfig = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: CMD_WRITE_SR,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_write(&buffer, transaction);
    }

    pub fn read_sr_lsb(&mut self) -> u8 {
        self.read_register(CMD_READ_SR)
    }
    pub fn read_sr_msb(&mut self) -> u8 {
        self.read_register(CMD_READ_CR)
    }

    pub fn reset_memory(&mut self) {
        self.exec_command(CMD_ENABLE_RESET);
        self.exec_command(CMD_RESET);
        self.wait_write_finish();
    }
    fn exec_command(&mut self, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: cmd,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_command(transaction);
    }
    fn wait_write_finish(&mut self) {
        while (self.read_sr_lsb() & 0x01) != 0 {}
    }

    pub fn read_mid(&mut self) -> [u8; 2] {
        let mut buffer = [0; 2];
        let transaction: TransferConfig = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::SING,
            instruction: CMD_READ_MID,
            address: Some(0),
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_read(&mut buffer, transaction);
        buffer
    }
    pub fn read_uuid(&mut self) -> [u8; 16] {
        let mut buffer = [0; 16];
        let transaction: TransferConfig = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::SING,
            instruction: CMD_READ_UUID,
            address: Some(0),
            dummy: DummyCycles::_8,
        };
        self.qspi.blocking_read(&mut buffer, transaction);
        buffer
    }
    pub fn read_id(&mut self) -> [u8; 3] {
        let mut buffer = [0; 3];
        let transaction: TransferConfig = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: CMD_READ_ID,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_read(&mut buffer, transaction);
        buffer
    }

    pub fn enable_mmap(&mut self) {
        let transaction: TransferConfig = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::QUAD,
            instruction: CMD_QUAD_READ,
            address: Some(0),
            dummy: DummyCycles::_8,
        };
        self.qspi.enable_memory_map(&transaction);
    }
    fn perform_erase(&mut self, addr: u32, cmd: u8) {
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::NONE,
            instruction: cmd,
            address: Some(addr),
            dummy: DummyCycles::_0,
        };
        self.enable_write();
        self.qspi.blocking_command(transaction);
        self.wait_write_finish();
    }
    pub fn enable_write(&mut self) {
        self.exec_command(CMD_WRITE_ENABLE);
    }
    pub fn erase_sector(&mut self, addr: u32) {
        self.perform_erase(addr, CMD_SECTOR_ERASE);
    }
    fn write_page(&mut self, addr: u32, buffer: &[u8], len: usize, use_dma: bool) {
        assert!(
            (len as u32 + (addr & 0x000000ff)) <= MEMORY_PAGE_SIZE as u32,
            "write_page(): page write length exceeds page boundary (len = {}, addr = {:X}",
            len,
            addr
        );

        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::QUAD,
            instruction: CMD_QUAD_WRITE_PG,
            address: Some(addr),
            dummy: DummyCycles::_0,
        };
        self.enable_write();
        if use_dma {
            self.qspi.blocking_write_dma(buffer, transaction);
        } else {
            self.qspi.blocking_write(buffer, transaction);
        }
        self.wait_write_finish();
    }
    pub fn write_memory(&mut self, addr: u32, buffer: &[u8], use_dma: bool) {
        let mut left = buffer.len();
        let mut place = addr;
        let mut chunk_start = 0;

        while left > 0 {
            let max_chunk_size = MEMORY_PAGE_SIZE - (place & 0x000000ff) as usize;
            let chunk_size = if left >= max_chunk_size { max_chunk_size } else { left };
            let chunk = &buffer[chunk_start..(chunk_start + chunk_size)];
            self.write_page(place, chunk, chunk_size, use_dma);
            place += chunk_size as u32;
            left -= chunk_size;
            chunk_start += chunk_size;
        }
    }

    pub fn read_memory(&mut self, addr: u32, buffer: &mut [u8], use_dma: bool) {
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::QUAD,
            instruction: CMD_QUAD_READ,
            address: Some(addr),
            dummy: DummyCycles::_8,
        };
        if use_dma {
            self.qspi.blocking_read_dma(buffer, transaction);
        } else {
            self.qspi.blocking_read(buffer, transaction);
        }
    }
}

const MEMORY_ADDR: u32 = 0x00000000 as u32;

bind_interrupts!(struct Irqs {
    DMA2_CHANNEL7 => dma::InterruptHandler<peripherals::DMA2_CH7>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut config = qspi::Config::default();
    config.memory_size = MemorySize::_16MiB;
    config.address_size = AddressSize::_24bit;
    config.prescaler = 200;
    config.cs_high_time = ChipSelectHighTime::_1Cycle;
    config.fifo_threshold = FIFOThresholdLevel::_16Bytes;
    config.sample_shifting = SampleShifting::None;

    let driver = qspi::Qspi::new_bank1(
        p.QUADSPI, p.PB1, p.PB0, p.PA7, p.PA6, p.PA3, p.PA2, p.DMA2_CH7, Irqs, config,
    );
    let mut flash = FlashMemory::new(driver);
    let mut wr_buf = [0u8; 256];
    for i in 0..32 {
        wr_buf[i] = i as u8;
    }
    let mut rd_buf = [0u8; 32];
    flash.erase_sector(MEMORY_ADDR);
    flash.write_memory(MEMORY_ADDR, &wr_buf, false);
    flash.read_memory(MEMORY_ADDR, &mut rd_buf, false);

    info!("data read from indirect mode: {}", rd_buf);
    flash.enable_mmap();
    let qspi_base = unsafe { core::slice::from_raw_parts(0x9000_0000 as *const u8, 32) };
    info!("data read from mmap: {}", qspi_base);
    loop {
        Timer::after_millis(1000).await;
    }
}
