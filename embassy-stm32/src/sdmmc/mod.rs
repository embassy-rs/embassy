//! Secure Digital / MultiMedia Card (SDMMC)
#![macro_use]

use core::default::Default;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::slice;
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use sdio_host::common_cmd::{self, R1, R2, R3, Resp, ResponseLen, Rz};
use sdio_host::emmc::{EMMC, ExtCSD};
use sdio_host::sd::{BusWidth, CIC, CID, CSD, CardCapacity, CardStatus, CurrentState, OCR, RCA, SCR, SD, SDStatus};
use sdio_host::sd_cmd::{R6, R7};
use sdio_host::{Cmd, emmc_cmd, sd_cmd};

#[cfg(sdmmc_v1)]
use crate::dma::ChannelAndRequest;
#[cfg(gpio_v2)]
use crate::gpio::Pull;
use crate::gpio::{AfType, AnyPin, OutputType, SealedPin, Speed};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::sdmmc::Sdmmc as RegBlock;
use crate::rcc::{self, RccInfo, RccPeripheral, SealedRccPeripheral};
use crate::time::{Hertz, mhz};
use crate::{interrupt, peripherals};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::state().waker.wake();
        let status = T::info().regs.star().read();
        T::info().regs.maskr().modify(|w| {
            if status.dcrcfail() {
                w.set_dcrcfailie(false)
            }
            if status.dtimeout() {
                w.set_dtimeoutie(false)
            }
            if status.dataend() {
                w.set_dataendie(false)
            }
            if status.dbckend() {
                w.set_dbckendie(false)
            }
            #[cfg(sdmmc_v1)]
            if status.stbiterr() {
                w.set_stbiterre(false)
            }
            #[cfg(sdmmc_v2)]
            if status.dabort() {
                w.set_dabortie(false)
            }
        });
    }
}

struct U128(pub u128);

trait TypedResp: Resp {
    type Word: From<U128>;
}

impl From<U128> for () {
    fn from(value: U128) -> Self {
        match value.0 {
            0 => (),
            _ => unreachable!(),
        }
    }
}

impl From<U128> for u32 {
    fn from(value: U128) -> Self {
        unwrap!(value.0.try_into())
    }
}

impl From<U128> for u128 {
    fn from(value: U128) -> Self {
        value.0
    }
}

impl TypedResp for Rz {
    type Word = ();
}

impl TypedResp for R1 {
    type Word = u32;
}

impl TypedResp for R2 {
    type Word = u128;
}

impl TypedResp for R3 {
    type Word = u32;
}

impl TypedResp for R6 {
    type Word = u32;
}

impl TypedResp for R7 {
    type Word = u32;
}

/// Frequency used for SD Card initialization. Must be no higher than 400 kHz.
const SD_INIT_FREQ: Hertz = Hertz(400_000);

/// The signalling scheme used on the SDMMC bus
#[non_exhaustive]
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Signalling {
    SDR12,
    SDR25,
    SDR50,
    SDR104,
    DDR50,
}

impl Default for Signalling {
    fn default() -> Self {
        Signalling::SDR12
    }
}

/// Aligned data block for SDMMC transfers.
///
/// This is a 512-byte array, aligned to 4 bytes to satisfy DMA requirements.
#[repr(align(4))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DataBlock(pub [u8; 512]);

impl Deref for DataBlock {
    type Target = [u8; 512];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DataBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn slice8_mut(x: &mut [u32]) -> &mut [u8] {
    let len = x.len() * 4;
    unsafe { slice::from_raw_parts_mut(x.as_mut_ptr() as _, len) }
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

/// Errors
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Timeout reported by the hardware
    Timeout,
    /// Timeout reported by the software driver.
    SoftwareTimeout,
    /// Unsupported card version.
    UnsupportedCardVersion,
    /// Unsupported card type.
    UnsupportedCardType,
    /// Unsupported voltage.
    UnsupportedVoltage,
    /// CRC error.
    Crc,
    /// No card inserted.
    NoCard,
    /// 8-lane buses are not supported for SD cards.
    BusWidth,
    /// Bad clock supplied to the SDMMC peripheral.
    BadClock,
    /// Signaling switch failed.
    SignalingSwitchFailed,
    /// Underrun error
    Underrun,
    /// ST bit error.
    #[cfg(sdmmc_v1)]
    StBitErr,
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
    pub async fn acquire(&mut self, cmd_block: &mut CmdBlock, freq: Hertz) -> Result<(), Error> {
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
        self.sdmmc.cmd(sd_cmd::send_if_cond(1, 0xAA), false)?;
        let cic = CIC::from(regs.respr(0).read().cardstatus());

        if cic.pattern() != 0xAA {
            return Err(Error::UnsupportedCardVersion);
        }

        if cic.voltage_accepted() & 1 == 0 {
            return Err(Error::UnsupportedVoltage);
        }

        let ocr = loop {
            // Signal that next command is a app command
            self.sdmmc.cmd(common_cmd::app_cmd(0), false)?; // CMD55

            // 3.2-3.3V
            let voltage_window = 1 << 5;
            // Initialize card
            match self
                .sdmmc
                .cmd(sd_cmd::sd_send_op_cond(true, false, true, voltage_window), false)
            {
                // ACMD41
                Ok(_) => (),
                Err(Error::Crc) => (),
                Err(err) => return Err(err),
            }

            let ocr: OCR<SD> = regs.respr(0).read().cardstatus().into();
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

        self.sdmmc.cmd(sd_cmd::send_relative_address(), false)?;
        let rca = RCA::<SD>::from(regs.respr(0).read().cardstatus());
        self.info.rca = rca.address();

        self.info.csd = self.sdmmc.get_csd(self.info.get_address())?.into();
        self.sdmmc.select_card(Some(self.info.get_address()))?;

        self.info.scr = self.get_scr(cmd_block).await?;

        let (bus_width, acmd_arg) = if !self.info.scr.bus_width_four() {
            (BusWidth::One, 0)
        } else {
            (BusWidth::Four, 2)
        };

        self.sdmmc.cmd(common_cmd::app_cmd(self.info.rca), false)?;
        self.sdmmc.cmd(sd_cmd::cmd6(acmd_arg), false)?;

        self.sdmmc.clkcr_set_clkdiv(freq.clamp(mhz(0), mhz(25)), bus_width)?;

        // Read status
        self.info.status = self.read_sd_status(cmd_block).await?;

        if freq > mhz(25) {
            // Switch to SDR25
            self.sdmmc.signalling = self.switch_signalling_mode(cmd_block, Signalling::SDR25).await?;

            if self.sdmmc.signalling == Signalling::SDR25 {
                // Set final clock frequency
                self.sdmmc.clkcr_set_clkdiv(freq, bus_width)?;

                if self.sdmmc.read_status(&self.info)?.state() != CurrentState::Transfer {
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

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let on_drop = OnDrop::new(|| self.sdmmc.on_drop());

        let transfer = self.sdmmc.prepare_datapath_read(cmd_block.as_mut(), 64, 6);
        self.sdmmc.cmd(sd_cmd::cmd6(set_function), true)?; // CMD6

        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        // Host is allowed to use the new functions at least 8
        // clocks after the end of the switch command
        // transaction. We know the current clock period is < 80ns,
        // so a total delay of 640ns is required here
        for _ in 0..300 {
            cortex_m::asm::nop();
        }

        on_drop.defuse();

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
        self.sdmmc.cmd(common_cmd::set_block_length(8), false)?; // CMD16
        self.sdmmc.cmd(common_cmd::app_cmd(self.info.rca), false)?;

        let scr = &mut cmd_block.0[..2];

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let on_drop = OnDrop::new(|| self.sdmmc.on_drop());

        let transfer = self.sdmmc.prepare_datapath_read(scr, 8, 3);
        self.sdmmc.cmd(sd_cmd::send_scr(), true)?;

        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        on_drop.defuse();

        Ok(SCR(u64::from_be_bytes(unwrap!(slice8_mut(scr).try_into()))))
    }

    /// Reads the SD Status (ACMD13)
    ///
    /// SD only.
    async fn read_sd_status(&self, cmd_block: &mut CmdBlock) -> Result<SDStatus, Error> {
        let rca = self.info.rca;

        self.sdmmc.cmd(common_cmd::set_block_length(64), false)?; // CMD16
        self.sdmmc.cmd(common_cmd::app_cmd(rca), false)?; // APP

        let status = cmd_block;

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let on_drop = OnDrop::new(|| self.sdmmc.on_drop());

        let transfer = self.sdmmc.prepare_datapath_read(status.as_mut(), 64, 6);
        self.sdmmc.cmd(sd_cmd::sd_status(), true)?;

        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        on_drop.defuse();

        for byte in status.iter_mut() {
            *byte = u32::from_be(*byte);
        }

        Ok(status.0.into())
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
            match self.sdmmc.cmd(emmc_cmd::send_op_cond(op_cond), false) {
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
            .cmd(emmc_cmd::assign_relative_address(self.info.rca), false)?;

        self.info.csd = self.sdmmc.get_csd(self.info.get_address())?.into();
        self.sdmmc.select_card(Some(self.info.get_address()))?;

        let (widbus, _) = bus_width_vals(bus_width);

        // Write bus width to ExtCSD byte 183
        self.sdmmc.cmd(
            emmc_cmd::modify_ext_csd(emmc_cmd::AccessMode::WriteByte, 183, widbus),
            false,
        )?;

        // Wait for ready after R1b response
        loop {
            let status = self.sdmmc.read_status(&self.info)?;

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
        let mut data_block = DataBlock([0u8; 512]);

        // NOTE(unsafe) DataBlock uses align 4
        let buffer = unsafe { &mut *((&mut data_block.0) as *mut [u8; 512] as *mut [u32; 128]) };

        self.sdmmc.cmd(common_cmd::set_block_length(512), false).unwrap(); // CMD16

        let transfer = self.sdmmc.prepare_datapath_read(buffer, 512, 9);
        self.sdmmc.cmd(emmc_cmd::send_ext_csd(), true)?;

        // Arm `OnDrop` after the buffer, so it will be dropped first
        let on_drop = OnDrop::new(|| self.sdmmc.on_drop());

        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        on_drop.defuse();

        Ok(unsafe { core::mem::transmute::<_, [u32; 128]>(data_block.0) }.into())
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
    pub async fn read_block(&mut self, block_idx: u32, buffer: &mut DataBlock) -> Result<(), Error> {
        let _scoped_block_stop = self.sdmmc.info.rcc.block_stop();
        let card_capacity = self.info.get_capacity();

        // NOTE(unsafe) DataBlock uses align 4
        let buffer = unsafe { &mut *((&mut buffer.0) as *mut [u8; 512] as *mut [u32; 128]) };

        // Always read 1 block of 512 bytes
        // SDSC cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match card_capacity {
            CardCapacity::StandardCapacity => block_idx * 512,
            _ => block_idx,
        };
        self.sdmmc.cmd(common_cmd::set_block_length(512), false)?; // CMD16

        let on_drop = OnDrop::new(|| self.sdmmc.on_drop());

        let transfer = self.sdmmc.prepare_datapath_read(buffer, 512, 9);
        self.sdmmc.cmd(common_cmd::read_single_block(address), true)?;

        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        on_drop.defuse();

        Ok(())
    }

    /// Read multiple data blocks.
    #[inline]
    pub async fn read_blocks(&mut self, block_idx: u32, blocks: &mut [DataBlock]) -> Result<(), Error> {
        let _scoped_block_stop = self.sdmmc.info.rcc.block_stop();
        let card_capacity = self.info.get_capacity();

        // NOTE(unsafe) reinterpret buffer as &mut [u32]
        let buffer = unsafe {
            let ptr = blocks.as_mut_ptr() as *mut u32;
            let len = blocks.len() * 128;
            core::slice::from_raw_parts_mut(ptr, len)
        };

        // Always read 1 block of 512 bytes
        // SDSC cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match card_capacity {
            CardCapacity::StandardCapacity => block_idx * 512,
            _ => block_idx,
        };
        self.sdmmc.cmd(common_cmd::set_block_length(512), false)?; // CMD16

        let on_drop = OnDrop::new(|| self.sdmmc.on_drop());
        let transfer = self.sdmmc.prepare_datapath_read(buffer, 512 * blocks.len() as u32, 9);
        self.sdmmc.cmd(common_cmd::read_multiple_blocks(address), true)?;

        self.sdmmc.complete_datapath_transfer(transfer, false).await?;

        self.sdmmc.cmd(common_cmd::stop_transmission(), false)?; // CMD12
        self.sdmmc.clear_interrupt_flags();

        on_drop.defuse();

        Ok(())
    }

    /// Write a data block.
    pub async fn write_block(&mut self, block_idx: u32, buffer: &DataBlock) -> Result<(), Error>
    where
        CardStatus<A::Ext>: From<u32>,
    {
        let _scoped_block_stop = self.sdmmc.info.rcc.block_stop();

        // NOTE(unsafe) DataBlock uses align 4
        let buffer = unsafe { &*((&buffer.0) as *const [u8; 512] as *const [u32; 128]) };

        // Always read 1 block of 512 bytes
        //  cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match self.info.get_capacity() {
            CardCapacity::StandardCapacity => block_idx * 512,
            _ => block_idx,
        };
        self.sdmmc.cmd(common_cmd::set_block_length(512), false)?; // CMD16

        let on_drop = OnDrop::new(|| self.sdmmc.on_drop());

        // sdmmc_v1 uses different cmd/dma order than v2, but only for writes
        #[cfg(sdmmc_v1)]
        self.sdmmc.cmd(common_cmd::write_single_block(address), true)?;

        let transfer = self.sdmmc.prepare_datapath_write(buffer, 512, 9);

        #[cfg(sdmmc_v2)]
        self.sdmmc.cmd(common_cmd::write_single_block(address), true)?;

        self.sdmmc.complete_datapath_transfer(transfer, true).await?;

        on_drop.defuse();

        // TODO: Make this configurable
        let mut timeout: u32 = 0x00FF_FFFF;

        while timeout > 0 {
            let ready_for_data = self.sdmmc.read_status(&self.info)?.ready_for_data();
            if ready_for_data {
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
            let ptr = blocks.as_ptr() as *const u32;
            let len = blocks.len() * 128;
            core::slice::from_raw_parts(ptr, len)
        };

        // Always read 1 block of 512 bytes
        // SDSC cards are byte addressed hence the blockaddress is in multiples of 512 bytes
        let address = match self.info.get_capacity() {
            CardCapacity::StandardCapacity => block_idx * 512,
            _ => block_idx,
        };

        self.sdmmc.cmd(common_cmd::set_block_length(512), false)?; // CMD16

        let block_count = blocks.len();

        let on_drop = OnDrop::new(|| self.sdmmc.on_drop());

        #[cfg(sdmmc_v1)]
        self.sdmmc.cmd(common_cmd::write_multiple_blocks(address), true)?; // CMD25

        // Setup write command
        let transfer = self.sdmmc.prepare_datapath_write(buffer, 512 * block_count as u32, 9);

        #[cfg(sdmmc_v2)]
        self.sdmmc.cmd(common_cmd::write_multiple_blocks(address), true)?; // CMD25

        self.sdmmc.complete_datapath_transfer(transfer, false).await?;

        self.sdmmc.cmd(common_cmd::stop_transmission(), false)?; // CMD12
        self.sdmmc.clear_interrupt_flags();

        on_drop.defuse();

        // TODO: Make this configurable
        let mut timeout: u32 = 0x00FF_FFFF;

        while timeout > 0 {
            let ready_for_data = self.sdmmc.read_status(&self.info)?.ready_for_data();

            if ready_for_data {
                return Ok(());
            }
            timeout -= 1;
        }
        Err(Error::SoftwareTimeout)
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

#[repr(u8)]
enum PowerCtrl {
    Off = 0b00,
    On = 0b11,
}

fn get_waitresp_val(rlen: ResponseLen) -> u8 {
    match rlen {
        common_cmd::ResponseLen::Zero => 0,
        common_cmd::ResponseLen::R48 => 1,
        common_cmd::ResponseLen::R136 => 3,
    }
}

/// Calculate clock divisor. Returns a SDMMC_CK less than or equal to
/// `sdmmc_ck` in Hertz.
///
/// Returns `(bypass, clk_div, clk_f)`, where `bypass` enables clock divisor bypass (only sdmmc_v1),
/// `clk_div` is the divisor register value and `clk_f` is the resulting new clock frequency.
#[cfg(sdmmc_v1)]
fn clk_div(ker_ck: Hertz, sdmmc_ck: u32) -> Result<(bool, u8, Hertz), Error> {
    // sdmmc_v1 maximum clock is 50 MHz
    if sdmmc_ck > 50_000_000 {
        return Err(Error::BadClock);
    }

    // bypass divisor
    if ker_ck.0 <= sdmmc_ck {
        return Ok((true, 0, ker_ck));
    }

    let clk_div = match ker_ck.0.div_ceil(sdmmc_ck) {
        0 | 1 => Ok(0),
        x @ 2..=258 => Ok((x - 2) as u8),
        _ => Err(Error::BadClock),
    }?;

    // SDIO_CK frequency = SDIOCLK / [CLKDIV + 2]
    let clk_f = Hertz(ker_ck.0 / (clk_div as u32 + 2));
    Ok((false, clk_div, clk_f))
}

fn bus_width_vals(bus_width: BusWidth) -> (u8, u32) {
    match bus_width {
        BusWidth::One => (0, 1u32),
        BusWidth::Four => (1, 4u32),
        BusWidth::Eight => (2, 8u32),
        _ => panic!("Invalid Bus Width"),
    }
}

/// Calculate clock divisor. Returns a SDMMC_CK less than or equal to
/// `sdmmc_ck` in Hertz.
///
/// Returns `(bypass, clk_div, clk_f)`, where `bypass` enables clock divisor bypass (only sdmmc_v1),
/// `clk_div` is the divisor register value and `clk_f` is the resulting new clock frequency.
#[cfg(sdmmc_v2)]
fn clk_div(ker_ck: Hertz, sdmmc_ck: u32) -> Result<(bool, u16, Hertz), Error> {
    match ker_ck.0.div_ceil(sdmmc_ck) {
        0 | 1 => Ok((false, 0, ker_ck)),
        x @ 2..=2046 => {
            // SDMMC_CK frequency = SDMMCCLK / [CLKDIV * 2]
            let clk_div = x.div_ceil(2) as u16;
            let clk = Hertz(ker_ck.0 / (clk_div as u32 * 2));

            Ok((false, clk_div, clk))
        }
        _ => Err(Error::BadClock),
    }
}

#[cfg(sdmmc_v1)]
type Transfer<'a> = crate::dma::Transfer<'a>;
#[cfg(sdmmc_v2)]
struct Transfer<'a> {
    _dummy: PhantomData<&'a ()>,
}

#[cfg(all(sdmmc_v1, dma))]
const DMA_TRANSFER_OPTIONS: crate::dma::TransferOptions = crate::dma::TransferOptions {
    pburst: crate::dma::Burst::Incr4,
    mburst: crate::dma::Burst::Incr4,
    flow_ctrl: crate::dma::FlowControl::Peripheral,
    fifo_threshold: Some(crate::dma::FifoThreshold::Full),
    priority: crate::dma::Priority::VeryHigh,
    circular: false,
    half_transfer_ir: false,
    complete_transfer_ir: true,
};
#[cfg(all(sdmmc_v1, not(dma)))]
const DMA_TRANSFER_OPTIONS: crate::dma::TransferOptions = crate::dma::TransferOptions {
    priority: crate::dma::Priority::VeryHigh,
    circular: false,
    half_transfer_ir: false,
    complete_transfer_ir: true,
};

/// SDMMC configuration
///
/// Default values:
/// data_transfer_timeout: 5_000_000
#[non_exhaustive]
pub struct Config {
    /// The timeout to be set for data transfers, in card bus clock periods
    pub data_transfer_timeout: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_transfer_timeout: 5_000_000,
        }
    }
}

/// Sdmmc device
pub struct Sdmmc<'d> {
    info: &'static Info,
    state: &'static State,
    ker_clk: Hertz,
    #[cfg(sdmmc_v1)]
    dma: ChannelAndRequest<'d>,

    clk: Peri<'d, AnyPin>,
    cmd: Peri<'d, AnyPin>,
    d0: Peri<'d, AnyPin>,
    d1: Option<Peri<'d, AnyPin>>,
    d2: Option<Peri<'d, AnyPin>>,
    d3: Option<Peri<'d, AnyPin>>,
    d4: Option<Peri<'d, AnyPin>>,
    d5: Option<Peri<'d, AnyPin>>,
    d6: Option<Peri<'d, AnyPin>>,
    d7: Option<Peri<'d, AnyPin>>,

    config: Config,
    /// Current clock to card
    clock: Hertz,
    /// Current signalling scheme to card
    signalling: Signalling,
}

const CLK_AF: AfType = AfType::output(OutputType::PushPull, Speed::VeryHigh);
#[cfg(gpio_v1)]
const CMD_AF: AfType = AfType::output(OutputType::PushPull, Speed::VeryHigh);
#[cfg(gpio_v2)]
const CMD_AF: AfType = AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up);
const DATA_AF: AfType = CMD_AF;

#[cfg(sdmmc_v1)]
impl<'d> Sdmmc<'d> {
    /// Create a new SDMMC driver, with 1 data lane.
    pub fn new_1bit<T: Instance>(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        dma: Peri<'d, impl SdmmcDma<T>>,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            set_as_af!(clk, CLK_AF);
            set_as_af!(cmd, CMD_AF);
            set_as_af!(d0, DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            new_dma_nonopt!(dma),
            clk.into(),
            cmd.into(),
            d0.into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            config,
        )
    }

    /// Create a new SDMMC driver, with 4 data lanes.
    pub fn new_4bit<T: Instance>(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        dma: Peri<'d, impl SdmmcDma<T>>,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            set_as_af!(clk, CLK_AF);
            set_as_af!(cmd, CMD_AF);
            set_as_af!(d0, DATA_AF);
            set_as_af!(d1, DATA_AF);
            set_as_af!(d2, DATA_AF);
            set_as_af!(d3, DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            new_dma_nonopt!(dma),
            clk.into(),
            cmd.into(),
            d0.into(),
            Some(d1.into()),
            Some(d2.into()),
            Some(d3.into()),
            None,
            None,
            None,
            None,
            config,
        )
    }
}

#[cfg(sdmmc_v1)]
impl<'d> Sdmmc<'d> {
    /// Create a new SDMMC driver, with 8 data lanes.
    pub fn new_8bit<T: Instance>(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        dma: Peri<'d, impl SdmmcDma<T>>,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            set_as_af!(clk, CLK_AF);
            set_as_af!(cmd, CMD_AF);
            set_as_af!(d0, DATA_AF);
            set_as_af!(d1, DATA_AF);
            set_as_af!(d2, DATA_AF);
            set_as_af!(d3, DATA_AF);
            set_as_af!(d4, DATA_AF);
            set_as_af!(d5, DATA_AF);
            set_as_af!(d6, DATA_AF);
            set_as_af!(d7, DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            new_dma_nonopt!(dma),
            clk.into(),
            cmd.into(),
            d0.into(),
            Some(d1.into()),
            Some(d2.into()),
            Some(d3.into()),
            Some(d4.into()),
            Some(d5.into()),
            Some(d6.into()),
            Some(d7.into()),
            config,
        )
    }
}

#[cfg(sdmmc_v2)]
impl<'d> Sdmmc<'d> {
    /// Create a new SDMMC driver, with 1 data lane.
    pub fn new_1bit<T: Instance>(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            set_as_af!(clk, CLK_AF);
            set_as_af!(cmd, CMD_AF);
            set_as_af!(d0, DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            clk.into(),
            cmd.into(),
            d0.into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            config,
        )
    }

    /// Create a new SDMMC driver, with 4 data lanes.
    pub fn new_4bit<T: Instance>(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            set_as_af!(clk, CLK_AF);
            set_as_af!(cmd, CMD_AF);
            set_as_af!(d0, DATA_AF);
            set_as_af!(d1, DATA_AF);
            set_as_af!(d2, DATA_AF);
            set_as_af!(d3, DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            clk.into(),
            cmd.into(),
            d0.into(),
            Some(d1.into()),
            Some(d2.into()),
            Some(d3.into()),
            None,
            None,
            None,
            None,
            config,
        )
    }
}

#[cfg(sdmmc_v2)]
impl<'d> Sdmmc<'d> {
    /// Create a new SDMMC driver, with 8 data lanes.
    pub fn new_8bit<T: Instance>(
        sdmmc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        clk: Peri<'d, impl CkPin<T>>,
        cmd: Peri<'d, impl CmdPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        config: Config,
    ) -> Self {
        critical_section::with(|_| {
            set_as_af!(clk, CLK_AF);
            set_as_af!(cmd, CMD_AF);
            set_as_af!(d0, DATA_AF);
            set_as_af!(d1, DATA_AF);
            set_as_af!(d2, DATA_AF);
            set_as_af!(d3, DATA_AF);
            set_as_af!(d4, DATA_AF);
            set_as_af!(d5, DATA_AF);
            set_as_af!(d6, DATA_AF);
            set_as_af!(d7, DATA_AF);
        });

        Self::new_inner(
            sdmmc,
            clk.into(),
            cmd.into(),
            d0.into(),
            Some(d1.into()),
            Some(d2.into()),
            Some(d3.into()),
            Some(d4.into()),
            Some(d5.into()),
            Some(d6.into()),
            Some(d7.into()),
            config,
        )
    }
}

impl<'d> Sdmmc<'d> {
    fn enable_interrupts(&self) {
        let regs = self.info.regs;
        regs.maskr().write(|w| {
            w.set_dcrcfailie(true);
            w.set_dtimeoutie(true);
            w.set_dataendie(true);
            w.set_dbckendie(true);

            #[cfg(sdmmc_v1)]
            w.set_stbiterre(true);
            #[cfg(sdmmc_v2)]
            w.set_dabortie(true);
        });
    }

    fn new_inner<T: Instance>(
        _sdmmc: Peri<'d, T>,
        #[cfg(sdmmc_v1)] dma: ChannelAndRequest<'d>,
        clk: Peri<'d, AnyPin>,
        cmd: Peri<'d, AnyPin>,
        d0: Peri<'d, AnyPin>,
        d1: Option<Peri<'d, AnyPin>>,
        d2: Option<Peri<'d, AnyPin>>,
        d3: Option<Peri<'d, AnyPin>>,
        d4: Option<Peri<'d, AnyPin>>,
        d5: Option<Peri<'d, AnyPin>>,
        d6: Option<Peri<'d, AnyPin>>,
        d7: Option<Peri<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        rcc::enable_and_reset_without_stop::<T>();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let info = T::info();
        let state = T::state();
        let ker_clk = T::frequency();

        info.regs.clkcr().write(|w| {
            w.set_pwrsav(false);
            w.set_negedge(false);

            // Hardware flow control is broken on SDIOv1 and causes clock glitches, which result in CRC errors.
            // See chip erratas for more details.
            #[cfg(sdmmc_v1)]
            w.set_hwfc_en(false);
            #[cfg(sdmmc_v2)]
            w.set_hwfc_en(true);

            #[cfg(sdmmc_v1)]
            w.set_clken(true);
        });

        // Power off, writen 00: Clock to the card is stopped;
        // D[7:0], CMD, and CK are driven high.
        info.regs.power().modify(|w| w.set_pwrctrl(PowerCtrl::Off as u8));

        Self {
            info,
            state,
            ker_clk,
            #[cfg(sdmmc_v1)]
            dma,

            clk,
            cmd,
            d0,
            d1,
            d2,
            d3,
            d4,
            d5,
            d6,
            d7,

            config,
            clock: SD_INIT_FREQ,
            signalling: Default::default(),
        }
    }

    /// Data transfer is in progress
    #[inline]
    fn data_active(&self) -> bool {
        let regs = self.info.regs;

        let status = regs.star().read();
        #[cfg(sdmmc_v1)]
        return status.rxact() || status.txact();
        #[cfg(sdmmc_v2)]
        return status.dpsmact();
    }

    /// Coammand transfer is in progress
    #[inline]
    fn cmd_active(&self) -> bool {
        let regs = self.info.regs;

        let status = regs.star().read();
        #[cfg(sdmmc_v1)]
        return status.cmdact();
        #[cfg(sdmmc_v2)]
        return status.cpsmact();
    }

    /// Wait idle on CMDACT, RXACT and TXACT (v1) or DOSNACT and CPSMACT (v2)
    #[inline]
    fn wait_idle(&self) {
        while self.data_active() || self.cmd_active() {}
    }

    fn bus_width(&self) -> BusWidth {
        match (self.d3.is_some(), self.d7.is_some()) {
            (true, true) => BusWidth::Eight,
            (true, false) => BusWidth::Four,
            _ => BusWidth::One,
        }
    }

    /// # Safety
    ///
    /// `buffer` must be valid for the whole transfer and word aligned
    #[allow(unused_variables)]
    fn prepare_datapath_read<'a>(&'a self, buffer: &'a mut [u32], length_bytes: u32, block_size: u8) -> Transfer<'a> {
        assert!(block_size <= 14, "Block size up to 2^14 bytes");
        let regs = self.info.regs;

        // Command AND Data state machines must be idle
        self.wait_idle();
        self.clear_interrupt_flags();

        regs.dlenr().write(|w| w.set_datalength(length_bytes));

        // SAFETY: No other functions use the dma
        #[cfg(sdmmc_v1)]
        let transfer = unsafe {
            self.dma
                .read_unchecked(regs.fifor().as_ptr() as *mut u32, buffer, DMA_TRANSFER_OPTIONS)
        };
        #[cfg(sdmmc_v2)]
        let transfer = {
            regs.idmabase0r().write(|w| w.set_idmabase0(buffer.as_mut_ptr() as u32));
            regs.idmactrlr().modify(|w| w.set_idmaen(true));
            Transfer {
                _dummy: core::marker::PhantomData,
            }
        };

        regs.dctrl().modify(|w| {
            w.set_dblocksize(block_size);
            w.set_dtdir(true);
            #[cfg(sdmmc_v1)]
            {
                w.set_dmaen(true);
                w.set_dten(true);
            }
        });

        self.enable_interrupts();

        transfer
    }

    /// # Safety
    ///
    /// `buffer` must be valid for the whole transfer and word aligned
    fn prepare_datapath_write<'a>(&'a self, buffer: &'a [u32], length_bytes: u32, block_size: u8) -> Transfer<'a> {
        assert!(block_size <= 14, "Block size up to 2^14 bytes");
        let regs = self.info.regs;

        // Command AND Data state machines must be idle
        self.wait_idle();
        self.clear_interrupt_flags();

        regs.dlenr().write(|w| w.set_datalength(length_bytes));

        // SAFETY: No other functions use the dma
        #[cfg(sdmmc_v1)]
        let transfer = unsafe {
            self.dma
                .write_unchecked(buffer, regs.fifor().as_ptr() as *mut u32, DMA_TRANSFER_OPTIONS)
        };
        #[cfg(sdmmc_v2)]
        let transfer = {
            regs.idmabase0r().write(|w| w.set_idmabase0(buffer.as_ptr() as u32));
            regs.idmactrlr().modify(|w| w.set_idmaen(true));
            Transfer {
                _dummy: core::marker::PhantomData,
            }
        };

        regs.dctrl().modify(|w| {
            w.set_dblocksize(block_size);
            w.set_dtdir(false);
            #[cfg(sdmmc_v1)]
            {
                w.set_dmaen(true);
                w.set_dten(true);
            }
        });

        self.enable_interrupts();

        transfer
    }

    /// Stops the DMA datapath
    fn stop_datapath(&self) {
        let regs = self.info.regs;

        #[cfg(sdmmc_v1)]
        regs.dctrl().modify(|w| {
            w.set_dmaen(false);
            w.set_dten(false);
        });
        #[cfg(sdmmc_v2)]
        regs.idmactrlr().modify(|w| w.set_idmaen(false));
    }

    fn init_idle(&mut self) -> Result<(), Error> {
        let regs = self.info.regs;

        self.clkcr_set_clkdiv(SD_INIT_FREQ, BusWidth::One)?;
        regs.dtimer()
            .write(|w| w.set_datatime(self.config.data_transfer_timeout));

        regs.power().modify(|w| w.set_pwrctrl(PowerCtrl::On as u8));
        self.cmd(common_cmd::idle(), false)
    }

    /// Sets the CLKDIV field in CLKCR. Updates clock field in self
    fn clkcr_set_clkdiv(&mut self, freq: Hertz, width: BusWidth) -> Result<(), Error> {
        let regs = self.info.regs;

        let (widbus, width_u32) = bus_width_vals(width);
        let (_bypass, clkdiv, new_clock) = clk_div(self.ker_clk, freq.0)?;

        // Enforce AHB and SDMMC_CK clock relation. See RM0433 Rev 7
        // Section 55.5.8
        let sdmmc_bus_bandwidth = new_clock.0 * width_u32;
        assert!(self.ker_clk.0 > 3 * sdmmc_bus_bandwidth / 32);
        self.clock = new_clock;

        // CPSMACT and DPSMACT must be 0 to set CLKDIV or WIDBUS
        self.wait_idle();
        regs.clkcr().modify(|w| {
            w.set_clkdiv(clkdiv);
            #[cfg(sdmmc_v1)]
            w.set_bypass(_bypass);
            w.set_widbus(widbus);
        });

        Ok(())
    }

    fn get_cid(&self) -> Result<u128, Error> {
        self.cmd(common_cmd::all_send_cid(), false) // CMD2
    }

    fn get_csd(&self, address: u16) -> Result<u128, Error> {
        self.cmd(common_cmd::send_csd(address), false)
    }

    /// Query the card status (CMD13, returns R1)
    fn read_status<A: Addressable>(&self, card: &A) -> Result<CardStatus<A::Ext>, Error>
    where
        CardStatus<A::Ext>: From<u32>,
    {
        let rca = card.get_address();

        Ok(self.cmd(common_cmd::card_status(rca, false), false)?.into()) // CMD13
    }

    /// Select one card and place it into the _Tranfer State_
    ///
    /// If `None` is specifed for `card`, all cards are put back into
    /// _Stand-by State_
    fn select_card(&self, rca: Option<u16>) -> Result<(), Error> {
        // Determine Relative Card Address (RCA) of given card
        let rca = rca.unwrap_or(0);

        let resp = self.cmd(common_cmd::select_card(rca), false);

        if let Err(Error::Timeout) = resp
            && rca == 0
        {
            return Ok(());
        }

        resp?;

        Ok(())
    }

    /// Clear flags in interrupt clear register
    #[inline]
    fn clear_interrupt_flags(&self) {
        let regs = self.info.regs;
        regs.icr().write(|w| {
            w.set_ccrcfailc(true);
            w.set_dcrcfailc(true);
            w.set_ctimeoutc(true);
            w.set_dtimeoutc(true);
            w.set_txunderrc(true);
            w.set_rxoverrc(true);
            w.set_cmdrendc(true);
            w.set_cmdsentc(true);
            w.set_dataendc(true);
            w.set_dbckendc(true);
            w.set_sdioitc(true);
            #[cfg(sdmmc_v1)]
            w.set_stbiterrc(true);

            #[cfg(sdmmc_v2)]
            {
                w.set_dholdc(true);
                w.set_dabortc(true);
                w.set_busyd0endc(true);
                w.set_ackfailc(true);
                w.set_acktimeoutc(true);
                w.set_vswendc(true);
                w.set_ckstopc(true);
                w.set_idmatec(true);
                w.set_idmabtcc(true);
            }
        });
    }

    /// Send command to card
    #[allow(unused_variables)]
    fn cmd<R: TypedResp>(&self, cmd: Cmd<R>, data: bool) -> Result<R::Word, Error> {
        let regs = self.info.regs;

        self.clear_interrupt_flags();
        // CP state machine must be idle
        while self.cmd_active() {}

        // Command arg
        regs.argr().write(|w| w.set_cmdarg(cmd.arg));

        // Command index and start CP State Machine
        regs.cmdr().write(|w| {
            w.set_waitint(false);
            w.set_waitresp(get_waitresp_val(cmd.response_len()));
            w.set_cmdindex(cmd.cmd);
            w.set_cpsmen(true);

            #[cfg(sdmmc_v2)]
            {
                // Special mode in CP State Machine
                // CMD12: Stop Transmission
                let cpsm_stop_transmission = cmd.cmd == 12;
                w.set_cmdstop(cpsm_stop_transmission);
                w.set_cmdtrans(data);
            }
        });

        let mut status;
        if cmd.response_len() == ResponseLen::Zero {
            // Wait for CMDSENT or a timeout
            while {
                status = regs.star().read();
                !(status.ctimeout() || status.cmdsent())
            } {}
        } else {
            // Wait for CMDREND or CCRCFAIL or a timeout
            while {
                status = regs.star().read();
                !(status.ctimeout() || status.cmdrend() || status.ccrcfail())
            } {}
        }

        if status.ctimeout() {
            return Err(Error::Timeout);
        } else if status.ccrcfail() {
            return Err(Error::Crc);
        }

        Ok(match R::LENGTH {
            ResponseLen::Zero => U128(0u128),
            ResponseLen::R48 => U128(self.info.regs.respr(0).read().cardstatus() as u128),
            ResponseLen::R136 => {
                let cid0 = self.info.regs.respr(0).read().cardstatus() as u128;
                let cid1 = self.info.regs.respr(1).read().cardstatus() as u128;
                let cid2 = self.info.regs.respr(2).read().cardstatus() as u128;
                let cid3 = self.info.regs.respr(3).read().cardstatus() as u128;

                U128((cid0 << 96) | (cid1 << 64) | (cid2 << 32) | (cid3))
            }
        }
        .into())
    }

    fn on_drop(&self) {
        let regs = self.info.regs;
        if self.data_active() {
            self.clear_interrupt_flags();
            // Send abort
            // CP state machine must be idle
            while self.cmd_active() {}

            // Command arg
            regs.argr().write(|w| w.set_cmdarg(0));

            // Command index and start CP State Machine
            regs.cmdr().write(|w| {
                w.set_waitint(false);
                w.set_waitresp(get_waitresp_val(ResponseLen::R48));
                w.set_cmdindex(12);
                w.set_cpsmen(true);

                #[cfg(sdmmc_v2)]
                {
                    w.set_cmdstop(true);
                    w.set_cmdtrans(false);
                }
            });

            // Wait for the abort
            while self.data_active() {}
        }
        regs.maskr().write(|_| ()); // disable irqs
        self.clear_interrupt_flags();
        self.stop_datapath();
    }

    /// Wait for a previously started datapath transfer to complete from an interrupt.
    #[inline]
    #[allow(unused)]
    async fn complete_datapath_transfer(&self, transfer: Transfer<'_>, block: bool) -> Result<(), Error> {
        let res = poll_fn(|cx| {
            // Compiler might not be sufficiently constrained here
            // https://github.com/embassy-rs/embassy/issues/4723
            self.state.waker.register(cx.waker());
            let status = self.info.regs.star().read();

            if status.dcrcfail() {
                return Poll::Ready(Err(Error::Crc));
            }
            if status.dtimeout() {
                return Poll::Ready(Err(Error::Timeout));
            }
            if status.txunderr() {
                return Poll::Ready(Err(Error::Underrun));
            }
            #[cfg(sdmmc_v1)]
            if status.stbiterr() {
                return Poll::Ready(Err(Error::StBitErr));
            }
            #[cfg(sdmmc_v1)]
            let done = match block {
                true => status.dbckend(),
                false => status.dataend(),
            };
            #[cfg(sdmmc_v2)]
            let done = status.dataend();
            if done {
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        self.clear_interrupt_flags();
        self.stop_datapath();

        drop(transfer);

        res
    }

    /// Get the current SDMMC bus clock
    pub fn clock(&self) -> Hertz {
        self.clock
    }
}

impl<'d> Drop for Sdmmc<'d> {
    fn drop(&mut self) {
        // T::Interrupt::disable();
        self.on_drop();

        critical_section::with(|_| {
            self.clk.set_as_disconnected();
            self.cmd.set_as_disconnected();
            self.d0.set_as_disconnected();
            if let Some(x) = &mut self.d1 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d2 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d3 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d4 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d5 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d6 {
                x.set_as_disconnected();
            }
            if let Some(x) = &mut self.d7 {
                x.set_as_disconnected();
            }
        });
    }
}

//////////////////////////////////////////////////////

type Regs = RegBlock;

struct Info {
    regs: Regs,
    rcc: RccInfo,
}

struct State {
    waker: AtomicWaker,
}

impl State {
    const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

trait SealedInstance {
    fn info() -> &'static Info;
    fn state() -> &'static State;
}

/// SDMMC instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral + 'static {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(CkPin, Instance);
pin_trait!(CmdPin, Instance);
pin_trait!(D0Pin, Instance);
pin_trait!(D1Pin, Instance);
pin_trait!(D2Pin, Instance);
pin_trait!(D3Pin, Instance);
pin_trait!(D4Pin, Instance);
pin_trait!(D5Pin, Instance);
pin_trait!(D6Pin, Instance);
pin_trait!(D7Pin, Instance);

#[cfg(sdmmc_v1)]
dma_trait!(SdmmcDma, Instance);

foreach_peripheral!(
    (sdmmc, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: unsafe { Regs::from_ptr(crate::pac::$inst.as_ptr()) },
                    rcc: crate::peripherals::$inst::RCC_INFO,
                };
                &INFO
            }

            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$inst;
        }
    };
);

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
            let block = unsafe { &mut *(&mut buf[0] as *mut _ as *mut crate::sdmmc::DataBlock) };
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
            let block = unsafe { &*(&buf[0] as *const _ as *const crate::sdmmc::DataBlock) };
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
