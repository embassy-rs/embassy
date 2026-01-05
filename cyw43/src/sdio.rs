use core::slice;

use aligned::{A4, Aligned};
use embassy_time::{Duration, Timer};

use crate::consts::*;
use crate::runner::{BusType, SealedBus};
use crate::try_until;
use crate::util::{aligned_mut, aligned_ref, slice8_mut};

// macro_rules! ALIGN_UINT {
//     ($val:expr, $align:expr) => {
//         ((($val) + ($align) - 1) & !(($align) - 1))
//     };
// }
//
// macro_rules! WRITE_BYTES_PAD {
//     ($len:expr) => {
//         ALIGN_UINT!($len, 64)
//     };
// }

#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(derive_more::Display))]
enum Mode {
    Block,
    Byte,
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(derive_more::Display))]
enum Word {
    U8,
    U16,
    U32,
}

const BLOCK_SIZE: usize = BACKPLANE_MAX_TRANSFER_SIZE;

fn cmd53_arg(write: bool, func: u32, addr: u32, mode: Mode, len: usize) -> u32 {
    let (len, block_mode) = match mode {
        Mode::Block => (len / BLOCK_SIZE, 1u32),
        Mode::Byte => (len, 0u32),
    };

    let op_code = 1;

    (write as u32) << 31 | func << 28 | block_mode << 27 | op_code << 26 | (addr & 0x1ffff) << 9 | len as u32
}

/// Custom Spi Trait that _only_ supports the bus operation of the cyw43
/// Implementors are expected to hold the CS pin low during an operation.
pub trait SdioBusCyw43<const SIZE: usize> {
    /// The error type for the BlockDevice implementation.
    type Error: core::fmt::Debug;

    /// Set the bus to high speed 4-bit frequency
    fn set_bus_to_high_speed(&mut self, frequency: u32) -> Result<(), Self::Error>;

    /// Issue CMD52
    async fn cmd52(&mut self, arg: u32) -> Result<u16, Self::Error>;

    /// Issue CMD53 in block read mode
    async fn cmd53_block_read(&mut self, arg: u32, blocks: &mut [Aligned<A4, [u8; SIZE]>]) -> Result<(), Self::Error>;

    /// Issue CMD53 in byte read mode
    async fn cmd53_byte_read(&mut self, arg: u32, buffer: &mut Aligned<A4, [u8]>) -> Result<(), Self::Error>;

    /// Issue CMD53 in block write mode
    async fn cmd53_block_write(&mut self, arg: u32, blocks: &[Aligned<A4, [u8; SIZE]>]) -> Result<(), Self::Error>;

    /// Issue CMD53 in byte write mode
    async fn cmd53_byte_write(&mut self, arg: u32, buffer: &Aligned<A4, [u8]>) -> Result<(), Self::Error>;

    /// Wait for events from the Device. A typical implementation would wait for the IRQ pin to be high.
    /// The default implementation always reports ready, resulting in active polling of the device.
    async fn wait_for_event(&mut self) {
        Timer::after_millis(800).await;
    }
}

impl<const SIZE: usize, T: SdioBusCyw43<SIZE>> SdioBusCyw43<SIZE> for &mut T {
    type Error = T::Error;

    fn set_bus_to_high_speed(&mut self, frequency: u32) -> Result<(), Self::Error> {
        T::set_bus_to_high_speed(self, frequency)
    }

    async fn cmd52(&mut self, arg: u32) -> Result<u16, Self::Error> {
        T::cmd52(self, arg).await
    }

    async fn cmd53_block_read(&mut self, arg: u32, blocks: &mut [Aligned<A4, [u8; SIZE]>]) -> Result<(), Self::Error> {
        T::cmd53_block_read(self, arg, blocks).await
    }

    async fn cmd53_byte_read(&mut self, arg: u32, buffer: &mut Aligned<A4, [u8]>) -> Result<(), Self::Error> {
        T::cmd53_byte_read(self, arg, buffer).await
    }

    async fn cmd53_block_write(&mut self, arg: u32, blocks: &[Aligned<A4, [u8; SIZE]>]) -> Result<(), Self::Error> {
        T::cmd53_block_write(self, arg, blocks).await
    }

    async fn cmd53_byte_write(&mut self, arg: u32, buffer: &Aligned<A4, [u8]>) -> Result<(), Self::Error> {
        T::cmd53_byte_write(self, arg, buffer).await
    }

    async fn wait_for_event(&mut self) {
        T::wait_for_event(self).await
    }
}

/// Doc
pub struct SdioBus<SDIO> {
    backplane_window: u32,
    sdio: SDIO,
}

impl<SDIO> SdioBus<SDIO>
where
    SDIO: SdioBusCyw43<BLOCK_SIZE>,
{
    pub(crate) fn new(sdio: SDIO) -> Self {
        Self {
            backplane_window: 0xAAAA_AAAA,
            sdio,
        }
    }

    async fn backplane_readn(&mut self, addr: u32, word: Word) -> u32 {
        trace!("backplane_readn addr = {:08x} len = {}", addr, word);

        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if word == Word::U32 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG;
        }

        let val = match word {
            Word::U8 => self.read8(FUNC_BACKPLANE, bus_addr).await as u32,
            Word::U16 => self.read16(FUNC_BACKPLANE, bus_addr).await as u32,
            Word::U32 => self.read32(FUNC_BACKPLANE, bus_addr).await,
        };

        trace!("backplane_readn addr = {:08x} len = {} val = {:08x}", addr, word, val);

        self.backplane_set_window(CHIPCOMMON_BASE_ADDRESS).await;

        return val;
    }

    async fn backplane_writen(&mut self, addr: u32, val: u32, word: Word) {
        trace!("backplane_writen addr = {:08x} len = {} val = {:08x}", addr, word, val);

        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if word == Word::U32 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG;
        }

        let _ = match word {
            Word::U8 => self.write8(FUNC_BACKPLANE, bus_addr, val.try_into().unwrap()).await,
            Word::U16 => self.write16(FUNC_BACKPLANE, bus_addr, val.try_into().unwrap()).await,
            Word::U32 => self.write32(FUNC_BACKPLANE, bus_addr, val).await,
        };

        self.backplane_set_window(CHIPCOMMON_BASE_ADDRESS).await;
    }

    async fn backplane_set_window(&mut self, addr: u32) {
        let new_window = addr & !BACKPLANE_ADDRESS_MASK;

        if (new_window >> 24) as u8 != (self.backplane_window >> 24) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_HIGH,
                (new_window >> 24) as u8,
            )
            .await;
        }
        if (new_window >> 16) as u8 != (self.backplane_window >> 16) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_MID,
                (new_window >> 16) as u8,
            )
            .await;
        }
        if (new_window >> 8) as u8 != (self.backplane_window >> 8) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_LOW,
                (new_window >> 8) as u8,
            )
            .await;
        }
        self.backplane_window = new_window;
    }

    async fn cmd52(&mut self, write: bool, func: u32, addr: u32, val: u8) -> u8 {
        let arg: u32 = (write as u32) << 31 | func << 28 | (addr & 0x1ffff) << 9 | (val as u32 & 0xff);

        let result = self.sdio.cmd52(arg).await.unwrap_or(u16::MAX) as u8;

        result
    }

    async fn cmd53_write(&mut self, func: u32, mut addr: u32, buf: &Aligned<A4, [u8]>) {
        let byte_part = size_of_val(buf) % BLOCK_SIZE;
        let block_part = size_of_val(buf) - byte_part;

        if block_part > 0 {
            let buf = &buf[..block_part];

            if self
                .sdio
                .cmd53_block_write(cmd53_arg(true, func, addr, Mode::Block, buf.len()), unsafe {
                    slice::from_raw_parts(buf.as_ptr() as *mut _, size_of_val(buf) / BLOCK_SIZE)
                })
                .await
                .is_err()
            {
                debug!("cmd53 block read failed");
            }

            addr += block_part as u32;
        }

        if byte_part > 0 {
            let buf = &buf[block_part..];

            if self
                .sdio
                .cmd53_byte_write(cmd53_arg(true, func, addr, Mode::Byte, buf.len()), buf)
                .await
                .is_err()
            {
                debug!("cmd53 byte read failed (size: {})", size_of_val(buf));
            }
        }
    }

    async fn cmd53_read(&mut self, func: u32, mut addr: u32, buf: &mut Aligned<A4, [u8]>) {
        let byte_part = size_of_val(buf) % BLOCK_SIZE;
        let block_part = size_of_val(buf) - byte_part;

        if block_part > 0 {
            let buf = &mut buf[..block_part];

            if self
                .sdio
                .cmd53_block_read(cmd53_arg(false, func, addr, Mode::Block, buf.len()), unsafe {
                    slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut _, size_of_val(buf) / BLOCK_SIZE)
                })
                .await
                .is_err()
            {
                debug!("cmd53 block read failed");
            }

            addr += block_part as u32;
        }

        if byte_part > 0 {
            let buf = &mut buf[block_part..];

            if self
                .sdio
                .cmd53_byte_read(cmd53_arg(false, func, addr, Mode::Byte, buf.len()), buf)
                .await
                .is_err()
            {
                debug!("cmd53 byte read failed (size: {})", size_of_val(buf));
            }
        }
    }
}

impl<SDIO> SealedBus for SdioBus<SDIO>
where
    SDIO: SdioBusCyw43<64>,
{
    const TYPE: BusType = BusType::Sdio;

    async fn init(&mut self, _bluetooth_enabled: bool) {
        // whd_bus_sdio_init

        // set up backplane
        if !try_until(
            async || {
                self.write8(BUS_FUNCTION, SDIOD_CCCR_IOEN, SDIO_FUNC_ENABLE_1 as u8)
                    .await;

                self.read8(BUS_FUNCTION, SDIOD_CCCR_IOEN).await as u32 == SDIO_FUNC_ENABLE_1
            },
            Duration::from_millis(500),
        )
        .await
        {
            debug!("timeout while setting up the backplane");
            return;
        }

        debug!("backplane is up");

        // Read the bus width and set to 4 bits (1-bit bus is not currently supported)
        let reg = self.read8(BUS_FUNCTION, SDIOD_CCCR_BICTRL).await as u32;

        self.write8(
            BUS_FUNCTION,
            SDIOD_CCCR_BICTRL,
            ((reg & !BUS_SD_DATA_WIDTH_MASK) | BUS_SD_DATA_WIDTH_4BIT) as u8,
        )
        .await;

        // Set the block size
        if !try_until(
            async || {
                self.write8(BUS_FUNCTION, SDIOD_CCCR_BLKSIZE_0, SDIO_64B_BLOCK as u8)
                    .await;

                self.read8(BUS_FUNCTION, SDIOD_CCCR_BLKSIZE_0).await as u32 == SDIO_64B_BLOCK
            },
            Duration::from_millis(500),
        )
        .await
        {
            debug!("timeout while setting block size");
            return;
        }

        self.write8(BUS_FUNCTION, SDIOD_CCCR_BLKSIZE_0, SDIO_64B_BLOCK as u8)
            .await;
        self.write8(BUS_FUNCTION, SDIOD_CCCR_F1BLKSIZE_0, SDIO_64B_BLOCK as u8)
            .await;
        self.write8(BUS_FUNCTION, SDIOD_CCCR_F2BLKSIZE_0, SDIO_64B_BLOCK as u8)
            .await;
        self.write8(BUS_FUNCTION, SDIOD_CCCR_F2BLKSIZE_1, 0).await;

        // Enable/Disable Client interrupts
        self.write8(
            BUS_FUNCTION,
            SDIOD_CCCR_INTEN,
            (INTR_CTL_MASTER_EN | INTR_CTL_FUNC1_EN | INTR_CTL_FUNC2_EN) as u8,
        )
        .await;

        self.sdio.set_bus_to_high_speed(25_000_000).unwrap();

        // enable more than 25MHz bus
        let reg = self.read8(BUS_FUNCTION, SDIOD_CCCR_SPEED_CONTROL).await as u32;
        if reg & 1 != 0 {
            self.write8(BUS_FUNCTION, SDIOD_CCCR_SPEED_CONTROL, (reg | SDIO_SPEED_EHS) as u8)
                .await;

            self.sdio.set_bus_to_high_speed(50_000_000).unwrap();
        }

        // Wait till the backplane is ready
        if !try_until(
            async || self.read8(BUS_FUNCTION, SDIOD_CCCR_IORDY).await as u32 & SDIO_FUNC_READY_1 != 0,
            Duration::from_millis(500),
        )
        .await
        {
            debug!("timeout while waiting for backplane to be ready");
            return;
        }
    }

    async fn wlan_read(&mut self, buf: &mut Aligned<A4, [u8]>) {
        self.cmd53_read(WLAN_FUNCTION, 0, buf).await;
    }

    async fn wlan_write(&mut self, buf: &Aligned<A4, [u8]>) {
        self.cmd53_write(WLAN_FUNCTION, 0, buf).await;
    }

    #[allow(unused)]
    async fn bp_read(&mut self, mut addr: u32, mut data: &mut [u8]) {
        unimplemented!()
    }

    /// A.K.A. cyw43_download_resource
    async fn bp_write(&mut self, mut addr: u32, mut data: &[u8]) {
        trace!("bp_write addr = {:08x}", addr);

        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr % 4 == 0);

        let mut buf = [0u32; BACKPLANE_MAX_TRANSFER_SIZE / 4];

        while !data.is_empty() {
            // Ensure transfer doesn't cross a window boundary.
            let window_offs = addr & BACKPLANE_ADDRESS_MASK;
            let window_remaining = BACKPLANE_WINDOW_SIZE - window_offs as usize;

            let len = data.len().min(BACKPLANE_MAX_TRANSFER_SIZE).min(window_remaining);

            slice8_mut(&mut buf)[..len].copy_from_slice(&data[..len]);

            self.backplane_set_window(addr).await;

            self.cmd53_write(
                BACKPLANE_FUNCTION,
                addr & BACKPLANE_ADDRESS_MASK as u32,
                &aligned_ref(&buf),
            )
            .await;

            // Advance ptr.
            addr += len as u32;
            data = &data[len..];
        }

        // TODO: implement verify download
    }

    async fn bp_read8(&mut self, addr: u32) -> u8 {
        self.backplane_readn(addr, Word::U8).await as u8
    }

    async fn bp_write8(&mut self, addr: u32, val: u8) {
        self.backplane_writen(addr, val as u32, Word::U8).await
    }

    async fn bp_read16(&mut self, addr: u32) -> u16 {
        self.backplane_readn(addr, Word::U16).await as u16
    }

    #[allow(unused)]
    async fn bp_write16(&mut self, addr: u32, val: u16) {
        self.backplane_writen(addr, val as u32, Word::U16).await
    }

    #[allow(unused)]
    async fn bp_read32(&mut self, addr: u32) -> u32 {
        self.backplane_readn(addr, Word::U32).await
    }

    async fn bp_write32(&mut self, addr: u32, val: u32) {
        self.backplane_writen(addr, val, Word::U32).await
    }

    async fn read8(&mut self, func: u32, addr: u32) -> u8 {
        self.cmd52(false, func, addr, 0).await.into()
    }

    async fn write8(&mut self, func: u32, addr: u32, val: u8) {
        self.cmd52(true, func, addr, val).await;
    }

    async fn read16(&mut self, func: u32, addr: u32) -> u16 {
        let mut val = [0u32];
        self.cmd53_read(func, addr, &mut aligned_mut(&mut val)[..2]).await;

        u16::from_be_bytes(val[0].to_be_bytes()[..2].try_into().unwrap())
    }

    #[allow(unused)]
    async fn write16(&mut self, func: u32, addr: u32, val: u16) {
        self.cmd53_write(func, addr, &aligned_ref(&[val as u32])[..2]).await;
    }

    async fn read32(&mut self, func: u32, addr: u32) -> u32 {
        let mut val = [0u32];
        self.cmd53_read(func, addr, &mut aligned_mut(&mut val)).await;

        val[0]
    }

    #[allow(unused)]
    async fn write32(&mut self, func: u32, addr: u32, val: u32) {
        self.cmd53_write(func, addr, &aligned_ref(&[val])).await;
    }

    async fn wait_for_event(&mut self) {
        Timer::after(Duration::from_millis(250)).await;
        // self.sdio.wait_for_event().await;
    }
}
