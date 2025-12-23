use core::default::Default;
use core::ops::{Deref, DerefMut};

use sdio_host::emmc::{EMMC, ExtCSD};
use sdio_host::sd::{BusWidth, CIC, CID, CSD, CardCapacity, CardStatus, CurrentState, OCR, RCA, SCR, SD, SDStatus};
use sdio_host::{common_cmd, emmc_cmd, sd_cmd};

use crate::sdmmc::{
    BlockSize, DatapathMode, Error, Sdmmc, Signalling, aligned_mut, aligned_ref, block_size, bus_width_vals,
    slice8_mut, slice8_ref,
};
use crate::time::{Hertz, mhz};

/// Aligned data block for SDMMC transfers.
///
/// This is a 512-byte array, aligned to 4 bytes to satisfy DMA requirements.
#[repr(align(4))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DataBlock(pub [u32; 128]);

impl DataBlock {
    /// Create a new DataBlock
    pub const fn new() -> Self {
        DataBlock([0u32; 128])
    }
}

impl Deref for DataBlock {
    type Target = [u8; 512];

    fn deref(&self) -> &Self::Target {
        unwrap!(slice8_ref(&self.0[..]).try_into())
    }
}

impl DerefMut for DataBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unwrap!(slice8_mut(&mut self.0[..]).try_into())
    }
}

/// Command Block buffer for SDMMC command transfers.
///
/// This is a 16-word array, exposed so that DMA commpatible memory can be used if required.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CmdBlock(pub [u32; 16]);

impl CmdBlock {
    /// Creates a new instance of CmdBlock
    pub const fn new() -> Self {
        Self([0u32; 16])
    }
}

impl Deref for CmdBlock {
    type Target = [u32; 16];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CmdBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Represents either an SD or EMMC card
pub trait Addressable: Sized + Clone {
    /// Associated type
    type Ext;

    /// Get this peripheral's address on the SDMMC bus
    fn get_address(&self) -> u16;

    /// Is this a standard or high capacity peripheral?
    fn get_capacity(&self) -> CardCapacity;

    /// Size in bytes
    fn size(&self) -> u64;
}

/// Storage Device
pub struct StorageDevice<'a, 'b, T: Addressable> {
    info: T,
    /// Inner member
    pub sdmmc: &'a mut Sdmmc<'b>,
}

/// Card Storage Device
impl<'a, 'b> StorageDevice<'a, 'b, Card> {
    /// Create a new SD card
    pub async fn new_sd_card(sdmmc: &'a mut Sdmmc<'b>, cmd_block: &mut CmdBlock, freq: Hertz) -> Result<Self, Error> {
        let mut s = Self {
            info: Card::default(),
            sdmmc,
        };

        s.acquire(cmd_block, freq).await?;

        Ok(s)
    }

    /// Initializes the card into a known state (or at least tries to).
    async fn acquire(&mut self, cmd_block: &mut CmdBlock, freq: Hertz) -> Result<(), Error> {
        let _scoped_block_stop = self.sdmmc.info.rcc.block_stop();
        let regs = self.sdmmc.info.regs;

        let _bus_width = match self.sdmmc.bus_width() {
            BusWidth::Eight => return Err(Error::BusWidth),
            bus_width => bus_width,
        };

        // While the SD/SDIO card or eMMC is in identification mode,
        // the SDMMC_CK frequency must be no more than 400 kHz.
        self.sdmmc.init_idle()?;

        // Check if cards supports CMD8 (with pattern)
        self.sdmmc.cmd(sd_cmd::send_if_cond(1, 0xAA), true, false)?;
        let cic = CIC::from(regs.respr(0).read().cardstatus());

        if cic.pattern() != 0xAA {
            return Err(Error::UnsupportedCardVersion);
        }

        if cic.voltage_accepted() & 1 == 0 {
            return Err(Error::UnsupportedVoltage);
        }

        let ocr = loop {
            // Signal that next command is a app command
            self.sdmmc.cmd(common_cmd::app_cmd(0), true, false)?; // CMD55

            // 3.2-3.3V
            let voltage_window = 1 << 5;
            // Initialize card

            let ocr: OCR<SD> = self
                .sdmmc
                .cmd(sd_cmd::sd_send_op_cond(true, false, true, voltage_window), false, false)?
                .into();

            if !ocr.is_busy() {
                // Power up done
                break ocr;
            }
        };

        if ocr.high_capacity() {
            // Card is SDHC or SDXC or SDUC
            self.info.card_type = CardCapacity::HighCapacity;
        } else {
            self.info.card_type = CardCapacity::StandardCapacity;
        }
        self.info.ocr = ocr;

        self.info.cid = self.sdmmc.get_cid()?.into();
        let rca: RCA<SD> = self.sdmmc.cmd(sd_cmd::send_relative_address(), true, false)?.into();
        self.info.rca = rca.address();
        self.info.csd = self.sdmmc.get_csd(self.info.get_address())?.into();
        self.sdmmc.select_card(Some(self.info.get_address()))?;
        self.info.scr = self.get_scr(cmd_block).await?;

        let (bus_width, acmd_arg) = if !self.info.scr.bus_width_four() {
            (BusWidth::One, 0)
        } else {
            (BusWidth::Four, 2)
        };

        self.sdmmc.cmd(common_cmd::app_cmd(self.info.rca), true, false)?;
        self.sdmmc.cmd(sd_cmd::cmd6(acmd_arg), true, false)?;

        self.sdmmc.clkcr_set_clkdiv(freq.clamp(mhz(0), mhz(25)), bus_width)?;

        // Read status
        self.info.status = self.read_sd_status(cmd_block).await?;

        if freq > mhz(25) {
            // Switch to SDR25
            let signalling = self.switch_signalling_mode(cmd_block, Signalling::SDR25).await?;

            if signalling == Signalling::SDR25 {
                // Set final clock frequency
                self.sdmmc.clkcr_set_clkdiv(freq, bus_width)?;

                let status: CardStatus<SD> = self.sdmmc.read_status(self.info.rca)?.into();
                if status.state() != CurrentState::Transfer {
                    return Err(Error::SignalingSwitchFailed);
                }
            }

            // Read status after signalling change
            self.read_sd_status(cmd_block).await?;
        }

        Ok(())
    }

    /// Switch mode using CMD6.
    ///
    /// Attempt to set a new signalling mode. The selected
    /// signalling mode is returned. Expects the current clock
    /// frequency to be > 12.5MHz.
    ///
    /// SD only.
    async fn switch_signalling_mode(
        &self,
        cmd_block: &mut CmdBlock,
        signalling: Signalling,
    ) -> Result<Signalling, Error> {
        // NB PLSS v7_10 4.3.10.4: "the use of SET_BLK_LEN command is not
        // necessary"

        let set_function = 0x8000_0000
            | match signalling {
                // See PLSS v7_10 Table 4-11
                Signalling::DDR50 => 0xFF_FF04,
                Signalling::SDR104 => 0xFF_1F03,
                Signalling::SDR50 => 0xFF_1F02,
                Signalling::SDR25 => 0xFF_FF01,
                Signalling::SDR12 => 0xFF_FF00,
            };

        let buffer = &mut aligned_mut(&mut cmd_block.0)[..64];
        let mode = DatapathMode::Block(block_size(size_of_val(buffer)));
        let transfer = self.sdmmc.prepare_datapath_read(buffer, mode);

        self.sdmmc.cmd(sd_cmd::cmd6(set_function), true, true)?; // CMD6

        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        // Host is allowed to use the new functions at least 8
        // clocks after the end of the switch command
        // transaction. We know the current clock period is < 80ns,
        // so a total delay of 640ns is required here
        for _ in 0..300 {
            cortex_m::asm::nop();
        }

        // Function Selection of Function Group 1
        let selection = (u32::from_be(cmd_block[4]) >> 24) & 0xF;

        match selection {
            0 => Ok(Signalling::SDR12),
            1 => Ok(Signalling::SDR25),
            2 => Ok(Signalling::SDR50),
            3 => Ok(Signalling::SDR104),
            4 => Ok(Signalling::DDR50),
            _ => Err(Error::UnsupportedCardType),
        }
    }

    /// Reads the SCR register.
    ///
    /// SD only.
    async fn get_scr(&self, cmd_block: &mut CmdBlock) -> Result<SCR, Error> {
        // Read the 64-bit SCR register
        self.sdmmc.cmd(common_cmd::set_block_length(8), true, false)?; // CMD16
        self.sdmmc.cmd(common_cmd::app_cmd(self.info.rca), true, false)?;

        let scr = &mut cmd_block.0[..2];

        // Arm `OnDrop` after the buffer, so it will be dropped first

        let transfer = self
            .sdmmc
            .prepare_datapath_read(aligned_mut(scr), DatapathMode::Block(BlockSize::Size8));
        self.sdmmc.cmd(sd_cmd::send_scr(), true, true)?;

        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        Ok(SCR(u64::from_be_bytes(unwrap!(slice8_mut(scr).try_into()))))
    }

    /// Reads the SD Status (ACMD13)
    ///
    /// SD only.
    async fn read_sd_status(&self, cmd_block: &mut CmdBlock) -> Result<SDStatus, Error> {
        let rca = self.info.rca;
        let buffer = &mut aligned_mut(&mut cmd_block.0)[..64];

        self.sdmmc
            .cmd(common_cmd::set_block_length(size_of_val(buffer) as u32), true, false)?; // CMD16
        self.sdmmc.cmd(common_cmd::app_cmd(rca), true, false)?; // APP

        let mode = DatapathMode::Block(block_size(size_of_val(buffer)));
        let transfer = self.sdmmc.prepare_datapath_read(buffer, mode);

        self.sdmmc.cmd(sd_cmd::sd_status(), true, true)?;
        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        for word in cmd_block.iter_mut() {
            *word = u32::from_be(*word);
        }

        Ok(cmd_block.0.into())
    }
}

/// Emmc storage device
impl<'a, 'b> StorageDevice<'a, 'b, Emmc> {
    /// Create a new EMMC card
    pub async fn new_emmc(sdmmc: &'a mut Sdmmc<'b>, cmd_block: &mut CmdBlock, freq: Hertz) -> Result<Self, Error> {
        let mut s = Self {
            info: Emmc::default(),
            sdmmc,
        };

        s.acquire(cmd_block, freq).await?;

        Ok(s)
    }

    async fn acquire(&mut self, _cmd_block: &mut CmdBlock, freq: Hertz) -> Result<(), Error> {
        let _scoped_block_stop = self.sdmmc.info.rcc.block_stop();
        let regs = self.sdmmc.info.regs;

        let bus_width = self.sdmmc.bus_width();

        // While the SD/SDIO card or eMMC is in identification mode,
        // the SDMMC_CK frequency must be no more than 400 kHz.
        self.sdmmc.init_idle()?;

        let ocr = loop {
            let high_voltage = 0b0 << 7;
            let access_mode = 0b10 << 29;
            let op_cond = high_voltage | access_mode | 0b1_1111_1111 << 15;
            // Initialize card
            match self.sdmmc.cmd(emmc_cmd::send_op_cond(op_cond), true, false) {
                Ok(_) => (),
                Err(Error::Crc) => (),
                Err(err) => return Err(err),
            }
            let ocr: OCR<EMMC> = regs.respr(0).read().cardstatus().into();
            if !ocr.is_busy() {
                // Power up done
                break ocr;
            }
        };

        self.info.capacity = if ocr.access_mode() == 0b10 {
            // Card is SDHC or SDXC or SDUC
            CardCapacity::HighCapacity
        } else {
            CardCapacity::StandardCapacity
        };
        self.info.ocr = ocr;
        self.info.cid = self.sdmmc.get_cid()?.into();
        self.info.rca = 1u16.into();

        self.sdmmc
            .cmd(emmc_cmd::assign_relative_address(self.info.rca), true, false)?;

        self.info.csd = self.sdmmc.get_csd(self.info.get_address())?.into();
        self.sdmmc.select_card(Some(self.info.get_address()))?;

        let (widbus, _) = bus_width_vals(bus_width);

        // Write bus width to ExtCSD byte 183
        self.sdmmc.cmd(
            emmc_cmd::modify_ext_csd(emmc_cmd::AccessMode::WriteByte, 183, widbus),
            true,
            false,
        )?;

        // Wait for ready after R1b response
        loop {
            let status: CardStatus<EMMC> = self.sdmmc.read_status(self.info.rca)?.into();
            if status.ready_for_data() {
                break;
            }
        }

        self.sdmmc.clkcr_set_clkdiv(freq.clamp(mhz(0), mhz(25)), bus_width)?;
        self.info.ext_csd = self.read_ext_csd().await?;

        Ok(())
    }

    /// Gets the EXT_CSD register.
    ///
    /// eMMC only.
    async fn read_ext_csd(&self) -> Result<ExtCSD, Error> {
        // Note: cmd_block can't be used because ExtCSD is too long to fit.
        let mut data_block = DataBlock::new();

        self.sdmmc
            .cmd(common_cmd::set_block_length(size_of::<DataBlock>() as u32), true, false)
            .unwrap(); // CMD16

        let transfer = self.sdmmc.prepare_datapath_read(
            aligned_mut(&mut data_block.0),
            DatapathMode::Block(block_size(size_of::<DataBlock>())),
        );
        self.sdmmc.cmd(emmc_cmd::send_ext_csd(), true, true)?;

        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        Ok(data_block.0.into())
    }
}

/// Card or Emmc storage device
impl<'a, 'b, A: Addressable> StorageDevice<'a, 'b, A> {
    /// Write a block
    pub fn card(&self) -> A {
        self.info.clone()
    }

    /// Read a data block.
    #[inline]
    pub async fn read_block(&mut self, block_idx: u32, data_block: &mut DataBlock) -> Result<(), Error> {
        let _scoped_block_stop = self.sdmmc.info.rcc.block_stop();
        let card_capacity = self.info.get_capacity();

        // Always read 1 block of 512 bytes
        // SDSC cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match card_capacity {
            CardCapacity::StandardCapacity => block_idx * size_of::<DataBlock>() as u32,
            _ => block_idx,
        };
        self.sdmmc
            .cmd(common_cmd::set_block_length(size_of::<DataBlock>() as u32), true, false)?; // CMD16

        let transfer = self.sdmmc.prepare_datapath_read(
            aligned_mut(&mut data_block.0),
            DatapathMode::Block(block_size(size_of::<DataBlock>())),
        );
        self.sdmmc.cmd(common_cmd::read_single_block(address), true, true)?;

        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        Ok(())
    }

    /// Read multiple data blocks.
    #[inline]
    pub async fn read_blocks(&mut self, block_idx: u32, blocks: &mut [DataBlock]) -> Result<(), Error> {
        let _scoped_block_stop = self.sdmmc.info.rcc.block_stop();
        let card_capacity = self.info.get_capacity();

        // NOTE(unsafe) reinterpret buffer as &mut [u32]
        let buffer = unsafe {
            core::slice::from_raw_parts_mut(
                blocks.as_mut_ptr() as *mut u32,
                blocks.len() * size_of::<DataBlock>() / size_of::<u32>(),
            )
        };

        // Always read 1 block of 512 bytes
        // SDSC cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match card_capacity {
            CardCapacity::StandardCapacity => block_idx * size_of::<DataBlock>() as u32,
            _ => block_idx,
        };
        self.sdmmc
            .cmd(common_cmd::set_block_length(size_of::<DataBlock>() as u32), true, false)?; // CMD16

        let transfer = self.sdmmc.prepare_datapath_read(
            aligned_mut(buffer),
            DatapathMode::Block(block_size(size_of::<DataBlock>())),
        );
        self.sdmmc.cmd(common_cmd::read_multiple_blocks(address), true, true)?;

        self.sdmmc.complete_datapath_transfer(transfer, false).await?;

        self.sdmmc.cmd(common_cmd::stop_transmission(), true, false)?; // CMD12
        self.sdmmc.clear_interrupt_flags();

        Ok(())
    }

    /// Write a data block.
    pub async fn write_block(&mut self, block_idx: u32, buffer: &DataBlock) -> Result<(), Error>
    where
        CardStatus<A::Ext>: From<u32>,
    {
        let _scoped_block_stop = self.sdmmc.info.rcc.block_stop();

        // Always read 1 block of 512 bytes
        //  cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match self.info.get_capacity() {
            CardCapacity::StandardCapacity => block_idx * size_of::<DataBlock>() as u32,
            _ => block_idx,
        };
        self.sdmmc
            .cmd(common_cmd::set_block_length(size_of::<DataBlock>() as u32), true, false)?; // CMD16

        // sdmmc_v1 uses different cmd/dma order than v2, but only for writes
        #[cfg(sdmmc_v1)]
        self.sdmmc.cmd(common_cmd::write_single_block(address), true, true)?;

        let transfer = self.sdmmc.prepare_datapath_write(
            aligned_ref(&buffer.0),
            DatapathMode::Block(block_size(size_of::<DataBlock>())),
        );

        #[cfg(sdmmc_v2)]
        self.sdmmc.cmd(common_cmd::write_single_block(address), true, true)?;

        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        // TODO: Make this configurable
        let mut timeout: u32 = 0x00FF_FFFF;

        while timeout > 0 {
            let status: CardStatus<A::Ext> = self.sdmmc.read_status(self.info.get_address())?.into();
            if status.ready_for_data() {
                return Ok(());
            }
            timeout -= 1;
        }

        Err(Error::SoftwareTimeout)
    }

    /// Write multiple data blocks.
    pub async fn write_blocks(&mut self, block_idx: u32, blocks: &[DataBlock]) -> Result<(), Error>
    where
        CardStatus<A::Ext>: From<u32>,
    {
        let _scoped_block_stop = self.sdmmc.info.rcc.block_stop();

        // NOTE(unsafe) reinterpret buffer as &[u32]
        let buffer = unsafe {
            core::slice::from_raw_parts(
                blocks.as_ptr() as *const u32,
                blocks.len() * size_of::<DataBlock>() / size_of::<u32>(),
            )
        };
        // Always read 1 block of 512 bytes
        // SDSC cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match self.info.get_capacity() {
            CardCapacity::StandardCapacity => block_idx * size_of::<DataBlock>() as u32,
            _ => block_idx,
        };

        self.sdmmc
            .cmd(common_cmd::set_block_length(size_of::<DataBlock>() as u32), true, false)?; // CMD16

        #[cfg(sdmmc_v1)]
        self.sdmmc.cmd(common_cmd::write_multiple_blocks(address), true, true)?; // CMD25

        // Setup write command
        let transfer = self.sdmmc.prepare_datapath_write(
            aligned_ref(buffer),
            DatapathMode::Block(block_size(size_of::<DataBlock>())),
        );
        #[cfg(sdmmc_v2)]
        self.sdmmc.cmd(common_cmd::write_multiple_blocks(address), true, true)?; // CMD25

        self.sdmmc.complete_datapath_transfer(transfer, false).await?;

        self.sdmmc.cmd(common_cmd::stop_transmission(), true, false)?; // CMD12
        self.sdmmc.clear_interrupt_flags();

        // TODO: Make this configurable
        let mut timeout: u32 = 0x00FF_FFFF;

        while timeout > 0 {
            let status: CardStatus<A::Ext> = self.sdmmc.read_status(self.info.get_address())?.into();
            if status.ready_for_data() {
                return Ok(());
            }
            timeout -= 1;
        }
        Err(Error::SoftwareTimeout)
    }
}

impl<'a, 'b, A: Addressable> Drop for StorageDevice<'a, 'b, A> {
    fn drop(&mut self) {
        self.sdmmc.on_drop();
    }
}

#[derive(Clone, Copy, Debug, Default)]
/// SD Card
pub struct Card {
    /// The type of this card
    pub card_type: CardCapacity,
    /// Operation Conditions Register
    pub ocr: OCR<SD>,
    /// Relative Card Address
    pub rca: u16,
    /// Card ID
    pub cid: CID<SD>,
    /// Card Specific Data
    pub csd: CSD<SD>,
    /// SD CARD Configuration Register
    pub scr: SCR,
    /// SD Status
    pub status: SDStatus,
}

impl Addressable for Card {
    type Ext = SD;

    /// Get this peripheral's address on the SDMMC bus
    fn get_address(&self) -> u16 {
        self.rca
    }

    /// Is this a standard or high capacity peripheral?
    fn get_capacity(&self) -> CardCapacity {
        self.card_type
    }

    /// Size in bytes
    fn size(&self) -> u64 {
        u64::from(self.csd.block_count()) * 512
    }
}

#[derive(Clone, Copy, Debug, Default)]
/// eMMC storage
pub struct Emmc {
    /// The capacity of this card
    pub capacity: CardCapacity,
    /// Operation Conditions Register
    pub ocr: OCR<EMMC>,
    /// Relative Card Address
    pub rca: u16,
    /// Card ID
    pub cid: CID<EMMC>,
    /// Card Specific Data
    pub csd: CSD<EMMC>,
    /// Extended Card Specific Data
    pub ext_csd: ExtCSD,
}

impl Addressable for Emmc {
    type Ext = EMMC;

    /// Get this peripheral's address on the SDMMC bus
    fn get_address(&self) -> u16 {
        self.rca
    }

    /// Is this a standard or high capacity peripheral?
    fn get_capacity(&self) -> CardCapacity {
        self.capacity
    }

    /// Size in bytes
    fn size(&self) -> u64 {
        u64::from(self.ext_csd.sector_count()) * 512
    }
}

impl<'d, 'e, A: Addressable> block_device_driver::BlockDevice<512> for StorageDevice<'d, 'e, A> {
    type Error = Error;
    type Align = aligned::A4;

    async fn read(
        &mut self,
        block_address: u32,
        buf: &mut [aligned::Aligned<Self::Align, [u8; 512]>],
    ) -> Result<(), Self::Error> {
        // TODO: I think block_address needs to be adjusted by the partition start offset
        if buf.len() == 1 {
            let block = unsafe { &mut *(&mut buf[0] as *mut _ as *mut DataBlock) };
            self.read_block(block_address, block).await?;
        } else {
            let blocks: &mut [DataBlock] =
                unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut DataBlock, buf.len()) };
            self.read_blocks(block_address, blocks).await?;
        }
        Ok(())
    }

    async fn write(
        &mut self,
        block_address: u32,
        buf: &[aligned::Aligned<Self::Align, [u8; 512]>],
    ) -> Result<(), Self::Error> {
        // TODO: I think block_address needs to be adjusted by the partition start offset
        if buf.len() == 1 {
            let block = unsafe { &*(&buf[0] as *const _ as *const DataBlock) };
            self.write_block(block_address, block).await?;
        } else {
            let blocks: &[DataBlock] =
                unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const DataBlock, buf.len()) };
            self.write_blocks(block_address, blocks).await?;
        }
        Ok(())
    }

    async fn size(&mut self) -> Result<u64, Self::Error> {
        Ok(self.info.size())
    }
}
