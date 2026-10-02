//! Expose the Daisy Seed's external QSPI flash as a USB mass-storage device.

#![no_std]
#![no_main]

use aligned::{A4, Aligned};
use block_device_driver::{BlockDevice, blocks_to_slice, blocks_to_slice_mut};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::dma;
use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::*;
use embassy_stm32::qspi::enums::{
    AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, MemorySize, QspiWidth,
};
use embassy_stm32::qspi::{self, Instance, MatchMode, Qspi, QuadDma, TransferConfig};
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, Peri, bind_interrupts, usb};
use embassy_time::{Duration, WithTimeout};
use embassy_usb::Builder;
use embassy_usb::class::msc::subclass::scsi::Scsi;
use embassy_usb::class::msc::subclass::scsi::block_device::BlockDeviceError;
use embassy_usb::class::msc::transport::bulk_only::BulkOnlyTransport;
use panic_probe as _;

const BLOCK_SIZE: usize = 512;
const SECTOR_SIZE: usize = 4096;
const FLASH_SIZE: usize = 8 * 1024 * 1024;
const PAGE_SIZE: usize = 256;

const WRITE_ENABLE: u8 = 0x06;
const PAGE_PROGRAM_QUAD: u8 = 0x32;
const SECTOR_ERASE: u8 = 0xD7;
const FAST_READ_QUAD_IO: u8 = 0xEB;
const READ_STATUS: u8 = 0x05;
const WRITE_STATUS: u8 = 0x01;
const SET_READ_PARAMETERS: u8 = 0xC0;
const RESET_ENABLE: u8 = 0x66;
const RESET_MEMORY: u8 = 0x99;
const STATUS_WIP: u8 = 1 << 0;
const STATUS_QE: u8 = 1 << 6;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<USB_OTG_FS>;
    QUADSPI => qspi::InterruptHandler<QUADSPI>;
    MDMA => dma::InterruptHandler<MDMA_CH0>;
});

struct QspiFlash<'d> {
    qspi: Qspi<'d, QUADSPI, Async>,
}

impl<'d> QspiFlash<'d> {
    fn new<D>(
        qspi: Peri<'d, QUADSPI>,
        dma: Peri<'d, D>,
        irq: impl Binding<D::Interrupt, dma::InterruptHandler<D>>
        + Binding<<QUADSPI as Instance>::Interrupt, qspi::InterruptHandler<QUADSPI>>
        + 'd,
        io0: Peri<'d, PF8>,
        io1: Peri<'d, PF9>,
        io2: Peri<'d, PF7>,
        io3: Peri<'d, PF6>,
        sck: Peri<'d, PF10>,
        cs: Peri<'d, PG6>,
    ) -> Self
    where
        D: QuadDma<QUADSPI>,
    {
        let mut config = qspi::Config::default();
        config.memory_size = MemorySize::_8MiB;
        config.prescaler = 1;
        config.cs_high_time = ChipSelectHighTime::_2Cycle;
        config.fifo_threshold = FIFOThresholdLevel::_1Bytes;

        let qspi = Qspi::new_bank1(qspi, io0, io1, io2, io3, sck, cs, dma, irq, config);
        let mut this = Self { qspi };
        this.command(RESET_ENABLE);
        this.command(RESET_MEMORY);
        this.write_register(WRITE_STATUS, STATUS_QE);
        this.write_register(SET_READ_PARAMETERS, 0b1111_0000);
        this
    }

    fn command(&mut self, instruction: u8) {
        self.qspi.blocking_command(TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction,
            address: None,
            address_size: AddressSize::_24Bit,
            dummy: DummyCycles::_0,
        });
    }

    fn write_register(&mut self, instruction: u8, value: u8) {
        self.command(WRITE_ENABLE);
        self.qspi.blocking_write(
            &[value],
            TransferConfig {
                iwidth: QspiWidth::SING,
                awidth: QspiWidth::NONE,
                dwidth: QspiWidth::SING,
                instruction,
                address: None,
                address_size: AddressSize::_24Bit,
                dummy: DummyCycles::_0,
            },
        );
        self.wait_blocking();
    }

    fn wait_blocking(&mut self) {
        loop {
            let mut status = [0];
            self.qspi.blocking_read(
                &mut status,
                TransferConfig {
                    iwidth: QspiWidth::SING,
                    awidth: QspiWidth::NONE,
                    dwidth: QspiWidth::SING,
                    instruction: READ_STATUS,
                    address: None,
                    address_size: AddressSize::_24Bit,
                    dummy: DummyCycles::_0,
                },
            );
            if status[0] & STATUS_WIP == 0 {
                break;
            }
        }
    }

    async fn wait(&mut self, timeout: Duration) -> Result<(), ()> {
        self.qspi
            .auto_poll(
                TransferConfig {
                    iwidth: QspiWidth::SING,
                    awidth: QspiWidth::NONE,
                    dwidth: QspiWidth::SING,
                    instruction: READ_STATUS,
                    address: None,
                    address_size: AddressSize::_24Bit,
                    dummy: DummyCycles::_0,
                },
                0x10,
                STATUS_WIP as u32,
                0,
                1,
                MatchMode::AND,
            )
            .with_timeout(timeout)
            .await
            .map_err(|_| ())?;
        Ok(())
    }

    async fn read(&mut self, address: u32, data: &mut [u8]) {
        self.qspi
            .read_dma(
                data,
                TransferConfig {
                    iwidth: QspiWidth::SING,
                    awidth: QspiWidth::QUAD,
                    dwidth: QspiWidth::QUAD,
                    instruction: FAST_READ_QUAD_IO,
                    address: Some(address),
                    address_size: AddressSize::_24Bit,
                    dummy: DummyCycles::_8,
                },
            )
            .await;
    }

    async fn erase_sector(&mut self, address: u32) -> Result<(), ()> {
        self.command(WRITE_ENABLE);
        self.qspi.blocking_command(TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::NONE,
            instruction: SECTOR_ERASE,
            address: Some(address),
            address_size: AddressSize::_24Bit,
            dummy: DummyCycles::_0,
        });
        self.wait(Duration::from_millis(600)).await
    }

    async fn program(&mut self, mut address: u32, mut data: &[u8]) -> Result<(), ()> {
        while !data.is_empty() {
            let len = data.len().min(PAGE_SIZE);
            self.command(WRITE_ENABLE);
            self.qspi
                .write_dma(
                    &data[..len],
                    TransferConfig {
                        iwidth: QspiWidth::SING,
                        awidth: QspiWidth::SING,
                        dwidth: QspiWidth::QUAD,
                        instruction: PAGE_PROGRAM_QUAD,
                        address: Some(address),
                        address_size: AddressSize::_24Bit,
                        dummy: DummyCycles::_0,
                    },
                )
                .await;
            self.wait(Duration::from_micros(1600)).await?;
            address += len as u32;
            data = &data[len..];
        }
        Ok(())
    }
}

struct FlashBlockDevice<'d> {
    flash: QspiFlash<'d>,
    sector: &'d mut [u8; SECTOR_SIZE],
}

impl BlockDevice<BLOCK_SIZE> for FlashBlockDevice<'_> {
    type Error = BlockDeviceError;
    type Align = A4;

    async fn read(&mut self, lba: u32, blocks: &mut [Aligned<A4, [u8; BLOCK_SIZE]>]) -> Result<(), Self::Error> {
        let address = lba
            .checked_mul(BLOCK_SIZE as u32)
            .ok_or(BlockDeviceError::LbaOutOfRange)?;
        self.flash.read(address, blocks_to_slice_mut(blocks)).await;
        Ok(())
    }

    async fn write(&mut self, lba: u32, blocks: &[Aligned<A4, [u8; BLOCK_SIZE]>]) -> Result<(), Self::Error> {
        let mut address = lba
            .checked_mul(BLOCK_SIZE as u32)
            .ok_or(BlockDeviceError::LbaOutOfRange)? as usize;
        let mut data = blocks_to_slice(blocks);

        while !data.is_empty() {
            let sector_address = address / SECTOR_SIZE * SECTOR_SIZE;
            let sector_offset = address - sector_address;
            let len = data.len().min(SECTOR_SIZE - sector_offset);

            self.flash.read(sector_address as u32, self.sector).await;
            self.sector[sector_offset..sector_offset + len].copy_from_slice(&data[..len]);
            self.flash
                .erase_sector(sector_address as u32)
                .await
                .map_err(|_| BlockDeviceError::WriteError)?;
            self.flash
                .program(sector_address as u32, self.sector)
                .await
                .map_err(|_| BlockDeviceError::WriteError)?;

            address += len;
            data = &data[len..];
        }
        Ok(())
    }

    async fn size(&mut self) -> Result<u64, Self::Error> {
        Ok(FLASH_SIZE as u64)
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul50,
            #[cfg(feature = "stm32h743bi")]
            fracn: None,
            divp: Some(PllDiv::Div2),
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div2;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.apb3_pre = APBPrescaler::Div2;
        config.rcc.apb4_pre = APBPrescaler::Div2;
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.usbsel = mux::Usbsel::Hsi48;
    }
    let p = embassy_stm32::init(config);

    let mut ep_out_buffer = [0; 256];
    let mut usb_config = embassy_stm32::usb::Config::default();
    usb_config.vbus_detection = false;
    let driver = Driver::new_fs(p.USB_OTG_FS, p.PA12, p.PA11, Irqs, &mut ep_out_buffer, usb_config);

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("Daisy QSPI MSC");
    config.serial_number = Some("12345678");

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut state = Default::default();
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    let flash = QspiFlash::new(p.QUADSPI, p.MDMA_CH0, Irqs, p.PF8, p.PF9, p.PF7, p.PF6, p.PF10, p.PG6);
    let mut sector = [0; SECTOR_SIZE];
    let mut scsi_buffer = [Aligned::<A4, _>([0; BLOCK_SIZE]); SECTOR_SIZE / BLOCK_SIZE];
    let scsi = Scsi::new(
        FlashBlockDevice {
            flash,
            sector: &mut sector,
        },
        &mut scsi_buffer,
        "Embassy",
        "Daisy QSPI",
    );
    let mut msc = BulkOnlyTransport::new(&mut builder, &mut state, 64, scsi);
    let mut usb = builder.build();

    info!("USB MSC QSPI flash disk running");
    join(usb.run(), msc.run()).await;
}
