//! Example driver for IS25LP064 Flash chip connected via QSPI. Tested on Daisy Seed.
#![no_std]
#![no_main]
#![allow(unused)]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::mode::{Async, Blocking, Mode};
use embassy_stm32::peripherals::*;
use embassy_stm32::qspi::enums::{
    AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, MemorySize, QspiWidth,
};
use embassy_stm32::qspi::{self, Instance, InterruptHandler, MatchMode, Qspi, QuadDma, TransferConfig};
use embassy_stm32::{Peri, bind_interrupts, dma};
use embassy_time::{Duration, WithTimeout};
use {defmt_rtt as _, panic_probe as _};

// Commands from IS25LP064 datasheet.
const WRITE_CMD: u8 = 0x32; // PPQ
const WRITE_ENABLE_CMD: u8 = 0x06; // WREN
const SECTOR_ERASE_CMD: u8 = 0xD7; // SER
const FAST_READ_QUAD_IO_CMD: u8 = 0xEB; // FRQIO
const RESET_ENABLE_CMD: u8 = 0x66;
const RESET_MEMORY_CMD: u8 = 0x99;

const WRITE_STATUS_REGISTER_CMD: u8 = 0x01; // WRSR
const READ_STATUS_REGISTER_CMD: u8 = 0x05; // RDSR
const STATUS_BIT_WIP: u8 = 1 << 0;
const STATUS_BIT_WEL: u8 = 1 << 1;
const STATUS_BIT_BP0: u8 = 1 << 2;
const STATUS_BIT_BP1: u8 = 1 << 3;
const STATUS_BIT_BP2: u8 = 1 << 4;
const STATUS_BIT_BP3: u8 = 1 << 5;
const STATUS_BIT_QE: u8 = 1 << 6;
const STATUS_BIT_SRWD: u8 = 1 << 7;

const SET_READ_PARAMETERS_CMD: u8 = 0xC0; // SRP
const READ_PARAMS_BIT_BL0: u8 = 1 << 0;
const READ_PARAMS_BIT_BL1: u8 = 1 << 1;
const READ_PARAMS_BIT_WE: u8 = 1 << 2;
const READ_PARAMS_BIT_DC0: u8 = 1 << 3;
const READ_PARAMS_BIT_DC1: u8 = 1 << 4;
const READ_PARAMS_BIT_ODS0: u8 = 1 << 5;
const READ_PARAMS_BIT_ODS1: u8 = 1 << 6;
const READ_PARAMS_BIT_ODS2: u8 = 1 << 7;

// Memory array specifications as defined in the datasheet.
const SECTOR_SIZE: u32 = 4096;
const PAGE_SIZE: u32 = 256;
const MAX_ADDRESS: u32 = 0x7FFFFF;

// Max Sector Erase time is 300ms
const SECTOR_ERASE_TIMEOUT: Duration = Duration::from_millis(600);

// Max Page Write time is 0.8ms
const PAGE_WRITE_TIMEOUT: Duration = Duration::from_micros(1600);

#[allow(non_snake_case)]
pub struct FlashPins<'a> {
    pub IO0: Peri<'a, PF8>, // (SI)
    pub IO1: Peri<'a, PF9>, // (SO)
    pub IO2: Peri<'a, PF7>,
    pub IO3: Peri<'a, PF6>,
    pub SCK: Peri<'a, PF10>,
    pub CS: Peri<'a, PG6>,
}

pub struct Flash<'a, MODE: Mode> {
    qspi: Qspi<'a, QUADSPI, MODE>,
}

impl<MODE: Mode> Flash<'_, MODE> {
    fn config() -> qspi::Config {
        let mut config = qspi::Config::default();

        config.memory_size = MemorySize::_8MiB;
        config.address_size = AddressSize::_24bit;
        config.prescaler = 1;
        config.cs_high_time = ChipSelectHighTime::_2Cycle;
        config.fifo_threshold = FIFOThresholdLevel::_1Bytes;
        config
    }

    pub fn read(&mut self, address: u32, buffer: &mut [u8]) {
        assert!(address + buffer.len() as u32 <= MAX_ADDRESS);

        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::QUAD,
            dwidth: QspiWidth::QUAD,
            instruction: FAST_READ_QUAD_IO_CMD,
            address: Some(address),
            dummy: DummyCycles::_8,
        };
        self.qspi.blocking_read(buffer, transaction);
    }

    pub fn read_uuid(&mut self) -> [u8; 16] {
        let mut buffer = [0; 16];
        let transaction: TransferConfig = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::SING,
            instruction: 0x4B,
            address: Some(0x00),
            dummy: DummyCycles::_8,
        };
        self.qspi.blocking_read(&mut buffer, transaction);
        buffer
    }

    pub fn write(&mut self, mut address: u32, data: &[u8]) {
        assert!(address <= MAX_ADDRESS);
        assert!(!data.is_empty());
        self.erase(address, data.len() as u32);

        let mut length = data.len() as u32;
        let mut start_cursor = 0;

        //WRITE_CMD(or PPQ) allows to write up to 256 bytes, which is as much as PAGE_SIZE.
        //Let's divide the data into chunks of page size to write to flash
        loop {
            // Calculate number of bytes between address and end of the page.
            let page_remainder = PAGE_SIZE - (address & (PAGE_SIZE - 1));
            let size = page_remainder.min(length) as usize;
            self.enable_write();
            let transaction = TransferConfig {
                iwidth: QspiWidth::SING,
                awidth: QspiWidth::SING,
                dwidth: QspiWidth::QUAD,
                instruction: WRITE_CMD,
                address: Some(address),
                dummy: DummyCycles::_0,
            };

            self.qspi
                .blocking_write(&data[start_cursor..start_cursor + size], transaction);
            self.wait_for_write();
            start_cursor += size;

            // Stop if this was the last needed page.
            if length <= page_remainder {
                break;
            }
            length -= page_remainder;

            // Jump to the next page.
            address += page_remainder;
            address %= MAX_ADDRESS;
        }
    }

    pub fn erase(&mut self, mut address: u32, mut length: u32) {
        assert!(address <= MAX_ADDRESS);
        assert!(length > 0);

        loop {
            // Erase the sector.
            self.enable_write();
            let transaction = TransferConfig {
                iwidth: QspiWidth::SING,
                awidth: QspiWidth::SING,
                dwidth: QspiWidth::NONE,
                instruction: SECTOR_ERASE_CMD,
                address: Some(address),
                dummy: DummyCycles::_0,
            };

            self.qspi.blocking_command(transaction);
            self.wait_for_write();

            // Calculate number of bytes between address and end of the sector.
            let sector_remainder = SECTOR_SIZE - (address & (SECTOR_SIZE - 1));

            // Stop if this was the last affected sector.
            if length <= sector_remainder {
                break;
            }
            length -= sector_remainder;

            // Jump to the next sector.
            address += sector_remainder;
            address %= MAX_ADDRESS;
        }
    }

    fn enable_write(&mut self) {
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: WRITE_ENABLE_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_command(transaction);
    }

    fn wait_for_write(&mut self) {
        loop {
            if self.read_status() & STATUS_BIT_WIP == 0 {
                break;
            }
        }
    }

    fn read_status(&mut self) -> u8 {
        let mut status: [u8; 1] = [0xFF; 1];
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: READ_STATUS_REGISTER_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_read(&mut status, transaction);
        status[0]
    }

    fn reset_memory(&mut self) {
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: RESET_ENABLE_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_command(transaction);

        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: RESET_MEMORY_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_command(transaction);
    }

    /// Reset status registers into driver's defaults. This makes sure that the
    /// peripheral is configured as expected.
    fn reset_status_register(&mut self) {
        self.enable_write();
        let value = STATUS_BIT_QE;
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: WRITE_STATUS_REGISTER_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_write(&[value], transaction);
        self.wait_for_write();
    }

    /// Reset read registers into driver's defaults. This makes sure that the
    /// peripheral is configured as expected.
    fn reset_read_register(&mut self) {
        let value = READ_PARAMS_BIT_ODS2 | READ_PARAMS_BIT_ODS1 | READ_PARAMS_BIT_ODS0 | READ_PARAMS_BIT_DC1;
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: SET_READ_PARAMETERS_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_write(&[value], transaction);
        self.wait_for_write();
    }

    fn reset(&mut self) {
        self.reset_memory();
        self.reset_status_register();
        self.reset_read_register();
    }
}

impl<'a> Flash<'a, Blocking> {
    pub fn new_blocking(peri: Peri<'a, QUADSPI>, pins: FlashPins<'a>) -> Self {
        let qspi = Qspi::new_blocking_bank1(
            peri,
            pins.IO0,
            pins.IO1,
            pins.IO2,
            pins.IO3,
            pins.SCK,
            pins.CS,
            Self::config(),
        );
        let mut flash = Flash { qspi };
        flash.reset();
        flash
    }
}

impl<'a> Flash<'a, Async> {
    pub fn new_async<D, I>(peri: Peri<'a, QUADSPI>, pins: FlashPins<'a>, ch: Peri<'a, D>, irq: I) -> Flash<'a, Async>
    where
        D: QuadDma<QUADSPI>,
        I: Binding<D::Interrupt, dma::InterruptHandler<D>>
            + Binding<<QUADSPI as Instance>::Interrupt, InterruptHandler<QUADSPI>>
            + 'a,
    {
        let qspi = Qspi::new_bank1(
            peri,
            pins.IO0,
            pins.IO1,
            pins.IO2,
            pins.IO3,
            pins.SCK,
            pins.CS,
            ch,
            irq,
            Self::config(),
        );
        let mut flash = Flash { qspi };
        flash.reset();
        flash
    }

    pub async fn read_async(&mut self, address: u32, buffer: &mut [u8]) {
        assert!(address + buffer.len() as u32 <= MAX_ADDRESS);

        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::QUAD,
            dwidth: QspiWidth::QUAD,
            instruction: FAST_READ_QUAD_IO_CMD,
            address: Some(address),
            dummy: DummyCycles::_8,
        };
        self.qspi.read_dma(buffer, transaction).await;
    }

    pub async fn write_async(&mut self, mut address: u32, data: &[u8]) {
        assert!(address <= MAX_ADDRESS);
        assert!(!data.is_empty());
        self.erase_async(address, data.len() as u32).await;
        let mut length = data.len() as u32;
        let mut start_cursor = 0;

        //WRITE_CMD(or PPQ) allows to write up to 256 bytes, which is as much as PAGE_SIZE.
        //Let's divide the data into chunks of page size to write to flash
        loop {
            // Calculate number of bytes between address and end of the page.
            let page_remainder = PAGE_SIZE - (address & (PAGE_SIZE - 1));
            let size = page_remainder.min(length) as usize;
            self.enable_write();
            let transaction = TransferConfig {
                iwidth: QspiWidth::SING,
                awidth: QspiWidth::SING,
                dwidth: QspiWidth::QUAD,
                instruction: WRITE_CMD,
                address: Some(address),
                dummy: DummyCycles::_0,
            };

            self.qspi
                .write_dma(&data[start_cursor..start_cursor + size], transaction)
                .await;
            self.wait_for_write_async(PAGE_WRITE_TIMEOUT).await;
            start_cursor += size;

            // Stop if this was the last needed page.
            if length <= page_remainder {
                break;
            }
            length -= page_remainder;

            // Jump to the next page.
            address += page_remainder;
            address %= MAX_ADDRESS;
        }
    }

    pub async fn erase_async(&mut self, mut address: u32, mut length: u32) {
        assert!(address <= MAX_ADDRESS);
        assert!(length > 0);

        loop {
            // Erase the sector.
            self.enable_write();
            let transaction = TransferConfig {
                iwidth: QspiWidth::SING,
                awidth: QspiWidth::SING,
                dwidth: QspiWidth::NONE,
                instruction: SECTOR_ERASE_CMD,
                address: Some(address),
                dummy: DummyCycles::_0,
            };
            self.qspi.blocking_command(transaction);

            self.wait_for_write_async(SECTOR_ERASE_TIMEOUT).await;

            // Calculate number of bytes between address and end of the sector.
            let sector_remainder = SECTOR_SIZE - (address & (SECTOR_SIZE - 1));

            // Stop if this was the last affected sector.
            if length <= sector_remainder {
                break;
            }
            length -= sector_remainder;

            // Jump to the next sector.
            address += sector_remainder;
            address %= MAX_ADDRESS;
        }
    }

    async fn wait_for_write_async(&mut self, timeout: Duration) {
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: READ_STATUS_REGISTER_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };

        self.qspi
            .auto_poll(transaction, 0x10, STATUS_BIT_WIP as u32, 0, 1, MatchMode::AND)
            .with_timeout(timeout)
            .await
            .expect("Flash Timed out waiting for status match")
    }
}

bind_interrupts!(pub struct Irqs {
    QUADSPI => qspi::InterruptHandler<embassy_stm32::peripherals::QUADSPI>;
    MDMA => dma::InterruptHandler<embassy_stm32::peripherals::MDMA_CH0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    const ADDRESS: u32 = 0x00;
    const SIZE: usize = 8000;

    let pins = FlashPins {
        IO0: p.PF8,
        IO1: p.PF9,
        IO2: p.PF7,
        IO3: p.PF6,
        SCK: p.PF10,
        CS: p.PG6,
    };

    let mut flash = Flash::new_async(p.QUADSPI, pins, p.MDMA_CH0, Irqs);

    info!("uuid: {}", flash.read_uuid());
    // Create an array of data to write.
    let mut data: [u8; SIZE] = [0; SIZE];
    for (i, x) in data.iter_mut().enumerate() {
        *x = (i % 256) as u8;
    }
    info!("Write buffer: {:?}", data[0..32]);

    // Write it to the flash memory.
    info!("Writting to flash");
    flash.write_async(ADDRESS, &data).await;

    // Read it back.
    info!("Reading from flash");
    let mut buffer: [u8; SIZE] = [0; SIZE];
    flash.read_async(ADDRESS, &mut buffer).await;
    info!("Read buffer: {:?}", buffer[0..32]);

    if data == buffer {
        info!("Everything went as expected");
    } else {
        error!("Read value does not match what was written");
    }

    loop {}
}
