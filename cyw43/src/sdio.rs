use core::mem;

use aligned::{A4, Aligned, Alignment};
use embassy_time::{Delay, Duration, Timer};

use crate::WithContext;
use crate::consts::*;
use crate::runner::{BusConfig, BusType, SealedBus};
use crate::util::try_until;

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

/// Create an aligned value from a non-aligned value
pub trait ToAligned {
    /// Element
    type Element: ?Sized;

    /// Create a type-checked aligned value from a value that is aligned.
    fn to_aligned<A: Alignment>(&self) -> &Aligned<A, Self::Element>;
}

impl<T: ?Sized> ToAligned for &T {
    type Element = T;

    #[inline]
    fn to_aligned<A: Alignment>(&self) -> &Aligned<A, Self::Element> {
        assert!(self as *const _ as usize % A::ALIGN == 0);

        unsafe { mem::transmute(*self) }
    }
}

/// Create an aligned value from a non-aligned value
pub trait ToMutAligned {
    /// Element
    type Element: ?Sized;

    /// Create a type-checked aligned value from a value that is aligned.
    fn to_mut_aligned<A: Alignment>(&mut self) -> &mut Aligned<A, Self::Element>;
}

impl<T: ?Sized> ToMutAligned for T {
    type Element = T;

    #[inline]
    fn to_mut_aligned<A: Alignment>(&mut self) -> &mut Aligned<A, Self::Element> {
        assert!(self as *mut _ as *mut u8 as usize % A::ALIGN == 0);

        unsafe { mem::transmute(&mut *self) }
    }
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

fn to_blocks<const BLOCK_SIZE: usize>(bytes: &Aligned<A4, [u8]>) -> &[Aligned<A4, [u8; BLOCK_SIZE]>] {
    assert!(bytes.len() % BLOCK_SIZE == 0);

    let ptr = bytes.as_ptr() as *const Aligned<A4, [u8; BLOCK_SIZE]>;
    let len = bytes.len() / BLOCK_SIZE;

    unsafe { core::slice::from_raw_parts(ptr, len) }
}

fn to_blocks_mut<const BLOCK_SIZE: usize>(bytes: &mut Aligned<A4, [u8]>) -> &mut [Aligned<A4, [u8; BLOCK_SIZE]>] {
    assert!(bytes.len() % BLOCK_SIZE == 0);

    let ptr = bytes.as_mut_ptr() as *mut Aligned<A4, [u8; BLOCK_SIZE]>;
    let len = bytes.len() / BLOCK_SIZE;

    unsafe { core::slice::from_raw_parts_mut(ptr, len) }
}

pub struct Config {
    pub max_f: u32,
    pub out_of_band_irq: bool,
}

/// Doc
pub struct SdioBus<SDIO>
where
    SDIO: ::sdio::MmcBus,
{
    backplane_window: u32,
    sdio: ::sdio::sdio::SdioCard<SDIO, Delay>,
}

impl<SDIO> SdioBus<SDIO>
where
    SDIO: ::sdio::MmcBus,
{
    pub(crate) fn new(sdio: SDIO) -> Self {
        Self {
            backplane_window: 0xAAAA_AAAA,
            sdio: ::sdio::sdio::SdioCard::new_uninit(sdio, Delay),
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

    async fn cmd53_write(&mut self, func: u8, mut addr: u32, buf: &Aligned<A4, [u8]>) -> crate::Result<()> {
        // Use buf.len() (Deref to [u8]) not size_of_val, which rounds up to 4 bytes.
        let byte_part = buf.len() % BLOCK_SIZE;
        let block_part = buf.len() - byte_part;

        if block_part > 0 {
            let buf = &buf[..block_part];

            self.sdio
                .cmd53_write_blocks(func as u8, true, addr, to_blocks::<BLOCK_SIZE>(buf))
                .await
                .map_err(|_| crate::Error)
                .ctx("cmd53 block write failed")?;

            addr += block_part as u32;
        }

        if byte_part > 0 {
            let buf = &buf[block_part..];

            self.sdio
                .cmd53_write_bytes(func as u8, true, addr, buf)
                .await
                .map_err(|_| crate::Error)
                .ctx("cmd53 byte write failed")?;
        }

        Ok(())
    }

    async fn cmd53_read(&mut self, func: u8, mut addr: u32, buf: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
        // Use buf.len() (Deref to [u8]) not size_of_val, which rounds up to 4 bytes.
        let byte_part = buf.len() % BLOCK_SIZE;
        let block_part = buf.len() - byte_part;

        if block_part > 0 {
            let buf = &mut buf[..block_part];

            self.sdio
                .cmd53_read_blocks(func as u8, true, addr, to_blocks_mut::<BLOCK_SIZE>(buf))
                .await
                .map_err(|_| crate::Error)
                .ctx("cmd53 block write failed")?;

            addr += block_part as u32;
        }

        if byte_part > 0 {
            let buf = &mut buf[block_part..];

            self.sdio
                .cmd53_read_bytes(func as u8, true, addr, buf)
                .await
                .map_err(|_| crate::Error)
                .ctx("cmd53 byte write failed")?;
        }

        Ok(())
    }
}

impl<SDIO> SealedBus for SdioBus<SDIO>
where
    SDIO: ::sdio::MmcBus,
{
    const TYPE: BusType = BusType::Sdio;
    type Config = Config;

    async fn init<'a>(&mut self, _bluetooth_enabled: bool, config: &'a Config) -> crate::Result<BusConfig<'a>> {
        // acquire the bus
        self.sdio.reacquire(config.max_f).await.map_err(|_| crate::Error)?;

        // whd_bus_sdio_init

        // set up backplane
        try_until(
            async || {
                self.write8(FUNC_BUS, SDIOD_CCCR_IOEN, SDIO_FUNC_ENABLE_1 as u8).await;

                self.read8(FUNC_BUS, SDIOD_CCCR_IOEN).await as u32 == SDIO_FUNC_ENABLE_1
            },
            Duration::from_millis(500),
        )
        .await
        .ctx("timeout while setting up the backplane")?;

        debug!("backplane is up");

        // Set the block size
        try_until(
            async || {
                self.write8(FUNC_BUS, SDIOD_CCCR_BLKSIZE_0, SDIO_64B_BLOCK as u8).await;

                self.read8(FUNC_BUS, SDIOD_CCCR_BLKSIZE_0).await as u32 == SDIO_64B_BLOCK
            },
            Duration::from_millis(500),
        )
        .await
        .ctx("timeout while setting block size")?;

        self.write8(FUNC_BUS, SDIOD_CCCR_BLKSIZE_0, SDIO_64B_BLOCK as u8).await;
        self.write8(FUNC_BUS, SDIOD_CCCR_F1BLKSIZE_0, SDIO_64B_BLOCK as u8)
            .await;
        self.write8(FUNC_BUS, SDIOD_CCCR_F2BLKSIZE_0, SDIO_64B_BLOCK as u8)
            .await;
        self.write8(FUNC_BUS, SDIOD_CCCR_F2BLKSIZE_1, 0).await;

        // Enable/Disable Client interrupts
        self.write8(
            FUNC_BUS,
            SDIOD_CCCR_INTEN,
            (INTR_CTL_MASTER_EN | INTR_CTL_FUNC1_EN | INTR_CTL_FUNC2_EN) as u8,
        )
        .await;

        // Wait till the backplane is ready
        try_until(
            async || self.read8(FUNC_BUS, SDIOD_CCCR_IORDY).await as u32 & SDIO_FUNC_READY_1 != 0,
            Duration::from_millis(500),
        )
        .await
        .ctx("timeout while waiting for backplane to be ready")?;

        Ok(BusConfig::Sdio(config))
    }

    async fn wlan_read(&mut self, buf: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
        if let Err(e) = self.cmd53_read(FUNC_WLAN, 0, buf).await {
            buf.fill(0);
            // A timed-out partial F2 read leaves the same packet pending forever.
            // Mirror WHD's abort path so the device can reset its F2 read state.
            self.write8(FUNC_BUS, SDIOD_CCCR_IOABORT, FUNC_WLAN as u8).await;
            self.write8(FUNC_BACKPLANE, REG_BACKPLANE_FRAME_CONTROL, SFC_RF_TERM)
                .await;

            Err(e)
        } else {
            Ok(())
        }
    }

    async fn wlan_write(&mut self, buf: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
        self.cmd53_write(FUNC_WLAN, 0, &buf[4..]).await
    }

    async fn bp_read(&mut self, mut addr: u32, data: &mut [u8]) -> crate::Result<()> {
        trace!("bp_read addr = {:08x}, len = {}", addr, data.len());

        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr % 4 == 0);

        let mut data: &mut Aligned<A4, [u8]> = data.to_mut_aligned();

        loop {
            // Ensure transfer doesn't cross a window boundary.
            let window_offs = addr & BACKPLANE_ADDRESS_MASK;
            let window_remaining = BACKPLANE_WINDOW_SIZE - window_offs as usize;

            let len = data.len().min(BLOCK_BUFFER_SIZE).min(window_remaining);
            let buf = &mut data[..len];

            self.backplane_set_window(addr).await;

            self.cmd53_read(FUNC_BACKPLANE, addr & BACKPLANE_ADDRESS_MASK as u32, buf)
                .await?;

            // Advance ptr.
            addr += len as u32;
            if data.len() == len {
                break;
            } else {
                data = &mut data[len..];
            }
        }

        self.backplane_set_window(CHIPCOMMON_BASE_ADDRESS).await;

        Ok(())
    }

    /// A.K.A. cyw43_download_resource
    async fn bp_write(&mut self, mut addr: u32, data: &[u8]) -> crate::Result<()> {
        trace!("bp_write addr = {:08x}, len = {}", addr, data.len());

        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr % 4 == 0);

        let mut data: &Aligned<A4, [u8]> = data.to_aligned();

        loop {
            // Ensure transfer doesn't cross a window boundary.
            let window_offs = addr & BACKPLANE_ADDRESS_MASK;
            let window_remaining = BACKPLANE_WINDOW_SIZE - window_offs as usize;

            let len = data.len().min(BLOCK_BUFFER_SIZE).min(window_remaining);
            let buf = &data[..len];

            self.backplane_set_window(addr).await;

            self.cmd53_write(FUNC_BACKPLANE, addr & BACKPLANE_ADDRESS_MASK as u32, buf)
                .await?;

            // Advance ptr.
            addr += len as u32;
            if data.len() == len {
                break;
            } else {
                data = &data[len..];
            }
        }

        self.backplane_set_window(CHIPCOMMON_BASE_ADDRESS).await;

        Ok(())
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

    async fn read8(&mut self, func: u8, addr: u32) -> u8 {
        self.sdio.cmd52_read(func as u8, addr).await.unwrap_or_default()
    }

    async fn write8(&mut self, func: u8, addr: u32, val: u8) {
        let _ = self.sdio.cmd52_write(func as u8, addr, val).await;
    }

    async fn read16(&mut self, func: u8, addr: u32) -> u16 {
        let mut val: Aligned<A4, [u8; _]> = Aligned([0u8; 2]);
        let _ = self.cmd53_read(func, addr, &mut val).await;

        u16::from_le_bytes(*val)
    }

    async fn write16(&mut self, func: u8, addr: u32, val: u16) {
        let val: Aligned<A4, [u8; 2]> = Aligned(val.to_le_bytes().into());
        let _ = self.cmd53_write(func, addr, &val).await;
    }

    async fn read32(&mut self, func: u8, addr: u32) -> u32 {
        let mut val: Aligned<A4, [u8; _]> = Aligned([0u8; 4]);
        let _ = self.cmd53_read(func, addr, &mut val).await;

        u32::from_le_bytes(*val)
    }

    async fn write32(&mut self, func: u8, addr: u32, val: u32) {
        let val: Aligned<A4, [u8; 4]> = Aligned(val.to_le_bytes().into());
        let _ = self.cmd53_write(func, addr, &val).await;
    }

    async fn wait_for_event(&mut self) {
        Timer::after(Duration::from_millis(10)).await;
        // self.sdio.wait_for_event().await;
    }
}
